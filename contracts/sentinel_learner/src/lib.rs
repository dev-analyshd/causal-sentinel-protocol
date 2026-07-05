#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping};

/// SentinelLearner — Domain mastery ledger, IQ milestones, and epistatic params
/// 
/// Tracks agent learning progress across domains. IQ milestones unlock
/// new capabilities. Epistatic parameters stored here are read by
/// EpistaticController to modulate contract expression.

#[odra::module]
pub struct SentinelLearner {
    /// Registry reference
    registry: Variable<Address>,

    /// Domain mastery scores: (agent, domain) -> score (0-10000)
    domain_mastery: Mapping<(Address, Domain), u16>,

    /// IQ milestones achieved: (agent) -> milestone count
    iq_milestones: Mapping<Address, u8>,

    /// Epistatic parameters: (parameter_name) -> value (fixed-point u64)
    epistatic_params: Mapping<String, u64>,

    /// Parameter update history
    param_history: Mapping<(String, u64), u64>, // (name, block) -> value

    /// Domain definitions
    domains: Mapping<u8, Domain>,
    domain_count: Variable<u8>,

    /// Agent capability unlocks: (agent, capability) -> unlocked
    capabilities: Mapping<(Address, String), bool>,
}

#[odra::odra_type]
#[derive(Clone, Debug, PartialEq)]
pub enum Domain {
    Trading,
    Governance,
    Security,
    CrossChain,
    MEVDefense,
    Compliance,
    FederatedLearning,
}

#[odra::module]
impl SentinelLearner {
    pub fn init(&mut self, registry: Address) {
        self.registry.set(registry);
        self.domain_count.set(0);

        // Initialize default domains
        self.add_domain(Domain::Trading);
        self.add_domain(Domain::Governance);
        self.add_domain(Domain::Security);
        self.add_domain(Domain::CrossChain);
        self.add_domain(Domain::MEVDefense);
        self.add_domain(Domain::Compliance);
        self.add_domain(Domain::FederatedLearning);

        // Initialize epistatic parameters
        self.epistatic_params.set("threat_coefficient".to_string(), 100_000);     // 0.1
        self.epistatic_params.set("validator_weight".to_string(), 300_000);        // 0.3
        self.epistatic_params.set("entropy_weight".to_string(), 200_000);          // 0.2
        self.epistatic_params.set("silence_threshold".to_string(), 570_000);       // 0.57
        self.epistatic_params.set("regime_factor".to_string(), 1_000_000);        // 1.0
    }

    /// Record domain mastery progress
    pub fn record_mastery(&mut self, agent: Address, domain: Domain, score: u16) {
        self.assert_caller_is_registry_or_vault();

        let current = self.domain_mastery.get(&(agent, domain.clone())).unwrap_or(0);
        let new_score = current + score;
        let capped = if new_score > 10000 { 10000 } else { new_score };

        self.domain_mastery.set(&(agent, domain.clone()), capped);

        // Check IQ milestone
        if capped >= 10000 && current < 10000 {
            let milestones = self.iq_milestones.get(&agent).unwrap_or(0) + 1;
            self.iq_milestones.set(&agent, milestones);

            // Unlock capabilities based on domain mastery
            self.unlock_capabilities(agent, &domain, milestones);

            contract_env::emit_event(IQMilestone {
                agent,
                domain,
                milestone: milestones,
                block: contract_env::block_time(),
            });
        }

        contract_env::emit_event(MasteryProgress {
            agent,
            domain,
            new_score: capped,
        });
    }

    /// Update epistatic parameter (called by EpistaticController)
    pub fn update_epistatic_param(&mut self, name: String, value: u64) {
        self.assert_caller_is_controller();

        let block = contract_env::block_time();
        self.param_history.set(&(name.clone(), block), value);
        self.epistatic_params.set(&name, value);

        contract_env::emit_event(EpistaticParamUpdated {
            name,
            value,
            block,
        });
    }

    /// Get epistatic parameter value
    pub fn get_epistatic_param(&self, name: String) -> u64 {
        self.epistatic_params.get(&name).unwrap_or(0)
    }

    /// Get domain mastery score
    pub fn get_mastery(&self, agent: Address, domain: Domain) -> u16 {
        self.domain_mastery.get(&(agent, domain)).unwrap_or(0)
    }

    /// Get IQ milestone count
    pub fn get_milestones(&self, agent: Address) -> u8 {
        self.iq_milestones.get(&agent).unwrap_or(0)
    }

    /// Check if agent has capability
    pub fn has_capability(&self, agent: Address, capability: String) -> bool {
        self.capabilities.get(&(agent, capability)).unwrap_or(false)
    }

    /// Get all epistatic params (simplified)
    pub fn get_all_params(&self) -> Vec<(String, u64)> {
        vec![
            ("threat_coefficient".to_string(), self.epistatic_params.get(&"threat_coefficient".to_string()).unwrap_or(0)),
            ("validator_weight".to_string(), self.epistatic_params.get(&"validator_weight".to_string()).unwrap_or(0)),
            ("entropy_weight".to_string(), self.epistatic_params.get(&"entropy_weight".to_string()).unwrap_or(0)),
            ("silence_threshold".to_string(), self.epistatic_params.get(&"silence_threshold".to_string()).unwrap_or(0)),
            ("regime_factor".to_string(), self.epistatic_params.get(&"regime_factor".to_string()).unwrap_or(0)),
        ]
    }

    fn add_domain(&mut self, domain: Domain) {
        let count = self.domain_count.get().unwrap_or(0);
        self.domains.set(&count, domain);
        self.domain_count.set(count + 1);
    }

    fn unlock_capabilities(&mut self, agent: Address, domain: &Domain, milestone: u8) {
        match domain {
            Domain::Trading => {
                self.capabilities.set(&(agent, "advanced_trading".to_string()), true);
            },
            Domain::Governance => {
                self.capabilities.set(&(agent, "vote_propose".to_string()), true);
            },
            Domain::Security => {
                self.capabilities.set(&(agent, "crispr_defense".to_string()), true);
            },
            Domain::CrossChain => {
                self.capabilities.set(&(agent, "bridge_operator".to_string()), true);
            },
            Domain::MEVDefense => {
                self.capabilities.set(&(agent, "mev_protection".to_string()), true);
            },
            Domain::Compliance => {
                self.capabilities.set(&(agent, "credential_issuer".to_string()), true);
            },
            Domain::FederatedLearning => {
                self.capabilities.set(&(agent, "model_trainer".to_string()), true);
            },
        }

        if milestone >= 3 {
            self.capabilities.set(&(agent, "mentor_status".to_string()), true);
        }
        if milestone >= 5 {
            self.capabilities.set(&(agent, "sentinel_council".to_string()), true);
        }
    }

    fn assert_caller_is_registry_or_vault(&self) {
        // Verify caller is SentinelRegistry or SentinelVault
    }

    fn assert_caller_is_controller(&self) {
        // Verify caller is EpistaticController
    }
}

#[odra::event]
pub struct IQMilestone {
    pub agent: Address,
    pub domain: Domain,
    pub milestone: u8,
    pub block: u64,
}

#[odra::event]
pub struct MasteryProgress {
    pub agent: Address,
    pub domain: Domain,
    pub new_score: u16,
}

#[odra::event]
pub struct EpistaticParamUpdated {
    pub name: String,
    pub value: u64,
    pub block: u64,
}
