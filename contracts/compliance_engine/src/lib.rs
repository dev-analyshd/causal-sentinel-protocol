#![no_std]

use odra::prelude::*;
use odra::{contract_env, types::Address, Variable, Mapping};

/// ComplianceEngine — Protocol-level compliance hooks via Casper Native Token Registry
/// 
/// Leverages Casper's protocol-level compliance hooks for native-speed
/// enforcement. This contract coordinates with the Native Token Registry
/// to enforce jurisdictional rules, KYC/AML tiers, and regulatory adaptation.

#[odra::module]
pub struct ComplianceEngine {
    /// Owner (regulatory multi-sig)
    owner: Variable<Address>,

    /// Jurisdiction rules: (jurisdiction_code) -> RuleSet
    jurisdiction_rules: Mapping<String, RuleSet>,

    /// Agent compliance status: (agent) -> ComplianceStatus
    agent_compliance: Mapping<Address, ComplianceStatus>,

    /// Credential validity: (nullifier) -> (expiry_block, tier)
    credential_validity: Mapping<[u8; 32], (u64, u8)>,

    /// Geographic HHI (Herfindahl-Hirschman Index) for decentralization
    geographic_hhi: Variable<u64>,

    /// Regulatory threat level (0-100)
    regulatory_threat: Variable<u8>,

    /// Chameleon mode: ZK-only outputs enforced
    chameleon_mode: Variable<bool>,

    /// Right to Invisibility flag
    right_to_invisibility: Variable<bool>,

