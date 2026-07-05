import { useState } from 'react';
import { Outlet, NavLink } from 'react-router-dom';
import { Shield, Activity, Users, Zap, FileCode, Menu } from 'lucide-react';

export default function Layout() {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const navItems = [
    { to: '/', icon: Activity, label: 'Dashboard' },
    { to: '/agents', icon: Users, label: 'Agents' },
    { to: '/zk', icon: Shield, label: 'ZK Wizard' },
    { to: '/silence', icon: Zap, label: 'Silence Log' },
    { to: '/contracts', icon: FileCode, label: 'Contracts' },
  ];

  return (
    <div className="min-h-screen flex bg-sentinel-900">
      {/* Sidebar */}
      <aside className={`fixed inset-y-0 left-0 z-50 w-64 bg-sentinel-800 border-r border-sentinel-700 transform transition-transform duration-300 ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'} lg:translate-x-0 lg:static`}>
        <div className="p-6 border-b border-sentinel-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-sentinel-300 to-sentinel-400 flex items-center justify-center">
              <Shield className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="font-bold text-lg tracking-tight">Causal Sentinel</h1>
              <p className="text-xs text-sentinel-400 font-mono">PROTOCOL v0.1.0</p>
            </div>
          </div>
        </div>

        <nav className="p-4 space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `flex items-center gap-3 px-4 py-3 rounded-lg transition-all ${
                  isActive
                    ? 'bg-sentinel-600 text-white shadow-lg shadow-sentinel-500/20'
                    : 'text-sentinel-400 hover:bg-sentinel-700 hover:text-white'
                }`
              }
              onClick={() => setSidebarOpen(false)}
            >
              <item.icon className="w-5 h-5" />
              <span className="font-medium">{item.label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-sentinel-700">
          <div className="glass-panel p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-sentinel-400 font-mono">NETWORK</span>
              <span className="text-xs text-green-400 font-mono">● LIVE</span>
            </div>
            <div className="text-sm font-mono">Casper Testnet</div>
            <div className="text-xs text-sentinel-400 font-mono mt-1">Block #1,234,567</div>
          </div>
        </div>
      </aside>

      {/* Mobile overlay */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-16 bg-sentinel-800/50 backdrop-blur-md border-b border-sentinel-700 flex items-center justify-between px-6 lg:hidden">
          <button
            onClick={() => setSidebarOpen(true)}
            className="p-2 rounded-lg hover:bg-sentinel-700"
          >
            <Menu className="w-6 h-6" />
          </button>
          <span className="font-bold">Causal Sentinel</span>
          <div className="w-10" />
        </header>

        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
