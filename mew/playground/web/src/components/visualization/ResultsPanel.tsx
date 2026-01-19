import React from 'react';
import { useResultsStore } from '../../stores';

export const ResultsPanel: React.FC = () => {
  // Subscribe to shared results store (updated by session.executeQuery)
  const lastResult = useResultsStore((state) => state.lastResult);

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

  // Handle error state
  if (lastResult && !lastResult.success) {
    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-error">{lastResult.error ?? 'Unknown error'}</div>
      </div>
    );
  }

  // Handle mutation result
  if (lastResult && lastResult.result_type === 'mutation') {
    const parts = [];
    if (lastResult.nodes_created) parts.push(`${lastResult.nodes_created} nodes created`);
    if (lastResult.nodes_modified) parts.push(`${lastResult.nodes_modified} nodes modified`);
    if (lastResult.nodes_deleted) parts.push(`${lastResult.nodes_deleted} nodes deleted`);
    if (lastResult.edges_created) parts.push(`${lastResult.edges_created} edges created`);
    if (lastResult.edges_deleted) parts.push(`${lastResult.edges_deleted} edges deleted`);
    const summary = parts.join(', ') || 'Mutation completed';

    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-summary">{summary}</div>
      </div>
    );
  }

  // Handle empty or no result
  if (!lastResult || !lastResult.columns || !lastResult.rows) {
    return (
      <div className="results-panel">
        <div className="results-header">Results</div>
        <div className="results-empty">Run a query to see results</div>
      </div>
    );
  }

  const columnWidths = calculateColumnWidths(lastResult.columns, lastResult.rows);

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
        Results <span className="results-count">({lastResult.rows.length} rows)</span>
      </div>
      <div className="results-table-container">
        <pre className="results-ascii-table">
          {buildSeparator()}
          {'\n'}
          {buildRow(lastResult.columns)}
          {'\n'}
          {buildSeparator()}
          {'\n'}
          {lastResult.rows.map((row, idx) => (
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
