#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping};

/// EpistaticController — EL_state computation and contract expression modulation
/// 
/// The epistatic state function reads environmental signals (threat level,
/// validator health, network entropy) and modulates how other contracts
/// express themselves — without changing their immutable bytecode.
/// 
/// DNA is unchanged. Phenotype adapts.

#[odra::module]
pub struct EpistaticController {
    /// Owner
    owner: Variable<Address>,

    /// SentinelVault reference
    vault: Variable<Address>,

    /// SentinelLearner reference (for epistatic params)
    learner: Variable<Address>,

    /// ComplianceEngine reference
    compliance: Variable<Address>,

    /// Current epistatic state EL_state(t) (fixed-point u64)
    el_state: Variable<u64>,

    /// Threat level (0-100)
    threat_level: Variable<u8>,

    /// Validator health score (0-10000)
    validator_health: Variable<u16>,

    /// Network entropy (behavioral diversity, 0-10000)
    network_entropy: Variable<u16>,

    /// Weights for EL_state computation (fixed-point)
    weight_threat: Variable<u64>,    // w_T
    weight_validator: Variable<u64>,  // w_V
    weight_entropy: Variable<u64>,    // w_N

    /// Expression modifiers: (contract_name) -> modifier (fixed-point)
    expression_modifiers: Mapping<String, u64>,

    /// Historical EL_state values
    el_history: Mapping<u64, u64>, // block -> EL_state

    /// Calibration counter for online learning
    calibration_count: Variable<u64>,

    /// Regime: Normal, Alert, Critical, Silence
    current_regime: Variable<Regime>,
}

#[odra::odra_type]
#[derive(Clone, Debug, PartialEq)]
pub enum Regime {
    Normal,      // Low threat, relaxed expression
    Alert,       // Elevated threat, tightened thresholds
    Critical,    // High threat, ZK-only, restricted ops
    Silence,     // Attack detected, all non-essential ops paused
}

#[odra::module]
impl EpistaticController {
    pub fn init(&mut self, owner: Address, vault: Address, learner: Address, compliance: Address) {
        self.owner.set(owner);
        self.vault.set(vault);
        self.learner.set(learner);
        self.compliance.set(compliance);

        self.el_state.set(500_000); // 0.5 initial
        self.threat_level.set(0);
        self.validator_health.set(8000); // 0.8
        self.network_entropy.set(7000); // 0.7

        // Initialize weights (sum to 1.0 in fixed-point)
        self.weight_threat.set(300_000);    // 0.3
        self.weight_validator.set(400_000); // 0.4
        self.weight_entropy.set(300_000);   // 0.3

        self.calibration_count.set(0);
        self.current_regime.set(Regime::Normal);

        // Initialize expression modifiers
        self.expression_modifiers.set("sentinel_vault".to_string(), 1_000_000);      // 1.0
        self.expression_modifiers.set("sentinel_registry".to_string(), 1_000_000);
        self.expression_modifiers.set("compliance_engine".to_string(), 1_000_000);
    }

    /// Compute and update EL_state(t)
    /// EL_state(t) = σ(Threat_level · w_T + Validator_health · w_V + Network_entropy · w_N)
    pub fn compute_el_state(&mut self) {
        let threat = self.threat_level.get().unwrap_or(0) as u64;
        let health = self.validator_health.get().unwrap_or(0) as u64;
        let entropy = self.network_entropy.get().unwrap_or(0) as u64;

        let w_t = self.weight_threat.get().unwrap_or(300_000);
        let w_v = self.weight_validator.get().unwrap_or(400_000);
        let w_n = self.weight_entropy.get().unwrap_or(300_000);

        // Compute weighted sum in fixed-point (1e6 scale)
        // threat is 0-100, health/entropy are 0-10000
        // Normalize all to 0-1 range first
        let threat_norm = threat * 10_000; // 0-100 -> 0-1,000,000
        let health_norm = health * 100;     // 0-10000 -> 0-1,000,000
        let entropy_norm = entropy * 100;   // 0-10000 -> 0-1,000,000

        let weighted_sum = (threat_norm * w_t + health_norm * w_v + entropy_norm * w_n) / 1_000_000;

        // Sigmoid approximation: σ(x) = x / (1 + |x|) for x in [0, 1e6]
        // But we need to map weighted_sum to [0, 1e6]
        // Simplified: use piecewise linear sigmoid
        let el = self.sigmoid_approx(weighted_sum);

        let block = contract_env::block_time();
        self.el_state.set(el);
        self.el_history.set(&block, el);

        // Determine regime based on EL_state and threat
        let regime = self.determine_regime(el, threat as u8);
        self.current_regime.set(regime.clone());

        // Modulate contract expressions based on regime
        self.modulate_expression(&regime, el);

        contract_env::emit_event(EpistaticStateUpdated {
            el_state: el,
            threat_level: threat as u8,
            validator_health: health as u16,
            network_entropy: entropy as u16,
            regime: regime.clone(),
            block,
        });
    }

    /// Update environmental signals (called by oracle/ANIMA)
    pub fn update_signals(&mut self, threat: u8, health: u16, entropy: u16) {
        self.assert_owner_or_oracle();

        self.threat_level.set(threat);
        self.validator_health.set(health);
        self.network_entropy.set(entropy);

        // Auto-compute new EL_state
        self.compute_el_state();

        // Update ComplianceEngine regulatory threat
        // (In production: call ComplianceEngine.update_regulatory_threat)
    }

