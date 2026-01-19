/**
 * ContentRenderer - Renders query results in world coordinates
 *
 * This renderer draws content (graphs, tables, etc.) in world space.
 * It doesn't handle navigation - that's the WorldCanvas's job.
 */

import type { ExecuteResult, GraphData, Layout } from '../types';
import { analyzeResult, type ResultAnalysis, type RenderStrategy, formatCellValue } from './result-analyzer';
import { buildHitMapFromLayout, buildHitMapFromTable, type HitMap, type HitRegion, hitTest } from './hitmap';

// Theme colors
const COLORS = {
  bgSecondary: '#1a1a1a',
  bgTertiary: '#2a2a2a',
  borderPrimary: '#333',
  borderSecondary: '#444',
  textPrimary: '#e5e5e5',
  textSecondary: '#a0a0a0',
  textMuted: '#666',
  accent: '#6366f1',
  success: '#22c55e',
  error: '#ef4444',
};

// Layout constants
const TABLE_CELL_WIDTH = 150;
const TABLE_CELL_HEIGHT = 32;
const TABLE_HEADER_HEIGHT = 36;
const DETAIL_CARD_WIDTH = 400;
const DETAIL_ROW_HEIGHT = 28;

export interface ContentBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RenderResult {
  strategy: RenderStrategy;
  bounds: ContentBounds;
  hitMap: HitMap;
}

export interface ContentRendererOptions {
  graphData?: GraphData;
  layout?: Layout;
  focal?: Set<number>;
  selected?: number | null;
}

/**
 * Render query result content in world coordinates
 * Returns the bounds of rendered content and a HitMap for interaction
 */
export function renderContent(
  ctx: CanvasRenderingContext2D,
  result: ExecuteResult,
  options: ContentRendererOptions = {}
): RenderResult {
  const analysis = analyzeResult(result);

  switch (analysis.strategy) {
    case 'graph':
      return renderGraph(ctx, result, analysis, options);
    case 'table':
      return renderTable(ctx, result, analysis);
    case 'detail':
      return renderDetail(ctx, result, analysis);
    case 'key-value':
      return renderKeyValue(ctx, result, analysis);
    case 'empty':
      return renderEmpty(ctx, 'No results');
    case 'error':
      return renderError(ctx, result.error ?? 'Unknown error');
    default:
      return renderEmpty(ctx, 'Unknown result type');
  }
}

/**
 * Render graph visualization
 */
