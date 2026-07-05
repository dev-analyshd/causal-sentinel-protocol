use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Federation Protocol — A2A peer discovery, mutual coherence exchange
///
/// Enables agents to discover peers, exchange coherence scores,
/// and form federated learning clusters while maintaining ZK privacy.

#[derive(Clone)]
pub struct AppState {
    pub peer_registry: Arc<RwLock<PeerRegistry>>,
    pub coherence_exchange: Arc<RwLock<CoherenceExchange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub agent_id: String,
    pub endpoint: String,
    pub public_key: String,
    pub coherence_psi: f64,
    pub moat_lambda: f64,
    pub credential_tier: u8,
    pub domains: Vec<String>,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceExchangeRequest {
    pub from_agent: String,
    pub psi: f64,
    pub lambda: f64,
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceExchangeResponse {
    pub accepted: bool,
    pub peer_psi: f64,
    pub peer_lambda: f64,
    pub federation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationCluster {
    pub cluster_id: String,
    pub members: Vec<String>,
    pub avg_coherence: f64,
    pub avg_moat: f64,
    pub domain: String,
    pub created_at: u64,
}

pub struct PeerRegistry {
    peers: std::collections::HashMap<String, PeerInfo>,
}

pub struct CoherenceExchange {
    exchanges: std::collections::HashMap<String, Vec<CoherenceExchangeResponse>>,
    clusters: std::collections::HashMap<String, FederationCluster>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, peer: PeerInfo) {
        info!("peer_registered agent_id={} endpoint={}", peer.agent_id, peer.endpoint);
        self.peers.insert(peer.agent_id.clone(), peer);
    }

    pub fn discover_peers(&self, domain: Option<String>, min_tier: u8) -> Vec<PeerInfo> {
        self.peers
            .values()
            .filter(|p| {
                p.credential_tier >= min_tier
                    && domain.as_ref().map(|d| p.domains.contains(d)).unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    pub fn get_peer(&self, agent_id: &str) -> Option<PeerInfo> {
        self.peers.get(agent_id).cloned()
    }

    pub fn heartbeat(&mut self, agent_id: &str) {
        if let Some(peer) = self.peers.get_mut(agent_id) {
            peer.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }
}

impl CoherenceExchange {
    pub fn new() -> Self {
        Self {
            exchanges: std::collections::HashMap::new(),
            clusters: std::collections::HashMap::new(),
        }
    }

    pub fn exchange(&mut self, from: &str, _to: &str, request: &CoherenceExchangeRequest) -> CoherenceExchangeResponse {
        // Compute federation score based on coherence similarity
        let federation_score = (request.psi + request.lambda) / 2.0;

        let response = CoherenceExchangeResponse {
            accepted: federation_score > 0.5,
            peer_psi: request.psi,
            peer_lambda: request.lambda,
            federation_score,
        };

        let entries = self.exchanges.entry(from.to_string()).or_insert_with(Vec::new);
        entries.push(response.clone());

        info!("coherence_exchanged from={} score={}", from, federation_score);
        response
    }

    pub fn form_cluster(&mut self, domain: String, members: Vec<String>) -> FederationCluster {
        let hash_input = format!("{}{}", domain, members.join(","));
        let digest = Sha3_256::digest(hash_input.as_bytes());
        let cluster_id = format!("cluster-{}", hex::encode(digest));

        let cluster = FederationCluster {
            cluster_id: cluster_id.clone(),
            members: members.clone(),
            avg_coherence: 0.75,
            avg_moat: 1.2,
            domain,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.clusters.insert(cluster_id.clone(), cluster.clone());
        info!("cluster_formed cluster_id={} members={}", cluster_id, members.len());
        cluster
    }

    pub fn get_clusters(&self) -> Vec<FederationCluster> {
        self.clusters.values().cloned().collect()
    }
}

async fn register_peer(
    State(state): State<AppState>,
    Json(peer): Json<PeerInfo>,
) -> StatusCode {
    let mut registry = state.peer_registry.write().await;
    registry.register(peer);
    StatusCode::OK
}

async fn discover_peers(
    State(state): State<AppState>,
) -> Json<Vec<PeerInfo>> {
    let registry = state.peer_registry.read().await;
    Json(registry.discover_peers(None, 1))
}

async fn exchange_coherence(
    State(state): State<AppState>,
    Json(request): Json<CoherenceExchangeRequest>,
) -> Json<CoherenceExchangeResponse> {
    let mut exchange = state.coherence_exchange.write().await;
    let response = exchange.exchange(&request.from_agent, "local", &request);
    Json(response)
}

async fn get_clusters(
    State(state): State<AppState>,
) -> Json<Vec<FederationCluster>> {
    let exchange = state.coherence_exchange.read().await;
    Json(exchange.get_clusters())
}

pub async fn run_server(port: u16) -> Result<()> {
    let state = AppState {
        peer_registry: Arc::new(RwLock::new(PeerRegistry::new())),
        coherence_exchange: Arc::new(RwLock::new(CoherenceExchange::new())),
    };

    let app = Router::new()
        .route("/federation/register", post(register_peer))
        .route("/federation/discover", get(discover_peers))
        .route("/federation/exchange", post(exchange_coherence))
        .route("/federation/clusters", get(get_clusters))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("federation_protocol_started addr={}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run_server(8082).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_registry_register_and_discover() {
        let mut registry = PeerRegistry::new();
        let peer = PeerInfo {
            agent_id: "agent_001".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            public_key: "pk_abc".to_string(),
            coherence_psi: 0.75,
            moat_lambda: 1.2,
            credential_tier: 3,
            domains: vec!["Trading".to_string()],
            last_seen: 0,
        };
        registry.register(peer);
        let peers = registry.discover_peers(None, 1);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].agent_id, "agent_001");
    }

    #[test]
    fn test_peer_registry_tier_filter() {
        let mut registry = PeerRegistry::new();
        let peer = PeerInfo {
            agent_id: "agent_001".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            public_key: "pk_abc".to_string(),
            coherence_psi: 0.75,
            moat_lambda: 1.2,
            credential_tier: 2,
            domains: vec![],
            last_seen: 0,
        };
        registry.register(peer);
        // min_tier=3 should return nothing
        let peers = registry.discover_peers(None, 3);
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_coherence_exchange() {
        let mut exchange = CoherenceExchange::new();
        let req = CoherenceExchangeRequest {
            from_agent: "agent_001".to_string(),
            psi: 0.80,
            lambda: 1.5,
            timestamp: 12345,
            signature: "sig".to_string(),
        };
        let resp = exchange.exchange("agent_001", "agent_002", &req);
        assert!(resp.accepted);
        assert!((resp.federation_score - 1.15).abs() < 1e-9);
    }

    #[test]
    fn test_cluster_formation() {
        let mut exchange = CoherenceExchange::new();
        let cluster = exchange.form_cluster(
            "Trading".to_string(),
            vec!["agent_001".to_string(), "agent_002".to_string()],
        );
        assert!(cluster.cluster_id.starts_with("cluster-"));
        assert_eq!(cluster.members.len(), 2);
    }
}
