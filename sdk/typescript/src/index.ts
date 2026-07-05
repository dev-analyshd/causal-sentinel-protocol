/**
 * Causal Sentinel Protocol — TypeScript SDK
 *
 * Type-safe developer interface for CSP agent operations.
 * The type system structurally prevents SilenceSignal from being cast to
 * an actionable signal — coherence gates are enforced at compile time.
 */

// ─── Core types ──────────────────────────────────────────────────────────────

/** Opaque brand tag — never cross-assignable between signal kinds */
type Brand<T, B extends string> = T & { readonly __brand: B };

/** A signal that carries a valuation and confidence interval */
export type ValuationSignal = Brand<{
  action: "EXECUTE";
  value: number;
  ci_95: [number, number]; // Always non-null
  psi: number;
  delta: number;
  lambda: number;
  tier: ComplianceTier;
  sentinel_score: number;
}, "ValuationSignal">;

/** A structured silence — gate closed, provides ETA and limiting plane */
export type SilenceSignal = Brand<{
  action: "SILENCE" | "EMERGENCY_HALT";
  psi: number;
  delta: number;
  gap: number;          // delta - psi: how far below threshold
  limiting_plane: CoherencePlane;
  trend: "improving" | "stable" | "degrading";
  eta_seconds: number | null; // null = unknown ETA
  reason: string;
}, "SilenceSignal">;

export type CoherenceResult = ValuationSignal | SilenceSignal;

export type CoherencePlane = "perceptual" | "inferential" | "consensus" | "self_reflection" | "world_model";

export type ComplianceTier = 1 | 2 | 3 | 4 | 5;
export const TierNames: Record<ComplianceTier, string> = {
  1: "Basic",
  2: "Bronze",
  3: "Silver",
  4: "Gold",
  5: "Platinum",
};

export type ActionType = "transfer" | "trade" | "stake" | "unstake" | "bridge" | "credential_renewal";

export type Regime = "Normal" | "Alert" | "Critical" | "Silence";

// ─── Request/response types ───────────────────────────────────────────────────

export interface CoherenceRequest {
  agent_id: string;
  action_type: ActionType;
  amount?: number;        // In CSPR motes
  target?: string;        // Target address for transfer/bridge
  jurisdiction?: string;  // ISO code: "US" | "EU" | "SG" | ...
}

export interface ZKProofRequest {
  agent_id: string;
  credential_type: "behavioral_integrity" | "causal_identity" | "sentinel_compliance";
  target_tier?: ComplianceTier;
}

export interface ZKProof {
  circuit_type: string;
  proof_bytes: Uint8Array;
  public_inputs: Uint8Array;
  nullifier: string;       // Hex-encoded [u8; 32]
  compliance_tier: ComplianceTier;
  expiry_block: number;
  generated_at: Date;
}

export interface MoatStatus {
  agent_id: string;
  lambda: number;          // Current Λ(t) value
  tier: ComplianceTier;
  registration_block: number;
  age_days: number;
  moat_growth_rate: number; // κ × Ψ(t) per cycle
  next_tier_eta_days: number | null;
}

export interface AgentHeartbeat {
  agent_id: string;
  psi: number;
  lambda: number;
  iq: number;
  block: number;
  tx_hash?: string;
}

export interface RegistrationResult {
  agent_address: string;
  registration_block: number;
  stake_amount: number;
  initial_tier: ComplianceTier;
  tx_hash: string;
}

// ─── Type guards ─────────────────────────────────────────────────────────────

export function isValuationSignal(result: CoherenceResult): result is ValuationSignal {
  return result.action === "EXECUTE";
}

export function isSilenceSignal(result: CoherenceResult): result is SilenceSignal {
  return result.action === "SILENCE" || result.action === "EMERGENCY_HALT";
}

// ─── SDK Client ──────────────────────────────────────────────────────────────

export interface CSPClientConfig {
  api_url: string;
  agent_id: string;
  api_key: string;
  timeout_ms?: number;
}

export class CausalSentinelSDK {
  private readonly config: Required<CSPClientConfig>;

  constructor(config: CSPClientConfig) {
    this.config = {
      timeout_ms: 10_000,
      ...config,
    };
  }

