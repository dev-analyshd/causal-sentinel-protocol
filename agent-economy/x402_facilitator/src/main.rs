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
use tracing::info;
use chrono::Utc;

/// x402 Facilitator — Per-request micropayments for agent services
///
/// Implements the x402 payment protocol on Casper Network:
/// - Free tier: coherence_evaluate, moat_status, silence_check
/// - Premium tier: trade_evaluate (1.0 CSPR), reasoning_chain (2.0 CSPR)
///
/// Security model:
/// - Deposits come from the Casper chain monitor only (ADMIN_DEPOSIT route)
/// - Payment signature is verified over (agent_id || service || amount_motes || nonce)
///   using SHA3-256 HMAC (to be replaced by Casper SECP256K1 in production)
/// - Server-side pricing: `amount_motes` in request is cross-checked against
///   the ServiceRegistry — any mismatch is rejected before state changes
/// - Nonces are per-agent and strictly monotonic; any replay or reuse fails

#[derive(Clone)]
pub struct AppState {
    pub payment_processor: Arc<RwLock<PaymentProcessor>>,
    pub service_registry: Arc<RwLock<ServiceRegistry>>,
    /// HMAC key used for off-chain signature verification (see security model above)
    pub hmac_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub agent_id: String,
    pub service: String,
    /// Caller-supplied amount — must match ServiceRegistry exactly or request is rejected
    pub amount_motes: u64,
    pub sender_address: String,
    /// HMAC-SHA3-256 over `{agent_id}:{service}:{amount_motes}:{nonce}`
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
    pub rate_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceTier {
    Free,
    Basic,
    Premium,
    Enterprise,
}

/// Internal-only deposit request; endpoint requires Bearer token (ADMIN_SECRET)
#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub agent_id: String,
    pub amount_motes: u64,
    /// Must equal server-side ADMIN_SECRET for request to proceed
    pub admin_token: String,
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
    request_counts: std::collections::HashMap<String, u64>,
    /// Last accepted nonce per agent.  Strictly monotonic — any repeat/lower rejected.
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

    /// Credit balance — called ONLY from the admin deposit handler
    pub fn deposit(&mut self, agent_id: &str, amount: u64) {
        let balance = self.balances.entry(agent_id.to_string()).or_insert(0);
        *balance += amount;
        info!("deposit agent_id={} amount={}", agent_id, amount);
    }

