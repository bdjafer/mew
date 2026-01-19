/**
 * Auto-Renderer - Adaptive rendering based on query result shape
 *
 * This implements the "mew:render/auto" strategy that detects result shape
 * and renders appropriately: graph, table, detail card, or key-value pairs.
 */

import type { ExecuteResult, GraphData, Layout } from '../types';
import { analyzeResult, type ResultAnalysis, type RenderStrategy, formatCellValue } from './result-analyzer';
import { buildHitMapFromLayout, buildHitMapFromTable, type HitMap, type HitRegion, hitTest } from './hitmap';

// Theme colors
const COLORS = {
  bgPrimary: '#0f0f0f',
  bgSecondary: '#1a1a1a',
  bgTertiary: '#2a2a2a',
  borderPrimary: '#333',
  borderSecondary: '#444',
  textPrimary: '#e5e5e5',
  textSecondary: '#a0a0a0',
  textMuted: '#666',
  accent: '#6366f1',
  accentHover: '#818cf8',
  success: '#22c55e',
  warning: '#f59e0b',
  error: '#ef4444',
};

// Layout constants
const TABLE_CELL_WIDTH = 150;
const TABLE_CELL_HEIGHT = 32;
const TABLE_HEADER_HEIGHT = 36;
const TABLE_PADDING = 16;
const DETAIL_CARD_WIDTH = 400;
const DETAIL_ROW_HEIGHT = 28;

export interface AutoRendererOptions {
  canvas: HTMLCanvasElement;
  onEntityClick?: (kind: 'node' | 'edge', id: number) => void;
  onRowClick?: (resultIndex: number, rowIndex: number) => void;
  onCellClick?: (resultIndex: number, row: number, col: number) => void;
}

export interface RenderContext {
  result: ExecuteResult;
  analysis: ResultAnalysis;
  graphData?: GraphData;
  layout?: Layout;
  focal?: Set<number>;
  selected?: number | null;
}

/**
 * AutoRenderer - Renders query results adaptively based on shape detection
 */
