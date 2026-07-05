#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping, List};

/// SentinelRegistry — Agent identity, Λ state, and credential lifecycle
/// 
/// Every agent must register here before entering the machine economy.
/// The registry maintains the compounding moat Λ(t) and the behavioral
/// history commitment that anchors causal identity recovery.

#[odra::module]
pub struct SentinelRegistry {
    /// Contract owner (sentinel council multi-sig)
    owner: Variable<Address>,

    /// Minimum stake required for agent registration (in CSPR motes)
    min_stake: Variable<u64>,

    /// Registered agent count
    agent_count: Variable<u64>,

    /// Agent metadata: (address) -> AgentProfile
    agents: Mapping<Address, AgentProfile>,

    /// Behavioral history commitment: (address) -> SHA3-256 hash
    behavioral_commitments: Mapping<Address, [u8; 32]>,

    /// Λ (moat) score: (address) -> f64 encoded as u64 (fixed-point 1e6)
    moat_scores: Mapping<Address, u64>,

    /// Credential tier: (address) -> u8 (1-5)
    credential_tiers: Mapping<Address, u8>,

    /// Registration block height
    registration_blocks: Mapping<Address, u64>,

    /// Active agent list (for iteration)
    active_agents: List<Address>,

    /// Suspended agents (SILENCE mode)
    suspended_agents: Mapping<Address, bool>,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct AgentProfile {
    pub identity: Address,
    pub registration_block: u64,
    pub last_heartbeat_block: u64,
    pub moat_lambda: u64,        // Fixed-point: actual = lambda / 1_000_000
    pub coherence_psi: u64,        // Fixed-point: actual = psi / 1_000_000
    pub iq_score: u64,           // Fixed-point
    pub credential_tier: u8,       // 1=Basic, 5=Platinum
    pub total_actions: u64,
    pub silence_events: u64,
    pub manipulation_count: u8,
    pub is_active: bool,
    pub dna_code_hash: [u8; 32], // User-defined change schedule commitment
}

#[odra::module]
impl SentinelRegistry {
    /// Initialize the registry with owner and minimum stake
    pub fn init(&mut self, owner: Address, min_stake: u64) {
        self.owner.set(owner);
        self.min_stake.set(min_stake);
        self.agent_count.set(0);
    }

    /// Register a new sentinel agent
    /// Requires: minimum CSPR stake, unique identity, valid DNA code commitment
    pub fn register_agent(
        &mut self,
        agent_address: Address,
        dna_code_hash: [u8; 32],
        behavioral_commitment: [u8; 32],
    ) {
        let caller = contract_env::caller();
        let stake = contract_env::attached_value();

        assert!(
            stake >= self.min_stake.get().unwrap_or(0),
            "Insufficient stake. Minimum: {}",
            self.min_stake.get().unwrap_or(0)
        );

        assert!(
            !self.agents.get(&agent_address).is_some(),
            "Agent already registered"
        );

        let current_block = contract_env::block_time();

        let profile = AgentProfile {
            identity: agent_address,
            registration_block: current_block,
            last_heartbeat_block: current_block,
            moat_lambda: 0,          // Λ starts at 0
            coherence_psi: 0,
            iq_score: 0,
            credential_tier: 1,      // Basic tier
            total_actions: 0,
            silence_events: 0,
            manipulation_count: 0,
            is_active: true,
            dna_code_hash,
        };

        self.agents.set(&agent_address, profile);
        self.behavioral_commitments.set(&agent_address, behavioral_commitment);
        self.moat_scores.set(&agent_address, 0);
        self.credential_tiers.set(&agent_address, 1);
        self.registration_blocks.set(&agent_address, current_block);
        self.active_agents.push(agent_address);
        self.suspended_agents.set(&agent_address, false);

        let count = self.agent_count.get().unwrap_or(0) + 1;
        self.agent_count.set(count);

        contract_env::emit_event(AgentRegistered {
            agent: agent_address,
            block: current_block,
            stake,
            dna_code_hash,
        });
    }

    /// Update agent state after coherence evaluation (called by SentinelVault)
    pub fn update_agent_state(
        &mut self,
        agent: Address,
        psi: u64,
        lambda: u64,
        iq: u64,
        manipulation_detected: bool,
    ) {
        self.assert_caller_is_vault();

        let mut profile = self.agents.get(&agent).expect("Agent not found");

        profile.coherence_psi = psi;
        profile.moat_lambda = lambda;
        profile.iq_score = iq;
        profile.last_heartbeat_block = contract_env::block_time();
        profile.total_actions += 1;

        if manipulation_detected {
            profile.manipulation_count += 1;
            if profile.manipulation_count >= 4 {
                profile.is_active = false;
                self.suspended_agents.set(&agent, true);
                contract_env::emit_event(AgentSuspended {
                    agent,
                    reason: SuspensionReason::ManipulationLimitExceeded,
                });
            }
        }

        // Update credential tier based on Λ and history
        let new_tier = self.compute_tier(lambda, profile.manipulation_count, profile.registration_block);
        profile.credential_tier = new_tier;
        self.credential_tiers.set(&agent, new_tier);

        self.agents.set(&agent, profile);

        contract_env::emit_event(AgentStateUpdated {
            agent,
            psi,
            lambda,
            tier: new_tier,
        });
    }