function renderGraph(
  ctx: CanvasRenderingContext2D,
  _result: ExecuteResult,
  analysis: ResultAnalysis,
  options: ContentRendererOptions
): RenderResult {
  const { graphData, layout, focal = new Set(), selected = null } = options;

  if (!graphData || !layout) {
    return renderEmpty(ctx, 'No graph data');
  }

  // Draw edges
  for (const edge of graphData.edges) {
    const el = layout.edges.get(edge.id);
    if (!el) continue;

    const isFocal = focal.size === 0 || edge.targets.some(t => focal.has(t));

    ctx.beginPath();
    ctx.strokeStyle = isFocal ? COLORS.accent : COLORS.borderSecondary;
    ctx.lineWidth = isFocal ? 2 : 1;
    ctx.globalAlpha = isFocal ? 1 : 0.4;

    if (el.path.length >= 2) {
      ctx.moveTo(el.path[0][0], el.path[0][1]);
      for (let i = 1; i < el.path.length; i++) {
        ctx.lineTo(el.path[i][0], el.path[i][1]);
      }
    }
    ctx.stroke();
    ctx.globalAlpha = 1;

    // Arrow
    if (el.path.length >= 2) {
      drawArrow(ctx, el.path[el.path.length - 2], el.path[el.path.length - 1], isFocal);
    }
  }

  // Draw nodes
  for (const node of graphData.nodes) {
    const nl = layout.nodes.get(node.id);
    if (!nl) continue;

    const isFocal = focal.size === 0 || focal.has(node.id);
    const isSelected = selected === node.id;

    ctx.globalAlpha = isFocal ? 1 : 0.4;
    ctx.fillStyle = isFocal ? COLORS.bgTertiary : COLORS.bgSecondary;
    ctx.strokeStyle = isSelected ? COLORS.success : COLORS.borderSecondary;
    ctx.lineWidth = isSelected ? 2 : 1;
    roundRect(ctx, nl.x, nl.y, nl.width, nl.height, 6);
    ctx.fill();
    ctx.stroke();
    ctx.globalAlpha = 1;

    // Type label
    ctx.font = 'bold 11px Inter, sans-serif';
    ctx.fillStyle = isFocal ? COLORS.accent : COLORS.textMuted;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.fillText(node.type, nl.x + nl.width / 2, nl.y + 8);

    // Separator
    ctx.beginPath();
    ctx.strokeStyle = COLORS.borderPrimary;
    ctx.lineWidth = 0.5;
    ctx.moveTo(nl.x + 8, nl.y + 24);
    ctx.lineTo(nl.x + nl.width - 8, nl.y + 24);
    ctx.stroke();

    // Attributes
    const attrs = getDisplayAttrs(node.attrs, 3);
    ctx.font = '10px Inter, sans-serif';
    ctx.textAlign = 'left';
    let yOffset = 30;
    for (const [key, value] of attrs) {
      ctx.fillStyle = isFocal ? COLORS.textSecondary : COLORS.textMuted;
      const keyText = `${key}: `;
      ctx.fillText(keyText, nl.x + 8, nl.y + yOffset);
      const keyWidth = ctx.measureText(keyText).width;
      ctx.fillStyle = isFocal ? COLORS.textPrimary : COLORS.textMuted;
      const valueText = truncateText(ctx, String(value), nl.width - 16 - keyWidth);
      ctx.fillText(valueText, nl.x + 8 + keyWidth, nl.y + yOffset);
      yOffset += 14;
    }
  }

  // Calculate bounds
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const nl of layout.nodes.values()) {
    minX = Math.min(minX, nl.x);
    minY = Math.min(minY, nl.y);
    maxX = Math.max(maxX, nl.x + nl.width);
    maxY = Math.max(maxY, nl.y + nl.height);
  }

  const bounds = {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };

  // Build hit map
  const edgeLayouts = new Map<number, { path: [number, number][] }>();
  for (const [id, el] of layout.edges) {
    edgeLayouts.set(id, { path: el.path });
  }
  const hitMap = buildHitMapFromLayout(layout.nodes, edgeLayouts, focal);

  return { strategy: analysis.strategy, bounds, hitMap };
}

/**
 * Render table
 */
function renderTable(
  ctx: CanvasRenderingContext2D,
  result: ExecuteResult,
  analysis: ResultAnalysis
): RenderResult {
  if (!result.columns || !result.rows) {
    return renderEmpty(ctx, 'No data');
  }

  const columns = result.columns;
  const rows = result.rows;
  const startX = 0;
  const startY = 0;

  // Header background
  ctx.fillStyle = COLORS.bgSecondary;
  ctx.fillRect(startX, startY, columns.length * TABLE_CELL_WIDTH, TABLE_HEADER_HEIGHT);

  // Header text
  ctx.font = 'bold 12px Inter, sans-serif';
  ctx.fillStyle = COLORS.textPrimary;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'middle';

  for (let i = 0; i < columns.length; i++) {
    const x = startX + i * TABLE_CELL_WIDTH + 12;
    ctx.fillText(
      truncateText(ctx, columns[i], TABLE_CELL_WIDTH - 24),
      x,
      startY + TABLE_HEADER_HEIGHT / 2
    );
  }

  // Header border
  ctx.strokeStyle = COLORS.borderPrimary;
  ctx.lineWidth = 1;
  ctx.strokeRect(startX, startY, columns.length * TABLE_CELL_WIDTH, TABLE_HEADER_HEIGHT);

  // Rows
  ctx.font = '12px Inter, sans-serif';
  for (let rowIdx = 0; rowIdx < rows.length; rowIdx++) {
    const row = rows[rowIdx];
    const y = startY + TABLE_HEADER_HEIGHT + rowIdx * TABLE_CELL_HEIGHT;

    // Alternate background
    if (rowIdx % 2 === 1) {
      ctx.fillStyle = COLORS.bgSecondary;
      ctx.fillRect(startX, y, columns.length * TABLE_CELL_WIDTH, TABLE_CELL_HEIGHT);
    }

    // Cells
    for (let colIdx = 0; colIdx < columns.length; colIdx++) {
      const x = startX + colIdx * TABLE_CELL_WIDTH + 12;
      const value = row[colIdx];
      const isRef = analysis.nodeRefColumns.includes(colIdx) || analysis.edgeRefColumns.includes(colIdx);
      ctx.fillStyle = isRef ? COLORS.accent : COLORS.textSecondary;
      ctx.fillText(
        truncateText(ctx, formatCellValue(value), TABLE_CELL_WIDTH - 24),
        x,
        y + TABLE_CELL_HEIGHT / 2
      );
    }

    // Row border
    ctx.strokeStyle = COLORS.borderPrimary;
    ctx.strokeRect(startX, y, columns.length * TABLE_CELL_WIDTH, TABLE_CELL_HEIGHT);
  }

  const bounds = {
    x: startX,
    y: startY,
    width: columns.length * TABLE_CELL_WIDTH,
    height: TABLE_HEADER_HEIGHT + rows.length * TABLE_CELL_HEIGHT,
  };

  const hitMap = buildHitMapFromTable(
    0,
    columns,
    rows,
    TABLE_CELL_WIDTH,
    TABLE_CELL_HEIGHT,
    startX,
    startY,
    TABLE_HEADER_HEIGHT
  );

  return { strategy: 'table', bounds, hitMap };
}

