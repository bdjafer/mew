/**
 * Result Analyzer - Detects query result shape to pick rendering strategy
 *
 * This implements the "auto" renderer's intelligence for adapting to any result shape.
 */

import type { ExecuteResult } from '../types';

// Rendering strategies based on result shape
export type RenderStrategy =
  | 'graph'      // Multiple rows with NodeRef/EdgeRef → graph visualization
  | 'table'      // Multiple rows without refs → tabular view
  | 'detail'     // Single row with entity refs → detail card
  | 'key-value'  // Single row without refs → key-value pairs
  | 'empty'      // No results → empty state
  | 'error';     // Error state

// Analysis result
export interface ResultAnalysis {
  strategy: RenderStrategy;
  hasNodeRefs: boolean;
  hasEdgeRefs: boolean;
  rowCount: number;
  columnCount: number;
  columns: string[];
  types: string[];
  // For graph strategy: which columns contain refs
  nodeRefColumns: number[];
  edgeRefColumns: number[];
}

/**
 * Analyze query result to determine rendering strategy
 */
export function analyzeResult(result: ExecuteResult): ResultAnalysis {
  // Handle error case
  if (!result.success || result.result_type === 'error') {
    return {
      strategy: 'error',
      hasNodeRefs: false,
      hasEdgeRefs: false,
      rowCount: 0,
      columnCount: 0,
      columns: [],
      types: [],
      nodeRefColumns: [],
      edgeRefColumns: [],
    };
  }

  // Handle empty/mutation results
  if (result.result_type !== 'query' || !result.columns || !result.rows) {
    return {
      strategy: 'empty',
      hasNodeRefs: false,
      hasEdgeRefs: false,
      rowCount: 0,
      columnCount: 0,
      columns: [],
      types: [],
      nodeRefColumns: [],
      edgeRefColumns: [],
    };
  }

  const columns = result.columns;
  const rows = result.rows;
  const rowCount = rows.length;
  const columnCount = columns.length;

  // Detect ref columns by analyzing actual values
  const nodeRefColumns: number[] = [];
  const edgeRefColumns: number[] = [];

  for (let col = 0; col < columnCount; col++) {
    let hasNodeRef = false;
    let hasEdgeRef = false;

    for (const row of rows) {
      const value = row[col];
      if (isNodeRef(value)) {
        hasNodeRef = true;
      } else if (isEdgeRef(value)) {
        hasEdgeRef = true;
      }
    }

    if (hasNodeRef) nodeRefColumns.push(col);
    if (hasEdgeRef) edgeRefColumns.push(col);
  }

  const hasNodeRefs = nodeRefColumns.length > 0;
  const hasEdgeRefs = edgeRefColumns.length > 0;
  const hasRefs = hasNodeRefs || hasEdgeRefs;

  // Determine strategy based on shape
  let strategy: RenderStrategy;

  if (rowCount === 0) {
    strategy = 'empty';
  } else if (rowCount === 1) {
    // Single row
    strategy = hasRefs ? 'detail' : 'key-value';
  } else {
    // Multiple rows
    strategy = hasRefs ? 'graph' : 'table';
  }

  // Infer types from values (since we don't have explicit type info in ExecuteResult)
  const types = columns.map((_, colIdx) => {
    if (nodeRefColumns.includes(colIdx)) return 'NodeRef';
    if (edgeRefColumns.includes(colIdx)) return 'EdgeRef';
    // Sample first non-null value to infer type
    for (const row of rows) {
      const value = row[colIdx];
      if (value !== null && value !== undefined) {
        return inferValueType(value);
      }
    }
    return 'unknown';
  });

  return {
    strategy,
    hasNodeRefs,
    hasEdgeRefs,
    rowCount,
    columnCount,
    columns,
    types,
    nodeRefColumns,
    edgeRefColumns,
  };
}

/**
 * Check if a value is a NodeRef
 * NodeRefs come from WASM as objects with _type: 'node' and _id
 */
function isNodeRef(value: unknown): boolean {
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>;
    return obj._type === 'node' && typeof obj._id === 'number';
  }
  return false;
}

/**
 * Check if a value is an EdgeRef
 */
function isEdgeRef(value: unknown): boolean {
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>;
    return obj._type === 'edge' && typeof obj._id === 'number';
  }
  return false;
}

/**
 * Extract ID from a ref value
 */
export function extractRefId(value: unknown): number | null {
  if (typeof value === 'object' && value !== null) {
    const obj = value as Record<string, unknown>;
    if (typeof obj._id === 'number') {
      return obj._id;
    }
  }
  return null;
}

/**
 * Infer type from a value
 */
function inferValueType(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'boolean') return 'Bool';
  if (typeof value === 'number') {
    return Number.isInteger(value) ? 'Int' : 'Float';
  }
  if (typeof value === 'string') return 'String';
  if (Array.isArray(value)) return 'List';
  if (typeof value === 'object') {
    const obj = value as Record<string, unknown>;
    if (obj._type === 'node') return 'NodeRef';
    if (obj._type === 'edge') return 'EdgeRef';
    return 'Map';
  }
  return 'unknown';
}

/**
 * Get display value for a cell
 */
export function formatCellValue(value: unknown): string {
  if (value === null || value === undefined) return '—';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') return String(value);
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) return `[${value.length} items]`;
  if (typeof value === 'object') {
    const obj = value as Record<string, unknown>;
    if (obj._type === 'node') return `#node_${obj._id}`;
    if (obj._type === 'edge') return `#edge_${obj._id}`;
    return JSON.stringify(value);
  }
  return String(value);
}

/**
 * Extract all entity IDs from query results (for graph rendering)
 */
export function extractEntityIds(result: ExecuteResult, analysis: ResultAnalysis): {
  nodeIds: number[];
  edgeIds: number[];
} {
  const nodeIds = new Set<number>();
  const edgeIds = new Set<number>();

  if (!result.rows) return { nodeIds: [], edgeIds: [] };

  for (const row of result.rows) {
    for (const colIdx of analysis.nodeRefColumns) {
      const id = extractRefId(row[colIdx]);
      if (id !== null) nodeIds.add(id);
    }
    for (const colIdx of analysis.edgeRefColumns) {
      const id = extractRefId(row[colIdx]);
      if (id !== null) edgeIds.add(id);
    }
  }

  return {
    nodeIds: [...nodeIds],
    edgeIds: [...edgeIds],
  };
}
