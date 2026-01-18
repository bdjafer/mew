import type { GraphData, Analysis, Layout, NodeLayout, EdgeLayout } from '../../types';
import type { LayoutStrategy } from '../strategy';

const NODE_WIDTH = 160;
const NODE_HEIGHT = 80;
const NODE_GAP = 30;

export function computeLayout(graph: GraphData, strategy: LayoutStrategy, analysis: Analysis): Layout {
  switch (strategy) {
    case 'detail':
      return detailLayout(graph);
    case 'hierarchical':
      return hierarchicalLayout(graph, analysis);
    case 'layered':
      return layeredLayout(graph, analysis);
    case 'bipartite':
      return bipartiteLayout(graph);
    case 'force':
      return forceLayout(graph);
    case 'cluster':
      return clusterLayout(graph);
    default:
      return forceLayout(graph);
  }
}

function detailLayout(graph: GraphData): Layout {
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  if (graph.nodes.length === 0) return { nodes, edges };
  const centerX = 400;
  const centerY = 300;
  if (graph.nodes.length === 1) {
    nodes.set(graph.nodes[0].id, { x: centerX - NODE_WIDTH / 2, y: centerY - NODE_HEIGHT / 2, width: NODE_WIDTH, height: NODE_HEIGHT });
  } else {
    const angle = (2 * Math.PI) / graph.nodes.length;
    const radius = Math.max(180, graph.nodes.length * 40);
    graph.nodes.forEach((node, i) => {
      const x = centerX + radius * Math.cos(i * angle - Math.PI / 2) - NODE_WIDTH / 2;
      const y = centerY + radius * Math.sin(i * angle - Math.PI / 2) - NODE_HEIGHT / 2;
      nodes.set(node.id, { x, y, width: NODE_WIDTH, height: NODE_HEIGHT });
    });
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        const path: [number, number][] = [
          [from.x + from.width / 2, from.y + from.height / 2],
          [to.x + to.width / 2, to.y + to.height / 2],
        ];
        edges.set(edge.id, { path });
      }
    }
  }
  return { nodes, edges };
}

function hierarchicalLayout(graph: GraphData, analysis: Analysis): Layout {
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  if (graph.nodes.length === 0) return { nodes, edges };
  const levelGap = 120;
  const root = analysis.root_candidates[0] ?? graph.nodes[0].id;
  const adjacency = new Map<number, number[]>();
  for (const node of graph.nodes) {
    adjacency.set(node.id, []);
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      adjacency.get(edge.targets[0])?.push(edge.targets[1]);
    }
  }
  const levels = new Map<number, number>();
  const queue = [root];
  levels.set(root, 0);
  while (queue.length > 0) {
    const current = queue.shift()!;
    const currentLevel = levels.get(current)!;
    for (const child of adjacency.get(current) ?? []) {
      if (!levels.has(child)) {
        levels.set(child, currentLevel + 1);
        queue.push(child);
      }
    }
  }
  const levelNodes = new Map<number, number[]>();
  for (const [nodeId, level] of levels) {
    if (!levelNodes.has(level)) levelNodes.set(level, []);
    levelNodes.get(level)!.push(nodeId);
  }
  const startX = 50;
  const startY = 50;
  for (const [level, nodeIds] of levelNodes) {
    const y = startY + level * levelGap;
    const totalWidth = nodeIds.length * NODE_WIDTH + (nodeIds.length - 1) * NODE_GAP;
    let x = startX + (800 - totalWidth) / 2;
    for (const nodeId of nodeIds) {
      nodes.set(nodeId, { x, y, width: NODE_WIDTH, height: NODE_HEIGHT });
      x += NODE_WIDTH + NODE_GAP;
    }
  }
  for (const node of graph.nodes) {
    if (!nodes.has(node.id)) {
      nodes.set(node.id, { x: startX, y: startY, width: NODE_WIDTH, height: NODE_HEIGHT });
    }
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        edges.set(edge.id, {
          path: [
            [from.x + from.width / 2, from.y + from.height],
            [to.x + to.width / 2, to.y],
          ],
        });
      }
    }
  }
  return { nodes, edges };
}

function layeredLayout(graph: GraphData, _analysis: Analysis): Layout {
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  if (graph.nodes.length === 0) return { nodes, edges };
  const layerGap = 200;
  const inDegree = new Map<number, number>();
  const adjacency = new Map<number, number[]>();
  for (const node of graph.nodes) {
    inDegree.set(node.id, 0);
    adjacency.set(node.id, []);
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      adjacency.get(edge.targets[0])?.push(edge.targets[1]);
      inDegree.set(edge.targets[1], (inDegree.get(edge.targets[1]) ?? 0) + 1);
    }
  }
  const layers: number[][] = [];
  const assigned = new Set<number>();
  while (assigned.size < graph.nodes.length) {
    const layer: number[] = [];
    for (const node of graph.nodes) {
      if (!assigned.has(node.id) && (inDegree.get(node.id) ?? 0) === 0) {
        layer.push(node.id);
      }
    }
    if (layer.length === 0) {
      for (const node of graph.nodes) {
        if (!assigned.has(node.id)) {
          layer.push(node.id);
          break;
        }
      }
    }
    for (const nodeId of layer) {
      assigned.add(nodeId);
      for (const child of adjacency.get(nodeId) ?? []) {
        inDegree.set(child, (inDegree.get(child) ?? 0) - 1);
      }
    }
    layers.push(layer);
  }
  const startX = 50;
  const startY = 50;
  for (let i = 0; i < layers.length; i++) {
    const x = startX + i * layerGap;
    const layer = layers[i];
    const totalHeight = layer.length * NODE_HEIGHT + (layer.length - 1) * NODE_GAP;
    let y = startY + (500 - totalHeight) / 2;
    for (const nodeId of layer) {
      nodes.set(nodeId, { x, y, width: NODE_WIDTH, height: NODE_HEIGHT });
      y += NODE_HEIGHT + NODE_GAP;
    }
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        edges.set(edge.id, {
          path: [
            [from.x + from.width, from.y + from.height / 2],
            [to.x, to.y + to.height / 2],
          ],
        });
      }
    }
  }
  return { nodes, edges };
}

