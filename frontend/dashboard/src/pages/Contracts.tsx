import { motion } from 'framer-motion';
import { FileCode, CheckCircle, Activity } from 'lucide-react';

const contracts = [
  {
    name: 'SentinelRegistry',
    hash: 'hash-...a3f2',
    status: 'live',
    calls: 1247,
    last_call: '2 min ago',
    description: 'Agent identity + Λ state + credential lifecycle',
  },
  {
    name: 'SentinelVault',
    hash: 'hash-...b8c1',
    status: 'live',
    calls: 3892,
    last_call: '30 sec ago',
    description: 'ZK-gated capital + coherence gate + heartbeat',
  },
  {
    name: 'SentinelLearner',
    hash: 'hash-...d4e5',
    status: 'live',
    calls: 567,
    last_call: '5 min ago',
    description: 'Domain mastery + IQ milestones + epistatic params',
  },
  {
    name: 'ComplianceEngine',
    hash: 'hash-...f6a7',
    status: 'live',
    calls: 2341,
    last_call: '1 min ago',
    description: 'Protocol-level compliance hooks + tier enforcement',
  },
  {
    name: 'EpistaticController',
    hash: 'hash-...g8h9',
    status: 'live',
    calls: 89,
    last_call: '12 min ago',
    description: 'EL_state computation + contract expression modulation',
  },
  {
    name: 'ZKVerifier',
    hash: 'hash-...i0j1',
    status: 'live',
    calls: 456,
    last_call: '3 min ago',
    description: 'UltraHonk proof verification (WASM-native)',
  },
];

export default function Contracts() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Smart Contracts</h2>
        <p className="text-sentinel-400 text-sm mt-1">Casper Testnet deployment status</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {contracts.map((contract, i) => (
          <motion.div
            key={contract.name}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            className="glass-panel p-6"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-sentinel-600/30">
                  <FileCode className="w-5 h-5 text-sentinel-300" />
                </div>
                <div>
                  <div className="font-bold">{contract.name}</div>
                  <div className="text-xs text-sentinel-500 font-mono">{contract.hash}</div>
                </div>
              </div>
              <span className="flex items-center gap-1.5 text-xs text-green-400">
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
                LIVE
              </span>
            </div>

            <p className="text-sm text-sentinel-300 mb-4">{contract.description}</p>

            <div className="flex items-center gap-6 text-xs text-sentinel-400">
              <span className="flex items-center gap-1">
                <Activity className="w-3 h-3" /> {contract.calls.toLocaleString()} calls
              </span>
              <span className="flex items-center gap-1">
                <CheckCircle className="w-3 h-3" /> {contract.last_call}
              </span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
