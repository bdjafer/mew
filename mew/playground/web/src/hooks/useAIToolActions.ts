import { useEffect } from 'react';
import { useEditorStore, useSessionStore, useChatStore } from '../stores';

interface AIToolActionDetail {
  type: 'edit_ontology' | 'edit_query' | 'execute_query' | 'generate_seed';
  content?: string;
  explanation?: string;
  waitForResults?: boolean;
}

/**
 * Format query rows for AI consumption.
 * Limits to first 20 rows and formats values for readability.
 */
function formatRowsForAI(columns: string[] | undefined, rows: unknown[][]): string {
  if (!columns || rows.length === 0) return 'No results';

  const MAX_ROWS = 20;
  const displayRows = rows.slice(0, MAX_ROWS);
  const truncated = rows.length > MAX_ROWS;

  // Format each row as an object with column names
  const formatted = displayRows.map(row => {
    const obj: Record<string, unknown> = {};
    columns.forEach((col, i) => {
      obj[col] = formatValue(row[i]);
    });
    return obj;
  });

  let result = JSON.stringify(formatted, null, 2);
  if (truncated) {
    result += `\n... and ${rows.length - MAX_ROWS} more rows`;
  }
  return result;
}

/**
 * Format a single value for display
 */
function formatValue(value: unknown): unknown {
  if (value === null || value === undefined) return null;
  if (typeof value === 'object') {
    const obj = value as Record<string, unknown>;
    // Handle NodeRef and EdgeRef
    if (obj._type === 'node') return `Node#${obj._id}`;
    if (obj._type === 'edge') return `Edge#${obj._id}`;
    if (obj._type === 'timestamp') return new Date(obj.value as number).toISOString();
  }
  return value;
}

/**
 * Hook that listens for AI tool actions and executes them.
 * This bridges the AI chat's tool calls to actual editor/query actions.
 */
export function useAIToolActions() {
  const setOntologyContent = useEditorStore((state) => state.setOntologyContent);
  const setQueryContent = useEditorStore((state) => state.setQueryContent);
  const queryContent = useEditorStore((state) => state.queryContent);
  const loadOntology = useSessionStore((state) => state.loadOntology);
  const executeQuery = useSessionStore((state) => state.executeQuery);
  const executeSeed = useSessionStore((state) => state.executeSeed);
  const addToolResult = useChatStore((state) => state.addToolResult);

  useEffect(() => {
    const handleToolAction = (event: Event) => {
      const customEvent = event as CustomEvent<AIToolActionDetail>;
      const { type, content, explanation } = customEvent.detail;

      console.log('AI Tool Action received:', type, explanation);

      switch (type) {
        case 'edit_ontology': {
          if (content) {
            setOntologyContent(content);
            // Also reload the ontology
            const result = loadOntology(content);
            addToolResult({
              tool: 'edit_ontology',
              success: result.success,
              error: result.error,
              message: result.success ? 'Ontology updated and loaded' : undefined,
            });
          }
          break;
        }

        case 'edit_query': {
          if (content) {
            setQueryContent(content);
            // Also execute the query immediately
            const result = executeQuery(content);
            addToolResult({
              tool: 'edit_query',
              success: result.success,
              error: result.error,
              message: result.success ? 'Query updated and executed' : 'Query updated but execution failed',
              summary: result.summary,
              rowCount: result.rowCount,
              columns: result.columns,
              rows: result.rows ? formatRowsForAI(result.columns, result.rows) : undefined,
            });
          }
          break;
        }

        case 'execute_query': {
          // Execute the current query content
          const result = executeQuery(queryContent);
          addToolResult({
            tool: 'execute_query',
            success: result.success,
            error: result.error,
            summary: result.summary,
            rowCount: result.rowCount,
            columns: result.columns,
            rows: result.rows ? formatRowsForAI(result.columns, result.rows) : undefined,
          });
          break;
        }

        case 'generate_seed': {
          if (content) {
            const result = executeSeed(content);
            addToolResult({
              tool: 'generate_seed',
              success: result.success,
              error: result.error,
              summary: result.summary,
            });
          }
          break;
        }
      }
    };

    window.addEventListener('ai-tool-action', handleToolAction);

    return () => {
      window.removeEventListener('ai-tool-action', handleToolAction);
    };
  }, [
    setOntologyContent,
    setQueryContent,
    queryContent,
    loadOntology,
    executeQuery,
    executeSeed,
    addToolResult,
  ]);
}
