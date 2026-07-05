export interface AgentProfile {
  identity: string;
  registration_block: number;
  last_heartbeat_block: number;
  moat_lambda: number;
  coherence_psi: number;
  iq_score: number;
  credential_tier: number;
  total_actions: number;
  silence_events: number;
  manipulation_count: number;
  is_active: boolean;
  dna_code_hash: string;
}

export interface CoherenceState {
  agent_id: string;
  psi: number;
  p_t: number;
  i_t: number;
  c_t: number;
  s_t: number;
  w_t: number;
  delta_t: number;
  lambda_t: number;
  regime: string;
  gate_open: boolean;
}

export interface SilenceEvent {
  agent: string;
  block: number;
  reason: string;
  psi: number;
  delta: number;
}

export interface ServiceDefinition {
  name: string;
  description: string;
  cost_motes: number;
  tier: 'Free' | 'Basic' | 'Premium' | 'Enterprise';
  requires_zk: boolean;
  rate_limit: number;
}
