import { motion } from 'framer-motion';
import { AlertTriangle, Clock, Filter } from 'lucide-react';

const silenceEvents = [
  {
    id: 'sil_001',
    agent_id: 'agent_002',
    block: 1234567,
    timestamp: '2026-07-04T14:32:11Z',
    psi: 0.45,
    delta: 0.60,
    gap: 0.15,
    reason: 'Coherence below threshold',
    planes: { p_t: 0.52, i_t: 0.41, c_t: 0.38, s_t: 0.55, w_t: 0.42 },
    regime: 'Alert',
  },
  {
    id: 'sil_002',
    agent_id: 'agent_005',
    block: 1234562,
    timestamp: '2026-07-04T14:31:33Z',
    psi: 0.0,
    delta: 0.58,
    gap: 0.58,
    reason: 'World model anomaly (z-score > 3σ)',
    planes: { p_t: 0.65, i_t: 0.70, c_t: 0.60, s_t: 0.55, w_t: 0.0 },
    regime: 'Silence',
  },
  {
    id: 'sil_003',
    agent_id: 'agent_008',
    block: 1234551,
    timestamp: '2026-07-04T14:30:05Z',
    psi: 0.38,
    delta: 0.62,
    gap: 0.24,
    reason: 'Inferential inconsistency across chains',
    planes: { p_t: 0.55, i_t: 0.20, c_t: 0.45, s_t: 0.50, w_t: 0.30 },
    regime: 'Critical',
  },
];

export default function SilenceLog() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">SILENCE Log</h2>
          <p className="text-sentinel-400 text-sm mt-1">Structured silence events with causal tracing</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 bg-sentinel-800 rounded-lg text-sm hover:bg-sentinel-700 transition-colors">
          <Filter className="w-4 h-4" /> Filter
        </button>
      </div>

      <div className="space-y-4">
        {silenceEvents.map((event, i) => (
          <motion.div
            key={event.id}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: i * 0.1 }}
            className="glass-panel p-6 hover:border-sentinel-200/30 transition-all"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${
                  event.regime === 'Silence' ? 'bg-red-500/20' :
                  event.regime === 'Critical' ? 'bg-orange-500/20' :
                  'bg-yellow-500/20'
                }`}>
                  <AlertTriangle className={`w-5 h-5 ${
                    event.regime === 'Silence' ? 'text-red-400' :
                    event.regime === 'Critical' ? 'text-orange-400' :
                    'text-yellow-400'
                  }`} />
                </div>
                <div>
                  <div className="font-mono text-sm">{event.id}</div>
                  <div className="text-xs text-sentinel-400 flex items-center gap-2 mt-1">
                    <span className="text-sentinel-300">{event.agent_id}</span>
                    <span>•</span>
                    <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> {event.timestamp}</span>
                    <span>•</span>
                    <span>Block #{event.block}</span>
                  </div>
                </div>
              </div>
              <span className={`px-3 py-1 rounded-full text-xs font-bold ${
                event.regime === 'Silence' ? 'bg-red-500/20 text-red-400' :
                event.regime === 'Critical' ? 'bg-orange-500/20 text-orange-400' :
                'bg-yellow-500/20 text-yellow-400'
              }`}>
                {event.regime.toUpperCase()}
              </span>
            </div>

            <div className="grid grid-cols-3 gap-4 mb-4">
              <div className="bg-sentinel-900/50 p-3 rounded-lg">
                <div className="text-xs text-sentinel-400 mb-1">Ψ (Coherence)</div>
                <div className="text-lg font-bold text-sentinel-200">{event.psi.toFixed(2)}</div>
              </div>
              <div className="bg-sentinel-900/50 p-3 rounded-lg">
                <div className="text-xs text-sentinel-400 mb-1">Δ (Threshold)</div>
                <div className="text-lg font-bold text-sentinel-300">{event.delta.toFixed(2)}</div>
              </div>
              <div className="bg-sentinel-900/50 p-3 rounded-lg">
                <div className="text-xs text-sentinel-400 mb-1">Gap</div>
                <div className="text-lg font-bold text-red-400">+{event.gap.toFixed(2)}</div>
              </div>
            </div>

            <div className="text-sm text-sentinel-300 mb-3">{event.reason}</div>

            {/* Plane breakdown */}
            <div className="grid grid-cols-5 gap-2">
              {Object.entries(event.planes).map(([plane, value]) => (
                <div key={plane} className="text-center">
                  <div className="text-xs text-sentinel-500 mb-1">{plane.replace('_', ' ')}</div>
                  <div className={`text-sm font-mono font-bold ${
                    value === 0 ? 'text-red-400' : value < 0.4 ? 'text-yellow-400' : 'text-green-400'
                  }`}>
                    {value.toFixed(2)}
                  </div>
                  <div className="w-full h-1 bg-sentinel-700 rounded-full mt-1 overflow-hidden">
                    <div
                      className="h-full rounded-full"
                      style={{
                        width: `${value * 100}%`,
                        backgroundColor: value === 0 ? '#ef4444' : value < 0.4 ? '#eab308' : '#10b981',
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
