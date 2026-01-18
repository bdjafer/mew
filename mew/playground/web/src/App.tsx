import { useEffect, useRef } from 'react';
import { WasmProvider, useWasm } from './providers/WasmProvider';
import { Header } from './components/layout/Header';
import { Sidebar } from './components/layout/Sidebar';
import { ResizeHandle } from './components/layout/ResizeHandle';
import { Visualization } from './components/visualization/Visualization';
import { DetailPanel } from './components/layout/DetailPanel';
import { ChatPanel } from './components/chat/ChatPanel';
import { SettingsModal } from './components/settings/SettingsModal';
import { useChatStore, useUIStore, useSessionStore, useEditorStore } from './stores';
import { DEFAULT_SEED } from './stores/editor';

function AppContent() {
  const { isReady } = useWasm();
  const loadSettings = useChatStore((state) => state.loadSettings);
  const loadHistory = useChatStore((state) => state.loadHistory);
  const sidebarWidth = useUIStore((state) => state.sidebarWidth);
  const loadOntology = useSessionStore((state) => state.loadOntology);
  const executeSeed = useSessionStore((state) => state.executeSeed);
  const ontologyContent = useEditorStore((state) => state.ontologyContent);
  const initializedRef = useRef(false);

  useEffect(() => {
    loadSettings();
    loadHistory();
  }, [loadSettings, loadHistory]);

  // Auto-load ontology and seed data on initialization
  useEffect(() => {
    if (isReady && !initializedRef.current) {
      initializedRef.current = true;
      const result = loadOntology(ontologyContent);
      if (result.success) {
        executeSeed(DEFAULT_SEED);
      }
    }
  }, [isReady, loadOntology, executeSeed, ontologyContent]);

  return (
    <div id="app">
      <Header />
      <main
        id="main"
        style={{ '--sidebar-width': `${sidebarWidth}px` } as React.CSSProperties}
      >
        <Sidebar />
        <ResizeHandle />
        <Visualization />
        <DetailPanel />
      </main>
      <ChatPanel />
      <SettingsModal />
    </div>
  );
}

export default function App() {
  return (
    <WasmProvider>
      <AppContent />
    </WasmProvider>
  );
}
