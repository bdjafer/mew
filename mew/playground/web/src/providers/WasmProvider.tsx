import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { initWasm } from '../api';

interface WasmContextValue {
  isReady: boolean;
  error: string | null;
}

const WasmContext = createContext<WasmContextValue | null>(null);

export function useWasm() {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error('useWasm must be used within WasmProvider');
  }
  return context;
}

interface WasmProviderProps {
  children: ReactNode;
}

export function WasmProvider({ children }: WasmProviderProps) {
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then(() => setIsReady(true))
      .catch((err) => setError(err.message || 'Failed to initialize WASM'));
  }, []);

  if (error) {
    return (
      <div className="wasm-error">
        <h2>Failed to load WASM module</h2>
        <p>{error}</p>
      </div>
    );
  }

  if (!isReady) {
    return (
      <div className="wasm-loading">
        <p>Loading WASM module...</p>
      </div>
    );
  }

  return <WasmContext.Provider value={{ isReady, error }}>{children}</WasmContext.Provider>;
}
