import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { FileMetadata, DecryptResult } from './types';

type Tab = 'files' | 'audit' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('files');
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [passwordSetup, setPasswordSetup] = useState(false);
  const [darkMode, setDarkMode] = useState(false);

  useEffect(() => {
    checkAuthStatus();
    const savedTheme = localStorage.getItem('vault_dark_mode');
    if (savedTheme === 'true') {
      setDarkMode(true);
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, []);

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('vault_dark_mode', String(darkMode));
  }, [darkMode]);

  function toggleDarkMode() {
    setDarkMode(!darkMode);
  }

  async function checkAuthStatus() {
    try {
      const hasPassword = await invoke<boolean>('check_auth_status');
      setPasswordSetup(hasPassword);
      setIsAuthenticated(hasPassword);
    } catch (err) {
      console.error('Error checking auth status:', err);
    } finally {
      setIsLoading(false);
    }
  }

  const handleLogin = () => {
    setIsAuthenticated(true);
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-gray-500 dark:text-gray-400">Cargando...</div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <AuthScreen 
      isSetup={passwordSetup} 
      onLogin={handleLogin}
    />;
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: 'files', label: 'Archivos' },
    { id: 'audit', label: 'Auditoría' },
    { id: 'settings', label: 'Ajustes' },
  ];

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <svg className="w-8 h-8 text-gray-800 dark:text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            <h1 className="text-xl font-semibold text-gray-800 dark:text-white">Vault</h1>
          </div>
          <div className="flex items-center gap-4">
            <button
              onClick={toggleDarkMode}
              className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              title={darkMode ? 'Modo claro' : 'Modo oscuro'}
            >
              {darkMode ? (
                <svg className="w-5 h-5 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clipRule="evenodd" />
                </svg>
              ) : (
                <svg className="w-5 h-5 text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
                </svg>
              )}
            </button>
            <button
              onClick={() => setIsAuthenticated(false)}
              className="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-white"
            >
              Cerrar sesión
            </button>
          </div>
        </div>
      </header>

      <nav className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6">
        <div className="flex gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'text-gray-900 dark:text-white border-b-2 border-gray-900 dark:border-white'
                  : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </nav>

      <main className="p-6">
        {activeTab === 'files' && <FileManager />}
        {activeTab === 'audit' && <AuditLog />}
        {activeTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

function FileManager() {
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [loading, setLoading] = useState(false);
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);
  const [decryptKey, setDecryptKey] = useState('');
  const [showDecryptInput, setShowDecryptInput] = useState(false);
  const [targetType, setTargetType] = useState<'file' | 'directory'>('file');

  async function handleEncrypt() {
    try {
      setLoading(true);
      setMessage(null);
      setGeneratedKey(null);

      if (targetType === 'file') {
        const filePath = await open({
          multiple: false,
          title: 'Seleccionar archivo a cifrar',
          directory: false,
        });

        if (!filePath) return;

        const result = await invoke<FileMetadata>('encrypt_file_cmd', {
          filePath,
        });

        setGeneratedKey(result.key);
        setMessage({
          type: 'success',
          text: `Archivo cifrado. Guarda esta clave - sin ella no podrás descifrar el archivo:`,
        });
      } else {
        const dirPath = await open({
          multiple: false,
          title: 'Seleccionar directorio a cifrar',
          directory: true,
        });

        if (!dirPath) return;

        const result = await invoke<any>('encrypt_dir_cmd', {
          inputDir: dirPath,
        });

        setGeneratedKey(result.key);
        setMessage({
          type: 'success',
          text: `Directorio cifrado. Se generó un archivo .vault. Guarda esta clave:`,
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

      if (targetType === 'file') {
        const filePath = await open({
          multiple: false,
          title: 'Seleccionar archivo .vault para descifrar',
          directory: false,
          filters: [{ name: 'Vault Files', extensions: ['vault'] }],
        });

        if (!filePath) return;

        const result = await invoke<DecryptResult>('decrypt_file_cmd', {
          filePath: filePath,
          keyBase64: decryptKey,
        });

        setMessage({ type: 'success', text: `Archivo descifrado: ${result.output_path}` });
      } else {
        const filePath = await open({
          multiple: false,
          title: 'Seleccionar archivo .vault para descifrar',
          directory: false,
          filters: [{ name: 'Vault Container', extensions: ['vault'] }],
        });

        if (!filePath) return;

        const result = await invoke<any>('decrypt_dir_cmd', {
          inputFile: filePath,
          keyBase64: decryptKey,
        });

        setMessage({ type: 'success', text: `Directorio descifrado: ${result.files_decrypted[0]}` });
      }

      setDecryptKey('');
      setShowDecryptInput(false);
    } catch (err) {
      setMessage({ type: 'error', text: `Error: Clave incorrecta o archivo no válido` });
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

  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-800 dark:text-white">Gestor de Archivos</h2>
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
        <p className="text-gray-500 dark:text-gray-400 text-sm mb-4">
          Al cifrar un archivo, se crea un archivo .vault junto con un archivo .vault-meta. Necesitas la clave para descifrarlo después.
        </p>
        
        <div className="relative mb-6">
          <div className="flex bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
            <button
              onClick={() => { setTargetType('file'); setShowDecryptInput(false); setGeneratedKey(null); }}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-all duration-200 ${
                targetType === 'file'
                  ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              <span className="flex items-center justify-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                Archivo
              </span>
            </button>
            <button
              onClick={() => { setTargetType('directory'); setShowDecryptInput(false); setGeneratedKey(null); }}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-all duration-200 ${
                targetType === 'directory'
                  ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              <span className="flex items-center justify-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                Directorio
              </span>
            </button>
          </div>
        </div>
        
        {message && (
          <div className={`p-3 rounded mb-4 text-sm ${
            message.type === 'success' ? 'bg-green-50 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300'
          }`}>
            {message.text}
          </div>
        )}

        {generatedKey && (
          <div className="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900 border border-yellow-200 dark:border-yellow-700 rounded">
            <p className="text-sm text-yellow-800 dark:text-yellow-200 mb-2 font-medium">CLAVE DE CIFRADO:</p>
            <code className="block text-sm bg-white dark:bg-gray-700 p-2 border rounded font-mono break-all text-gray-800 dark:text-gray-200">{generatedKey}</code>
            <button
              onClick={copyKey}
              className="mt-2 text-xs text-yellow-700 dark:text-yellow-400 underline"
            >
              Copiar clave
            </button>
          </div>
        )}

        {showDecryptInput && (
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Ingresa la clave para descifrar:</label>
            <input
              type="text"
              value={decryptKey}
              onChange={(e) => setDecryptKey(e.target.value)}
              placeholder="Pega la clave aquí..."
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm font-mono bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={handleEncrypt}
            disabled={loading}
            className="px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors disabled:opacity-50"
          >
            {loading ? 'Procesando...' : 'Cifrar archivo'}
          </button>
          <button
            onClick={() => { setShowDecryptInput(!showDecryptInput); setMessage(null); setDecryptKey(''); }}
            className="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 text-sm font-medium rounded hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
          >
            {showDecryptInput ? 'Cancelar' : 'Seleccionar archivo .vault'}
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
  const [events, setEvents] = useState<any[]>([]);
  const [watcherEvents, setWatcherEvents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [integrityResult, setIntegrityResult] = useState<any>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [filterDate, setFilterDate] = useState<string>('');

  useEffect(() => {
    loadEvents();
    loadWatcherEvents();
    const interval = setInterval(loadWatcherEvents, 3000);
    return () => clearInterval(interval);
  }, []);

  async function loadWatcherEvents() {
    try {
      const evts = await invoke<any[]>('get_watcher_events', { limit: 20 });
      setWatcherEvents(evts);
    } catch (err) {
      console.error('Error loading watcher events:', err);
    }
  }

  async function loadEvents() {
    try {
      setLoading(true);
      const logs = await invoke<any[]>('get_audit_logs', { limit: 100 });
      setEvents(logs);
    } catch (err) {
      console.error('Error loading audit logs:', err);
    } finally {
      setLoading(false);
    }
  }

  const filteredEvents = events.filter(event => {
    if (filterType !== 'all' && event.event_type !== filterType) return false;
    if (filterDate) {
      const eventDate = new Date(event.timestamp).toISOString().split('T')[0];
      if (eventDate !== filterDate) return false;
    }
    return true;
  });

  async function validateIntegrity() {
    try {
      const result = await invoke<any>('validate_audit_integrity');
      setIntegrityResult(result);
    } catch (err) {
      setIntegrityResult({
        is_valid: false,
        status: 'ERROR',
        last_valid_id: 0,
        failed_at: null,
        details: String(err),
      });
    }
  }

  async function repairIntegrity() {
    try {
      const result = await invoke<any>('repair_audit_integrity');
      setIntegrityResult(result);
      loadEvents();
    } catch (err) {
      setIntegrityResult({
        is_valid: false,
        status: 'ERROR',
        last_valid_id: 0,
        failed_at: null,
        details: String(err),
      });
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
        <h2 className="text-lg font-semibold text-gray-800 dark:text-white">Log de Auditoría</h2>
        <button
          onClick={validateIntegrity}
          className="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 text-sm font-medium rounded hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
        >
          Validar Integridad
        </button>
      </div>

      <div className="flex gap-4 items-center bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-600 dark:text-gray-400">Tipo:</label>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            <option value="all">Todos</option>
            <option value="encrypt">Encrypt</option>
            <option value="decrypt">Decrypt</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-600 dark:text-gray-400">Fecha:</label>
          <input
            type="date"
            value={filterDate}
            onChange={(e) => setFilterDate(e.target.value)}
            className="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          />
        </div>
        <button
          onClick={() => { setFilterType('all'); setFilterDate(''); }}
          className="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
        >
          Limpiar filtros
        </button>
      </div>

      {integrityResult && (
        <div className={`p-4 rounded border text-sm ${
          integrityResult.is_valid ? 'bg-green-50 border-green-200 text-green-700 dark:bg-green-900 dark:border-green-800 dark:text-green-300' : 'bg-red-50 border-red-200 text-red-700 dark:bg-red-900 dark:border-red-800 dark:text-red-300'
        }`}>
          <div className="font-medium mb-1">
            Estado: {integrityResult.status === 'VALID' ? 'VÁLIDA' : 'COMPROMETIDA'}
          </div>
          <div className="text-sm">{integrityResult.details}</div>
          
          {!integrityResult.is_valid && integrityResult.failed_at && (
            <div className="text-xs mt-1 opacity-75">
              Detectado en: {integrityResult.failed_at}
            </div>
          )}
          
          {!integrityResult.is_valid && (
            <button
              onClick={repairIntegrity}
              className="mt-3 px-3 py-1.5 bg-red-600 text-white text-xs font-medium rounded hover:bg-red-700 transition-colors"
            >
              Reparar automáticamente
            </button>
          )}
        </div>
      )}

      {watcherEvents.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
          <div className="px-4 py-3 bg-yellow-50 dark:bg-yellow-900 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">Eventos en tiempo real (File Watcher)</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Hora</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Tipo</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Ruta</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
                {watcherEvents.map((event: any, idx: number) => (
                  <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400">{event.timestamp || new Date().toLocaleTimeString()}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        event.event_type === 'create' ? 'bg-green-100 text-green-700 dark:bg-green-800 dark:text-green-300' :
                        event.event_type === 'modify' ? 'bg-blue-100 text-blue-700 dark:bg-blue-800 dark:text-blue-300' :
                        event.event_type === 'delete' ? 'bg-red-100 text-red-700 dark:bg-red-800 dark:text-red-300' :
                        'bg-gray-100 text-gray-700 dark:bg-gray-600 dark:text-gray-300'
                      }`}>
                        {event.event_type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400 max-w-xs truncate">{event.path}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        {filteredEvents.length === 0 ? (
          <div className="p-6 text-gray-500 dark:text-gray-400 text-center">No hay eventos registrados.</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Timestamp</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Tipo</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Ruta</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">Descripción</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
                {filteredEvents.map((event: any) => (
                  <tr key={event.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400">{formatTimestamp(event.timestamp)}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        event.event_type === 'encrypt' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' :
                        event.event_type === 'decrypt' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' :
                        'bg-gray-100 text-gray-700 dark:bg-gray-600 dark:text-gray-300'
                      }`}>
                        {event.event_type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400 max-w-xs truncate">{event.path}</td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400">{event.description}</td>
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
      <h2 className="text-lg font-semibold text-gray-800 dark:text-white">Ajustes</h2>

      {message && (
        <div className={`p-3 rounded text-sm ${
          message.type === 'success' ? 'bg-green-50 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300'
        }`}>
          {message.text}
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Algoritmo de cifrado</label>
          <select
            value={algorithm}
            onChange={(e) => handleAlgorithmChange(e.target.value)}
            className="w-full max-w-xs px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            <option value="AES-256">AES-256</option>
            <option value="ChaCha20">ChaCha20</option>
          </select>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4">
        <h3 className="font-medium text-gray-800 dark:text-white">Email de Alertas</h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Configura el email donde recibirás las alertas de actividad sospechosa.
        </p>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Email de destino para alertas</label>
          <input
            type="email"
            value={alertEmail}
            onChange={(e) => setAlertEmail(e.target.value)}
            placeholder="tu@email.com"
            className="w-full max-w-md px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
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
            className="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 text-sm font-medium rounded hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors disabled:opacity-50"
          >
            Probar email
          </button>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4">
        <h3 className="font-medium text-gray-800 dark:text-white">Monitoreo de Archivos</h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Monitorea directorios para detectar cambios en archivos.
        </p>
        
        <FileWatcherSettings />
      </div>
    </div>
  );
}

function FileWatcherSettings() {
  const [watchedPaths, setWatchedPaths] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadWatchedPaths();
  }, []);

  async function loadWatchedPaths() {
    try {
      const paths = await invoke<string[]>('get_watched_paths');
      setWatchedPaths(paths);
    } catch (err) {
      console.error('Error loading watched paths:', err);
    }
  }

  async function handleAddWatch() {
    try {
      const dirPath = await open({
        multiple: false,
        title: 'Seleccionar directorio a monitorear',
        directory: true,
      });

      if (!dirPath) return;

      setLoading(true);
      await invoke('start_watching', { path: dirPath });
      setMessage({ type: 'success', text: 'Directorio agregado al monitoreo' });
      loadWatchedPaths();
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  async function handleRemoveWatch(path: string) {
    try {
      setLoading(true);
      await invoke('stop_watching', { path });
      setMessage({ type: 'success', text: 'Directorio removido del monitoreo' });
      loadWatchedPaths();
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-4">
{message && (
        <div className={`p-3 rounded text-sm ${
          message.type === 'success' ? 'bg-green-50 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300'
        }`}>
          {message.text}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        {watchedPaths.length === 0 ? (
          <p className="text-sm text-gray-500 dark:text-gray-400">No hay directorios siendo monitoreados</p>
        ) : (
          watchedPaths.map((path) => (
            <div key={path} className="flex items-center gap-2 bg-gray-100 dark:bg-gray-700 px-3 py-2 rounded">
              <span className="text-sm text-gray-700 dark:text-gray-300 truncate max-w-xs">{path}</span>
              <button
                onClick={() => handleRemoveWatch(path)}
                className="text-red-500 hover:text-red-700 text-sm"
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>

      <button
        onClick={handleAddWatch}
        disabled={loading}
        className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 transition-colors disabled:opacity-50"
      >
        + Agregar directorio
      </button>
    </div>
  );
}

function AuthScreen({ isSetup, onLogin }: { isSetup: boolean; onLogin: () => void }) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSetupPassword() {
    if (password.length < 4) {
      setMessage({ type: 'error', text: 'La contraseña debe tener al menos 4 caracteres' });
      return;
    }
    if (password !== confirmPassword) {
      setMessage({ type: 'error', text: 'Las contraseñas no coinciden' });
      return;
    }

    setLoading(true);
    setMessage(null);

    try {
      const result = await invoke<{ success: boolean; message: string }>('setup_password', { password });
      if (result.success) {
        setMessage({ type: 'success', text: 'Contraseña configurada correctamente' });
        setTimeout(() => onLogin(), 1000);
      } else {
        setMessage({ type: 'error', text: result.message });
      }
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  async function handleLogin() {
    if (!password) {
      setMessage({ type: 'error', text: 'Ingresa tu contraseña' });
      return;
    }

    setLoading(true);
    setMessage(null);

    try {
      const result = await invoke<{ success: boolean; message: string }>('login', { password });
      if (result.success) {
        onLogin();
      } else {
        setMessage({ type: 'error', text: 'Contraseña incorrecta' });
      }
    } catch (err) {
      setMessage({ type: 'error', text: `Error: ${err}` });
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
      <div className="max-w-md w-full p-8">
        <div className="text-center mb-8">
          <svg className="w-16 h-16 mx-auto text-gray-800 dark:text-white mb-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
          <h1 className="text-2xl font-semibold text-gray-800 dark:text-white">Vault</h1>
          <p className="text-gray-500 dark:text-gray-400 mt-2">
            {isSetup ? 'Ingresa tu contraseña para acceder' : 'Configura una contraseña para proteger tu bóveda'}
          </p>
        </div>

        {message && (
          <div className={`p-3 rounded mb-4 text-sm ${
            message.type === 'success' ? 'bg-green-50 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300'
          }`}>
            {message.text}
          </div>
        )}

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              {isSetup ? 'Contraseña' : 'Nueva Contraseña'}
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Ingresa tu contraseña"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          {!isSetup && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirmar Contraseña</label>
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirma tu contraseña"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          )}

          <button
            onClick={isSetup ? handleLogin : handleSetupPassword}
            disabled={loading}
            className="w-full px-4 py-2 bg-gray-900 text-white text-sm font-medium rounded hover:bg-gray-800 transition-colors disabled:opacity-50"
          >
            {loading ? 'Procesando...' : (isSetup ? 'Iniciar Sesión' : 'Configurar Contraseña')}
          </button>
        </div>

        <p className="text-xs text-gray-400 dark:text-gray-500 text-center mt-4">
          Tu contraseña se almacena de forma segura usando Argon2
        </p>
      </div>
    </div>
  );
}

export default App;