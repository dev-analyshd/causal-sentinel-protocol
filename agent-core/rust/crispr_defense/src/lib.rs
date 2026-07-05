use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// CRISPR Defense — Pre-execution attack interception (mempool layer)
///
/// Monitors the Casper mempool for attack patterns:
/// - Front-running sequences
/// - Sandwich attack bundles
/// - Replay attempts
/// - Credential nullifier reuse
/// - Anomalous gas price spikes
///
/// Intercepts attacks before they reach consensus.

pub struct CRISPRDefense {
    /// Known attack pattern signatures
    attack_signatures: RwLock<HashMap<String, AttackPattern>>,

    /// Seen nullifiers (prevent replay)
    seen_nullifiers: RwLock<HashSet<[u8; 32]>>,

    /// Mempool transaction cache
    mempool: RwLock<HashMap<[u8; 32], MempoolTx>>,

    /// Suspicious account tracking
    suspicious_accounts: RwLock<HashMap<String, u32>>,

    /// Defense statistics
    stats: RwLock<DefenseStats>,
}

#[derive(Debug, Clone)]
pub struct AttackPattern {
    pub name: String,
    pub signature_hash: [u8; 32],
    pub severity: AttackSeverity,
    pub detection_rules: Vec<DetectionRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum DetectionRule {
    GasPriceSpike { threshold: u64 },
    SequencePattern { pattern: Vec<String> },
    NullifierReuse,
    AccountAnomaly { max_tx_per_block: u32 },
    ValueDiscrepancy { max_ratio: f64 },
}

#[derive(Debug, Clone)]
pub struct MempoolTx {
    pub hash: [u8; 32],
    pub from: String,
    pub to: String,
    pub value: u64,
    pub gas_price: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub nullifier: Option<[u8; 32]>,
}

#[derive(Debug, Default, Clone)]
pub struct DefenseStats {
    pub total_scanned: u64,
    pub attacks_detected: u64,
    pub attacks_blocked: u64,
    pub false_positives: u64,
}

#[derive(Debug)]
pub struct DefenseResult {
    pub tx_hash: [u8; 32],
    pub allowed: bool,
    pub reason: Option<String>,
    pub severity: Option<AttackSeverity>,
}

impl CRISPRDefense {
    pub fn new() -> Self {
        let mut signatures = HashMap::new();

        // Initialize known attack patterns
        signatures.insert("front_run".to_string(), AttackPattern {
            name: "Front-Running".to_string(),
            signature_hash: [0u8; 32],
            severity: AttackSeverity::High,
            detection_rules: vec![
                DetectionRule::GasPriceSpike { threshold: 1000 },
                DetectionRule::SequencePattern {
                    pattern: vec!["watch".to_string(), "copy".to_string(), "execute".to_string()]
                },
            ],
        });

        signatures.insert("sandwich".to_string(), AttackPattern {
            name: "Sandwich Attack".to_string(),
            signature_hash: [0u8; 32],
            severity: AttackSeverity::Critical,
            detection_rules: vec![
                DetectionRule::SequencePattern {
                    pattern: vec!["buy".to_string(), "victim".to_string(), "sell".to_string()]
                },
                DetectionRule::ValueDiscrepancy { max_ratio: 0.05 },
            ],
        });

        signatures.insert("replay".to_string(), AttackPattern {
            name: "Replay Attack".to_string(),
            signature_hash: [0u8; 32],
            severity: AttackSeverity::Critical,
            detection_rules: vec![
                DetectionRule::NullifierReuse,
            ],
        });

        Self {
            attack_signatures: RwLock::new(signatures),
            seen_nullifiers: RwLock::new(HashSet::new()),
            mempool: RwLock::new(HashMap::new()),
            suspicious_accounts: RwLock::new(HashMap::new()),
            stats: RwLock::new(DefenseStats::default()),
        }
    }

