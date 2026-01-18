import { useRef, useEffect } from 'react';
import { useSessionStore, useEditorStore, useUIStore } from '../../stores';
import { mewLanguage, mewTheme } from '../../editor/tokens';
import * as monaco from 'monaco-editor';

// Track if language has been registered
let languageRegistered = false;

function setupMewLanguage() {
  if (languageRegistered) return;

  // Register MEW language for ontology
  monaco.languages.register({ id: 'mew' });
  monaco.languages.setMonarchTokensProvider('mew', mewLanguage);

  // Register MEW-GQL language for queries (uses same tokenizer)
  monaco.languages.register({ id: 'mew-gql' });
  monaco.languages.setMonarchTokensProvider('mew-gql', mewLanguage);

  // Define theme
  monaco.editor.defineTheme('mew-dark', mewTheme);

  languageRegistered = true;
}

export function Sidebar() {
  const ontologyEditorRef = useRef<HTMLDivElement>(null);
  const queryEditorRef = useRef<HTMLDivElement>(null);
  const ontologyModelRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const queryModelRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);

  const ontologyContent = useEditorStore((state) => state.ontologyContent);
  const queryContent = useEditorStore((state) => state.queryContent);
  const setOntologyContent = useEditorStore((state) => state.setOntologyContent);
  const setQueryContent = useEditorStore((state) => state.setQueryContent);

  const loadOntology = useSessionStore((state) => state.loadOntology);
  const executeQuery = useSessionStore((state) => state.executeQuery);
  const executeSeed = useSessionStore((state) => state.executeSeed);
  const schema = useSessionStore((state) => state.schema);
  const isGeneratingSeed = useUIStore((state) => state.isGeneratingSeed);

  // Initialize Monaco editors
  useEffect(() => {
    setupMewLanguage();

    if (ontologyEditorRef.current && !ontologyModelRef.current) {
      ontologyModelRef.current = monaco.editor.create(ontologyEditorRef.current, {
        value: ontologyContent,
        language: 'mew',
        theme: 'mew-dark',
        minimap: { enabled: false },
        fontSize: 13,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
      });

      ontologyModelRef.current.onDidChangeModelContent(() => {
        const value = ontologyModelRef.current?.getValue() || '';
        setOntologyContent(value);
      });
    }

    if (queryEditorRef.current && !queryModelRef.current) {
      queryModelRef.current = monaco.editor.create(queryEditorRef.current, {
        value: queryContent,
        language: 'mew-gql',
        theme: 'mew-dark',
        minimap: { enabled: false },
        fontSize: 13,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
      });

      queryModelRef.current.onDidChangeModelContent(() => {
        const value = queryModelRef.current?.getValue() || '';
        setQueryContent(value);
      });
    }

    return () => {
      ontologyModelRef.current?.dispose();
      queryModelRef.current?.dispose();
      ontologyModelRef.current = null;
      queryModelRef.current = null;
    };
  }, []);

  // Sync editor content with store
  useEffect(() => {
    if (ontologyModelRef.current && ontologyModelRef.current.getValue() !== ontologyContent) {
      ontologyModelRef.current.setValue(ontologyContent);
    }
  }, [ontologyContent]);

  useEffect(() => {
    if (queryModelRef.current && queryModelRef.current.getValue() !== queryContent) {
      queryModelRef.current.setValue(queryContent);
    }
  }, [queryContent]);

  const handleLoadOntology = () => {
    loadOntology(ontologyContent);
  };

  const handleExecuteQuery = () => {
    // Use the window function that updates ResultsPanel
    const executeWithResults = (window as unknown as { __executeQueryWithResults?: (query: string) => void }).__executeQueryWithResults;
    if (executeWithResults) {
      executeWithResults(queryContent);
    } else {
      executeQuery(queryContent);
    }
  };

  const handleGenerateSeed = () => {
    // TODO: Implement AI-powered seed generation
    executeSeed(queryContent);
  };

  return (
    <aside id="sidebar">
      <div id="editor-panel">
        <h2>Ontology</h2>
        <div id="editor-container" ref={ontologyEditorRef} />
        <div className="editor-buttons">
          <button onClick={handleLoadOntology}>Load Ontology</button>
          <button
            className={`btn-secondary ${isGeneratingSeed ? 'btn-loading' : ''}`}
            onClick={handleGenerateSeed}
            disabled={!schema || isGeneratingSeed}
          >
            Generate Seed
          </button>
        </div>
      </div>
      <div id="query-panel">
        <h2>Query</h2>
        <div id="query-container" ref={queryEditorRef} />
        <div className="editor-buttons">
          <button onClick={handleExecuteQuery} disabled={!schema}>
            Execute Query
          </button>
        </div>
      </div>
    </aside>
  );
}
