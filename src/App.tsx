import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { CryptoStats, AuditEvent, EncryptResult } from './types';

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
  const [stats, setStats] = useState<CryptoStats>({ files_encrypted: 0, dirs_watched: 0, audit_events: 0 });
  const [algorithm, setAlgorithm] = useState<string>('AES-256');
  const [emailConfigured, setEmailConfigured] = useState<boolean>(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboardData();
  }, []);

  async function loadDashboardData() {
    try {
      setLoading(true);
      const [statsData, algo, email] = await Promise.all([
        invoke<CryptoStats>('get_stats'),
        invoke<string>('get_algorithm'),
        invoke<boolean>('is_email_configured'),
      ]);
      setStats(statsData);
      setAlgorithm(algo);
      setEmailConfigured(email);
    } catch (err) {
      console.error('Error loading dashboard:', err);
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return <div className="p-6 text-gray-500">Cargando...</div>;
  }

  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Dashboard</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">{stats.audit_events}</div>
          <div className="text-sm text-gray-500">Eventos de auditoría</div>
        </div>
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">{stats.dirs_watched}</div>
          <div className="text-sm text-gray-500">Directorios vigilados</div>
        </div>
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <div className="text-2xl font-bold text-gray-900">-</div>
          <div className="text-sm text-gray-500">Archivos cifrados</div>
        </div>
      </div>
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <h3 className="font-medium text-gray-800 mb-4">Estado del sistema</h3>
        <div className="space-y-2 text-sm text-gray-600">
          <p>Cifrado: {algorithm}</p>
          <p>Email: {emailConfigured ? 'Configurado' : 'No configurado'}</p>
          <p>USB: No configurado</p>
        </div>
      </div>
    </div>
  );
}

import type { DirEncryptResult, DirDecryptResult } from './types';

