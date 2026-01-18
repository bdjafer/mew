import { initWasm, createSession, execute, getGraph, getSchema, parse } from './api';
import { setupMonaco, getEditorValue, setEditorValue, getQueryValue, setQueryValue, setDiagnostics } from './editor/setup';
import { analyzeGraph } from './viz/analyzer';
import { selectStrategy } from './viz/strategy';
import { computeLayout } from './viz/layout';
import { Renderer } from './viz/renderer';
import type { GraphData, NodeData, ViewState, SchemaData } from './types';

let currentSessionId: number | null = null;
let currentSchema: SchemaData | null = null;
let currentGraph: GraphData | null = null;
let renderer: Renderer | null = null;
let viewState: ViewState = {
  query: '',
  viewport: { x: 0, y: 0 },
  zoom: 1,
  focal: new Set(),
  selected: null,
};

const SAMPLE_ONTOLOGY = `ontology TodoApp {
  node User {
    name: String [required]
    email: String [required, unique]
  }

  node Task {
    title: String [required]
    status: String = "todo"
    priority: Int = 3
  }

  edge owns(user: User, task: Task)
  edge assigned_to(task: Task, user: User)
}`;

const SAMPLE_SEED = `SPAWN User { name = "Alice", email = "alice@example.com" }
SPAWN User { name = "Bob", email = "bob@example.com" }
SPAWN Task { title = "Design API", status = "done", priority = 5 }
SPAWN Task { title = "Implement WASM", status = "in_progress", priority = 4 }
SPAWN Task { title = "Write tests", status = "todo", priority = 3 }
LINK owns(SPAWN User { name = "Carol", email = "carol@example.com" } AS u1, SPAWN Task { title = "Review code", status = "in_progress" } AS t1)
LINK owns(SPAWN User { name = "Dave", email = "dave@example.com" } AS u2, SPAWN Task { title = "Deploy app", status = "todo" } AS t2)
LINK assigned_to(SPAWN Task { title = "Fix bug" } AS t3, SPAWN User { name = "Eve", email = "eve@example.com" } AS u3)`;

async function init() {
  try {
    setStatus('Loading WASM module...');
    await initWasm();
    setStatus('Setting up editor...');
    await setupMonaco();
    setEditorValue(SAMPLE_ONTOLOGY);
    setQueryValue('MATCH t: Task RETURN t.title, t.status, t.priority');
    const canvas = document.getElementById('graph-canvas') as HTMLCanvasElement;
    renderer = new Renderer(canvas);
    renderer.onNodeClick = handleNodeClick;
    renderer.onNodeHover = handleNodeHover;
    renderer.onZoomChange = updateZoomDisplay;
    setupEventListeners();
    setStatus('Ready. Click "Load Ontology" to begin.');
  } catch (error) {
    console.error('Initialization error:', error);
    setStatus(`Error: ${error}`);
  }
}

function setupEventListeners() {
  document.getElementById('load-ontology')!.addEventListener('click', loadOntology);
  document.getElementById('execute-query')!.addEventListener('click', executeQuery);
  document.getElementById('zoom-in')!.addEventListener('click', () => renderer?.zoomIn());
  document.getElementById('zoom-out')!.addEventListener('click', () => renderer?.zoomOut());
  document.getElementById('zoom-fit')!.addEventListener('click', () => renderer?.fitToView());
  window.addEventListener('resize', () => renderer?.resize());
  setupSidebarResize();
}

function setupSidebarResize() {
  const handle = document.getElementById('resize-handle')!;
  const sidebar = document.getElementById('sidebar')!;
  let isDragging = false;
  let startX = 0;
  let startWidth = 0;

  handle.addEventListener('mousedown', (e) => {
    isDragging = true;
    startX = e.clientX;
    startWidth = sidebar.offsetWidth;
    handle.classList.add('dragging');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    e.preventDefault();
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    const delta = e.clientX - startX;
    const newWidth = Math.min(800, Math.max(280, startWidth + delta));
    document.documentElement.style.setProperty('--sidebar-width', `${newWidth}px`);
    renderer?.resize();
  });

  document.addEventListener('mouseup', () => {
    if (isDragging) {
      isDragging = false;
      handle.classList.remove('dragging');
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }
  });
}

function loadOntology() {
  const source = getEditorValue();
  const parseResult = parse(source, 'ontology');
  if (!parseResult.success) {
    setDiagnostics(parseResult.errors);
    setStatus(`Parse error: ${parseResult.errors[0]?.message}`);
    return;
  }
  setDiagnostics([]);
  const result = createSession(source);
  if (!result.success) {
    setStatus(`Session error: ${result.error ?? 'Unknown error'}`);
    return;
  }
  currentSessionId = result.session_id!;
  currentSchema = getSchema(currentSessionId);
  const lines = SAMPLE_SEED.split('\n');
  let nodesCreated = 0;
  let edgesCreated = 0;
  for (const line of lines) {
    if (line.trim()) {
      const res = execute(currentSessionId, line);
      if (res.success) {
        nodesCreated += res.nodes_created ?? 0;
        edgesCreated += res.edges_created ?? 0;
      }
    }
  }
  currentGraph = getGraph(currentSessionId);
  currentGraph = getGraph(currentSessionId);
  document.getElementById('session-info')!.textContent = `Session: ${currentSessionId}`;
  setStatus(`Ontology loaded. Created ${nodesCreated} nodes, ${edgesCreated} edges.`);
  updateGraphStats();
  updateVisualization();
  updateMinimap();
}

