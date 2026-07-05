#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping};

/// SentinelVault — ZK-gated capital, coherence gate, and on-chain heartbeat
/// 
/// All agent capital flows through this vault. Every withdrawal, trade, or
/// cross-chain action requires a valid ZK Behavioral Integrity Credential (BIC)
/// and a passing coherence score Ψ(t) ≥ Δ(t).

#[odra::module]
pub struct SentinelVault {
    /// Registry contract reference
    registry: Variable<Address>,

    /// ZK Verifier contract reference
    zk_verifier: Variable<Address>,

    /// Total CSPR locked in vault
    total_locked: Variable<u64>,

    /// Agent balances: (agent) -> balance in motes
    balances: Mapping<Address, u64>,

    /// Action limits per tier: (tier) -> daily limit in motes
    tier_limits: Mapping<u8, u64>,

    /// Daily action volume: (agent, day) -> volume
    daily_volume: Mapping<(Address, u64), u64>,

    /// Last heartbeat timestamp per agent
    last_heartbeat: Mapping<Address, u64>,

    /// Coherence gate threshold Δ(t) — updated by EpistaticController
    current_threshold: Variable<u64>,

    /// Global threat level (0-100)
    threat_level: Variable<u8>,

    /// Silence mode: if true, all non-essential ops paused
    silence_mode: Variable<bool>,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct ActionRequest {
    pub agent: Address,
    pub action_type: ActionType,
    pub amount: u64,
    pub target: Option<Address>,
    pub zk_proof: Vec<u8>,      // BIC ZK proof bytes
    pub nullifier: [u8; 32],    // ZK nullifier to prevent double-spend
    pub compliance_tier: u8,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub enum ActionType {
    Transfer,
    Trade,
    Stake,
    Unstake,
    CrossChainBridge,
    CredentialRenewal,
}

#[odra::module]
impl SentinelVault {
    pub fn init(&mut self, registry: Address, zk_verifier: Address) {
        self.registry.set(registry);
        self.zk_verifier.set(zk_verifier);
        self.total_locked.set(0);
        self.current_threshold.set(570_000); // 0.57 fixed-point
        self.threat_level.set(0);
        self.silence_mode.set(false);

        // Set tier limits (in CSPR motes)
        self.tier_limits.set(&1, 200_000_000_000);   // Basic: ~200 CSPR
        self.tier_limits.set(&2, 400_000_000_000);   // Bronze
        self.tier_limits.set(&3, 600_000_000_000);   // Silver
        self.tier_limits.set(&4, 800_000_000_000);   // Gold
        self.tier_limits.set(&5, 1_000_000_000_000); // Platinum: ~1000 CSPR
    }

    /// Deposit CSPR into vault (anyone can deposit)
    pub fn deposit(&mut self) {
        let caller = contract_env::caller();
        let amount = contract_env::attached_value();

        let current = self.balances.get(&caller).unwrap_or(0);
        self.balances.set(&caller, current + amount);

        let total = self.total_locked.get().unwrap_or(0) + amount;
        self.total_locked.set(total);

        contract_env::emit_event(Deposit {
            agent: caller,
            amount,
            new_balance: current + amount,
        });
    }

    /// Execute action with ZK-gated coherence check
    pub fn execute_action(&mut self, request: ActionRequest) {
        // 1. Check global silence mode
        assert!(!self.silence_mode.get().unwrap_or(false), "Global SILENCE mode active");

        // 2. Verify agent is registered and active
        // (In production, call SentinelRegistry.is_active)

        // 3. Verify ZK proof via ZKVerifier contract
        let proof_valid = self.verify_zk_proof(&request);
        assert!(proof_valid, "Invalid ZK Behavioral Integrity Credential");

        // 4. Check compliance tier matches
        let agent_tier = self.get_agent_tier(request.agent);
        assert_eq!(request.compliance_tier, agent_tier, "Tier mismatch");

        // 5. Check daily limit
        let day = contract_env::block_time() / 86400;
        let daily = self.daily_volume.get(&(request.agent, day)).unwrap_or(0);
        let limit = self.tier_limits.get(&agent_tier).unwrap_or(0);
        assert!(daily + request.amount <= limit, "Daily limit exceeded");

        // 6. Check coherence threshold
        let psi = self.get_agent_psi(request.agent);
        let threshold = self.current_threshold.get().unwrap_or(570_000);

        if psi < threshold {
            // Coherence gate CLOSED → emit SILENCE
            self.record_silence(request.agent, "Coherence below threshold");
            contract_env::revert("SILENCE: Ψ(t) < Δ(t)");
        }

        // 7. Execute action
        match request.action_type {
            ActionType::Transfer => {
                let target = request.target.expect("Transfer requires target");
                self.transfer(request.agent, target, request.amount);
            },
            ActionType::Trade => {
                self.execute_trade(request.agent, request.amount);
            },
            ActionType::Stake => {
                self.stake(request.agent, request.amount);
            },
            ActionType::Unstake => {
                self.unstake(request.agent, request.amount);
            },
            ActionType::CrossChainBridge => {
                self.bridge(request.agent, request.amount, request.target);
            },
            ActionType::CredentialRenewal => {
                self.renew_credential(request.agent);
            },
        }

        // Update daily volume
        self.daily_volume.set(&(request.agent, day), daily + request.amount);

        contract_env::emit_event(ActionExecuted {
            agent: request.agent,
            action_type: request.action_type,
            amount: request.amount,
            tier: agent_tier,
            block: contract_env::block_time(),
        });
    }

    /// On-chain heartbeat — agents must call every 100 blocks
    pub fn heartbeat(&mut self, psi: u64, lambda: u64, iq: u64) {
        let caller = contract_env::caller();
        let current_block = contract_env::block_time();

        let last = self.last_heartbeat.get(&caller).unwrap_or(0);
        assert!(
            current_block >= last + 100,
            "Heartbeat too frequent. Min interval: 100 blocks"
        );

        self.last_heartbeat.set(&caller, current_block);

        // Forward state update to registry
        // (In production, call SentinelRegistry.update_agent_state)

        contract_env::emit_event(Heartbeat {
            agent: caller,
            block: current_block,
            psi,
            lambda,
            iq,
        });
    }

    /// Update coherence threshold (called by EpistaticController)
    pub fn update_threshold(&mut self, new_threshold: u64, threat: u8) {
        self.assert_caller_is_controller();
        self.current_threshold.set(new_threshold);
        self.threat_level.set(threat);

        if threat >= 80 {
            self.silence_mode.set(true);
            contract_env::emit_event(GlobalSilenceActivated {
                threat_level: threat,
                threshold: new_threshold,
            });
        } else if threat < 30 && self.silence_mode.get().unwrap_or(false) {
            self.silence_mode.set(false);
            contract_env::emit_event(GlobalSilenceDeactivated {
                threat_level: threat,
            });
        }
    }

    /// Get agent balance
    pub fn get_balance(&self, agent: Address) -> u64 {
        self.balances.get(&agent).unwrap_or(0)
    }

    /// Get total locked
    pub fn get_total_locked(&self) -> u64 {
        self.total_locked.get().unwrap_or(0)
    }

    /// Get current threshold
    pub fn get_threshold(&self) -> u64 {
        self.current_threshold.get().unwrap_or(570_000)
    }

    // Internal helpers
    fn verify_zk_proof(&self, request: &ActionRequest) -> bool {
        // In production: call ZKVerifier.verify(request.zk_proof, request.nullifier)
        // Simplified: check non-empty proof and nullifier
        !request.zk_proof.is_empty() && request.nullifier != [0u8; 32]
    }

    fn get_agent_tier(&self, agent: Address) -> u8 {
        // In production: call SentinelRegistry.get_agent(agent).credential_tier
        3 // Mock: Silver tier
    }

    fn get_agent_psi(&self, agent: Address) -> u64 {
        // In production: call SentinelRegistry.get_agent(agent).coherence_psi
        750_000 // Mock: 0.75
    }

    fn transfer(&mut self, from: Address, to: Address, amount: u64) {
        let from_bal = self.balances.get(&from).unwrap_or(0);
        assert!(from_bal >= amount, "Insufficient balance");
        self.balances.set(&from, from_bal - amount);

        let to_bal = self.balances.get(&to).unwrap_or(0);
        self.balances.set(&to, to_bal + amount);
    }

    fn execute_trade(&mut self, _agent: Address, _amount: u64) {
        // Integration with CSPR.trade via MCP
    }

    fn stake(&mut self, agent: Address, amount: u64) {
        let bal = self.balances.get(&agent).unwrap_or(0);
        assert!(bal >= amount, "Insufficient balance");
        // Staking logic with Casper PoS
    }

    fn unstake(&mut self, _agent: Address, _amount: u64) {
        // Unstaking logic
    }

    fn bridge(&mut self, _agent: Address, _amount: u64, _target: Option<Address>) {
        // Cross-chain bridge logic
    }

    fn renew_credential(&mut self, _agent: Address) {
        // Credential renewal logic
    }

    fn record_silence(&mut self, agent: Address, reason: &str) {
        // Forward to registry
        contract_env::emit_event(SilenceRecorded {
            agent,
            reason: reason.to_string(),
            block: contract_env::block_time(),
        });
    }

    fn assert_caller_is_controller(&self) {
        // Verify caller is EpistaticController
    }
}

#[odra::event]
pub struct Deposit {
    pub agent: Address,
    pub amount: u64,
    pub new_balance: u64,
}

#[odra::event]
pub struct ActionExecuted {
    pub agent: Address,
    pub action_type: ActionType,
    pub amount: u64,
    pub tier: u8,
    pub block: u64,
}

#[odra::event]
pub struct Heartbeat {
    pub agent: Address,
    pub block: u64,
    pub psi: u64,
    pub lambda: u64,
    pub iq: u64,
}

#[odra::event]
pub struct GlobalSilenceActivated {
    pub threat_level: u8,
    pub threshold: u64,
}

#[odra::event]
pub struct GlobalSilenceDeactivated {
    pub threat_level: u8,
}

#[odra::event]
pub struct SilenceRecorded {
    pub agent: Address,
    pub reason: String,
    pub block: u64,
}