function bipartiteLayout(graph: GraphData): Layout {
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  if (graph.nodes.length === 0) return { nodes, edges };
  const types = [...new Set(graph.nodes.map(n => n.type))];
  const typeA = types[0] ?? '';
  const typeB = types[1] ?? types[0] ?? '';
  const groupA = graph.nodes.filter(n => n.type === typeA);
  const groupB = graph.nodes.filter(n => n.type === typeB);
  const columnGap = 280;
  const startY = 50;
  const leftX = 80;
  const rightX = leftX + NODE_WIDTH + columnGap;
  let y = startY;
  for (const node of groupA) {
    nodes.set(node.id, { x: leftX, y, width: NODE_WIDTH, height: NODE_HEIGHT });
    y += NODE_HEIGHT + NODE_GAP;
  }
  y = startY;
  for (const node of groupB) {
    nodes.set(node.id, { x: rightX, y, width: NODE_WIDTH, height: NODE_HEIGHT });
    y += NODE_HEIGHT + NODE_GAP;
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        edges.set(edge.id, {
          path: [
            [from.x + from.width, from.y + from.height / 2],
            [to.x, to.y + to.height / 2],
          ],
        });
      }
    }
  }
  return { nodes, edges };
}

function forceLayout(graph: GraphData): Layout {
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  if (graph.nodes.length === 0) return { nodes, edges };
  const positions = new Map<number, { x: number; y: number; vx: number; vy: number }>();
  const centerX = 400;
  const centerY = 300;
  for (const node of graph.nodes) {
    positions.set(node.id, {
      x: centerX + (Math.random() - 0.5) * 400,
      y: centerY + (Math.random() - 0.5) * 300,
      vx: 0,
      vy: 0,
    });
  }
  const iterations = 100;
  const repulsion = 5000;
  const attraction = 0.01;
  const damping = 0.9;
  for (let iter = 0; iter < iterations; iter++) {
    for (const nodeA of graph.nodes) {
      const posA = positions.get(nodeA.id)!;
      let fx = 0, fy = 0;
      for (const nodeB of graph.nodes) {
        if (nodeA.id === nodeB.id) continue;
        const posB = positions.get(nodeB.id)!;
        const dx = posA.x - posB.x;
        const dy = posA.y - posB.y;
        const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
        const force = repulsion / (dist * dist);
        fx += (dx / dist) * force;
        fy += (dy / dist) * force;
      }
      posA.vx += fx;
      posA.vy += fy;
    }
    for (const edge of graph.edges) {
      if (edge.targets.length >= 2) {
        const posA = positions.get(edge.targets[0]);
        const posB = positions.get(edge.targets[1]);
        if (posA && posB) {
          const dx = posB.x - posA.x;
          const dy = posB.y - posA.y;
          const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
          const force = dist * attraction;
          posA.vx += (dx / dist) * force;
          posA.vy += (dy / dist) * force;
          posB.vx -= (dx / dist) * force;
          posB.vy -= (dy / dist) * force;
        }
      }
    }
    for (const node of graph.nodes) {
      const pos = positions.get(node.id)!;
      pos.x += pos.vx;
      pos.y += pos.vy;
      pos.vx *= damping;
      pos.vy *= damping;
      pos.x = Math.max(50, Math.min(750, pos.x));
      pos.y = Math.max(50, Math.min(550, pos.y));
    }
  }
  for (const node of graph.nodes) {
    const pos = positions.get(node.id)!;
    nodes.set(node.id, { x: pos.x - NODE_WIDTH / 2, y: pos.y - NODE_HEIGHT / 2, width: NODE_WIDTH, height: NODE_HEIGHT });
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        edges.set(edge.id, {
          path: [
            [from.x + from.width / 2, from.y + from.height / 2],
            [to.x + to.width / 2, to.y + to.height / 2],
          ],
        });
      }
    }
  }
  return { nodes, edges };
}

function clusterLayout(graph: GraphData): Layout {
  const typeGroups = new Map<string, typeof graph.nodes>();
  for (const node of graph.nodes) {
    if (!typeGroups.has(node.type)) typeGroups.set(node.type, []);
    typeGroups.get(node.type)!.push(node);
  }
  const nodes = new Map<number, NodeLayout>();
  const edges = new Map<number, EdgeLayout>();
  const clusterGap = 280;
  let clusterX = 50;
  for (const [, group] of typeGroups) {
    let y = 50;
    for (const node of group) {
      nodes.set(node.id, { x: clusterX, y, width: NODE_WIDTH, height: NODE_HEIGHT });
      y += NODE_HEIGHT + NODE_GAP;
    }
    clusterX += NODE_WIDTH + clusterGap;
  }
  for (const edge of graph.edges) {
    if (edge.targets.length >= 2) {
      const from = nodes.get(edge.targets[0]);
      const to = nodes.get(edge.targets[1]);
      if (from && to) {
        edges.set(edge.id, {
          path: [
            [from.x + from.width / 2, from.y + from.height / 2],
            [to.x + to.width / 2, to.y + to.height / 2],
          ],
        });
      }
    }
  }
  return { nodes, edges };
}

export { detailLayout, hierarchicalLayout, layeredLayout, bipartiteLayout, forceLayout, clusterLayout };
