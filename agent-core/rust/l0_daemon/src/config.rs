use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub node_url: String,
    pub websocket_port: u16,
    pub event_buffer_size: usize,
    pub reconnect_interval_secs: u64,
    pub batch_size: usize,
    pub sentinel_contracts: SentinelContracts,
    pub hashing: HashingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelContracts {
    pub registry: String,
    pub vault: String,
    pub learner: String,
    pub compliance: String,
    pub epistatic: String,
    pub zk_verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashingConfig {
    pub dual_hash_enabled: bool,
    pub primary_algorithm: String, // "sha3_256"
    pub secondary_algorithm: String, // "sha3_256_double"
    pub behavioral_vector_dim: usize,
    pub history_window_blocks: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            node_url: "http://localhost:7777".to_string(),
            websocket_port: 9001,
            event_buffer_size: 10000,
            reconnect_interval_secs: 5,
            batch_size: 100,
            sentinel_contracts: SentinelContracts {
                registry: "hash-...a3f2".to_string(),
                vault: "hash-...b8c1".to_string(),
                learner: "hash-...d4e5".to_string(),
                compliance: "hash-...f6a7".to_string(),
                epistatic: "hash-...g8h9".to_string(),
                zk_verifier: "hash-...i0j1".to_string(),
            },
            hashing: HashingConfig {
                dual_hash_enabled: true,
                primary_algorithm: "sha3_256".to_string(),
                secondary_algorithm: "sha3_256_double".to_string(),
                behavioral_vector_dim: 128,
                history_window_blocks: 100,
            },
        }
    }
}

impl DaemonConfig {
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path).await?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