    /// Record a SILENCE event (coherence gate closed)
    pub fn record_silence(&mut self, agent: Address, reason: String) {
        self.assert_caller_is_vault();

        let mut profile = self.agents.get(&agent).expect("Agent not found");
        profile.silence_events += 1;
        self.agents.set(&agent, profile);

        contract_env::emit_event(SilenceEvent {
            agent,
            block: contract_env::block_time(),
            reason,
        });
    }

    /// Causal Identity Recovery: prove behavioral history match
    pub fn recover_identity(
        &mut self,
        new_address: Address,
        old_address: Address,
        causal_proof_hash: [u8; 32],
        temporal_challenge: u64,
    ) {
        let old_profile = self.agents.get(&old_address).expect("Old identity not found");

        // Verify old identity is inactive (lost/compromised)
        assert!(!old_profile.is_active, "Old identity still active");

        // Verify causal proof against stored commitment
        let stored_commitment = self.behavioral_commitments.get(&old_address).expect("No commitment");
        let challenge_hash = self.hash_causal_challenge(&old_address, &new_address, temporal_challenge);

        assert_eq!(causal_proof_hash, challenge_hash, "Invalid causal proof");

        // Transfer Λ and history to new address
        let mut new_profile = old_profile.clone();
        new_profile.identity = new_address;
        new_profile.is_active = true;

        self.agents.set(&new_address, new_profile);
        self.behavioral_commitments.set(&new_address, stored_commitment);
        self.moat_scores.set(&new_address, old_profile.moat_lambda);
        self.credential_tiers.set(&new_address, old_profile.credential_tier);
        self.active_agents.push(new_address);

        contract_env::emit_event(IdentityRecovered {
            old_address,
            new_address,
            lambda_transferred: old_profile.moat_lambda,
        });
    }

    /// Get agent profile
    pub fn get_agent(&self, agent: Address) -> Option<AgentProfile> {
        self.agents.get(&agent)
    }

    /// Get total registered agents
    pub fn get_agent_count(&self) -> u64 {
        self.agent_count.get().unwrap_or(0)
    }

    /// Check if agent is active
    pub fn is_active(&self, agent: Address) -> bool {
        self.agents.get(&agent).map(|p| p.is_active).unwrap_or(false)
    }

    /// Get moat score
    pub fn get_moat(&self, agent: Address) -> u64 {
        self.moat_scores.get(&agent).unwrap_or(0)
    }

    /// Compute credential tier based on Λ, manipulation count, and age
    fn compute_tier(&self, lambda: u64, manipulations: u8, registration_block: u64) -> u8 {
        let age_blocks = contract_env::block_time() - registration_block;
        let age_months = age_blocks / (30 * 24 * 60 * 60 / 1000); // Approximate

        match (lambda, manipulations, age_months) {
            (l, 0, a) if l >= 2_000_000 && a >= 12 => 5, // Platinum
            (l, 0, a) if l >= 1_500_000 && a >= 9  => 4, // Gold
            (l, m, a) if l >= 1_000_000 && m <= 1 && a >= 6 => 3, // Silver
            (l, m, a) if l >= 500_000 && m <= 2 && a >= 3 => 2,  // Bronze
            _ => 1, // Basic
        }
    }

    fn hash_causal_challenge(&self, old: &Address, new: &Address, challenge: u64) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(old.as_bytes());
        hasher.update(new.as_bytes());
        hasher.update(&challenge.to_le_bytes());
        hasher.finalize().into()
    }

    fn assert_caller_is_vault(&self) {
        // In production, this checks against SentinelVault contract hash
        // Simplified for demo
    }
}

// Events
#[odra::event]
pub struct AgentRegistered {
    pub agent: Address,
    pub block: u64,
    pub stake: u64,
    pub dna_code_hash: [u8; 32],
}

#[odra::event]
pub struct AgentStateUpdated {
    pub agent: Address,
    pub psi: u64,
    pub lambda: u64,
    pub tier: u8,
}

#[odra::event]
pub struct SilenceEvent {
    pub agent: Address,
    pub block: u64,
    pub reason: String,
}

#[odra::event]
pub struct IdentityRecovered {
    pub old_address: Address,
    pub new_address: Address,
    pub lambda_transferred: u64,
}

#[odra::event]
pub struct AgentSuspended {
    pub agent: Address,
    pub reason: SuspensionReason,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub enum SuspensionReason {
    ManipulationLimitExceeded,
    CoherenceViolation,
    GovernanceVote,
}
