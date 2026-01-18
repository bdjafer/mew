import type * as Monaco from 'monaco-editor';

export const mewLanguage: Monaco.languages.IMonarchLanguage = {
  defaultToken: 'invalid',
  tokenPostfix: '.mew',
  keywords: [
    'ontology', 'node', 'edge', 'constraint', 'rule', 'type',
    'MATCH', 'SPAWN', 'KILL', 'LINK', 'UNLINK', 'SET', 'RETURN',
    'WHERE', 'ORDER', 'BY', 'ASC', 'DESC', 'LIMIT', 'OFFSET',
    'GROUP', 'HAVING', 'BEGIN', 'COMMIT', 'ROLLBACK', 'INSPECT',
    'WALK', 'FROM', 'TO', 'VIA', 'EXPLAIN', 'PROFILE', 'AS',
    'AND', 'OR', 'NOT', 'IN', 'true', 'false', 'null',
    'on_kill_source', 'on_kill_target', 'cascade', 'restrict', 'delete',
    'required', 'unique', 'readonly', 'abstract', 'sealed', 'symmetric',
    'no_self', 'acyclic', 'deferred', 'hard', 'soft', 'auto',
  ],
  typeKeywords: [
    'String', 'Int', 'Float', 'Bool', 'Timestamp', 'Duration', 'ID', 'any',
  ],
  operators: [
    '=', '!=', '<', '>', '<=', '>=', '+', '-', '*', '/', '%',
    '&&', '||', '!', '..', '->', '<-',
  ],
  symbols: /[=><!~?:&|+\-*\/\^%]+/,
  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
  tokenizer: {
    root: [
      [/--.*$/, 'comment'],
      [/[a-zA-Z_][a-zA-Z0-9_]*/, {
        cases: {
          '@keywords': 'keyword',
          '@typeKeywords': 'type',
          '@default': 'identifier',
        },
      }],
      [/#[0-9]+/, 'number.ref'],
      [/[0-9]+\.[0-9]*([eE][-+]?[0-9]+)?/, 'number.float'],
      [/[0-9]+/, 'number'],
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, 'string', '@string'],
      { include: '@whitespace' },
      [/[{}()\[\]]/, '@brackets'],
      [/[;,.]/, 'delimiter'],
      [/@symbols/, {
        cases: {
          '@operators': 'operator',
          '@default': '',
        },
      }],
    ],
    string: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, 'string', '@pop'],
    ],
    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/--.*$/, 'comment'],
    ],
  },
};

export const mewTheme: Monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'keyword', foreground: '818CF8', fontStyle: 'bold' },
    { token: 'type', foreground: '22D3EE' },
    { token: 'identifier', foreground: 'E5E5E5' },
    { token: 'number', foreground: 'F59E0B' },
    { token: 'number.float', foreground: 'F59E0B' },
    { token: 'number.ref', foreground: '10B981' },
    { token: 'string', foreground: 'A5F3FC' },
    { token: 'string.escape', foreground: '67E8F9' },
    { token: 'comment', foreground: '6B7280', fontStyle: 'italic' },
    { token: 'operator', foreground: 'F472B6' },
    { token: 'delimiter', foreground: '9CA3AF' },
  ],
  colors: {
    'editor.background': '#1a1a1a',
    'editor.foreground': '#e5e5e5',
    'editorCursor.foreground': '#6366f1',
    'editor.lineHighlightBackground': '#252525',
    'editorLineNumber.foreground': '#4b5563',
    'editor.selectionBackground': '#6366f150',
    'editor.inactiveSelectionBackground': '#6366f130',
  },
};