export class AutoRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private hitMap: HitMap | null = null;
  private viewport = { x: 0, y: 0 };
  private zoom = 1;
  private isDragging = false;
  private lastMouse = { x: 0, y: 0 };
  private currentStrategy: RenderStrategy = 'empty';

  // Event handlers
  onEntityClick: ((kind: 'node' | 'edge', id: number) => void) | null = null;
  onRowClick: ((resultIndex: number, rowIndex: number) => void) | null = null;
  onCellClick: ((resultIndex: number, row: number, col: number) => void) | null = null;
  onHover: ((region: HitRegion | null) => void) | null = null;

  constructor(options: AutoRendererOptions) {
    this.canvas = options.canvas;
    this.ctx = this.canvas.getContext('2d')!;
    this.onEntityClick = options.onEntityClick ?? null;
    this.onRowClick = options.onRowClick ?? null;
    this.onCellClick = options.onCellClick ?? null;
    this.setupEventListeners();
    this.resize();
  }

  resize() {
    const rect = this.canvas.parentElement?.getBoundingClientRect();
    if (rect) {
      this.canvas.width = rect.width;
      this.canvas.height = rect.height;
    }
  }

  private setupEventListeners() {
    this.canvas.addEventListener('mousedown', this.handleMouseDown.bind(this));
    this.canvas.addEventListener('mousemove', this.handleMouseMove.bind(this));
    this.canvas.addEventListener('mouseup', this.handleMouseUp.bind(this));
    this.canvas.addEventListener('wheel', this.handleWheel.bind(this));
    this.canvas.addEventListener('click', this.handleClick.bind(this));
  }

  private handleMouseDown(e: MouseEvent) {
    this.isDragging = true;
    this.lastMouse = { x: e.clientX, y: e.clientY };
    this.canvas.style.cursor = 'grabbing';
  }

  private handleMouseMove(e: MouseEvent) {
    if (this.isDragging) {
      const dx = e.clientX - this.lastMouse.x;
      const dy = e.clientY - this.lastMouse.y;
      this.viewport.x += dx / this.zoom;
      this.viewport.y += dy / this.zoom;
      this.lastMouse = { x: e.clientX, y: e.clientY };
      // Note: need to redraw, but we don't have context here
    } else {
      const pos = this.screenToWorld(e.clientX, e.clientY);
      if (this.hitMap) {
        const hit = hitTest(this.hitMap, pos.x, pos.y);
        this.canvas.style.cursor = hit?.cursor ?? 'grab';
        this.onHover?.(hit);
      }
    }
  }

  private handleMouseUp() {
    this.isDragging = false;
    this.canvas.style.cursor = 'grab';
  }

  private handleWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    this.zoom = Math.max(0.1, Math.min(5, this.zoom * factor));
  }

  private handleClick(e: MouseEvent) {
    if (this.isDragging) return;
    const pos = this.screenToWorld(e.clientX, e.clientY);
    if (!this.hitMap) return;

    const hit = hitTest(this.hitMap, pos.x, pos.y);
    if (!hit) return;

    switch (hit.ref.type) {
      case 'entity':
        this.onEntityClick?.(hit.ref.kind, hit.ref.id);
        break;
      case 'row':
        this.onRowClick?.(hit.ref.result, hit.ref.index);
        break;
      case 'cell':
        this.onCellClick?.(hit.ref.result, hit.ref.row, hit.ref.col);
        break;
    }
  }

  private screenToWorld(screenX: number, screenY: number): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: (screenX - rect.left) / this.zoom - this.viewport.x,
      y: (screenY - rect.top) / this.zoom - this.viewport.y,
    };
  }

  /**
   * Render query result adaptively based on shape
   */
  render(context: RenderContext) {
    const { result, analysis } = context;
    this.currentStrategy = analysis.strategy;

    // Clear canvas
    this.ctx.fillStyle = COLORS.bgPrimary;
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

    // Apply viewport transform
    this.ctx.save();
    this.ctx.scale(this.zoom, this.zoom);
    this.ctx.translate(this.viewport.x, this.viewport.y);

    // Render based on strategy
    switch (analysis.strategy) {
      case 'graph':
        this.renderGraph(context);
        break;
      case 'table':
        this.renderTable(result, analysis);
        break;
      case 'detail':
        this.renderDetail(result, analysis);
        break;
      case 'key-value':
        this.renderKeyValue(result, analysis);
        break;
      case 'empty':
        this.renderEmpty();
        break;
      case 'error':
        this.renderError(result.error ?? 'Unknown error');
        break;
    }

    this.ctx.restore();
  }

  /**
   * Render graph visualization (delegates to existing graph renderer logic)
   */
  private renderGraph(context: RenderContext) {
    const { graphData, layout, focal, selected } = context;
    if (!graphData || !layout) {
      this.renderEmpty('No graph data');
      return;
    }

    const focalSet = focal ?? new Set<number>();

    // Draw edges first (lower layer)
    this.drawEdges(graphData, layout, focalSet);

    // Draw nodes (upper layer)
    this.drawNodes(graphData, layout, focalSet, selected ?? null);

    // Build hit map
    const edgeLayouts = new Map<number, { path: [number, number][] }>();
    for (const [id, el] of layout.edges) {
      edgeLayouts.set(id, { path: el.path });
    }
    this.hitMap = buildHitMapFromLayout(layout.nodes, edgeLayouts, focalSet);
  }

  private drawEdges(graph: GraphData, layout: Layout, focal: Set<number>) {
    for (const edge of graph.edges) {
      const el = layout.edges.get(edge.id);
      if (!el) continue;

      const isFocal = focal.size === 0 || edge.targets.some(t => focal.has(t));

      this.ctx.beginPath();
      this.ctx.strokeStyle = isFocal ? COLORS.accent : COLORS.borderSecondary;
      this.ctx.lineWidth = isFocal ? 2 : 1;
      this.ctx.globalAlpha = isFocal ? 1 : 0.4;

      if (el.path.length >= 2) {
        this.ctx.moveTo(el.path[0][0], el.path[0][1]);
        for (let i = 1; i < el.path.length; i++) {
          this.ctx.lineTo(el.path[i][0], el.path[i][1]);
        }
      }
      this.ctx.stroke();
      this.ctx.globalAlpha = 1;

      // Draw arrow
      if (el.path.length >= 2) {
        this.drawArrow(el.path[el.path.length - 2], el.path[el.path.length - 1], isFocal);
      }
    }
  }

  private drawArrow(from: [number, number], to: [number, number], isFocal: boolean) {
    const headLen = 10;
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const angle = Math.atan2(dy, dx);

    this.ctx.beginPath();
    this.ctx.moveTo(to[0], to[1]);
    this.ctx.lineTo(to[0] - headLen * Math.cos(angle - Math.PI / 6), to[1] - headLen * Math.sin(angle - Math.PI / 6));
    this.ctx.lineTo(to[0] - headLen * Math.cos(angle + Math.PI / 6), to[1] - headLen * Math.sin(angle + Math.PI / 6));
    this.ctx.closePath();
    this.ctx.fillStyle = isFocal ? COLORS.accent : COLORS.borderSecondary;
    this.ctx.globalAlpha = isFocal ? 1 : 0.4;
    this.ctx.fill();
    this.ctx.globalAlpha = 1;
  }

  private drawNodes(graph: GraphData, layout: Layout, focal: Set<number>, selected: number | null) {
    for (const node of graph.nodes) {
      const nl = layout.nodes.get(node.id);
      if (!nl) continue;

      const isFocal = focal.size === 0 || focal.has(node.id);
      const isSelected = selected === node.id;

      // Node background
      this.ctx.globalAlpha = isFocal ? 1 : 0.4;
      this.ctx.fillStyle = isFocal ? COLORS.bgTertiary : COLORS.bgSecondary;
      this.ctx.strokeStyle = isSelected ? COLORS.success : COLORS.borderSecondary;
      this.ctx.lineWidth = isSelected ? 2 : 1;
      this.roundRect(nl.x, nl.y, nl.width, nl.height, 6);
      this.ctx.fill();
      this.ctx.stroke();
      this.ctx.globalAlpha = 1;

      // Type label
      this.ctx.font = 'bold 11px Inter, sans-serif';
      this.ctx.fillStyle = isFocal ? COLORS.accent : COLORS.textMuted;
      this.ctx.textAlign = 'center';
      this.ctx.textBaseline = 'top';
      this.ctx.fillText(node.type, nl.x + nl.width / 2, nl.y + 8);

      // Separator line
      this.ctx.beginPath();
      this.ctx.strokeStyle = COLORS.borderPrimary;
      this.ctx.lineWidth = 0.5;
      this.ctx.moveTo(nl.x + 8, nl.y + 24);
      this.ctx.lineTo(nl.x + nl.width - 8, nl.y + 24);
      this.ctx.stroke();

      // Attributes
      const attrs = this.getDisplayAttrs(node.attrs, 3);
      this.ctx.font = '10px Inter, sans-serif';
      this.ctx.textAlign = 'left';
      let yOffset = 30;
      for (const [key, value] of attrs) {
        this.ctx.fillStyle = isFocal ? COLORS.textSecondary : COLORS.textMuted;
        const keyText = `${key}: `;
        this.ctx.fillText(keyText, nl.x + 8, nl.y + yOffset);
        const keyWidth = this.ctx.measureText(keyText).width;
        this.ctx.fillStyle = isFocal ? COLORS.textPrimary : COLORS.textMuted;
        const valueText = this.truncateText(String(value), nl.width - 16 - keyWidth);
        this.ctx.fillText(valueText, nl.x + 8 + keyWidth, nl.y + yOffset);
        yOffset += 14;
      }
    }
  }

  /**
   * Render tabular data
   */
  private renderTable(result: ExecuteResult, analysis: ResultAnalysis) {
    if (!result.columns || !result.rows) {
      this.renderEmpty('No data');
      return;
    }

    const columns = result.columns;
    const rows = result.rows;
    const startX = TABLE_PADDING;
    const startY = TABLE_PADDING;

    // Draw header
    this.ctx.fillStyle = COLORS.bgSecondary;
    this.ctx.fillRect(startX, startY, columns.length * TABLE_CELL_WIDTH, TABLE_HEADER_HEIGHT);

    this.ctx.font = 'bold 12px Inter, sans-serif';
    this.ctx.fillStyle = COLORS.textPrimary;
    this.ctx.textAlign = 'left';
    this.ctx.textBaseline = 'middle';

    for (let i = 0; i < columns.length; i++) {
      const x = startX + i * TABLE_CELL_WIDTH + 12;
      this.ctx.fillText(
        this.truncateText(columns[i], TABLE_CELL_WIDTH - 24),
        x,
        startY + TABLE_HEADER_HEIGHT / 2
      );
    }

    // Draw header border
    this.ctx.strokeStyle = COLORS.borderPrimary;
    this.ctx.lineWidth = 1;
    this.ctx.strokeRect(startX, startY, columns.length * TABLE_CELL_WIDTH, TABLE_HEADER_HEIGHT);

    // Draw rows
    this.ctx.font = '12px Inter, sans-serif';
    for (let rowIdx = 0; rowIdx < rows.length; rowIdx++) {
      const row = rows[rowIdx];
      const y = startY + TABLE_HEADER_HEIGHT + rowIdx * TABLE_CELL_HEIGHT;

      // Alternate row background
      if (rowIdx % 2 === 1) {
        this.ctx.fillStyle = COLORS.bgSecondary;
        this.ctx.fillRect(startX, y, columns.length * TABLE_CELL_WIDTH, TABLE_CELL_HEIGHT);
      }

      // Draw cells
      for (let colIdx = 0; colIdx < columns.length; colIdx++) {
        const x = startX + colIdx * TABLE_CELL_WIDTH + 12;
        const value = row[colIdx];

        // Color ref values differently
        const isRef = analysis.nodeRefColumns.includes(colIdx) || analysis.edgeRefColumns.includes(colIdx);
        this.ctx.fillStyle = isRef ? COLORS.accent : COLORS.textSecondary;

        this.ctx.fillText(
          this.truncateText(formatCellValue(value), TABLE_CELL_WIDTH - 24),
          x,
          y + TABLE_CELL_HEIGHT / 2
        );
      }

      // Row border
      this.ctx.strokeStyle = COLORS.borderPrimary;
      this.ctx.strokeRect(startX, y, columns.length * TABLE_CELL_WIDTH, TABLE_CELL_HEIGHT);
    }

    // Build hit map
    this.hitMap = buildHitMapFromTable(
      0, // result index
      columns,
      rows,
      TABLE_CELL_WIDTH,
      TABLE_CELL_HEIGHT,
      startX,
      startY,
      TABLE_HEADER_HEIGHT
    );
  }

  /**
   * Render detail card (single row with refs)
   */
  private renderDetail(result: ExecuteResult, analysis: ResultAnalysis) {
    if (!result.columns || !result.rows || result.rows.length === 0) {
      this.renderEmpty('No data');
      return;
    }

    const columns = result.columns;
    const row = result.rows[0];
    const x = (this.canvas.width / this.zoom - DETAIL_CARD_WIDTH) / 2 - this.viewport.x;
    const y = TABLE_PADDING - this.viewport.y;

    // Card background
    const cardHeight = 48 + columns.length * DETAIL_ROW_HEIGHT + 16;
    this.ctx.fillStyle = COLORS.bgSecondary;
    this.roundRect(x, y, DETAIL_CARD_WIDTH, cardHeight, 8);
    this.ctx.fill();
    this.ctx.strokeStyle = COLORS.borderPrimary;
    this.ctx.stroke();

    // Title
    this.ctx.font = 'bold 14px Inter, sans-serif';
    this.ctx.fillStyle = COLORS.textPrimary;
    this.ctx.textAlign = 'left';
    this.ctx.textBaseline = 'middle';
    this.ctx.fillText('Detail View', x + 16, y + 24);

    // Separator
    this.ctx.beginPath();
    this.ctx.strokeStyle = COLORS.borderPrimary;
    this.ctx.moveTo(x + 16, y + 44);
    this.ctx.lineTo(x + DETAIL_CARD_WIDTH - 16, y + 44);
    this.ctx.stroke();

    // Fields
    this.ctx.font = '12px Inter, sans-serif';
    for (let i = 0; i < columns.length; i++) {
      const fieldY = y + 56 + i * DETAIL_ROW_HEIGHT;

      // Label
      this.ctx.fillStyle = COLORS.textSecondary;
      this.ctx.fillText(columns[i], x + 16, fieldY + DETAIL_ROW_HEIGHT / 2);

      // Value
      const isRef = analysis.nodeRefColumns.includes(i) || analysis.edgeRefColumns.includes(i);
      this.ctx.fillStyle = isRef ? COLORS.accent : COLORS.textPrimary;
      this.ctx.textAlign = 'right';
      this.ctx.fillText(
        this.truncateText(formatCellValue(row[i]), DETAIL_CARD_WIDTH / 2 - 32),
        x + DETAIL_CARD_WIDTH - 16,
        fieldY + DETAIL_ROW_HEIGHT / 2
      );
      this.ctx.textAlign = 'left';
    }

    // Build simple hit map for the card
    this.hitMap = {
      regions: [{
        bounds: { type: 'rect', x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
        ref: { type: 'row', result: 0, index: 0 },
        layer: 1,
        cursor: 'default',
      }],
      bounds: { x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
    };
  }

  /**
   * Render key-value pairs (single row without refs)
   */
  private renderKeyValue(result: ExecuteResult, _analysis: ResultAnalysis) {
    if (!result.columns || !result.rows || result.rows.length === 0) {
      this.renderEmpty('No data');
      return;
    }

    const columns = result.columns;
    const row = result.rows[0];
    const x = (this.canvas.width / this.zoom - DETAIL_CARD_WIDTH) / 2 - this.viewport.x;
    const y = TABLE_PADDING - this.viewport.y;

    // Card background
    const cardHeight = 48 + columns.length * DETAIL_ROW_HEIGHT + 16;
    this.ctx.fillStyle = COLORS.bgSecondary;
    this.roundRect(x, y, DETAIL_CARD_WIDTH, cardHeight, 8);
    this.ctx.fill();
    this.ctx.strokeStyle = COLORS.borderPrimary;
    this.ctx.stroke();

    // Title
    this.ctx.font = 'bold 14px Inter, sans-serif';
    this.ctx.fillStyle = COLORS.textPrimary;
    this.ctx.textAlign = 'left';
    this.ctx.textBaseline = 'middle';
    this.ctx.fillText('Result', x + 16, y + 24);

    // Separator
    this.ctx.beginPath();
    this.ctx.strokeStyle = COLORS.borderPrimary;
    this.ctx.moveTo(x + 16, y + 44);
    this.ctx.lineTo(x + DETAIL_CARD_WIDTH - 16, y + 44);
    this.ctx.stroke();

    // Key-value pairs
    this.ctx.font = '12px Inter, sans-serif';
    for (let i = 0; i < columns.length; i++) {
      const fieldY = y + 56 + i * DETAIL_ROW_HEIGHT;

      // Key
      this.ctx.fillStyle = COLORS.textSecondary;
      this.ctx.fillText(columns[i], x + 16, fieldY + DETAIL_ROW_HEIGHT / 2);

      // Value
      this.ctx.fillStyle = COLORS.textPrimary;
      this.ctx.textAlign = 'right';
      this.ctx.fillText(
        this.truncateText(formatCellValue(row[i]), DETAIL_CARD_WIDTH / 2 - 32),
        x + DETAIL_CARD_WIDTH - 16,
        fieldY + DETAIL_ROW_HEIGHT / 2
      );
      this.ctx.textAlign = 'left';
    }

    // Build hit map
    this.hitMap = {
      regions: [{
        bounds: { type: 'rect', x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
        ref: { type: 'row', result: 0, index: 0 },
        layer: 1,
        cursor: 'default',
      }],
      bounds: { x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
    };
  }

  /**
   * Render empty state
   */
  private renderEmpty(message: string = 'No results') {
    const centerX = this.canvas.width / 2 / this.zoom - this.viewport.x;
    const centerY = this.canvas.height / 2 / this.zoom - this.viewport.y;

    this.ctx.font = '16px Inter, sans-serif';
    this.ctx.fillStyle = COLORS.textMuted;
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';
    this.ctx.fillText(message, centerX, centerY);

    this.hitMap = { regions: [], bounds: { x: 0, y: 0, w: 0, h: 0 } };
  }

  /**
   * Render error state
   */
  private renderError(message: string) {
    const centerX = this.canvas.width / 2 / this.zoom - this.viewport.x;
    const centerY = this.canvas.height / 2 / this.zoom - this.viewport.y;

    // Error icon (simple X)
    this.ctx.beginPath();
    this.ctx.strokeStyle = COLORS.error;
    this.ctx.lineWidth = 3;
    this.ctx.moveTo(centerX - 20, centerY - 40);
    this.ctx.lineTo(centerX + 20, centerY - 10);
    this.ctx.moveTo(centerX + 20, centerY - 40);
    this.ctx.lineTo(centerX - 20, centerY - 10);
    this.ctx.stroke();

    // Error message
    this.ctx.font = '14px Inter, sans-serif';
    this.ctx.fillStyle = COLORS.error;
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'top';
    this.ctx.fillText(this.truncateText(message, 400), centerX, centerY + 10);

    this.hitMap = { regions: [], bounds: { x: 0, y: 0, w: 0, h: 0 } };
  }

  // Helper methods

  private getDisplayAttrs(attrs: Record<string, unknown>, max: number): [string, unknown][] {
    const priorityKeys = ['name', 'title', 'label', 'email', 'status', 'id'];
    const result: [string, unknown][] = [];

    for (const key of priorityKeys) {
      if (key in attrs && result.length < max) {
        result.push([key, attrs[key]]);
      }
    }

    for (const [key, value] of Object.entries(attrs)) {
      if (!priorityKeys.includes(key) && result.length < max) {
        result.push([key, value]);
      }
    }

    return result;
  }

  private truncateText(text: string, maxWidth: number): string {
    if (this.ctx.measureText(text).width <= maxWidth) return text;
    let truncated = text;
    while (truncated.length > 0 && this.ctx.measureText(truncated + '...').width > maxWidth) {
      truncated = truncated.slice(0, -1);
    }
    return truncated + '...';
  }

  private roundRect(x: number, y: number, w: number, h: number, r: number) {
    this.ctx.beginPath();
    this.ctx.moveTo(x + r, y);
    this.ctx.lineTo(x + w - r, y);
    this.ctx.quadraticCurveTo(x + w, y, x + w, y + r);
    this.ctx.lineTo(x + w, y + h - r);
    this.ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    this.ctx.lineTo(x + r, y + h);
    this.ctx.quadraticCurveTo(x, y + h, x, y + h - r);
    this.ctx.lineTo(x, y + r);
    this.ctx.quadraticCurveTo(x, y, x + r, y);
    this.ctx.closePath();
  }

  // Public API

  getHitMap(): HitMap | null {
    return this.hitMap;
  }

  getCurrentStrategy(): RenderStrategy {
    return this.currentStrategy;
  }

  setViewport(x: number, y: number) {
    this.viewport = { x, y };
  }

  setZoom(zoom: number) {
    this.zoom = Math.max(0.1, Math.min(5, zoom));
  }

  getViewport() {
    return { ...this.viewport };
  }

  getZoom() {
    return this.zoom;
  }

  fitToContent() {
    if (!this.hitMap || this.hitMap.bounds.w === 0) return;

    const bounds = this.hitMap.bounds;
    const padding = 50;
    const scaleX = (this.canvas.width - padding * 2) / bounds.w;
    const scaleY = (this.canvas.height - padding * 2) / bounds.h;
    this.zoom = Math.min(scaleX, scaleY, 2);
    this.viewport.x = -bounds.x + padding / this.zoom + (this.canvas.width / this.zoom - bounds.w) / 2;
    this.viewport.y = -bounds.y + padding / this.zoom + (this.canvas.height / this.zoom - bounds.h) / 2;
  }
}

/**
 * Helper to create render context from query result
 */
export function createRenderContext(
  result: ExecuteResult,
  graphData?: GraphData,
  layout?: Layout,
  focal?: Set<number>,
  selected?: number | null
): RenderContext {
  const analysis = analyzeResult(result);
  return {
    result,
    analysis,
    graphData,
    layout,
    focal,
    selected,
  };
}