/**
 * Render detail card
 */
function renderDetail(
  ctx: CanvasRenderingContext2D,
  result: ExecuteResult,
  analysis: ResultAnalysis
): RenderResult {
  if (!result.columns || !result.rows || result.rows.length === 0) {
    return renderEmpty(ctx, 'No data');
  }

  const columns = result.columns;
  const row = result.rows[0];
  const x = 0;
  const y = 0;
  const cardHeight = 48 + columns.length * DETAIL_ROW_HEIGHT + 16;

  // Card background
  ctx.fillStyle = COLORS.bgSecondary;
  roundRect(ctx, x, y, DETAIL_CARD_WIDTH, cardHeight, 8);
  ctx.fill();
  ctx.strokeStyle = COLORS.borderPrimary;
  ctx.stroke();

  // Title
  ctx.font = 'bold 14px Inter, sans-serif';
  ctx.fillStyle = COLORS.textPrimary;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'middle';
  ctx.fillText('Detail View', x + 16, y + 24);

  // Separator
  ctx.beginPath();
  ctx.strokeStyle = COLORS.borderPrimary;
  ctx.moveTo(x + 16, y + 44);
  ctx.lineTo(x + DETAIL_CARD_WIDTH - 16, y + 44);
  ctx.stroke();

  // Fields
  ctx.font = '12px Inter, sans-serif';
  for (let i = 0; i < columns.length; i++) {
    const fieldY = y + 56 + i * DETAIL_ROW_HEIGHT;
    ctx.fillStyle = COLORS.textSecondary;
    ctx.textAlign = 'left';
    ctx.fillText(columns[i], x + 16, fieldY + DETAIL_ROW_HEIGHT / 2);

    const isRef = analysis.nodeRefColumns.includes(i) || analysis.edgeRefColumns.includes(i);
    ctx.fillStyle = isRef ? COLORS.accent : COLORS.textPrimary;
    ctx.textAlign = 'right';
    ctx.fillText(
      truncateText(ctx, formatCellValue(row[i]), DETAIL_CARD_WIDTH / 2 - 32),
      x + DETAIL_CARD_WIDTH - 16,
      fieldY + DETAIL_ROW_HEIGHT / 2
    );
  }

  const bounds = { x, y, width: DETAIL_CARD_WIDTH, height: cardHeight };
  const hitMap: HitMap = {
    regions: [{
      bounds: { type: 'rect', x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
      ref: { type: 'row', result: 0, index: 0 },
      layer: 1,
      cursor: 'default',
    }],
    bounds: { x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
  };

  return { strategy: 'detail', bounds, hitMap };
}

/**
 * Render key-value pairs
 */
function renderKeyValue(
  ctx: CanvasRenderingContext2D,
  result: ExecuteResult,
  _analysis: ResultAnalysis
): RenderResult {
  if (!result.columns || !result.rows || result.rows.length === 0) {
    return renderEmpty(ctx, 'No data');
  }

  const columns = result.columns;
  const row = result.rows[0];
  const x = 0;
  const y = 0;
  const cardHeight = 48 + columns.length * DETAIL_ROW_HEIGHT + 16;

  // Card background
  ctx.fillStyle = COLORS.bgSecondary;
  roundRect(ctx, x, y, DETAIL_CARD_WIDTH, cardHeight, 8);
  ctx.fill();
  ctx.strokeStyle = COLORS.borderPrimary;
  ctx.stroke();

  // Title
  ctx.font = 'bold 14px Inter, sans-serif';
  ctx.fillStyle = COLORS.textPrimary;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'middle';
  ctx.fillText('Result', x + 16, y + 24);

  // Separator
  ctx.beginPath();
  ctx.strokeStyle = COLORS.borderPrimary;
  ctx.moveTo(x + 16, y + 44);
  ctx.lineTo(x + DETAIL_CARD_WIDTH - 16, y + 44);
  ctx.stroke();

  // Fields
  ctx.font = '12px Inter, sans-serif';
  for (let i = 0; i < columns.length; i++) {
    const fieldY = y + 56 + i * DETAIL_ROW_HEIGHT;
    ctx.fillStyle = COLORS.textSecondary;
    ctx.textAlign = 'left';
    ctx.fillText(columns[i], x + 16, fieldY + DETAIL_ROW_HEIGHT / 2);

    ctx.fillStyle = COLORS.textPrimary;
    ctx.textAlign = 'right';
    ctx.fillText(
      truncateText(ctx, formatCellValue(row[i]), DETAIL_CARD_WIDTH / 2 - 32),
      x + DETAIL_CARD_WIDTH - 16,
      fieldY + DETAIL_ROW_HEIGHT / 2
    );
  }

  const bounds = { x, y, width: DETAIL_CARD_WIDTH, height: cardHeight };
  const hitMap: HitMap = {
    regions: [{
      bounds: { type: 'rect', x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
      ref: { type: 'row', result: 0, index: 0 },
      layer: 1,
      cursor: 'default',
    }],
    bounds: { x, y, w: DETAIL_CARD_WIDTH, h: cardHeight },
  };

  return { strategy: 'key-value', bounds, hitMap };
}

/**
 * Render empty state
 */
function renderEmpty(ctx: CanvasRenderingContext2D, message: string): RenderResult {
  ctx.font = '16px Inter, sans-serif';
  ctx.fillStyle = COLORS.textMuted;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(message, 0, 0);

  return {
    strategy: 'empty',
    bounds: { x: -100, y: -20, width: 200, height: 40 },
    hitMap: { regions: [], bounds: { x: 0, y: 0, w: 0, h: 0 } },
  };
}

/**
 * Render error state
 */
function renderError(ctx: CanvasRenderingContext2D, message: string): RenderResult {
  // Error X
  ctx.beginPath();
  ctx.strokeStyle = COLORS.error;
  ctx.lineWidth = 3;
  ctx.moveTo(-20, -30);
  ctx.lineTo(20, 0);
  ctx.moveTo(20, -30);
  ctx.lineTo(-20, 0);
  ctx.stroke();

  // Message
  ctx.font = '14px Inter, sans-serif';
  ctx.fillStyle = COLORS.error;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';
  ctx.fillText(truncateText(ctx, message, 400), 0, 20);

  return {
    strategy: 'error',
    bounds: { x: -200, y: -30, width: 400, height: 80 },
    hitMap: { regions: [], bounds: { x: 0, y: 0, w: 0, h: 0 } },
  };
}

// Helper functions

function drawArrow(ctx: CanvasRenderingContext2D, from: [number, number], to: [number, number], isFocal: boolean) {
  const headLen = 10;
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const angle = Math.atan2(dy, dx);

  ctx.beginPath();
  ctx.moveTo(to[0], to[1]);
  ctx.lineTo(to[0] - headLen * Math.cos(angle - Math.PI / 6), to[1] - headLen * Math.sin(angle - Math.PI / 6));
  ctx.lineTo(to[0] - headLen * Math.cos(angle + Math.PI / 6), to[1] - headLen * Math.sin(angle + Math.PI / 6));
  ctx.closePath();
  ctx.fillStyle = isFocal ? COLORS.accent : COLORS.borderSecondary;
  ctx.globalAlpha = isFocal ? 1 : 0.4;
  ctx.fill();
  ctx.globalAlpha = 1;
}

function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, y + h - r);
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
  ctx.lineTo(x + r, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
}

function getDisplayAttrs(attrs: Record<string, unknown>, max: number): [string, unknown][] {
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

function truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
  if (ctx.measureText(text).width <= maxWidth) return text;
  let truncated = text;
  while (truncated.length > 0 && ctx.measureText(truncated + '...').width > maxWidth) {
    truncated = truncated.slice(0, -1);
  }
  return truncated + '...';
}

/**
 * Hit test helper that works with world coordinates
 */
export function hitTestWorld(
  hitMap: HitMap,
  worldX: number,
  worldY: number
): HitRegion | null {
  return hitTest(hitMap, worldX, worldY);
}
