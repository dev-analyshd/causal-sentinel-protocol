use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use tracing::{info, warn};

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
///
/// # Nullifier lifecycle
/// Every transaction that passes `scan()` and carries a nullifier MUST have
/// `record_nullifier()` called in the same logical commit path.  `scan()`
/// handles this automatically for accepted transactions so that no external
/// caller needs to remember the extra step.

pub struct CRISPRDefense {
    /// Seen nullifiers (prevent replay)
    seen_nullifiers: RwLock<HashSet<[u8; 32]>>,

    /// Mempool transaction cache
    mempool: RwLock<HashMap<[u8; 32], MempoolTx>>,

    /// Suspicious account tracking: account → anomaly count
    suspicious_accounts: RwLock<HashMap<String, u32>>,

    /// Defense statistics
    stats: RwLock<DefenseStats>,
}

#[derive(Debug, Clone)]
pub struct AttackPattern {
    pub name: String,
    pub severity: AttackSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttackSeverity {
    Low,
    Medium,
    High,
    Critical,
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
        Self {
            seen_nullifiers: RwLock::new(HashSet::new()),
            mempool: RwLock::new(HashMap::new()),
            suspicious_accounts: RwLock::new(HashMap::new()),
            stats: RwLock::new(DefenseStats::default()),
        }
    }

    /// Scan a transaction before execution.
    ///
    /// Accepted transactions whose nullifier is `Some(n)` have `n` automatically
    /// added to `seen_nullifiers` in the same method — no external call needed.
    pub async fn scan(&self, tx: MempoolTx) -> DefenseResult {
        {
            let mut stats = self.stats.write().await;
            stats.total_scanned += 1;
        }

        // ─── 1. Nullifier replay check ────────────────────────────────────
        if let Some(nullifier) = tx.nullifier {
            let seen = self.seen_nullifiers.read().await;
            if seen.contains(&nullifier) {
                drop(seen);
                let mut stats = self.stats.write().await;
                stats.attacks_detected += 1;
                stats.attacks_blocked += 1;
                drop(stats);

                warn!(
                    tx_hash = %hex::encode(tx.hash),
                    nullifier = %hex::encode(nullifier),
                    "replay_attack_detected"
                );
                return DefenseResult {
                    tx_hash: tx.hash,
                    allowed: false,
                    reason: Some("Nullifier reuse detected (replay attack)".to_string()),
                    severity: Some(AttackSeverity::Critical),
                };
            }
        }

        // ─── 2. Gas price anomaly ────────────────────────────────────────
        if self.is_gas_price_anomalous(&tx).await {
            warn!(
                tx_hash = %hex::encode(tx.hash),
                gas_price = tx.gas_price,
                "gas_price_anomaly"
            );
            let mut suspicious = self.suspicious_accounts.write().await;
            let count = suspicious.entry(tx.from.clone()).or_insert(0);
            *count += 1;
            if *count >= 3 {
                drop(suspicious);
                let mut stats = self.stats.write().await;
                stats.attacks_detected += 1;
                stats.attacks_blocked += 1;
                drop(stats);
                return DefenseResult {
                    tx_hash: tx.hash,
                    allowed: false,
                    reason: Some("Repeated gas price anomalies — account flagged".to_string()),
                    severity: Some(AttackSeverity::Medium),
                };
            }
        }

        // ─── 3. Sequence pattern detection ──────────────────────────────
        if let Some(pattern) = self.detect_sequence_pattern(&tx).await {
            let mut stats = self.stats.write().await;
            stats.attacks_detected += 1;
            stats.attacks_blocked += 1;
            drop(stats);
            warn!(
                tx_hash = %hex::encode(tx.hash),
                pattern = %pattern,
                "sequence_pattern_detected"
            );
            return DefenseResult {
                tx_hash: tx.hash,
                allowed: false,
                reason: Some(format!("Sequence pattern detected: {}", pattern)),
                severity: Some(AttackSeverity::High),
            };
        }

        // ─── 4. Accept: record nullifier (same path, cannot be skipped) ──
        let tx_hash = tx.hash;
        if let Some(nullifier) = tx.nullifier {
            let mut seen = self.seen_nullifiers.write().await;
            seen.insert(nullifier);
            info!(nullifier = %hex::encode(nullifier), "nullifier_recorded");
        }

        let mut mempool = self.mempool.write().await;
        mempool.insert(tx_hash, tx);

        DefenseResult {
            tx_hash,
            allowed: true,
            reason: None,
            severity: None,
        }
    }

    /// Manually record a nullifier (e.g. after on-chain confirmation).
    /// `scan()` already calls this for every accepted tx, so external callers
    /// only need this when processing confirmed chain events directly.
    pub async fn record_nullifier(&self, nullifier: [u8; 32]) {
        let mut seen = self.seen_nullifiers.write().await;
        seen.insert(nullifier);
        info!(nullifier = %hex::encode(nullifier), "nullifier_recorded_externally");
    }

    pub async fn get_stats(&self) -> DefenseStats {
        self.stats.read().await.clone()
    }

    async fn is_gas_price_anomalous(&self, tx: &MempoolTx) -> bool {
        // Production: compare against rolling 10-block average + 3σ
        // MVP: flag if gas_price > hard ceiling
        tx.gas_price > 1000
    }

    async fn detect_sequence_pattern(&self, _tx: &MempoolTx) -> Option<String> {
        // Production: analyse surrounding mempool for sandwich/front-run windows
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
        defense.record_nullifier(nullifier).await;

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
    async fn test_accepted_nullifier_recorded_automatically() {
        let defense = CRISPRDefense::new();
        let nullifier = [5u8; 32];

        // First scan: accepted, nullifier recorded inside scan()
        let tx1 = MempoolTx {
            hash: [10u8; 32],
            from: "honest".to_string(),
            to: "target".to_string(),
            value: 100,
            gas_price: 10,
            nonce: 1,
            timestamp: 1,
            nullifier: Some(nullifier),
        };
        let r1 = defense.scan(tx1).await;
        assert!(r1.allowed, "First use of nullifier should be allowed");

        // Second scan of same nullifier: must be rejected (recorded by scan above)
        let tx2 = MempoolTx {
            hash: [11u8; 32],
            from: "attacker".to_string(),
            to: "target".to_string(),
            value: 100,
            gas_price: 10,
            nonce: 2,
            timestamp: 2,
            nullifier: Some(nullifier),
        };
        let r2 = defense.scan(tx2).await;
        assert!(!r2.allowed, "Replay of same nullifier must be blocked");
        assert_eq!(r2.severity, Some(AttackSeverity::Critical));
    }

    #[tokio::test]
    async fn test_clean_transaction_allowed() {
        let defense = CRISPRDefense::new();
        let tx = MempoolTx {
            hash: [3u8; 32],
            from: "honest_agent".to_string(),
            to: "recipient".to_string(),
            value: 500,
            gas_price: 10,
            nonce: 1,
            timestamp: 12345,
            nullifier: None,
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
        assert_eq!(stats.attacks_detected, 0);
    }
}
