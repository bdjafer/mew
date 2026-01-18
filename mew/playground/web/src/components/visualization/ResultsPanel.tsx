import React, { useState, useCallback } from 'react';
import { useSessionStore } from '../../stores';

interface QueryResult {
  columns: string[];
  rows: unknown[][];
}

export const ResultsPanel: React.FC = () => {
  const [result, setResult] = useState<QueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<string | null>(null);

  const executeQuery = useSessionStore((state) => state.executeQuery);

  // Subscribe to query executions by wrapping the store's executeQuery
  const handleQueryExecution = useCallback((query: string) => {
    setError(null);
    setSummary(null);

    const queryResult = executeQuery(query);

    if (!queryResult.success) {
      setError(queryResult.error ?? 'Unknown error');
      setResult(null);
      return queryResult;
    }

    if (queryResult.resultType === 'query' && queryResult.columns && queryResult.rows) {
      setResult({
        columns: queryResult.columns,
        rows: queryResult.rows,
      });
      setSummary(null);
    } else if (queryResult.resultType === 'mutation') {
      setResult(null);
      setSummary(queryResult.summary ?? 'Mutation completed');
    }

    return queryResult;
  }, [executeQuery]);

  // Export the handler for external use
  React.useEffect(() => {
    (window as unknown as { __executeQueryWithResults: typeof handleQueryExecution }).__executeQueryWithResults = handleQueryExecution;
  }, [handleQueryExecution]);

  const formatCell = (value: unknown): string => {
    if (value === null || value === undefined) {
      return 'null';
    }
    if (typeof value === 'object') {
      return JSON.stringify(value);
    }
    return String(value);
  };

  const truncateCell = (value: string, maxLength: number = 50): string => {
    if (value.length <= maxLength) return value;
    return value.slice(0, maxLength - 3) + '...';
  };

  // Calculate column widths for ASCII table
  const calculateColumnWidths = (columns: string[], rows: unknown[][]): number[] => {
    const widths = columns.map((col) => col.length);
    for (const row of rows) {
      for (let i = 0; i < row.length; i++) {
        const cellWidth = truncateCell(formatCell(row[i])).length;
        widths[i] = Math.max(widths[i], cellWidth);
      }
    }
    return widths.map((w) => Math.min(w, 50)); // Cap width at 50
  };

  if (error) {
    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-error">{error}</div>
      </div>
    );
  }

  if (summary) {
    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-summary">{summary}</div>
      </div>
    );
  }

  if (!result) {
    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-empty">Run a query to see results</div>
      </div>
    );
  }

  const columnWidths = calculateColumnWidths(result.columns, result.rows);

  // Build ASCII table
  const buildSeparator = () => {
    return '+' + columnWidths.map((w) => '-'.repeat(w + 2)).join('+') + '+';
  };

  const buildRow = (cells: string[]) => {
    return (
      '|' +
      cells
        .map((cell, i) => ' ' + truncateCell(cell).padEnd(columnWidths[i]) + ' ')
        .join('|') +
      '|'
    );
  };

  return (
    <div className="results-panel">
      <div className="results-header">
        Results <span className="results-count">({result.rows.length} rows)</span>
      </div>
      <div className="results-table-container">
        <pre className="results-ascii-table">
          {buildSeparator()}
          {'\n'}
          {buildRow(result.columns)}
          {'\n'}
          {buildSeparator()}
          {'\n'}
          {result.rows.map((row, idx) => (
            <React.Fragment key={idx}>
              {buildRow(row.map(formatCell))}
              {'\n'}
            </React.Fragment>
          ))}
          {buildSeparator()}
        </pre>
      </div>
    </div>
  );
};
