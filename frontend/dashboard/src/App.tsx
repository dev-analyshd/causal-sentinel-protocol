import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import AgentExplorer from './pages/AgentExplorer';
import ZKWizard from './pages/ZKWizard';
import SilenceLog from './pages/SilenceLog';
import Contracts from './pages/Contracts';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="agents" element={<AgentExplorer />} />
          <Route path="zk" element={<ZKWizard />} />
          <Route path="silence" element={<SilenceLog />} />
          <Route path="contracts" element={<Contracts />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