    /// Scan a transaction before execution
    pub async fn scan(&self, tx: MempoolTx) -> DefenseResult {
        let mut stats = self.stats.write().await;
        stats.total_scanned += 1;
        drop(stats);

        // 1. Check nullifier reuse
        if let Some(nullifier) = tx.nullifier {
            let seen = self.seen_nullifiers.read().await;
            if seen.contains(&nullifier) {
                let mut stats = self.stats.write().await;
                stats.attacks_detected += 1;
                stats.attacks_blocked += 1;
                drop(stats);

                warn!("replay_attack_detected tx_hash={}", hex::encode(tx.hash));
                return DefenseResult {
                    tx_hash: tx.hash,
                    allowed: false,
                    reason: Some("Nullifier reuse detected (replay attack)".to_string()),
                    severity: Some(AttackSeverity::Critical),
                };
            }
        }

        // 2. Check gas price anomaly
        if self.is_gas_price_anomalous(&tx).await {
            warn!("gas_price_anomaly tx_hash={} gas_price={}", hex::encode(tx.hash), tx.gas_price);

            let mut suspicious = self.suspicious_accounts.write().await;
            let count = suspicious.entry(tx.from.clone()).or_insert(0);
            *count += 1;

            if *count >= 3 {
                let mut stats = self.stats.write().await;
                stats.attacks_detected += 1;
                drop(stats);

                return DefenseResult {
                    tx_hash: tx.hash,
                    allowed: false,
                    reason: Some("Repeated gas price anomalies".to_string()),
                    severity: Some(AttackSeverity::Medium),
                };
            }
        }

        // 3. Check sequence patterns
        if let Some(pattern) = self.detect_sequence_pattern(&tx).await {
            let mut stats = self.stats.write().await;
            stats.attacks_detected += 1;
            drop(stats);

            warn!("sequence_pattern_detected tx_hash={} pattern={}", hex::encode(tx.hash), pattern);

            return DefenseResult {
                tx_hash: tx.hash,
                allowed: false,
                reason: Some(format!("Sequence pattern detected: {}", pattern)),
                severity: Some(AttackSeverity::High),
            };
        }

        // 4. Store in mempool cache
        let tx_hash = tx.hash;
        let mut mempool = self.mempool.write().await;
        mempool.insert(tx.hash, tx);
        drop(mempool);

        DefenseResult {
            tx_hash,
            allowed: true,
            reason: None,
            severity: None,
        }
    }

    /// Record nullifier as spent
    pub async fn record_nullifier(&self, nullifier: [u8; 32]) {
        let mut seen = self.seen_nullifiers.write().await;
        seen.insert(nullifier);
        info!("nullifier_recorded nullifier={}", hex::encode(nullifier));
    }

    /// Get defense statistics
    pub async fn get_stats(&self) -> DefenseStats {
        self.stats.read().await.clone()
    }

    async fn is_gas_price_anomalous(&self, tx: &MempoolTx) -> bool {
        // In production: compare against rolling average
        // Mock: flag if gas price > 1000
        tx.gas_price > 1000
    }

    async fn detect_sequence_pattern(&self, _tx: &MempoolTx) -> Option<String> {
        // In production: analyze mempool sequences
        // Mock: no patterns detected
        None
    }
}

impl Default for CRISPRDefense {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute behavioral signature hash using dual SHA3-256
pub fn compute_signature(data: &[u8]) -> [u8; 32] {
    let primary = Sha3_256::digest(data);
    let secondary = Sha3_256::digest(&primary);
    secondary.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replay_attack_detection() {
        let defense = CRISPRDefense::new();
        let nullifier = [1u8; 32];

        // Record nullifier
        defense.record_nullifier(nullifier).await;

        // Try to use it again
        let tx = MempoolTx {
            hash: [2u8; 32],
            from: "attacker".to_string(),
            to: "victim".to_string(),
            value: 1000,
            gas_price: 100,
            nonce: 1,
            timestamp: 12345,
            nullifier: Some(nullifier),
        };

        let result = defense.scan(tx).await;
        assert!(!result.allowed);
        assert_eq!(result.severity, Some(AttackSeverity::Critical));
    }

    #[tokio::test]
    async fn test_clean_transaction_allowed() {
        let defense = CRISPRDefense::new();
        let tx = MempoolTx {
            hash: [3u8; 32],
            from: "honest_agent".to_string(),
            to: "recipient".to_string(),
            value: 500,
            gas_price: 10,    // Normal gas price
            nonce: 1,
            timestamp: 12345,
            nullifier: None,  // No nullifier
        };

        let result = defense.scan(tx).await;
        assert!(result.allowed);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_compute_signature_deterministic() {
        let data = b"behavioral_data_for_agent_001";
        let sig1 = compute_signature(data);
        let sig2 = compute_signature(data);
        assert_eq!(sig1, sig2);
        assert_ne!(sig1, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_defense_stats() {
        let defense = CRISPRDefense::new();
        let tx = MempoolTx {
            hash: [4u8; 32],
            from: "agent".to_string(),
            to: "target".to_string(),
            value: 100,
            gas_price: 10,
            nonce: 1,
            timestamp: 0,
            nullifier: None,
        };
        defense.scan(tx).await;
        let stats = defense.get_stats().await;
        assert_eq!(stats.total_scanned, 1);
    }
}
