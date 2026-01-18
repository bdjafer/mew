import type { GraphData, SchemaData, Analysis, GraphStructure } from '../types';

export function analyzeGraph(graph: GraphData, _schema: SchemaData | null): Analysis {
  const type_counts = new Map<string, number>();
  for (const node of graph.nodes) {
    type_counts.set(node.type, (type_counts.get(node.type) ?? 0) + 1);
  }
  const edge_types = [...new Set(graph.edges.map(e => e.type))];
  const cardinality = graph.nodes.length;
  const maxEdges = cardinality * (cardinality - 1) / 2;
  const density = maxEdges > 0 ? graph.edges.length / maxEdges : 0;
  const structure = detectStructure(graph);
  const root_candidates = findRootCandidates(graph);
  return { type_counts, edge_types, structure, cardinality, density, root_candidates };
}

function detectStructure(graph: GraphData): GraphStructure {
  if (graph.nodes.length === 0) return 'single';
  if (graph.nodes.length === 1) return 'single';
  const adjacency = buildAdjacency(graph);
  const hasCycle = detectCycle(adjacency, graph.nodes.length);
  if (hasCycle) return 'cyclic';
  const inDegree = new Map<number, number>();
  const outDegree = new Map<number, number>();
  for (const node of graph.nodes) {
    inDegree.set(node.id, 0);
    outDegree.set(node.id, 0);
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      outDegree.set(edge.targets[0], (outDegree.get(edge.targets[0]) ?? 0) + 1);
      for (let i = 1; i < edge.targets.length; i++) {
        inDegree.set(edge.targets[i], (inDegree.get(edge.targets[i]) ?? 0) + 1);
      }
    }
  }
  const roots = [...inDegree.entries()].filter(([, deg]) => deg === 0).map(([id]) => id);
  if (roots.length === 1) {
    const isTree = checkTree(adjacency, roots[0], graph.nodes.length);
    if (isTree) return 'tree';
  }
  const types = [...new Set(graph.nodes.map(n => n.type))];
  if (types.length === 2) {
    const typeA = types[0];
    const typeB = types[1];
    const edgesOnlyBetweenTypes = graph.edges.every(e => {
      if (e.targets.length < 2) return true;
      const t0 = graph.nodes.find(n => n.id === e.targets[0])?.type;
      const t1 = graph.nodes.find(n => n.id === e.targets[1])?.type;
      return (t0 === typeA && t1 === typeB) || (t0 === typeB && t1 === typeA);
    });
    if (edgesOnlyBetweenTypes) return 'bipartite';
  }
  if (graph.nodes.length > 50) return 'cluster';
  return 'dag';
}

function buildAdjacency(graph: GraphData): Map<number, number[]> {
  const adj = new Map<number, number[]>();
  for (const node of graph.nodes) {
    adj.set(node.id, []);
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = edge.targets[0];
      const to = edge.targets[1];
      adj.get(from)?.push(to);
    }
  }
  return adj;
}

function detectCycle(adjacency: Map<number, number[]>, _n: number): boolean {
  const visited = new Set<number>();
  const recStack = new Set<number>();
  function dfs(node: number): boolean {
    visited.add(node);
    recStack.add(node);
    for (const neighbor of adjacency.get(node) ?? []) {
      if (!visited.has(neighbor)) {
        if (dfs(neighbor)) return true;
      } else if (recStack.has(neighbor)) {
        return true;
      }
    }
    recStack.delete(node);
    return false;
  }
  for (const [node] of adjacency) {
    if (!visited.has(node)) {
      if (dfs(node)) return true;
    }
  }
  return false;
}

function checkTree(adjacency: Map<number, number[]>, root: number, n: number): boolean {
  const visited = new Set<number>();
  const queue = [root];
  while (queue.length > 0) {
    const node = queue.shift()!;
    if (visited.has(node)) return false;
    visited.add(node);
    for (const neighbor of adjacency.get(node) ?? []) {
      if (!visited.has(neighbor)) {
        queue.push(neighbor);
      }
    }
  }
  return visited.size === n;
}

function findRootCandidates(graph: GraphData): number[] {
  const inDegree = new Map<number, number>();
  for (const node of graph.nodes) {
    inDegree.set(node.id, 0);
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      for (let i = 1; i < edge.targets.length; i++) {
        inDegree.set(edge.targets[i], (inDegree.get(edge.targets[i]) ?? 0) + 1);
      }
    }
  }
  return [...inDegree.entries()].filter(([, deg]) => deg === 0).map(([id]) => id);
}