function FileManager() {
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [loading, setLoading] = useState(false);
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);
  const [decryptKey, setDecryptKey] = useState('');
  const [showDecryptInput, setShowDecryptInput] = useState(false);
  const [mode, setMode] = useState<'file' | 'dir'>('file');

  async function handleEncrypt() {
    try {
      setLoading(true);
      setMessage(null);
      setGeneratedKey(null);

      if (mode === 'file') {
        const inputPath = await open({
          multiple: false,
          title: 'Seleccionar archivo a cifrar',
          directory: false,
        });

        if (!inputPath) return;

        const outputPath = await save({
          title: 'Guardar archivo cifrado',
          defaultPath: (inputPath as string) + '.encrypted',
        });

        if (!outputPath) return;

        const result = await invoke<EncryptResult>('encrypt_file_cmd', {
          inputPath,
          outputPath,
        });

        setGeneratedKey(result.key);
        setMessage({
          type: 'success',
          text: `Archivo cifrado exitosamente. Guarda esta clave - la necesitarás para descifrar:`,
        });
      } else {
        const inputPath = await open({
          multiple: false,
          title: 'Seleccionar directorio a cifrar',
          directory: true,
        });

        if (!inputPath) return;

        const outputPath = await save({
          title: 'Guardar directorio cifrado',
          defaultPath: (inputPath as string) + '_encrypted',
        });

        if (!outputPath) return;

        const result = await invoke<DirEncryptResult>('encrypt_dir_cmd', {
          inputDir: inputPath,
          outputDir: outputPath,
        });

        setGeneratedKey(result.key);
        setMessage({
          type: 'success',
          text: `Directorio cifrado (${result.files_encrypted.length} archivos). Guarda esta clave - la necesitarás para descifrar todos:`,
        });
      }
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  async function handleDecrypt() {
    if (!decryptKey.trim()) {
      setMessage({ type: 'error', text: 'Por favor ingresa la clave de descifrado' });
      return;
    }
    
    try {
      setLoading(true);
      setMessage(null);

      if (mode === 'file') {
        const inputPath = await open({
          multiple: false,
          title: 'Seleccionar archivo cifrado',
          directory: false,
        });

        if (!inputPath) return;

        const outputPath = await save({
          title: 'Guardar archivo descifrado',
        });

        if (!outputPath) return;

        await invoke('decrypt_file_cmd', {
          inputPath,
          outputPath,
          keyBase64: decryptKey,
        });

        setMessage({ type: 'success', text: 'Archivo descifrado exitosamente.' });
        setDecryptKey('');
      } else {
        const inputPath = await open({
          multiple: false,
          title: 'Seleccionar directorio cifrado',
          directory: true,
        });

        if (!inputPath) return;

        const outputPath = await save({
          title: 'Guardar directorio descifrado',
          defaultPath: (inputPath as string) + '_decrypted',
        });

        if (!outputPath) return;

        const result = await invoke<DirDecryptResult>('decrypt_dir_cmd', {
          inputDir: inputPath,
          outputDir: outputPath,
          keyBase64: decryptKey,
        });

        setMessage({ type: 'success', text: `Directorio descifrado (${result.files_decrypted.length} archivos).` });
        setDecryptKey('');
      }
    } catch (err) {
      setMessage({ type: 'error', text: `Error: Clave incorrecta o archivo dañado` });
    } finally {
      setLoading(false);
    }
  }

  function copyKey() {
    if (generatedKey) {
      navigator.clipboard.writeText(generatedKey);
      setMessage({ type: 'success', text: 'Clave copiada al portapapeles' });
    }
  }

  function toggleMode(newMode: 'file' | 'dir') {
    setMode(newMode);
    setMessage(null);
    setGeneratedKey(null);
  }

  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Gestor de Archivos</h2>
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <div className="flex gap-2 mb-4">
          <button
            onClick={() => toggleMode('file')}
            className={`px-3 py-1 text-sm rounded ${
              mode === 'file' ? 'bg-gray-900 text-white' : 'bg-gray-100 text-gray-700'
            }`}
          >
            Archivo
          </button>
          <button
            onClick={() => toggleMode('dir')}
            className={`px-3 py-1 text-sm rounded ${
              mode === 'dir' ? 'bg-gray-900 text-white' : 'bg-gray-100 text-gray-700'
            }`}
          >
            Directorio
          </button>
        </div>
        
        <p className="text-gray-500 text-sm mb-4">
          {mode === 'file' 
            ? 'Al cifrar, el programa te mostrará una clave que debes guardar. Sin esa clave no podrás descifrar el archivo.'
            : 'Al cifrar un directorio, todos los archivos se cifrarán con una sola clave. Con esa clave podrás descifrar todos los archivos.'}
        </p>
        
        {message && (
          <div className={`p-3 rounded mb-4 text-sm ${
            message.type === 'success' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
          }`}>
            {message.text}
          </div>
        )}

        {generatedKey && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
            <p className="text-sm text-yellow-800 mb-2 font-medium">CLAVE DE CIFRADO:</p>
            <code className="block text-sm bg-white p-2 border rounded font-mono break-all">{generatedKey}</code>
            <button
              onClick={copyKey}
              className="mt-2 text-xs text-yellow-700 underline"
            >
              Copiar clave
            </button>
          </div>
        )}

        {showDecryptInput && (
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              {mode === 'dir' ? 'Ingresa la clave para descifrar todos los archivos:' : 'Ingresa la clave de descifrado:'}
            </label>
            <input
              type="text"
              value={decryptKey}
              onChange={(e) => setDecryptKey(e.target.value)}
              placeholder="Pega la clave aquí..."
              className="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
            />
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={handleEncrypt}
            disabled={loading}
            className="px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors disabled:opacity-50"
          >
            {loading ? 'Procesando...' : (mode === 'file' ? 'Cifrar archivo' : 'Cifrar directorio')}
          </button>
          <button
            onClick={() => { setShowDecryptInput(!showDecryptInput); setMessage(null); }}
            className="px-4 py-2 bg-white border border-gray-300 text-gray-700 text-sm font-medium rounded hover:bg-gray-50 transition-colors"
          >
            {showDecryptInput ? 'Cancelar' : (mode === 'file' ? 'Descifrar archivo' : 'Descifrar directorio')}
          </button>
          {showDecryptInput && (
            <button
              onClick={handleDecrypt}
              disabled={loading || !decryptKey.trim()}
              className="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded hover:bg-green-700 transition-colors disabled:opacity-50"
            >
              Descifrar
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function AuditLog() {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [integrityStatus, setIntegrityStatus] = useState<boolean | null>(null);

  useEffect(() => {
    loadEvents();
  }, []);

  async function loadEvents() {
    try {
      setLoading(true);
      const logs = await invoke<AuditEvent[]>('get_audit_logs', { limit: 100 });
      setEvents(logs);
    } catch (err) {
      console.error('Error loading audit logs:', err);
    } finally {
      setLoading(false);
    }
  }

  async function validateIntegrity() {
    try {
      const isValid = await invoke<boolean>('validate_audit_integrity');
      setIntegrityStatus(isValid);
    } catch (err) {
      setIntegrityStatus(false);
    }
  }

  function formatTimestamp(timestamp: string) {
    try {
      return new Date(timestamp).toLocaleString('es-ES');
    } catch {
      return timestamp;
    }
  }

  if (loading) {
    return <div className="p-6 text-gray-500">Cargando...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-800">Log de Auditoría</h2>
        <button
          onClick={validateIntegrity}
          className="px-4 py-2 bg-white border border-gray-300 text-gray-700 text-sm font-medium rounded hover:bg-gray-50 transition-colors"
        >
          Validar Integridad
        </button>
      </div>

      {integrityStatus !== null && (
        <div className={`p-3 rounded text-sm ${
          integrityStatus ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
        }`}>
          Integridad del log: {integrityStatus ? 'VÁLIDA' : 'COMPROMETIDA'}
        </div>
      )}

      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        {events.length === 0 ? (
          <div className="p-6 text-gray-500 text-center">No hay eventos registrados.</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Timestamp</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Tipo</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Ruta</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Descripción</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {events.map((event) => (
                  <tr key={event.id} className="hover:bg-gray-50">
                    <td className="px-4 py-3 text-gray-600">{formatTimestamp(event.timestamp)}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        event.event_type === 'encrypt' ? 'bg-blue-100 text-blue-700' :
                        event.event_type === 'decrypt' ? 'bg-green-100 text-green-700' :
                        'bg-gray-100 text-gray-700'
                      }`}>
                        {event.event_type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-gray-600 max-w-xs truncate">{event.path}</td>
                    <td className="px-4 py-3 text-gray-600">{event.description}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function Settings() {
  const [algorithm, setAlgorithmState] = useState<string>('AES-256');
  const [alertEmail, setAlertEmail] = useState<string>('');
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  async function loadSettings() {
    try {
      const [algo, email] = await Promise.all([
        invoke<string>('get_algorithm'),
        invoke<string>('get_alert_email'),
      ]);
      setAlgorithmState(algo);
      setAlertEmail(email);
    } catch (err) {
      console.error('Error loading settings:', err);
    }
  }

  async function handleAlgorithmChange(newAlgo: string) {
    try {
      await invoke('set_algorithm', { algorithm: newAlgo });
      setAlgorithmState(newAlgo);
      setMessage({ type: 'success', text: 'Algoritmo actualizado' });
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    }
  }

  async function handleSaveEmail() {
    try {
      setLoading(true);
      setMessage(null);

      if (!alertEmail.trim()) {
        setMessage({ type: 'error', text: 'Ingresa un email de destino para alertas' });
        return;
      }

      await invoke('set_alert_email', { email: alertEmail });
      setMessage({ type: 'success', text: 'Email de alertas configurado correctamente' });
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  async function handleTestEmail() {
    try {
      setLoading(true);
      setMessage(null);

      if (!alertEmail.trim()) {
        setMessage({ type: 'error', text: 'Primero guarda el email de destino' });
        return;
      }

      await invoke('test_email', { to: [alertEmail] });
      setMessage({ type: 'success', text: 'Email de prueba enviado correctamente' });
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800">Ajustes</h2>

      {message && (
        <div className={`p-3 rounded text-sm ${
          message.type === 'success' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
        }`}>
          {message.text}
        </div>
      )}

      <div className="bg-white p-6 rounded-lg border border-gray-200 space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Algoritmo de cifrado</label>
          <select
            value={algorithm}
            onChange={(e) => handleAlgorithmChange(e.target.value)}
            className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded text-sm"
          >
            <option value="AES-256">AES-256</option>
            <option value="ChaCha20">ChaCha20</option>
          </select>
        </div>
      </div>

      <div className="bg-white p-6 rounded-lg border border-gray-200 space-y-4">
        <h3 className="font-medium text-gray-800">Email de Alertas</h3>
        <p className="text-sm text-gray-500">
          Configura el email donde recibirás las alertas de actividad sospechosa.
        </p>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Email de destino para alertas</label>
          <input
            type="email"
            value={alertEmail}
            onChange={(e) => setAlertEmail(e.target.value)}
            placeholder="tu@email.com"
            className="w-full max-w-md px-3 py-2 border border-gray-300 rounded text-sm"
          />
        </div>

        <div className="flex gap-3 pt-2">
          <button
            onClick={handleSaveEmail}
            disabled={loading}
            className="px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors disabled:opacity-50"
          >
            Guardar email
          </button>
          <button
            onClick={handleTestEmail}
            disabled={loading}
            className="px-4 py-2 bg-white border border-gray-300 text-gray-700 text-sm font-medium rounded hover:bg-gray-50 transition-colors disabled:opacity-50"
          >
            Probar email
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;