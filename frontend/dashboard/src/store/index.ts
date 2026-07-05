import { create } from 'zustand';

interface Agent {
  id: string;
  psi: number;
  delta: number;
  lambda: number;
  tier: number;
  status: string;
  regime: string;
}

interface AppState {
  agents: Agent[];
  globalPsi: number;
  globalLambda: number;
  regime: string;
  setAgents: (agents: Agent[]) => void;
  updateAgent: (agent: Agent) => void;
  setGlobalState: (psi: number, lambda: number, regime: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  agents: [],
  globalPsi: 0.74,
  globalLambda: 1.84,
  regime: 'Normal',
  setAgents: (agents) => set({ agents }),
  updateAgent: (agent) => set((state) => ({
    agents: state.agents.map((a) => (a.id === agent.id ? agent : a)),
  })),
  setGlobalState: (psi, lambda, regime) => set({ globalPsi: psi, globalLambda: lambda, regime }),
}));
