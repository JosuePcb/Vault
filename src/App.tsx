import { useState } from 'react';

type Tab = 'dashboard' | 'files' | 'audit' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');

  const tabs: { id: Tab; label: string }[] = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'files', label: 'Archivos' },
    { id: 'audit', label: 'Auditoría' },
    { id: 'settings', label: 'Ajustes' },
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center gap-3">
          <svg className="w-8 h-8 text-gray-800" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
          <h1 className="text-xl font-semibold text-gray-800">Vault</h1>
        </div>
      </header>

      <nav className="bg-white border-b border-gray-200 px-6">
        <div className="flex gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'text-gray-900 border-b-2 border-gray-900'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </nav>

      <main className="p-6">
        {activeTab === 'dashboard' && <Dashboard />}
        {activeTab === 'files' && <FileManager />}
        {activeTab === 'audit' && <AuditLog />}
        {activeTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

function Dashboard() {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Dashboard</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">0</div>
          <div className="text-sm text-gray-500">Archivos cifrados</div>
        </div>
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">0</div>
          <div className="text-sm text-gray-500">Directorios vigilados</div>
        </div>
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">0</div>
          <div className="text-sm text-gray-500">Eventos de auditoría</div>
        </div>
      </div>
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <h3 className="font-medium text-gray-800 mb-4">Estado del sistema</h3>
        <div className="space-y-2 text-sm text-gray-600">
          <p>Cifrado: AES-256 (configurable)</p>
          <p>Email: No configurado</p>
          <p>USB: No configurado</p>
        </div>
      </div>
    </div>
  );
}

function FileManager() {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Gestor de Archivos</h2>
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <p className="text-gray-500 text-sm mb-4">Selecciona archivos o directorios para cifrar o descifrar.</p>
        <div className="flex gap-3">
          <button className="px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors">
            Seleccionar archivo
          </button>
          <button className="px-4 py-2 bg-white border border-gray-300 text-gray-700 text-sm font-medium rounded hover:bg-gray-50 transition-colors">
            Seleccionar directorio
          </button>
        </div>
      </div>
    </div>
  );
}

function AuditLog() {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Log de Auditoría</h2>
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <p className="text-gray-500 text-sm">No hay eventos registrados.</p>
      </div>
    </div>
  );
}

function Settings() {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Ajustes</h2>
      <div className="bg-white p-6 rounded-lg border border-gray-200 space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Algoritmo de cifrado</label>
          <select className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded text-sm">
            <option>AES-256</option>
            <option>ChaCha20</option>
          </select>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Email de alertas</label>
          <input type="email" placeholder="tu@email.com" className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded text-sm" />
        </div>
        <button className="px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors">
          Guardar ajustes
        </button>
      </div>
    </div>
  );
}

export default App;
