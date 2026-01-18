export interface ParseResult {
  success: boolean;
  errors: ParseError[];
}

export interface ParseError {
  message: string;
  line: number;
  column: number;
}

export interface ExecuteResult {
  success: boolean;
  result_type: 'query' | 'mutation' | 'transaction' | 'mixed' | 'empty' | 'error';
  columns?: string[];
  rows?: unknown[][];
  nodes_created?: number;
  nodes_modified?: number;
  nodes_deleted?: number;
  edges_created?: number;
  edges_modified?: number;
  edges_deleted?: number;
  error?: string;
}

export interface GraphData {
  nodes: NodeData[];
  edges: EdgeData[];
}

export interface NodeData {
  id: number;
  type: string;
  attrs: Record<string, unknown>;
}

export interface EdgeData {
  id: number;
  type: string;
  targets: number[];
  attrs: Record<string, unknown>;
}

export interface SchemaData {
  types: TypeInfo[];
  edge_types: EdgeTypeInfo[];
  type_graph: Connection[];
}

export interface TypeInfo {
  id: number;
  name: string;
  attributes: AttrInfo[];
}

export interface AttrInfo {
  name: string;
  type_name: string;
  required: boolean;
}

export interface EdgeTypeInfo {
  id: number;
  name: string;
  arity: number;
  params: ParamInfo[];
}

export interface ParamInfo {
  name: string;
  type_constraint: string;
}

export interface Connection {
  edge_type: string;
  from_type: string;
  to_type: string;
}

export interface CompletionItem {
  label: string;
  kind: 'keyword' | 'type' | 'function' | 'property' | 'snippet';
  detail?: string;
  insert_text?: string;
}

export interface Stats {
  node_count: number;
  edge_count: number;
  type_counts: [string, number][];
}

export interface CreateSessionResult {
  success: boolean;
  session_id?: number;
  error?: string;
}

export type Intent = 'find' | 'explore' | 'list' | 'understand' | 'compare' | 'overview' | 'track';

export type GraphStructure = 'single' | 'tree' | 'dag' | 'cyclic' | 'bipartite' | 'cluster';

export interface Analysis {
  type_counts: Map<string, number>;
  edge_types: string[];
  structure: GraphStructure;
  cardinality: number;
  density: number;
  root_candidates: number[];
}

export interface NodeLayout {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface EdgeLayout {
  path: [number, number][];
}

export interface Layout {
  nodes: Map<number, NodeLayout>;
  edges: Map<number, EdgeLayout>;
}

export interface ViewState {
  query: string;
  viewport: { x: number; y: number };
  zoom: number;
  focal: Set<number>;
  selected: number | null;
  actor?: number;
}
