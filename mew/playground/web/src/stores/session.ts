import { create } from 'zustand';
import { parse, createSession, execute, getGraph, getSchema } from '../api';
import type { GraphData, SchemaData, ExecuteResult } from '../types';
import { useResultsStore } from './results';

interface ParseError {
  message: string;
  line?: number;
  column?: number;
}

interface LoadResult {
  success: boolean;
  error?: string;
  message?: string;
}

interface QueryResult {
  success: boolean;
  error?: string;
  rowCount?: number;
  summary?: string;
  columns?: string[];
  rows?: unknown[][];
  resultType?: 'query' | 'mutation';
}

interface SessionState {
  sessionId: number | null;
  schema: SchemaData | null;
  graph: GraphData | null;
  status: string;
  parseErrors: ParseError[];
}

interface SessionActions {
  setStatus: (status: string) => void;
  loadOntology: (source: string) => LoadResult;
  executeQuery: (query: string) => QueryResult;
  executeSeed: (content: string) => QueryResult;
  refreshGraph: () => void;
}

export const useSessionStore = create<SessionState & SessionActions>((set, get) => ({
  sessionId: null,
  schema: null,
  graph: null,
  status: 'Ready. Click "Load Ontology" to begin.',
  parseErrors: [],

  setStatus: (status) => set({ status }),

  loadOntology: (source) => {
    const parseResult = parse(source, 'ontology');
    if (!parseResult.success) {
      const errors = parseResult.errors || [];
      const errorMsg = errors
        .map((e: ParseError) => (e.line ? `Line ${e.line}: ${e.message}` : e.message))
        .join('\n');
      set({ parseErrors: errors, status: `Parse error: ${errors[0]?.message}` });
      return { success: false, error: errorMsg };
    }

    const result = createSession(source);
    if (!result.success) {
      set({ parseErrors: [], status: `Session error: ${result.error ?? 'Unknown error'}` });
      return { success: false, error: result.error ?? 'Unknown session error' };
    }

    const sessionId = result.session_id!;
    const schema = getSchema(sessionId);
    const graph = getGraph(sessionId);

    set({
      sessionId,
      schema,
      graph,
      parseErrors: [],
      status: 'Ontology loaded successfully',
    });

    return { success: true, message: 'Ontology loaded successfully' };
  },

  executeQuery: (query) => {
    const { sessionId } = get();
    if (sessionId === null) {
      return { success: false, error: 'No session loaded. Load the ontology first.' };
    }

    const result = execute(sessionId, query);

    // Push result to results store for adaptive rendering
    useResultsStore.getState().setResult(result as ExecuteResult);

    if (!result.success) {
      set({ status: `Error: ${result.error}` });
      return { success: false, error: result.error };
    }

    if (result.result_type === 'query' && result.columns && result.rows) {
      set({ status: `Query returned ${result.rows.length} rows` });
      return {
        success: true,
        rowCount: result.rows.length,
        columns: result.columns,
        rows: result.rows,
        resultType: 'query',
      };
    } else if (result.result_type === 'mutation') {
      const graph = getGraph(sessionId);

      const parts = [];
      if (result.nodes_created) parts.push(`${result.nodes_created} nodes created`);
      if (result.nodes_modified) parts.push(`${result.nodes_modified} nodes modified`);
      if (result.nodes_deleted) parts.push(`${result.nodes_deleted} nodes deleted`);
      if (result.edges_created) parts.push(`${result.edges_created} edges created`);
      if (result.edges_deleted) parts.push(`${result.edges_deleted} edges deleted`);
      const msg = parts.join(', ') || 'No changes';

      set({ graph, status: msg });
      return { success: true, summary: msg, resultType: 'mutation' };
    }

    return { success: true };
  },

  executeSeed: (content) => {
    const { sessionId } = get();
    if (sessionId === null) {
      return { success: false, error: 'No session loaded. Load the ontology first.' };
    }

    const result = execute(sessionId, content);

    if (!result.success) {
      set({ status: `Seed error: ${result.error}` });
      return { success: false, error: result.error };
    }

    if (result.result_type === 'mutation') {
      const graph = getGraph(sessionId);

      const parts = [];
      if (result.nodes_created) parts.push(`${result.nodes_created} nodes created`);
      if (result.nodes_modified) parts.push(`${result.nodes_modified} nodes modified`);
      if (result.nodes_deleted) parts.push(`${result.nodes_deleted} nodes deleted`);
      if (result.edges_created) parts.push(`${result.edges_created} edges created`);
      if (result.edges_deleted) parts.push(`${result.edges_deleted} edges deleted`);
      const msg = parts.join(', ') || 'No changes';

      set({ graph, status: `Seed: ${msg}` });
      return { success: true, summary: msg };
    }

    return { success: true };
  },

  refreshGraph: () => {
    const { sessionId } = get();
    if (sessionId !== null) {
      const graph = getGraph(sessionId);
      set({ graph });
    }
  },
}));
