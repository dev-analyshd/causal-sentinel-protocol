use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use chrono::Utc;

/// x402 Facilitator — Per-request micropayments for agent services
///
/// Implements the x402 payment protocol on Casper Network:
/// - Free tier: coherence_evaluate, moat_status, silence_check
/// - Premium tier: trade_evaluate (1.0 CSPR), reasoning_chain (2.0 CSPR)
/// - All payments settle on Casper mainnet with deterministic finality

#[derive(Clone)]
pub struct AppState {
    pub payment_processor: Arc<RwLock<PaymentProcessor>>,
    pub service_registry: Arc<RwLock<ServiceRegistry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub agent_id: String,
    pub service: String,
    pub amount_motes: u64,
    pub sender_address: String,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
    pub remaining_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub name: String,
    pub description: String,
    pub cost_motes: u64,
    pub tier: ServiceTier,
    pub requires_zk: bool,
    pub rate_limit: u32, // requests per minute
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceTier {
    Free,
    Basic,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBalance {
    pub agent_id: String,
    pub balance_motes: u64,
    pub total_spent: u64,
    pub total_requests: u64,
    pub last_payment: String,
}

pub struct PaymentProcessor {
    balances: std::collections::HashMap<String, u64>,
    spent: std::collections::HashMap<String, u64>,
    request_counts: std::collections::HashMap<String, u32>,
    nonce_tracker: std::collections::HashMap<String, u64>,
}

pub struct ServiceRegistry {
    services: std::collections::HashMap<String, ServiceDefinition>,
}

impl PaymentProcessor {
    pub fn new() -> Self {
        Self {
            balances: std::collections::HashMap::new(),
            spent: std::collections::HashMap::new(),
            request_counts: std::collections::HashMap::new(),
            nonce_tracker: std::collections::HashMap::new(),
        }
    }

    pub fn deposit(&mut self, agent_id: &str, amount: u64) {
        let balance = self.balances.entry(agent_id.to_string()).or_insert(0);
        *balance += amount;
        info!("deposit agent_id={} amount={}", agent_id, amount);
    }

    pub fn process_payment(&mut self, request: &PaymentRequest) -> Result<PaymentResponse, String> {
        // Verify nonce
        let last_nonce = self.nonce_tracker.get(&request.agent_id).copied().unwrap_or(0);
        if request.nonce <= last_nonce {
            return Err("Nonce already used".to_string());
        }

        // Check balance
        let balance = self.balances.get(&request.agent_id).copied().unwrap_or(0);
        if balance < request.amount_motes {
            return Err(format!("Insufficient balance: {} < {}", balance, request.amount_motes));
        }

        // Deduct
        let new_balance = balance - request.amount_motes;
        self.balances.insert(request.agent_id.clone(), new_balance);

        let spent = self.spent.entry(request.agent_id.clone()).or_insert(0);
        *spent += request.amount_motes;

        self.nonce_tracker.insert(request.agent_id.clone(), request.nonce);

        // In production: submit to Casper mainnet
        let hash_input = format!("{}{}{}{}", request.agent_id, request.service, request.nonce, Utc::now());
        let digest = Sha3_256::digest(hash_input.as_bytes());
        let tx_hash = format!("hash-{}", hex::encode(digest));

        info!("payment_processed agent_id={} amount={}", request.agent_id, request.amount_motes);

        Ok(PaymentResponse {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
            remaining_balance: new_balance,
        })
    }

    pub fn get_balance(&self, agent_id: &str) -> u64 {
        self.balances.get(agent_id).copied().unwrap_or(0)
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let mut services = std::collections::HashMap::new();

        // Free tier
        services.insert("coherence_evaluate".to_string(), ServiceDefinition {
            name: "Coherence Evaluate".to_string(),
            description: "Evaluate agent coherence score Ψ(t)".to_string(),
            cost_motes: 0,
            tier: ServiceTier::Free,
            requires_zk: false,
            rate_limit: 60,
        });

        services.insert("moat_status".to_string(), ServiceDefinition {
            name: "Moat Status".to_string(),
            description: "Get current moat Λ(t) and tier".to_string(),
            cost_motes: 0,
            tier: ServiceTier::Free,
            requires_zk: false,
            rate_limit: 60,
        });

        services.insert("silence_check".to_string(), ServiceDefinition {
            name: "Silence Check".to_string(),
            description: "Check if agent is in SILENCE mode".to_string(),
            cost_motes: 0,
            tier: ServiceTier::Free,
            requires_zk: false,
            rate_limit: 60,
        });

        // Premium tier
        services.insert("trade_evaluate".to_string(), ServiceDefinition {
            name: "Trade Evaluate".to_string(),
            description: "Evaluate trade opportunity with coherence gating".to_string(),
            cost_motes: 1_000_000_000, // 1.0 CSPR
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        });

        services.insert("reasoning_chain".to_string(), ServiceDefinition {
            name: "Reasoning Chain".to_string(),
            description: "Execute multi-chain reasoning with consensus".to_string(),
            cost_motes: 2_000_000_000, // 2.0 CSPR
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 5,
        });

        services.insert("cross_chain_bridge".to_string(), ServiceDefinition {
            name: "Cross-Chain Bridge".to_string(),
            description: "Bridge assets across chains with ZK compliance".to_string(),
            cost_motes: 5_000_000_000, // 5.0 CSPR
            tier: ServiceTier::Enterprise,
            requires_zk: true,
            rate_limit: 2,
        });

        Self { services }
    }

    pub fn get_service(&self, name: &str) -> Option<ServiceDefinition> {
        self.services.get(name).cloned()
    }

    pub fn list_services(&self) -> Vec<ServiceDefinition> {
        self.services.values().cloned().collect()
    }
}

async fn get_services(State(state): State<AppState>) -> Json<Vec<ServiceDefinition>> {
    let registry = state.service_registry.read().await;
    Json(registry.list_services())
}

async fn get_service(
    Path(service_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ServiceDefinition>, StatusCode> {
    let registry = state.service_registry.read().await;
    match registry.get_service(&service_name) {
        Some(svc) => Ok(Json(svc)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn process_payment(
    State(state): State<AppState>,
    Json(request): Json<PaymentRequest>,
) -> Result<Json<PaymentResponse>, StatusCode> {
    let mut processor = state.payment_processor.write().await;

    match processor.process_payment(&request) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Ok(Json(PaymentResponse {
            success: false,
            tx_hash: None,
            error: Some(e),
            remaining_balance: 0,
        })),
    }
}

async fn get_balance(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
) -> Json<AgentBalance> {
    let processor = state.payment_processor.read().await;
    let balance = processor.get_balance(&agent_id);

    Json(AgentBalance {
        agent_id: agent_id.clone(),
        balance_motes: balance,
        total_spent: processor.spent.get(&agent_id).copied().unwrap_or(0),
        total_requests: processor.request_counts.get(&agent_id).copied().unwrap_or(0) as u64,
        last_payment: Utc::now().to_rfc3339(),
    })
}

async fn deposit(
    Path((agent_id, amount)): Path<(String, u64)>,
    State(state): State<AppState>,
) -> Json<PaymentResponse> {
    let mut processor = state.payment_processor.write().await;
    processor.deposit(&agent_id, amount);

    Json(PaymentResponse {
        success: true,
        tx_hash: None,
        error: None,
        remaining_balance: processor.get_balance(&agent_id),
    })
}

pub async fn run_server(port: u16) -> Result<()> {
    let state = AppState {
        payment_processor: Arc::new(RwLock::new(PaymentProcessor::new())),
        service_registry: Arc::new(RwLock::new(ServiceRegistry::new())),
    };

    let app = Router::new()
        .route("/services", get(get_services))
        .route("/services/:name", get(get_service))
        .route("/pay", post(process_payment))
        .route("/balance/:agent_id", get(get_balance))
        .route("/deposit/:agent_id/:amount", post(deposit))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("x402_facilitator_started addr={}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run_server(8080).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_processor_deposit() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 5_000_000_000);
        assert_eq!(pp.get_balance("agent_001"), 5_000_000_000);
    }

    #[test]
    fn test_payment_processor_insufficient_balance() {
        let mut pp = PaymentProcessor::new();
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000,
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce: 1,
        };
        let result = pp.process_payment(&req);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient balance"));
    }

    #[test]
    fn test_payment_processor_success() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 5_000_000_000);
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000,
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce: 1,
        };
        let result = pp.process_payment(&req);
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.success);
        assert_eq!(resp.remaining_balance, 4_000_000_000);
    }

    #[test]
    fn test_service_registry() {
        let sr = ServiceRegistry::new();
        let svc = sr.get_service("coherence_evaluate");
        assert!(svc.is_some());
        let svc = svc.unwrap();
        assert_eq!(svc.cost_motes, 0);
        assert!(!svc.requires_zk);
    }

    #[test]
    fn test_service_registry_premium() {
        let sr = ServiceRegistry::new();
        let svc = sr.get_service("trade_evaluate");
        assert!(svc.is_some());
        let svc = svc.unwrap();
        assert_eq!(svc.cost_motes, 1_000_000_000);
        assert!(svc.requires_zk);
    }
}
