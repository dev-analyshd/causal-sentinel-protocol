import React from 'react';
import { motion } from 'framer-motion';
import { Activity, TrendingUp, Shield, AlertTriangle, Lock, Globe } from 'lucide-react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from 'recharts';
import { useWebSocket } from '../hooks/useWebSocket';

const mockData = [
  { block: '1.2M', psi: 0.72, delta: 0.58, lambda: 1.2 },
  { block: '1.3M', psi: 0.68, delta: 0.59, lambda: 1.25 },
  { block: '1.4M', psi: 0.75, delta: 0.57, lambda: 1.3 },
  { block: '1.5M', psi: 0.71, delta: 0.60, lambda: 1.35 },
  { block: '1.6M', psi: 0.78, delta: 0.56, lambda: 1.4 },
  { block: '1.7M', psi: 0.74, delta: 0.58, lambda: 1.45 },
  { block: '1.8M', psi: 0.80, delta: 0.55, lambda: 1.5 },
];

const agents = [
  { id: 'agent_001', psi: 0.82, delta: 0.57, lambda: 2.1, tier: 5, status: 'active', regime: 'Normal' },
  { id: 'agent_002', psi: 0.45, delta: 0.60, lambda: 0.8, tier: 2, status: 'silence', regime: 'Alert' },
  { id: 'agent_003', psi: 0.91, delta: 0.55, lambda: 3.2, tier: 5, status: 'active', regime: 'Normal' },
];