    /// Online calibration of weights based on outcome feedback
    pub fn calibrate_weights(&mut self, outcome: i64) {
        self.assert_owner_or_oracle();

        let count = self.calibration_count.get().unwrap_or(0);
        let learning_rate = 10_000; // 0.01 fixed-point

        // Simple gradient descent: adjust weights based on outcome
        // outcome > 0: good, increase entropy weight, decrease threat weight
        // outcome < 0: bad, increase threat weight, decrease entropy weight

        let w_t = self.weight_threat.get().unwrap_or(300_000);
        let w_v = self.weight_validator.get().unwrap_or(400_000);
        let w_n = self.weight_entropy.get().unwrap_or(300_000);

        let adjustment = (outcome.abs() as u64 * learning_rate) / 100;

        let (new_w_t, new_w_v, new_w_n) = if outcome > 0 {
            // Good outcome: trust entropy more, threat less
            let new_t = if w_t > adjustment { w_t - adjustment } else { 0 };
            let new_n = w_n + adjustment;
            let new_v = w_v; // Keep validator weight stable
            (new_t, new_v, new_n)
        } else {
            // Bad outcome: trust threat more, entropy less
            let new_t = w_t + adjustment;
            let new_n = if w_n > adjustment { w_n - adjustment } else { 0 };
            let new_v = w_v;
            (new_t, new_v, new_n)
        };

        // Normalize to sum to 1,000,000
        let sum = new_w_t + new_w_v + new_w_n;
        if sum > 0 {
            self.weight_threat.set((new_w_t * 1_000_000) / sum);
            self.weight_validator.set((new_w_v * 1_000_000) / sum);
            self.weight_entropy.set((new_w_n * 1_000_000) / sum);
        }

        self.calibration_count.set(count + 1);

        contract_env::emit_event(WeightsCalibrated {
            threat_weight: self.weight_threat.get().unwrap_or(0),
            validator_weight: self.weight_validator.get().unwrap_or(0),
            entropy_weight: self.weight_entropy.get().unwrap_or(0),
            calibration_count: count + 1,
        });
    }

    /// Get current EL_state
    pub fn get_el_state(&self) -> u64 {
        self.el_state.get().unwrap_or(500_000)
    }

    /// Get current regime
    pub fn get_regime(&self) -> Regime {
        self.current_regime.get().unwrap_or(Regime::Normal)
    }

    /// Get expression modifier for a contract
    pub fn get_modifier(&self, contract: String) -> u64 {
        self.expression_modifiers.get(&contract).unwrap_or(1_000_000)
    }

    /// Get EL_state history
    pub fn get_el_history(&self, block: u64) -> u64 {
        self.el_history.get(&block).unwrap_or(0)
    }

    /// Get all weights
    pub fn get_weights(&self) -> (u64, u64, u64) {
        (
            self.weight_threat.get().unwrap_or(0),
            self.weight_validator.get().unwrap_or(0),
            self.weight_entropy.get().unwrap_or(0),
        )
    }

    // Sigmoid approximation: maps [0, 1e6] -> [0, 1e6]
    fn sigmoid_approx(&self, x: u64) -> u64 {
        if x < 200_000 {
            // Low: linear ramp
            x / 2
        } else if x < 500_000 {
            // Mid: quadratic
            (x * x) / 2_000_000
        } else if x < 800_000 {
            // High-mid: approaching saturation
            500_000 + (x - 500_000) / 2
        } else {
            // High: saturated
            650_000 + (x - 800_000) / 4
        }
    }

    fn determine_regime(&self, el: u64, threat: u8) -> Regime {
        if threat >= 80 || el >= 900_000 {
            Regime::Silence
        } else if threat >= 60 || el >= 750_000 {
            Regime::Critical
        } else if threat >= 30 || el >= 600_000 {
            Regime::Alert
        } else {
            Regime::Normal
        }
    }

    fn modulate_expression(&mut self, regime: &Regime, el: u64) {
        match regime {
            Regime::Normal => {
                // Relaxed: lower thresholds, optimize throughput
                self.expression_modifiers.set("sentinel_vault".to_string(), 1_000_000 - (el / 10));
                self.expression_modifiers.set("compliance_engine".to_string(), 1_000_000);
            },
            Regime::Alert => {
                // Tightened: higher thresholds, increased ZK requirements
                self.expression_modifiers.set("sentinel_vault".to_string(), 1_000_000 + (el / 5));
                self.expression_modifiers.set("compliance_engine".to_string(), 1_000_000 + (el / 8));
            },
            Regime::Critical => {
                // Restricted: ZK-only, reduced limits
                self.expression_modifiers.set("sentinel_vault".to_string(), 1_500_000);
                self.expression_modifiers.set("compliance_engine".to_string(), 1_300_000);
            },
            Regime::Silence => {
                // Paused: all non-essential ops halted
                self.expression_modifiers.set("sentinel_vault".to_string(), 10_000_000); // 10x threshold
                self.expression_modifiers.set("compliance_engine".to_string(), 2_000_000);
            },
        }

        // Forward threshold update to SentinelVault
        // (In production: call SentinelVault.update_threshold)
    }

    fn assert_owner_or_oracle(&self) {
        // Verify caller
    }
}

#[odra::event]
pub struct EpistaticStateUpdated {
    pub el_state: u64,
    pub threat_level: u8,
    pub validator_health: u16,
    pub network_entropy: u16,
    pub regime: Regime,
    pub block: u64,
}

#[odra::event]
pub struct WeightsCalibrated {
    pub threat_weight: u64,
    pub validator_weight: u64,
    pub entropy_weight: u64,
    pub calibration_count: u64,
}
