import React from 'react';
import { motion } from 'framer-motion';
import { Search, Filter, ArrowUpRight, Shield, Clock, Award } from 'lucide-react';

const agents = [
  {
    id: 'agent_001',
    address: 'account-hash-...a3f2',
    psi: 0.82,
    lambda: 2.1,
    tier: 5,
    age_days: 387,
    actions: 1247,
    silence_events: 3,
    manipulations: 0,
    domains: ['Trading', 'Security', 'CrossChain'],
    status: 'active',
  },
  {
    id: 'agent_002',
    address: 'account-hash-...b8c1',
    psi: 0.45,
    lambda: 0.8,
    tier: 2,
    age_days: 92,
    actions: 234,
    silence_events: 12,
    manipulations: 2,
    domains: ['Trading'],
    status: 'silence',
  },
  {
    id: 'agent_003',
    address: 'account-hash-...d4e5',
    psi: 0.91,
    lambda: 3.2,
    tier: 5,
    age_days: 512,
    actions: 3456,
    silence_events: 1,
    manipulations: 0,
    domains: ['Governance', 'Security', 'Compliance', 'FederatedLearning'],
    status: 'active',
  },
];

export default function AgentExplorer() {
  const [search, setSearch] = React.useState('');
  const [filterTier, setFilterTier] = React.useState<number | null>(null);

  const filtered = agents.filter(a => {
    const matchesSearch = a.id.includes(search) || a.address.includes(search);
    const matchesTier = filterTier === null || a.tier === filterTier;
    return matchesSearch && matchesTier;
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Agent Explorer</h2>
          <p className="text-sentinel-400 text-sm mt-1">Browse and inspect sentinel agents</p>
        </div>
      </div>

      {/* Search & Filter */}
      <div className="flex gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-sentinel-400" />
          <input
            type="text"
            placeholder="Search agents..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-10 pr-4 py-3 bg-sentinel-800 border border-sentinel-700 rounded-lg text-white placeholder-sentinel-500 focus:outline-none focus:border-sentinel-300"
          />
        </div>
        <div className="flex gap-2">
          {[1, 2, 3, 4, 5].map(tier => (
            <button
              key={tier}
              onClick={() => setFilterTier(filterTier === tier ? null : tier)}
              className={`px-3 py-2 rounded-lg text-sm font-medium transition-all ${
                filterTier === tier
                  ? 'bg-sentinel-300 text-white'
                  : 'bg-sentinel-800 text-sentinel-400 hover:bg-sentinel-700'
              }`}
            >
              T{tier}
            </button>
          ))}
        </div>
      </div>

      {/* Agent Cards */}
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
        {filtered.map((agent, i) => (
          <motion.div
            key={agent.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            className="glass-panel p-6 hover:border-sentinel-300/30 transition-all cursor-pointer group"
          >
            <div className="flex items-start justify-between mb-4">
              <div>
                <div className="font-mono text-sm text-sentinel-300">{agent.id}</div>
                <div className="text-xs text-sentinel-500 font-mono mt-1">{agent.address}</div>
              </div>
              <TierBadge tier={agent.tier} />
            </div>

            <div className="grid grid-cols-2 gap-4 mb-4">
              <div>
                <div className="text-xs text-sentinel-400 mb-1">Coherence Ψ</div>
                <div className="text-xl font-bold">{agent.psi.toFixed(2)}</div>
                <CoherenceBar value={agent.psi} threshold={0.57} />
              </div>
              <div>
                <div className="text-xs text-sentinel-400 mb-1">Moat Λ</div>
                <div className="text-xl font-bold text-sentinel-300">{agent.lambda.toFixed(2)}</div>
              </div>
            </div>

            <div className="flex items-center gap-4 text-xs text-sentinel-400 mb-4">
              <span className="flex items-center gap-1">
                <Clock className="w-3 h-3" /> {agent.age_days}d
              </span>
              <span className="flex items-center gap-1">
                <ArrowUpRight className="w-3 h-3" /> {agent.actions} actions
              </span>
              <span className="flex items-center gap-1">
                <Shield className="w-3 h-3" /> {agent.silence_events} silence
              </span>
            </div>

            <div className="flex flex-wrap gap-2">
              {agent.domains.map(d => (
                <span key={d} className="px-2 py-1 bg-sentinel-700/50 rounded text-xs text-sentinel-300">
                  {d}
                </span>
              ))}
            </div>

            <div className="mt-4 pt-4 border-t border-sentinel-700/50 flex items-center justify-between">
              <StatusBadge status={agent.status} />
              <button className="text-xs text-sentinel-300 hover:text-white transition-colors flex items-center gap-1">
                Details <ArrowUpRight className="w-3 h-3" />
              </button>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}

function CoherenceBar({ value, threshold }: { value: number; threshold: number }) {
  return (
    <div className="w-full h-1.5 bg-sentinel-700 rounded-full mt-2 overflow-hidden">
      <div
        className="h-full rounded-full transition-all duration-500"
        style={{
          width: `${value * 100}%`,
          backgroundColor: value >= threshold ? '#10b981' : '#e94560',
        }}
      />
    </div>
  );
}

function TierBadge({ tier }: { tier: number }) {
  const colors = ['', 'bg-gray-500', 'bg-amber-700', 'bg-slate-400', 'bg-yellow-500', 'bg-purple-500'];
  const labels = ['', 'Basic', 'Bronze', 'Silver', 'Gold', 'Platinum'];
  return (
    <div className={`px-2 py-1 rounded text-xs font-bold text-white ${colors[tier]}`}>
      {labels[tier]}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const isActive = status === 'active';
  return (
    <span className={`text-xs font-mono ${isActive ? 'text-green-400' : 'text-sentinel-200'}`}>
      {isActive ? '● ACTIVE' : '● SILENCE'}
    </span>
  );
}
