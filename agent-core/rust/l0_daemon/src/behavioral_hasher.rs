use sha3::{Digest, Sha3_256};
use tracing::{debug, info};
use std::collections::VecDeque;

/// BehavioralHasher: Computes dual SHA3-256 behavioral hashes
/// 
/// Primary hash: SHA3-256(event_data) → behavioral vector component
/// Secondary hash: SHA3-256(SHA3-256(event_data) + timestamp) → temporal anchor
/// 
/// The dual hash creates a non-reversible but verifiable behavioral
/// fingerprint that feeds into the coherence engine's perceptual
/// entropy calculation.

#[derive(Debug)]
pub struct BehavioralHasher {
    primary_history: VecDeque<[u8; 32]>,
    secondary_history: VecDeque<[u8; 32]>,
    behavioral_vector: Vec<f64>,
    total_events: u64,
    last_block: u64,
    entropy_accumulator: f64,
}

impl BehavioralHasher {
    pub fn new() -> Self {
        Self {
            primary_history: VecDeque::with_capacity(1000),
            secondary_history: VecDeque::with_capacity(1000),
            behavioral_vector: vec![0.0; 128],
            total_events: 0,
            last_block: 0,
            entropy_accumulator: 0.0,
        }
    }

    pub fn process_deploy(&mut self, deploy_hash: &[u8; 32], account: &str, timestamp: u64) {
        let mut data = deploy_hash.to_vec();
        data.extend_from_slice(account.as_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());

        let (primary, secondary) = self.compute_dual_hash(&data);
        self.store_hashes(primary, secondary);
        self.update_vector(&primary, &secondary);

        self.total_events += 1;
        debug!("Processed deploy hash for account {}", account);
    }

    pub fn process_block(&mut self, block_hash: &[u8; 32], height: u64, era_id: u64, timestamp: u64) {
        let mut data = block_hash.to_vec();
        data.extend_from_slice(&height.to_le_bytes());
        data.extend_from_slice(&era_id.to_le_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());

        let (primary, secondary) = self.compute_dual_hash(&data);
        self.store_hashes(primary, secondary);
        self.update_vector(&primary, &secondary);

        self.last_block = height;

        // Compute perceptual entropy from block spacing
        if self.total_events > 0 {
            let block_entropy = self.compute_block_entropy(height, timestamp);
            self.entropy_accumulator += block_entropy;
        }

        self.total_events += 1;
    }

    pub fn process_transfer(&mut self, from: &str, to: &str, amount: u64, deploy_hash: &[u8; 32]) {
        let mut data = from.as_bytes().to_vec();
        data.extend_from_slice(to.as_bytes());
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(deploy_hash);

        let (primary, secondary) = self.compute_dual_hash(&data);
        self.store_hashes(primary, secondary);
        self.update_vector(&primary, &secondary);

        self.total_events += 1;
    }

    pub fn process_consensus(&mut self, public_key: &str, era_id: u64, message_type: &str) {
        let mut data = public_key.as_bytes().to_vec();
        data.extend_from_slice(&era_id.to_le_bytes());
        data.extend_from_slice(message_type.as_bytes());

        let (primary, secondary) = self.compute_dual_hash(&data);
        self.store_hashes(primary, secondary);
        self.update_vector(&primary, &secondary);

        self.total_events += 1;
    }

    pub fn process_step(&mut self, era_id: u64, effects: &[String]) {
        let mut data = era_id.to_le_bytes().to_vec();
        for effect in effects {
            data.extend_from_slice(effect.as_bytes());
        }

        let (primary, secondary) = self.compute_dual_hash(&data);
        self.store_hashes(primary, secondary);
        self.update_vector(&primary, &secondary);

        self.total_events += 1;
    }

    pub fn get_behavioral_vector(&self) -> &[f64] {
        &self.behavioral_vector
    }

    pub fn get_perceptual_entropy(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            self.entropy_accumulator / self.total_events as f64
        }
    }

    pub fn get_total_events(&self) -> u64 {
        self.total_events
    }

    pub fn get_last_block(&self) -> u64 {
        self.last_block
    }

    fn compute_dual_hash(&self, data: &[u8]) -> ([u8; 32], [u8; 32]) {
        // Primary: SHA3-256(data)
        let primary = {
            let mut hasher = Sha3_256::new();
            hasher.update(data);
            hasher.finalize().into()
        };

        // Secondary: SHA3-256(SHA3-256(data) || timestamp_nonce)
        let secondary = {
            let mut hasher = Sha3_256::new();
            hasher.update(&primary);
            hasher.update(&self.total_events.to_le_bytes());
            hasher.finalize().into()
        };

        (primary, secondary)
    }

    fn store_hashes(&mut self, primary: [u8; 32], secondary: [u8; 32]) {
        self.primary_history.push_back(primary);
        self.secondary_history.push_back(secondary);

        if self.primary_history.len() > 1000 {
            self.primary_history.pop_front();
            self.secondary_history.pop_front();
        }
    }

    fn update_vector(&mut self, primary: &[u8; 32], secondary: &[u8; 32]) {
        // XOR-fold 32-byte hashes into 128-dim vector (4 bytes per dimension)
        for i in 0..32 {
            let dim = i % 128;
            let val = (primary[i] ^ secondary[i]) as f64 / 255.0;
            self.behavioral_vector[dim] = self.behavioral_vector[dim] * 0.9 + val * 0.1;
        }
    }

    fn compute_block_entropy(&self, height: u64, timestamp: u64) -> f64 {
        // Simplified entropy: inverse of block time regularity
        let expected_interval = 8.0; // 8 seconds
        let actual_interval = if self.last_block > 0 {
            (timestamp - self.last_block * 8) as f64 / 1000.0
        } else {
            expected_interval
        };

        let deviation = (actual_interval - expected_interval).abs();
        let entropy = 1.0 - (-deviation / expected_interval).exp();

        entropy
    }
}
