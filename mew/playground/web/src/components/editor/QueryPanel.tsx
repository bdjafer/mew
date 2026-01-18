import { useState } from 'react';
import { MonacoEditor } from './MonacoEditor';
import { useEditorStore, useSessionStore } from '../../stores';

interface QueryResult {
  success: boolean;
  error?: string;
  rowCount?: number;
  summary?: string;
  columns?: string[];
  rows?: unknown[][];
  resultType?: 'query' | 'mutation';
}

export function QueryPanel() {
  const queryContent = useEditorStore((state) => state.queryContent);
  const setQueryContent = useEditorStore((state) => state.setQueryContent);
  const executeQuery = useSessionStore((state) => state.executeQuery);
  const [result, setResult] = useState<QueryResult | null>(null);

  const handleExecute = () => {
    const queryResult = executeQuery(queryContent);
    setResult(queryResult);
  };

  return (
    <div className="query-panel">
      <div className="query-panel__header">
        <span className="query-panel__title">Query</span>
        <div className="query-panel__actions">
          <button
            className="query-panel__btn query-panel__btn--execute"
            onClick={handleExecute}
          >
            Execute
          </button>
        </div>
      </div>
      <div className="query-panel__editor">
        <MonacoEditor
          type="query"
          value={queryContent}
          onChange={setQueryContent}
          height="120px"
        />
      </div>
      {result && (
        <div className="query-panel__results">
          {result.error ? (
            <div className="query-panel__error">{result.error}</div>
          ) : result.resultType === 'query' && result.columns && result.rows ? (
            <div className="query-panel__table-container">
              <table className="query-panel__table">
                <thead>
                  <tr>
                    {result.columns.map((col, i) => (
                      <th key={i}>{col}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.rows.map((row, rowIdx) => (
                    <tr key={rowIdx}>
                      {row.map((cell, cellIdx) => (
                        <td key={cellIdx}>{formatCell(cell)}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
              <div className="query-panel__row-count">
                {result.rowCount} row{result.rowCount !== 1 ? 's' : ''}
              </div>
            </div>
          ) : result.resultType === 'mutation' ? (
            <div className="query-panel__summary">{result.summary}</div>
          ) : (
            <div className="query-panel__success">Query executed successfully</div>
          )}
        </div>
      )}
    </div>
  );
}

function formatCell(value: unknown): string {
  if (value === null || value === undefined) {
    return 'null';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
