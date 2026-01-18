import { useRef, useEffect } from 'react';
import * as monaco from 'monaco-editor';
import { mewLanguage, mewTheme } from '../../editor/tokens';
import { createCompletionProvider } from '../../editor/completions';
import { useSessionStore } from '../../stores';

interface MonacoEditorProps {
  type: 'ontology' | 'query';
  value: string;
  onChange: (value: string) => void;
  height?: string;
}

// Track if language has been registered globally
let languageRegistered = false;

// Register MEW language once
function registerMewLanguage(sessionIdRef: { current: number | null }) {
  if (languageRegistered) return;

  monaco.languages.register({ id: 'mew' });
  monaco.languages.setMonarchTokensProvider('mew', mewLanguage);
  monaco.editor.defineTheme('mew-dark', mewTheme);
  monaco.languages.registerCompletionItemProvider('mew', createCompletionProvider(sessionIdRef));

  languageRegistered = true;
}

export function MonacoEditor({ type, value, onChange, height = '100%' }: MonacoEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const sessionIdRef = useRef<number | null>(null);
  const sessionId = useSessionStore((state) => state.sessionId);

  // Keep sessionIdRef in sync
  useEffect(() => {
    sessionIdRef.current = sessionId;
  }, [sessionId]);

  // Initialize editor
  useEffect(() => {
    if (!containerRef.current) return;

    // Register language before creating editor
    registerMewLanguage(sessionIdRef);

    const editor = monaco.editor.create(containerRef.current, {
      value,
      language: 'mew',
      theme: 'mew-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: type === 'ontology' ? 'on' : 'off',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      tabSize: 2,
    });

    editorRef.current = editor;

    // Listen for changes
    const disposable = editor.onDidChangeModelContent(() => {
      const newValue = editor.getValue();
      onChange(newValue);
    });

    return () => {
      disposable.dispose();
      editor.dispose();
      editorRef.current = null;
    };
  }, []); // Only run once on mount

  // Update editor value when prop changes (from external source)
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;

    const currentValue = editor.getValue();
    if (value !== currentValue) {
      // Preserve cursor position
      const position = editor.getPosition();
      editor.setValue(value);
      if (position) {
        editor.setPosition(position);
      }
    }
  }, [value]);

  return (
    <div
      ref={containerRef}
      style={{ height, width: '100%' }}
    />
  );
}
