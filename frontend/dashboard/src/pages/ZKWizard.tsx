import React from 'react';
import { motion } from 'framer-motion';
import { Shield, CheckCircle, Loader, FileKey, Fingerprint } from 'lucide-react';

type CircuitType = 'behavioral_integrity' | 'causal_identity' | 'sentinel_compliance';

const circuits: { id: CircuitType; name: string; constraints: number; description: string }[] = [
  {
    id: 'behavioral_integrity',
    name: 'Behavioral Integrity Credential (BIC)',
    constraints: 12000,
    description: 'Proves agent has operated with sustained coherence above threshold',
  },
  {
    id: 'causal_identity',
    name: 'Causal Identity Proof (CIP)',
    constraints: 8500,
    description: 'Proves behavioral signature matches historical baseline for recovery',
  },
  {
    id: 'sentinel_compliance',
    name: 'Sentinel Compliance',
    constraints: 6000,
    description: 'Enforces tier-based transaction limits and jurisdiction rules',
  },
];

export default function ZKWizard() {
  const [selectedCircuit, setSelectedCircuit] = React.useState<CircuitType | null>(null);
  const [step, setStep] = React.useState(0);
  const [generating, setGenerating] = React.useState(false);
  const [proof, setProof] = React.useState<string | null>(null);

  const handleGenerate = async () => {
    setGenerating(true);
    // Simulate proof generation
    await new Promise(r => setTimeout(r, 3000));
    setProof(`0x${Array(64).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join('')}`);
    setGenerating(false);
    setStep(3);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">ZK Proof Wizard</h2>
        <p className="text-sentinel-400 text-sm mt-1">Generate zero-knowledge behavioral credentials</p>
      </div>

      {/* Stepper */}
      <div className="flex items-center gap-4">
        {['Select Circuit', 'Configure Inputs', 'Generate Proof', 'Verify'].map((label, i) => (
          <React.Fragment key={label}>
            <div className={`flex items-center gap-2 ${i <= step ? 'text-white' : 'text-sentinel-500'}`}>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold ${
                i < step ? 'bg-green-500' : i === step ? 'bg-sentinel-300' : 'bg-sentinel-700'
              }`}>
                {i < step ? <CheckCircle className="w-4 h-4" /> : i + 1}
              </div>
              <span className="text-sm hidden md:block">{label}</span>
            </div>
            {i < 3 && <div className={`flex-1 h-0.5 ${i < step ? 'bg-green-500' : 'bg-sentinel-700'}`} />}
          </React.Fragment>
        ))}
      </div>

      {/* Step Content */}
      <div className="glass-panel p-8">
        {step === 0 && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="space-y-4">
            <h3 className="text-lg font-semibold mb-4">Select Circuit Type</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {circuits.map((circuit) => (
                <button
                  key={circuit.id}
                  onClick={() => { setSelectedCircuit(circuit.id); setStep(1); }}
                  className={`p-6 rounded-xl border-2 text-left transition-all ${
                    selectedCircuit === circuit.id
                      ? 'border-sentinel-300 bg-sentinel-300/10'
                      : 'border-sentinel-700 hover:border-sentinel-500'
                  }`}
                >
                  <Shield className="w-8 h-8 text-sentinel-300 mb-3" />
                  <div className="font-semibold mb-2">{circuit.name}</div>
                  <div className="text-xs text-sentinel-400 mb-3">{circuit.constraints.toLocaleString()} constraints</div>
                  <div className="text-sm text-sentinel-300">{circuit.description}</div>
                </button>
              ))}
            </div>
          </motion.div>
        )}

        {step === 1 && selectedCircuit && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="space-y-6">
            <h3 className="text-lg font-semibold">Configure Private Inputs</h3>
            <div className="space-y-4">
              <InputField label="Agent Secret Key" type="password" placeholder="Enter secret key..." />
              <InputField label="Behavioral History Vector" placeholder="[0.82, 0.75, 0.91, ...]" />
              <InputField label="Manipulation Scores" placeholder="[0, 0, 0, 1, 0, ...]" />
              <InputField label="Lambda Trace" placeholder="[0.1, 0.3, 0.5, 0.8, 1.2, ...]" />
            </div>
            <div className="flex gap-4">
              <button
                onClick={() => setStep(0)}
                className="px-6 py-3 rounded-lg bg-sentinel-700 hover:bg-sentinel-600 transition-colors"
              >
                Back
              </button>
              <button
                onClick={() => setStep(2)}
                className="px-6 py-3 rounded-lg bg-sentinel-300 hover:bg-sentinel-400 text-white transition-colors"
              >
                Continue
              </button>
            </div>
          </motion.div>
        )}

        {step === 2 && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center py-12">
            {generating ? (
              <div className="space-y-4">
                <Loader className="w-12 h-12 animate-spin mx-auto text-sentinel-300" />
                <div className="text-lg font-semibold">Generating ZK Proof...</div>
                <div className="text-sm text-sentinel-400">This may take 30-60 seconds</div>
                <div className="w-64 h-2 bg-sentinel-700 rounded-full mx-auto overflow-hidden">
                  <motion.div
                    className="h-full bg-sentinel-300"
                    initial={{ width: 0 }}
                    animate={{ width: '100%' }}
                    transition={{ duration: 3 }}
                  />
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="text-lg font-semibold">Ready to Generate</div>
                <div className="text-sm text-sentinel-400">Circuit: {selectedCircuit}</div>
                <button
                  onClick={handleGenerate}
                  className="px-8 py-4 rounded-lg bg-sentinel-300 hover:bg-sentinel-400 text-white font-bold transition-all"
                >
                  Generate Proof
                </button>
              </div>
            )}
          </motion.div>
        )}

        {step === 3 && proof && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="space-y-6">
            <div className="flex items-center gap-3 text-green-400">
              <CheckCircle className="w-8 h-8" />
              <div className="text-lg font-bold">Proof Generated Successfully</div>
            </div>

            <div className="bg-sentinel-900/50 p-4 rounded-lg font-mono text-sm break-all border border-sentinel-700">
              <div className="text-sentinel-400 mb-2">Proof:</div>
              {proof}
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="glass-panel p-4">
                <div className="text-xs text-sentinel-400 mb-1">Nullifier</div>
                <div className="font-mono text-sm">0x{proof.slice(0, 32)}</div>
              </div>
              <div className="glass-panel p-4">
                <div className="text-xs text-sentinel-400 mb-1">Tier</div>
                <div className="font-bold text-sentinel-300">Platinum (5)</div>
              </div>
            </div>

            <div className="flex gap-4">
              <button
                onClick={() => { setStep(0); setProof(null); setSelectedCircuit(null); }}
                className="px-6 py-3 rounded-lg bg-sentinel-700 hover:bg-sentinel-600 transition-colors"
              >
                Generate Another
              </button>
              <button className="px-6 py-3 rounded-lg bg-green-600 hover:bg-green-700 text-white transition-colors flex items-center gap-2">
                <Fingerprint className="w-4 h-4" />
                Submit to Contract
              </button>
            </div>
          </motion.div>
        )}
      </div>
    </div>
  );
}

function InputField({ label, type = 'text', placeholder }: { label: string; type?: string; placeholder: string }) {
  return (
    <div>
      <label className="block text-sm font-medium text-sentinel-300 mb-2">{label}</label>
      <input
        type={type}
        placeholder={placeholder}
        className="w-full px-4 py-3 bg-sentinel-900 border border-sentinel-700 rounded-lg text-white placeholder-sentinel-600 focus:outline-none focus:border-sentinel-300 font-mono text-sm"
      />
    </div>
  );
}