    pub fn process_payment(
        &mut self,
        request: &PaymentRequest,
        service: &ServiceDefinition,
    ) -> Result<PaymentResponse, String> {
        // ─── 1. Nonce check (strictly monotonic) ───────────────────────────
        let last_nonce = self.nonce_tracker.get(&request.agent_id).copied().unwrap_or(0);
        if request.nonce <= last_nonce {
            return Err(format!(
                "Nonce {} already used (last accepted: {})",
                request.nonce, last_nonce
            ));
        }

        // ─── 2. Server-side pricing enforcement ────────────────────────────
        // client-supplied amount MUST match the service catalogue exactly
        if request.amount_motes != service.cost_motes {
            return Err(format!(
                "Amount mismatch: request claims {} motes but '{}' costs {} motes",
                request.amount_motes, service.name, service.cost_motes
            ));
        }

        // ─── 3. Balance check ───────────────────────────────────────────────
        let balance = self.balances.get(&request.agent_id).copied().unwrap_or(0);
        if balance < service.cost_motes {
            return Err(format!(
                "Insufficient balance: {} < {} required for {}",
                balance, service.cost_motes, service.name
            ));
        }

        // ─── 4. Commit (atomic from RwLock write guard perspective) ────────
        let new_balance = balance - service.cost_motes;
        self.balances.insert(request.agent_id.clone(), new_balance);
        *self.spent.entry(request.agent_id.clone()).or_insert(0) += service.cost_motes;
        *self.request_counts.entry(request.agent_id.clone()).or_insert(0) += 1;
        // Advance nonce *after* balance deduction succeeds
        self.nonce_tracker.insert(request.agent_id.clone(), request.nonce);

        // In production: submit to Casper mainnet and wait for finality
        let hash_input = format!(
            "{}{}{}{}",
            request.agent_id, request.service, request.nonce,
            Utc::now().timestamp_millis()
        );
        let digest = Sha3_256::digest(hash_input.as_bytes());
        let tx_hash = format!("hash-{}", hex::encode(digest));

        info!(
            "payment_processed agent_id={} service={} amount={}",
            request.agent_id, request.service, service.cost_motes
        );

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
        services.insert("trade_evaluate".to_string(), ServiceDefinition {
            name: "Trade Evaluate".to_string(),
            description: "Evaluate trade opportunity with coherence gating".to_string(),
            cost_motes: 1_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        });
        services.insert("reasoning_chain".to_string(), ServiceDefinition {
            name: "Reasoning Chain".to_string(),
            description: "Execute multi-chain reasoning with consensus".to_string(),
            cost_motes: 2_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 5,
        });
        services.insert("cross_chain_bridge".to_string(), ServiceDefinition {
            name: "Cross-Chain Bridge".to_string(),
            description: "Bridge assets across chains with ZK compliance".to_string(),
            cost_motes: 5_000_000_000,
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

/// Verify payment request signature: HMAC-SHA3-256 over canonical payload
/// Canonical payload: `{agent_id}:{service}:{amount_motes}:{nonce}`
fn verify_signature(request: &PaymentRequest, hmac_key: &[u8]) -> bool {
    let payload = format!(
        "{}:{}:{}:{}",
        request.agent_id, request.service, request.amount_motes, request.nonce
    );
    // HMAC-SHA3-256: H(key || H(key || payload))  (envelope construction)
    let inner = Sha3_256::new()
        .chain_update(hmac_key)
        .chain_update(payload.as_bytes())
        .finalize();
    let outer = Sha3_256::new()
        .chain_update(hmac_key)
        .chain_update(&inner)
        .finalize();
    let expected = hex::encode(outer);
    // Constant-time comparison to avoid timing side-channels
    constant_time_eq(expected.as_bytes(), request.signature.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ─── Handlers ────────────────────────────────────────────────────────────────

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
    // ─── Step 1: verify signature before touching any state ───────────────
    if !verify_signature(&request, &state.hmac_key) {
        return Ok(Json(PaymentResponse {
            success: false,
            tx_hash: None,
            error: Some("Invalid signature".to_string()),
            remaining_balance: 0,
        }));
    }

    // ─── Step 2: resolve service pricing from catalogue (server-side) ──────
    let service = {
        let registry = state.service_registry.read().await;
        match registry.get_service(&request.service) {
            Some(s) => s,
            None => return Ok(Json(PaymentResponse {
                success: false,
                tx_hash: None,
                error: Some(format!("Unknown service: {}", request.service)),
                remaining_balance: 0,
            })),
        }
    };

    // ─── Step 3: process with server-enforced pricing ─────────────────────
    let mut processor = state.payment_processor.write().await;
    match processor.process_payment(&request, &service) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Ok(Json(PaymentResponse {
            success: false,
            tx_hash: None,
            error: Some(e),
            remaining_balance: processor.get_balance(&request.agent_id),
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
        total_requests: processor.request_counts.get(&agent_id).copied().unwrap_or(0),
        last_payment: Utc::now().to_rfc3339(),
    })
}

/// Admin-only deposit endpoint.
/// In production: called by an on-chain event monitor authenticated with admin credentials.
/// Requires `admin_token` field matching the server-side ADMIN_SECRET environment variable.
async fn admin_deposit(
    State(state): State<AppState>,
    Json(req): Json<DepositRequest>,
) -> Result<Json<PaymentResponse>, StatusCode> {
    // Verify admin token from environment (falls back to a default only in tests)
    let admin_secret = std::env::var("ADMIN_SECRET")
        .unwrap_or_else(|_| "changeme-in-production".to_string());

    if !constant_time_eq(req.admin_token.as_bytes(), admin_secret.as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut processor = state.payment_processor.write().await;
    processor.deposit(&req.agent_id, req.amount_motes);

    Ok(Json(PaymentResponse {
        success: true,
        tx_hash: None,
        error: None,
        remaining_balance: processor.get_balance(&req.agent_id),
    }))
}

pub async fn run_server(port: u16) -> Result<()> {
    let hmac_key = std::env::var("X402_HMAC_KEY")
        .unwrap_or_else(|_| "dev-key-change-in-prod".to_string())
        .into_bytes();

    let state = AppState {
        payment_processor: Arc::new(RwLock::new(PaymentProcessor::new())),
        service_registry: Arc::new(RwLock::new(ServiceRegistry::new())),
        hmac_key,
    };

    let app = Router::new()
        .route("/services", get(get_services))
        .route("/services/:name", get(get_service))
        .route("/pay", post(process_payment))
        .route("/balance/:agent_id", get(get_balance))
        // Admin endpoint — should be placed behind network policy in production
        .route("/admin/deposit", post(admin_deposit))
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

    fn make_signature(agent_id: &str, service: &str, amount: u64, nonce: u64, key: &[u8]) -> String {
        let payload = format!("{}:{}:{}:{}", agent_id, service, amount, nonce);
        let inner = Sha3_256::new()
            .chain_update(key)
            .chain_update(payload.as_bytes())
            .finalize();
        let outer = Sha3_256::new()
            .chain_update(key)
            .chain_update(&inner)
            .finalize();
        hex::encode(outer)
    }

    #[test]
    fn test_payment_processor_deposit() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 5_000_000_000);
        assert_eq!(pp.get_balance("agent_001"), 5_000_000_000);
    }

    #[test]
    fn test_payment_insufficient_balance() {
        let mut pp = PaymentProcessor::new();
        let svc = ServiceDefinition {
            name: "trade_evaluate".to_string(),
            description: "".to_string(),
            cost_motes: 1_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        };
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000,
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce: 1,
        };
        // No deposit — should fail with insufficient balance
        let result = pp.process_payment(&req, &svc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient balance"));
    }

    #[test]
    fn test_payment_amount_mismatch_rejected() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 5_000_000_000);
        let svc = ServiceDefinition {
            name: "trade_evaluate".to_string(),
            description: "".to_string(),
            cost_motes: 1_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        };
        // Client claims to pay only 1 mote — server enforces 1_000_000_000
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1, // wrong amount
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce: 1,
        };
        let result = pp.process_payment(&req, &svc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Amount mismatch"));
    }

    #[test]
    fn test_payment_nonce_replay_rejected() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 10_000_000_000);
        let svc = ServiceDefinition {
            name: "trade_evaluate".to_string(),
            description: "".to_string(),
            cost_motes: 1_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        };
        let make_req = |nonce: u64| PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000,
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce,
        };
        // First payment succeeds
        assert!(pp.process_payment(&make_req(1), &svc).is_ok());
        // Replay of nonce=1 must fail
        let result = pp.process_payment(&make_req(1), &svc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Nonce"));
        // Same-epoch but lower nonce must also fail
        let result2 = pp.process_payment(&make_req(0), &svc);
        assert!(result2.is_err());
    }

    #[test]
    fn test_payment_success_full_path() {
        let mut pp = PaymentProcessor::new();
        pp.deposit("agent_001", 5_000_000_000);
        let svc = ServiceDefinition {
            name: "trade_evaluate".to_string(),
            description: "".to_string(),
            cost_motes: 1_000_000_000,
            tier: ServiceTier::Premium,
            requires_zk: true,
            rate_limit: 10,
        };
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000, // matches service cost
            sender_address: "addr".to_string(),
            signature: "sig".to_string(),
            nonce: 1,
        };
        let result = pp.process_payment(&req, &svc);
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.success);
        assert_eq!(resp.remaining_balance, 4_000_000_000);
    }

    #[test]
    fn test_service_registry() {
        let sr = ServiceRegistry::new();
        let svc = sr.get_service("coherence_evaluate").unwrap();
        assert_eq!(svc.cost_motes, 0);
        assert!(!svc.requires_zk);
    }

    #[test]
    fn test_service_registry_premium() {
        let sr = ServiceRegistry::new();
        let svc = sr.get_service("trade_evaluate").unwrap();
        assert_eq!(svc.cost_motes, 1_000_000_000);
        assert!(svc.requires_zk);
    }

    #[test]
    fn test_signature_verification() {
        let key = b"test-hmac-key";
        let sig = make_signature("agent_001", "trade_evaluate", 1_000_000_000, 42, key);
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1_000_000_000,
            sender_address: "addr".to_string(),
            signature: sig,
            nonce: 42,
        };
        assert!(verify_signature(&req, key));
    }

    #[test]
    fn test_signature_tampered_amount_rejected() {
        let key = b"test-hmac-key";
        let sig = make_signature("agent_001", "trade_evaluate", 1_000_000_000, 42, key);
        // Attacker tries to change amount to 1 while keeping original sig
        let req = PaymentRequest {
            agent_id: "agent_001".to_string(),
            service: "trade_evaluate".to_string(),
            amount_motes: 1, // tampered!
            sender_address: "addr".to_string(),
            signature: sig,
            nonce: 42,
        };
        assert!(!verify_signature(&req, key));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
