import * as monaco from 'monaco-editor';
import { mewLanguage, mewTheme } from './tokens';
import { createCompletionProvider } from './completions';
import type { ParseError } from '../types';

(self as unknown as { MonacoEnvironment: { getWorker: () => Worker } }).MonacoEnvironment = {
  getWorker() {
    return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), { type: 'module' });
  },
};

let ontologyEditor: monaco.editor.IStandaloneCodeEditor | null = null;
let queryEditor: monaco.editor.IStandaloneCodeEditor | null = null;
const sessionIdRef = { current: null as number | null };

export async function setupMonaco(): Promise<void> {
  monaco.languages.register({ id: 'mew' });
  monaco.languages.setMonarchTokensProvider('mew', mewLanguage);
  monaco.editor.defineTheme('mew-dark', mewTheme);
  monaco.languages.registerCompletionItemProvider('mew', createCompletionProvider(sessionIdRef));
  const editorContainer = document.getElementById('editor-container')!;
  ontologyEditor = monaco.editor.create(editorContainer, {
    language: 'mew',
    theme: 'mew-dark',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    tabSize: 2,
  });
  const queryContainer = document.getElementById('query-container')!;
  queryEditor = monaco.editor.create(queryContainer, {
    language: 'mew',
    theme: 'mew-dark',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'off',
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    tabSize: 2,
  });
}

export function getEditorValue(): string {
  return ontologyEditor?.getValue() ?? '';
}

export function setEditorValue(value: string): void {
  ontologyEditor?.setValue(value);
}

export function getQueryValue(): string {
  return queryEditor?.getValue() ?? '';
}

export function setQueryValue(value: string): void {
  queryEditor?.setValue(value);
}

export function setSessionId(id: number | null): void {
  sessionIdRef.current = id;
}

export function setDiagnostics(errors: ParseError[]): void {
  if (!ontologyEditor) return;
  const model = ontologyEditor.getModel();
  if (!model) return;
  const markers: monaco.editor.IMarkerData[] = errors.map(err => ({
    severity: monaco.MarkerSeverity.Error,
    message: err.message,
    startLineNumber: err.line,
    startColumn: err.column,
    endLineNumber: err.line,
    endColumn: err.column + 1,
  }));
  monaco.editor.setModelMarkers(model, 'mew', markers);
}