export default function Dashboard() {
  const { connected, data } = useWebSocket('ws://localhost:9001');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Dashboard</h2>
          <p className="text-sentinel-400 text-sm mt-1">Real-time coherence monitoring</p>
        </div>
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-400 animate-pulse' : 'bg-red-400'}`} />
          <span className="text-sm font-mono text-sentinel-400">{connected ? 'L0 CONNECTED' : 'DISCONNECTED'}</span>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={Activity}
          label="Active Agents"
          value="247"
          change="+12"
          color="text-green-400"
        />
        <StatCard
          icon={TrendingUp}
          label="Avg Coherence Ψ"
          value="0.74"
          change="+0.03"
          color="text-sentinel-100"
        />
        <StatCard
          icon={Shield}
          label="Avg Moat Λ"
          value="1.84"
          change="+0.12"
          color="text-sentinel-300"
        />
        <StatCard
          icon={AlertTriangle}
          label="SILENCE Events"
          value="23"
          change="-5"
          color="text-sentinel-200"
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="glass-panel p-6"
        >
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Activity className="w-5 h-5 text-sentinel-300" />
            Coherence Over Time
          </h3>
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={mockData}>
              <defs>
                <linearGradient id="psiGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#e94560" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#e94560" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#1a1a2e" />
              <XAxis dataKey="block" stroke="#533483" fontSize={12} />
              <YAxis stroke="#533483" fontSize={12} domain={[0, 1]} />
              <Tooltip
                contentStyle={{ backgroundColor: '#12121a', border: '1px solid #16213e', borderRadius: '8px' }}
                labelStyle={{ color: '#fff' }}
              />
              <Area type="monotone" dataKey="psi" stroke="#e94560" fill="url(#psiGradient)" strokeWidth={2} />
              <Area type="monotone" dataKey="delta" stroke="#ffd93d" fill="none" strokeWidth={2} strokeDasharray="5 5" />
            </AreaChart>
          </ResponsiveContainer>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="glass-panel p-6"
        >
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <TrendingUp className="w-5 h-5 text-sentinel-100" />
            Moat Compounding
          </h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={mockData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#1a1a2e" />
              <XAxis dataKey="block" stroke="#533483" fontSize={12} />
              <YAxis stroke="#533483" fontSize={12} />
              <Tooltip
                contentStyle={{ backgroundColor: '#12121a', border: '1px solid #16213e', borderRadius: '8px' }}
              />
              <Bar dataKey="lambda" fill="#533483" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </motion.div>
      </div>

      {/* Agent Table */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="glass-panel p-6"
      >
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Globe className="w-5 h-5 text-sentinel-400" />
          Active Agents
        </h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left text-xs font-mono text-sentinel-400 uppercase border-b border-sentinel-700">
                <th className="pb-3">Agent ID</th>
                <th className="pb-3">Ψ (Coherence)</th>
                <th className="pb-3">Δ (Threshold)</th>
                <th className="pb-3">Λ (Moat)</th>
                <th className="pb-3">Tier</th>
                <th className="pb-3">Status</th>
                <th className="pb-3">Regime</th>
              </tr>
            </thead>
            <tbody className="text-sm">
              {agents.map((agent) => (
                <tr key={agent.id} className="border-b border-sentinel-700/50 hover:bg-sentinel-700/30 transition-colors">
                  <td className="py-4 font-mono">{agent.id}</td>
                  <td className="py-4">
                    <div className="flex items-center gap-2">
                      <div className="w-24 h-2 bg-sentinel-700 rounded-full overflow-hidden">
                        <div
                          className="h-full rounded-full transition-all duration-500"
                          style={{
                            width: `${agent.psi * 100}%`,
                            backgroundColor: agent.psi >= agent.delta ? '#10b981' : '#e94560',
                          }}
                        />
                      </div>
                      <span className="font-mono">{agent.psi.toFixed(2)}</span>
                    </div>
                  </td>
                  <td className="py-4 font-mono text-sentinel-400">{agent.delta.toFixed(2)}</td>
                  <td className="py-4 font-mono text-sentinel-300">{agent.lambda.toFixed(2)}</td>
                  <td className="py-4">
                    <TierBadge tier={agent.tier} />
                  </td>
                  <td className="py-4">
                    <StatusBadge status={agent.status} />
                  </td>
                  <td className="py-4">
                    <RegimeBadge regime={agent.regime} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </motion.div>
    </div>
  );
}

function StatCard({ icon: Icon, label, value, change, color }: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  change: string;
  color: string;
}) {
  return (
    <motion.div
      whileHover={{ scale: 1.02 }}
      className="glass-panel p-6"
    >
      <div className="flex items-center justify-between mb-4">
        <div className="p-2 rounded-lg bg-sentinel-700/50">
          <Icon className="w-5 h-5 text-sentinel-400" />
        </div>
        <span className={`text-xs font-mono ${change.startsWith('+') ? 'text-green-400' : 'text-sentinel-200'}`}>
          {change}
        </span>
      </div>
      <div className={`text-2xl font-bold ${color}`}>{value}</div>
      <div className="text-sm text-sentinel-400 mt-1">{label}</div>
    </motion.div>
  );
}

function TierBadge({ tier }: { tier: number }) {
  const colors = [
    '',
    'bg-gray-500/20 text-gray-400',
    'bg-amber-700/20 text-amber-400',
    'bg-slate-400/20 text-slate-300',
    'bg-yellow-500/20 text-yellow-400',
    'bg-purple-500/20 text-purple-400',
  ];
  const labels = ['', 'Basic', 'Bronze', 'Silver', 'Gold', 'Platinum'];

  return (
    <span className={`px-2 py-1 rounded-full text-xs font-medium ${colors[tier]}`}>
      {labels[tier]}
    </span>
  );
}

function StatusBadge({ status }: { status: string }) {
  const isActive = status === 'active';
  return (
    <span className={`flex items-center gap-1.5 text-xs font-medium ${isActive ? 'text-green-400' : 'text-sentinel-200'}`}>
      <span className={`w-1.5 h-1.5 rounded-full ${isActive ? 'bg-green-400' : 'bg-sentinel-200'} ${isActive ? 'animate-pulse' : ''}`} />
      {isActive ? 'ACTIVE' : 'SILENCE'}
    </span>
  );
}

function RegimeBadge({ regime }: { regime: string }) {
  const colors: Record<string, string> = {
    Normal: 'text-green-400',
    Alert: 'text-yellow-400',
    Critical: 'text-orange-400',
    Silence: 'text-red-400',
  };
  return (
    <span className={`text-xs font-mono ${colors[regime] || 'text-sentinel-400'}`}>
      {regime.toUpperCase()}
    </span>
  );
}
