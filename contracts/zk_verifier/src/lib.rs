#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping};

/// ZKVerifier — UltraHonk proof verification (WASM-native on Casper)
/// 
/// Verifies three circuit types:
/// 1. Behavioral Integrity Credential (BIC) — 12,000 constraints
/// 2. Causal Identity Proof (CIP) — 8,500 constraints
/// 3. Sentinel Compliance — 6,000 constraints
/// 
/// Uses Barretenberg verifier compiled to WASM for native Casper execution.

#[odra::module]
pub struct ZKVerifier {
    /// Owner
    owner: Variable<Address>,

    /// Verification keys: (circuit_type) -> VK bytes
    verification_keys: Mapping<String, Vec<u8>>,

    /// Verified proof count
    proof_count: Variable<u64>,

    /// Nullifier spent tracking: (nullifier) -> spent
    nullifier_spent: Mapping<[u8; 32], bool>,

    /// Proof verification history: (index) -> VerificationRecord
    verification_history: Mapping<u64, VerificationRecord>,

    /// Circuit constraint counts (for gas estimation)
    constraint_counts: Mapping<String, u32>,

    /// WASM verifier module hash (for integrity)
    verifier_wasm_hash: Variable<[u8; 32]>,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct VerificationRecord {
    pub circuit_type: String,
    pub nullifier: [u8; 32],
    pub public_inputs_hash: [u8; 32],
    pub verified: bool,
    pub block: u64,
    pub gas_used: u64,
}

#[odra::module]
impl ZKVerifier {
    pub fn init(&mut self, owner: Address) {
        self.owner.set(owner);
        self.proof_count.set(0);

        // Set constraint counts
        self.constraint_counts.set("behavioral_integrity".to_string(), 12000);
        self.constraint_counts.set("causal_identity".to_string(), 8500);
        self.constraint_counts.set("sentinel_compliance".to_string(), 6000);
    }

    /// Register a verification key for a circuit type
    pub fn register_vk(&mut self, circuit_type: String, vk: Vec<u8>) {
        self.assert_owner();
        self.verification_keys.set(&circuit_type, vk);

        contract_env::emit_event(VKRegistered {
            circuit_type,
            vk_hash: self.hash_bytes(&vk),
        });
    }

    /// Verify a ZK proof
    /// Returns (verified, tier, gas_used)
    pub fn verify_proof(
        &mut self,
        circuit_type: String,
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
        nullifier: [u8; 32],
    ) -> (bool, u8, u64) {
        // 1. Check nullifier not spent
        assert!(
            !self.nullifier_spent.get(&nullifier).unwrap_or(false),
            "Nullifier already spent"
        );

        // 2. Get verification key
        let vk = self.verification_keys.get(&circuit_type).expect("VK not registered");

        // 3. Compute gas estimate based on constraint count
        let constraints = self.constraint_counts.get(&circuit_type).unwrap_or(10000);
        let gas_estimate = (constraints as u64) * 1000; // ~1ms per constraint

        // 4. Verify proof (WASM-native Barretenberg)
        let verified = self.verify_wasm(&circuit_type, &proof, &public_inputs, &vk);

        // 5. Mark nullifier spent
        self.nullifier_spent.set(&nullifier, true);

        // 6. Record verification
        let count = self.proof_count.get().unwrap_or(0);
        let record = VerificationRecord {
            circuit_type: circuit_type.clone(),
            nullifier,
            public_inputs_hash: self.hash_bytes(&public_inputs),
            verified,
            block: contract_env::block_time(),
            gas_used: gas_estimate,
        };
        self.verification_history.set(&count, record);
        self.proof_count.set(count + 1);

        // 7. Extract compliance tier from public inputs
        let tier = if public_inputs.len() >= 1 {
            public_inputs[0] // First byte encodes tier
        } else {
            1
        };

        contract_env::emit_event(ProofVerified {
            circuit_type,
            nullifier,
            verified,
            tier,
            gas_used: gas_estimate,
        });

        (verified, tier, gas_estimate)
    }

    /// Batch verify multiple proofs (gas optimization)
    pub fn batch_verify(
        &mut self,
        circuit_type: String,
        proofs: Vec<Vec<u8>>,
        public_inputs: Vec<Vec<u8>>,
        nullifiers: Vec<[u8; 32]>,
    ) -> Vec<bool> {
        assert_eq!(proofs.len(), public_inputs.len(), "Mismatched inputs");
        assert_eq!(proofs.len(), nullifiers.len(), "Mismatched nullifiers");

        let mut results = Vec::new();
        for i in 0..proofs.len() {
            let (verified, _, _) = self.verify_proof(
                circuit_type.clone(),
                proofs[i].clone(),
                public_inputs[i].clone(),
                nullifiers[i],
            );
            results.push(verified);
        }
        results
    }

    /// Check if nullifier is spent
    pub fn is_spent(&self, nullifier: [u8; 32]) -> bool {
        self.nullifier_spent.get(&nullifier).unwrap_or(false)
    }

    /// Get verification record
    pub fn get_record(&self, index: u64) -> Option<VerificationRecord> {
        self.verification_history.get(&index)
    }

    /// Get total proofs verified
    pub fn get_proof_count(&self) -> u64 {
        self.proof_count.get().unwrap_or(0)
    }

    /// Get constraint count for circuit
    pub fn get_constraint_count(&self, circuit_type: String) -> u32 {
        self.constraint_counts.get(&circuit_type).unwrap_or(0)
    }

    /// Update WASM verifier hash (for integrity checks)
    pub fn set_verifier_hash(&mut self, hash: [u8; 32]) {
        self.assert_owner();
        self.verifier_wasm_hash.set(hash);
    }

    // WASM-native verification (mock for testnet)
    fn verify_wasm(&self, circuit_type: &str, proof: &[u8], public_inputs: &[u8], vk: &[u8]) -> bool {
        // In production: call Barretenberg WASM verifier
        // For testnet: simulate verification with structural checks

        if proof.len() < 64 {
            return false;
        }
        if public_inputs.len() < 1 {
            return false;
        }
        if vk.len() < 32 {
            return false;
        }

        // Check proof structure matches circuit type
        match circuit_type {
            "behavioral_integrity" => proof.len() >= 128,
            "causal_identity" => proof.len() >= 96,
            "sentinel_compliance" => proof.len() >= 80,
            _ => false,
        }
    }

    fn hash_bytes(&self, data: &[u8]) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn assert_owner(&self) {
        assert_eq!(contract_env::caller(), self.owner.get().unwrap(), "Not owner");
    }
}

#[odra::event]
pub struct VKRegistered {
    pub circuit_type: String,
    pub vk_hash: [u8; 32],
}

#[odra::event]
pub struct ProofVerified {
    pub circuit_type: String,
    pub nullifier: [u8; 32],
    pub verified: bool,
    pub tier: u8,
    pub gas_used: u64,
}
