import type * as Monaco from 'monaco-editor';
import { getCompletions } from '../api';

export function createCompletionProvider(sessionIdRef: { current: number | null }): Monaco.languages.CompletionItemProvider {
  return {
    provideCompletionItems(model, position) {
      const wordInfo = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: wordInfo.startColumn,
        endColumn: wordInfo.endColumn,
      };
      const lineContent = model.getLineContent(position.lineNumber);
      const context = determineContext(lineContent, position.column);
      const items = getCompletions(sessionIdRef.current ?? undefined, wordInfo.word, context);
      return {
        suggestions: items.map(item => ({
          label: item.label,
          kind: mapCompletionKind(item.kind),
          detail: item.detail,
          insertText: item.insert_text ?? item.label,
          range,
        })),
      };
    },
  };
}

function determineContext(line: string, column: number): string {
  const beforeCursor = line.substring(0, column - 1).toLowerCase();
  if (beforeCursor.includes('node') || beforeCursor.includes('edge') || beforeCursor.includes('ontology')) {
    return 'ontology';
  }
  if (beforeCursor.includes('[')) {
    return 'ontology constraint';
  }
  return 'statement';
}

function mapCompletionKind(kind: string): number {
  const monaco = (window as unknown as { monaco: typeof Monaco }).monaco;
  switch (kind) {
    case 'keyword': return monaco.languages.CompletionItemKind.Keyword;
    case 'type': return monaco.languages.CompletionItemKind.Class;
    case 'function': return monaco.languages.CompletionItemKind.Function;
    case 'property': return monaco.languages.CompletionItemKind.Property;
    case 'snippet': return monaco.languages.CompletionItemKind.Snippet;
    default: return monaco.languages.CompletionItemKind.Text;
  }
}