function executeQuery() {
  if (currentSessionId === null) {
    setStatus('No session loaded. Load an ontology first.');
    return;
  }
  const query = getQueryValue();
  const result = execute(currentSessionId, query);
  if (!result.success) {
    setStatus(`Error: ${result.error}`);
    showResults(`Error: ${result.error}`);
    return;
  }
  viewState.query = query;
  if (result.result_type === 'query' && result.columns && result.rows) {
    const focal = new Set<number>();
    for (const row of result.rows) {
      for (const cell of row) {
        if (typeof cell === 'object' && cell !== null && '_type' in cell && '_id' in cell) {
          focal.add((cell as { _id: number })._id);
        }
      }
    }
    viewState.focal = focal;
    const output = formatQueryResult(result.columns, result.rows);
    showResults(output, result.rows.length);
    setStatus(`Query returned ${result.rows.length} rows`);
  } else if (result.result_type === 'mutation') {
    currentGraph = getGraph(currentSessionId);
    const parts = [];
    if (result.nodes_created) parts.push(`${result.nodes_created} nodes created`);
    if (result.nodes_modified) parts.push(`${result.nodes_modified} nodes modified`);
    if (result.nodes_deleted) parts.push(`${result.nodes_deleted} nodes deleted`);
    if (result.edges_created) parts.push(`${result.edges_created} edges created`);
    if (result.edges_deleted) parts.push(`${result.edges_deleted} edges deleted`);
    const msg = parts.join(', ') || 'No changes';
    setStatus(msg);
    showResults(msg);
    updateGraphStats();
    updateMinimap();
  }
  updateVisualization();
}

function formatQueryResult(columns: string[], rows: unknown[][]): string {
  if (rows.length === 0) return 'No results';
  const widths = columns.map((col, i) => {
    const maxDataWidth = Math.max(...rows.map(row => String(formatCell(row[i])).length));
    return Math.max(col.length, maxDataWidth);
  });
  const header = columns.map((col, i) => col.padEnd(widths[i])).join(' | ');
  const separator = widths.map(w => '-'.repeat(w)).join('-+-');
  const dataRows = rows.map(row => row.map((cell, i) => String(formatCell(cell)).padEnd(widths[i])).join(' | '));
  return [header, separator, ...dataRows].join('\n');
}

function formatCell(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'object' && '_type' in value && '_id' in value) {
    return `#${(value as { _id: number })._id}`;
  }
  return String(value);
}

function showResults(content: string, rowCount?: number) {
  const panel = document.getElementById('results-panel')!;
  const header = rowCount !== undefined ? `Results (${rowCount} rows)` : 'Results';
  panel.innerHTML = `<div class="results-header"><span>${header}</span><span class="results-close" onclick="this.parentElement.parentElement.innerHTML=''">✕</span></div><div class="results-content">${escapeHtml(content)}</div>`;
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function setStatus(message: string) {
  const statusEl = document.getElementById('graph-stats');
  if (statusEl) statusEl.textContent = message;
}

function updateGraphStats() {
  if (!currentGraph) return;
  const stats = document.getElementById('graph-stats');
  if (stats) {
    stats.textContent = `${currentGraph.nodes.length} nodes, ${currentGraph.edges.length} edges`;
  }
}

function updateZoomDisplay(zoom: number) {
  const el = document.getElementById('zoom-level');
  if (el) el.textContent = `${Math.round(zoom * 100)}%`;
}

function updateVisualization() {
  if (!renderer || !currentGraph) return;
  const analysis = analyzeGraph(currentGraph, currentSchema);
  const strategy = selectStrategy(analysis, viewState);
  const layout = computeLayout(currentGraph, strategy, analysis);
  renderer.render(currentGraph, layout, viewState.focal, viewState.selected);
}

function updateMinimap() {
  if (!currentSchema) return;
  const minimap = document.getElementById('minimap')!;
  minimap.innerHTML = '';
  for (const type of currentSchema.types) {
    const count = currentGraph?.nodes.filter(n => n.type === type.name).length ?? 0;
    const el = document.createElement('div');
    el.className = 'minimap-type';
    el.textContent = `${type.name}: ${count}`;
    el.addEventListener('click', () => {
      setQueryValue(`MATCH n: ${type.name} RETURN n`);
      executeQuery();
    });
    minimap.appendChild(el);
  }
  for (const edgeType of currentSchema.edge_types) {
    const count = currentGraph?.edges.filter(e => e.type === edgeType.name).length ?? 0;
    const el = document.createElement('div');
    el.className = 'minimap-type';
    el.style.fontStyle = 'italic';
    el.textContent = `${edgeType.name}: ${count}`;
    minimap.appendChild(el);
  }
}

function handleNodeClick(node: NodeData) {
  viewState.selected = node.id;
  updateDetailPanel(node);
  updateVisualization();
  document.getElementById('selection-info')!.textContent = `Selected: ${node.type} #${node.id}`;
}

function handleNodeHover(node: NodeData | null) {
  if (node) {
    renderer?.highlightNode(node.id);
  } else {
    renderer?.clearHighlight();
  }
}

function updateDetailPanel(node: NodeData) {
  const panel = document.getElementById('detail-content')!;
  const attrs = Object.entries(node.attrs).map(([k, v]) => `<div><strong>${k}:</strong> ${v}</div>`).join('');
  panel.innerHTML = `<div class="node-detail"><div class="node-type">${node.type}</div><div class="node-id">#${node.id}</div>${attrs}</div>`;
}

init();