  /**
   * Evaluate if an action should execute based on the coherence gate.
   * Returns either a ValuationSignal (execute) or SilenceSignal (hold).
   *
   * The return type is discriminated — you MUST handle both cases:
   *   const result = await sdk.evaluateAction(req);
   *   if (isValuationSignal(result)) { ... execute ... }
   *   else { ... handle silence ... }
   */
  async evaluateAction(request: CoherenceRequest): Promise<CoherenceResult> {
    const resp = await this.fetch("/api/v1/coherence/evaluate", {
      method: "POST",
      body: JSON.stringify({ ...request, agent_id: this.config.agent_id }),
    });
    return resp as CoherenceResult;
  }

  /**
   * Generate a ZK Behavioral Integrity Credential (BIC).
   * The proof is valid for 90 days (credential_ttl = 972,000 blocks).
   */
  async generateZKProof(request: ZKProofRequest): Promise<ZKProof> {
    const resp = await this.fetch("/api/v1/zk/generate", {
      method: "POST",
      body: JSON.stringify({ ...request, agent_id: this.config.agent_id }),
    });
    return {
      ...resp,
      proof_bytes: new Uint8Array(Buffer.from(resp.proof_bytes_hex, "hex")),
      public_inputs: new Uint8Array(Buffer.from(resp.public_inputs_hex, "hex")),
      generated_at: new Date(resp.generated_at),
    };
  }

  /**
   * Get current moat Λ(t) status and tier projection.
   */
  async getMoatStatus(agentId?: string): Promise<MoatStatus> {
    const id = agentId ?? this.config.agent_id;
    return this.fetch(`/api/v1/moat/${encodeURIComponent(id)}`);
  }

  /**
   * Submit on-chain heartbeat (call every 100 blocks).
   * Pushes Ψ(t), Λ(t), IQ(t) to SentinelVault.
   */
  async submitHeartbeat(psi: number, lambda: number, iq: number): Promise<AgentHeartbeat> {
    return this.fetch("/api/v1/heartbeat", {
      method: "POST",
      body: JSON.stringify({
        agent_id: this.config.agent_id,
        psi: Math.round(psi * 1_000_000),    // Convert to fixed-point
        lambda: Math.round(lambda * 1_000_000),
        iq: Math.round(iq * 1_000_000),
      }),
    });
  }

  /**
   * Register a new sentinel agent on-chain.
   * Requires attached CSPR stake (minimum 10 CSPR on testnet).
   */
  async registerAgent(
    dna_code_hash: string,
    behavioral_commitment: string,
    stake_motes: number,
  ): Promise<RegistrationResult> {
    return this.fetch("/api/v1/agents/register", {
      method: "POST",
      body: JSON.stringify({
        agent_id: this.config.agent_id,
        dna_code_hash,
        behavioral_commitment,
        stake_motes,
      }),
    });
  }

  /**
   * WebSocket subscription for real-time coherence events.
   * Emits on every coherence evaluation (~every 8 seconds).
   */
  subscribeToEvents(
    onEvent: (event: CoherenceResult) => void,
    onError?: (err: Error) => void,
  ): () => void {
    const wsUrl = this.config.api_url.replace(/^http/, "ws") + "/ws/coherence";
    const ws = new WebSocket(wsUrl + `?agent_id=${this.config.agent_id}&api_key=${this.config.api_key}`);

    ws.onmessage = (msg) => {
      try {
        onEvent(JSON.parse(msg.data) as CoherenceResult);
      } catch (e) {
        onError?.(e as Error);
      }
    };
    ws.onerror = (e) => onError?.(new Error("WebSocket error"));

    return () => ws.close();
  }

  // ─── Private helpers ────────────────────────────────────────────────────────

  private async fetch(path: string, init?: RequestInit): Promise<any> {
    const url = this.config.api_url + path;
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.config.timeout_ms);

    try {
      const resp = await globalThis.fetch(url, {
        ...init,
        signal: controller.signal,
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${this.config.api_key}`,
          ...(init?.headers ?? {}),
        },
      });

      if (!resp.ok) {
        const body = await resp.text().catch(() => "");
        throw new Error(`CSP API error ${resp.status}: ${body}`);
      }

      return resp.json();
    } finally {
      clearTimeout(timer);
    }
  }
}

// ─── Convenience factory ──────────────────────────────────────────────────────

export function createSDK(config: CSPClientConfig): CausalSentinelSDK {
  return new CausalSentinelSDK(config);
}

export default CausalSentinelSDK;