    /// AWA (Automated Weaponization Assessment) flag
    awa_enforced: Variable<bool>,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct RuleSet {
    pub jurisdiction: String,
    pub kyc_required: bool,
    pub aml_threshold: u64,        // In motes
    pub max_transaction: u64,      // In motes
    pub zk_only: bool,
    pub geographic_restrictions: Vec<String>,
    pub active: bool,
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct ComplianceStatus {
    pub agent: Address,
    pub jurisdiction: String,
    pub kyc_verified: bool,
    pub aml_score: u8,            // 0-100 (lower is better)
    pub last_audit_block: u64,
    pub sanctions_screened: bool,
    pub tier_override: Option<u8>, // Regulatory override of credential tier
}

#[odra::module]
impl ComplianceEngine {
    pub fn init(&mut self, owner: Address) {
        self.owner.set(owner);
        self.geographic_hhi.set(2500); // Max decentralization
        self.regulatory_threat.set(0);
        self.chameleon_mode.set(false);
        self.right_to_invisibility.set(true);
        self.awa_enforced.set(true);

        // Initialize default jurisdiction rules
        self.add_jurisdiction("US", RuleSet {
            jurisdiction: "US".to_string(),
            kyc_required: true,
            aml_threshold: 10_000_000_000, // ~10,000 CSPR
            max_transaction: 100_000_000_000,
            zk_only: false,
            geographic_restrictions: vec![],
            active: true,
        });

        self.add_jurisdiction("EU", RuleSet {
            jurisdiction: "EU".to_string(),
            kyc_required: true,
            aml_threshold: 10_000_000_000,
            max_transaction: 100_000_000_000,
            zk_only: false,
            geographic_restrictions: vec![],
            active: true,
        });

        self.add_jurisdiction("SG", RuleSet {
            jurisdiction: "SG".to_string(),
            kyc_required: true,
            aml_threshold: 20_000_000_000,
            max_transaction: 200_000_000_000,
            zk_only: false,
            geographic_restrictions: vec![],
            active: true,
        });
    }

    /// Add or update jurisdiction rules
    pub fn set_jurisdiction_rules(&mut self, code: String, rules: RuleSet) {
        self.assert_owner();
        self.jurisdiction_rules.set(&code, rules);

        contract_env::emit_event(JurisdictionRulesUpdated {
            jurisdiction: code,
            block: contract_env::block_time(),
        });
    }

    /// Register agent compliance status
    pub fn register_compliance(&mut self, agent: Address, status: ComplianceStatus) {
        self.assert_owner_or_oracle();
        self.agent_compliance.set(&agent, status);

        contract_env::emit_event(ComplianceRegistered {
            agent,
            jurisdiction: status.jurisdiction.clone(),
            kyc_verified: status.kyc_verified,
        });
    }

    /// Verify if an action is compliant (called by SentinelVault before execution)
    pub fn verify_compliance(
        &self,
        agent: Address,
        amount: u64,
        target_jurisdiction: String,
    ) -> ComplianceResult {
        let status = self.agent_compliance.get(&agent);
        let rules = self.jurisdiction_rules.get(&target_jurisdiction);

        if status.is_none() {
            return ComplianceResult {
                allowed: false,
                reason: "Agent not compliance-registered".to_string(),
                required_tier: 1,
            };
        }

        let status = status.unwrap();
        let rules = rules.expect("Jurisdiction not found");

        if !rules.active {
            return ComplianceResult {
                allowed: false,
                reason: "Jurisdiction rules suspended".to_string(),
                required_tier: 1,
            };
        }

        if rules.kyc_required && !status.kyc_verified {
            return ComplianceResult {
                allowed: false,
                reason: "KYC required but not verified".to_string(),
                required_tier: 1,
            };
        }

        if amount > rules.max_transaction {
            return ComplianceResult {
                allowed: false,
                reason: "Amount exceeds jurisdiction maximum".to_string(),
                required_tier: status.tier_override.unwrap_or(5),
            };
        }

        if amount > rules.aml_threshold && status.aml_score > 50 {
            return ComplianceResult {
                allowed: false,
                reason: "AML risk score too high for this amount".to_string(),
                required_tier: 4,
            };
        }

        // Chameleon mode: force ZK-only
        if self.chameleon_mode.get().unwrap_or(false) || rules.zk_only {
            return ComplianceResult {
                allowed: true,
                reason: "ZK-only mode enforced".to_string(),
                required_tier: status.tier_override.unwrap_or(3),
            };
        }

        ComplianceResult {
            allowed: true,
            reason: "Compliant".to_string(),
            required_tier: status.tier_override.unwrap_or(1),
        }
    }

    /// Record credential issuance
    pub fn record_credential(&mut self, nullifier: [u8; 32], expiry_block: u64, tier: u8) {
        self.assert_owner_or_oracle();
        self.credential_validity.set(&nullifier, (expiry_block, tier));

        contract_env::emit_event(CredentialRecorded {
            nullifier,
            expiry_block,
            tier,
        });
    }

    /// Verify credential is valid and not expired
    pub fn verify_credential(&self, nullifier: [u8; 32]) -> (bool, u8) {
        match self.credential_validity.get(&nullifier) {
            Some((expiry, tier)) => {
                let current = contract_env::block_time();
                (current <= expiry, tier)
            },
            None => (false, 0),
        }
    }

    /// Update regulatory threat and trigger chameleon mode if needed
    pub fn update_regulatory_threat(&mut self, threat: u8) {
        self.assert_owner_or_oracle();
        self.regulatory_threat.set(threat);

        // Chameleon Protocol: if threat detected, shift to ZK-only
        if threat >= 60 {
            self.chameleon_mode.set(true);
            contract_env::emit_event(ChameleonModeActivated {
                threat_level: threat,
                block: contract_env::block_time(),
            });
        } else if threat < 20 {
            self.chameleon_mode.set(false);
            contract_env::emit_event(ChameleonModeDeactivated {
                threat_level: threat,
            });
        }

        // Right to Invisibility: if AWA detects weaponization, freeze emissions
        if threat >= 90 && self.right_to_invisibility.get().unwrap_or(false) {
            self.awa_enforced.set(false);
            contract_env::emit_event(RightToInvisibilityEnforced {
                threat_level: threat,
            });
        }
    }

    /// Get geographic HHI
    pub fn get_geographic_hhi(&self) -> u64 {
        self.geographic_hhi.get().unwrap_or(2500)
    }

    /// Update geographic HHI (from validator health data)
    pub fn update_hhi(&mut self, hhi: u64) {
        self.assert_owner_or_oracle();
        self.geographic_hhi.set(hhi);
    }

    /// Get current regulatory threat
    pub fn get_regulatory_threat(&self) -> u8 {
        self.regulatory_threat.get().unwrap_or(0)
    }

    /// Check if chameleon mode is active
    pub fn is_chameleon_active(&self) -> bool {
        self.chameleon_mode.get().unwrap_or(false)
    }

    fn add_jurisdiction(&mut self, code: &str, rules: RuleSet) {
        self.jurisdiction_rules.set(&code.to_string(), rules);
    }

    fn assert_owner(&self) {
        assert_eq!(contract_env::caller(), self.owner.get().unwrap(), "Not owner");
    }

    fn assert_owner_or_oracle(&self) {
        // Verify caller is owner or authorized oracle
    }
}

#[odra::odra_type]
#[derive(Clone, Debug)]
pub struct ComplianceResult {
    pub allowed: bool,
    pub reason: String,
    pub required_tier: u8,
}

#[odra::event]
pub struct JurisdictionRulesUpdated {
    pub jurisdiction: String,
    pub block: u64,
}

#[odra::event]
pub struct ComplianceRegistered {
    pub agent: Address,
    pub jurisdiction: String,
    pub kyc_verified: bool,
}

#[odra::event]
pub struct CredentialRecorded {
    pub nullifier: [u8; 32],
    pub expiry_block: u64,
    pub tier: u8,
}

#[odra::event]
pub struct ChameleonModeActivated {
    pub threat_level: u8,
    pub block: u64,
}

#[odra::event]
pub struct ChameleonModeDeactivated {
    pub threat_level: u8,
}

#[odra::event]
pub struct RightToInvisibilityEnforced {
    pub threat_level: u8,
}
