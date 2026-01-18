import type {
  ParseResult,
  ExecuteResult,
  GraphData,
  SchemaData,
  CompletionItem,
  Stats,
  CreateSessionResult,
} from './types';
import init, { Playground } from '../pkg/mew_playground';

let playground: Playground | null = null;

export async function initWasm(): Promise<void> {
  if (playground) return;
  await init();
  playground = new Playground();
}

function getPlayground(): Playground {
  if (!playground) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }
  return playground;
}

export function parse(source: string, mode: 'ontology' | 'statement'): ParseResult {
  return getPlayground().parse(source, mode) as ParseResult;
}

export function createSession(ontology: string): CreateSessionResult {
  return getPlayground().create_session(ontology) as CreateSessionResult;
}

export function deleteSession(sessionId: number): boolean {
  return getPlayground().delete_session(sessionId);
}

export function execute(sessionId: number, statement: string): ExecuteResult {
  return getPlayground().execute(sessionId, statement) as ExecuteResult;
}

export function getGraph(sessionId: number): GraphData {
  return getPlayground().get_graph(sessionId) as GraphData;
}

export function getNodes(sessionId: number, nodeIds: number[]): GraphData {
  return getPlayground().get_nodes(sessionId, new BigUint64Array(nodeIds.map(BigInt))) as GraphData;
}

export function getNeighbors(sessionId: number, nodeIds: number[]): GraphData {
  return getPlayground().get_neighbors(sessionId, new BigUint64Array(nodeIds.map(BigInt))) as GraphData;
}

export function getSchema(sessionId: number): SchemaData {
  return getPlayground().get_schema(sessionId) as SchemaData;
}

export function getCompletions(sessionId: number | undefined, prefix: string, context: string): CompletionItem[] {
  return getPlayground().get_completions(sessionId ?? null, prefix, context) as CompletionItem[];
}

export function getStats(sessionId: number): Stats {
  return getPlayground().get_stats(sessionId) as Stats;
}
