/**
 * HitMap - Universal interactivity layer for visualization
 * Maps visual regions to data references for interaction handling
 */

import type { NodeLayout } from '../types';

// Reference types - what a hit region points to
export type Reference =
  | { type: 'row'; result: number; index: number }
  | { type: 'entity'; id: number; kind: 'node' | 'edge' }
  | { type: 'cell'; result: number; row: number; col: number }
  | { type: 'custom'; kind: string; data: unknown };

// Bounds variants for hit regions
export type Bounds =
  | { type: 'rect'; x: number; y: number; w: number; h: number }
  | { type: 'circle'; cx: number; cy: number; r: number };

// Cursor hints
export type CursorType = 'default' | 'pointer' | 'grab' | 'grabbing' | 'text';

// Affordance hints for discoverability
export type Affordance = 'clickable' | 'draggable' | 'expandable';

// Action hints for context menus
export interface ActionHint {
  id: string;
  label: string;
  shortcut?: string;
  destructive?: boolean;
}

// Tooltip content
export interface TooltipContent {
  title: string;
  subtitle?: string;
  details?: Array<{ label: string; value: string }>;
}

// A single hit region
export interface HitRegion {
  bounds: Bounds;
  ref: Reference;
  layer: number;  // Higher = on top
  cursor: CursorType;
  tooltip?: TooltipContent;
  affordance?: Affordance;
  actions?: ActionHint[];
}

// The HitMap structure
export interface HitMap {
  regions: HitRegion[];
  bounds: { x: number; y: number; w: number; h: number };
}

/**
 * Build a HitMap from graph layout
 */
export function buildHitMapFromLayout(
  nodes: Map<number, NodeLayout>,
  edges: Map<number, { path: [number, number][] }>,
  focal: Set<number>
): HitMap {
  const regions: HitRegion[] = [];

  // Add node hit regions
  for (const [id, layout] of nodes) {
    const isFocal = focal.size === 0 || focal.has(id);
    regions.push({
      bounds: {
        type: 'rect',
        x: layout.x,
        y: layout.y,
        w: layout.width,
        h: layout.height,
      },
      ref: { type: 'entity', id, kind: 'node' },
      layer: isFocal ? 10 : 5,
      cursor: 'pointer',
      affordance: 'clickable',
    });
  }

  // Add edge hit regions (simple line approximation with padding)
  for (const [id, layout] of edges) {
    if (layout.path.length >= 2) {
      // Create a bounding box around the edge path
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      for (const [x, y] of layout.path) {
        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        maxX = Math.max(maxX, x);
        maxY = Math.max(maxY, y);
      }
      const padding = 5;
      regions.push({
        bounds: {
          type: 'rect',
          x: minX - padding,
          y: minY - padding,
          w: maxX - minX + padding * 2,
          h: maxY - minY + padding * 2,
        },
        ref: { type: 'entity', id, kind: 'edge' },
        layer: 1,
        cursor: 'pointer',
      });
    }
  }

  // Calculate overall bounds
  let boundsX = 0, boundsY = 0, boundsW = 800, boundsH = 600;
  if (regions.length > 0) {
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const region of regions) {
      if (region.bounds.type === 'rect') {
        minX = Math.min(minX, region.bounds.x);
        minY = Math.min(minY, region.bounds.y);
        maxX = Math.max(maxX, region.bounds.x + region.bounds.w);
        maxY = Math.max(maxY, region.bounds.y + region.bounds.h);
      }
    }
    boundsX = minX;
    boundsY = minY;
    boundsW = maxX - minX;
    boundsH = maxY - minY;
  }

  return {
    regions,
    bounds: { x: boundsX, y: boundsY, w: boundsW, h: boundsH },
  };
}

/**
 * Build a HitMap from table results
 */
export function buildHitMapFromTable(
  resultIndex: number,
  columns: string[],
  rows: unknown[][],
  cellWidth: number,
  cellHeight: number,
  startX: number,
  startY: number,
  headerHeight: number
): HitMap {
  const regions: HitRegion[] = [];

  // Add cell hit regions
  for (let rowIdx = 0; rowIdx < rows.length; rowIdx++) {
    for (let colIdx = 0; colIdx < columns.length; colIdx++) {
      const x = startX + colIdx * cellWidth;
      const y = startY + headerHeight + rowIdx * cellHeight;
      regions.push({
        bounds: { type: 'rect', x, y, w: cellWidth, h: cellHeight },
        ref: { type: 'cell', result: resultIndex, row: rowIdx, col: colIdx },
        layer: 5,
        cursor: 'default',
      });
    }
  }

  // Add row hit regions (for row selection)
  for (let rowIdx = 0; rowIdx < rows.length; rowIdx++) {
    const y = startY + headerHeight + rowIdx * cellHeight;
    regions.push({
      bounds: { type: 'rect', x: startX, y, w: columns.length * cellWidth, h: cellHeight },
      ref: { type: 'row', result: resultIndex, index: rowIdx },
      layer: 1,
      cursor: 'pointer',
      affordance: 'clickable',
    });
  }

  return {
    regions,
    bounds: {
      x: startX,
      y: startY,
      w: columns.length * cellWidth,
      h: headerHeight + rows.length * cellHeight,
    },
  };
}

/**
 * Hit test - find what's at a given point
 */
export function hitTest(hitMap: HitMap, x: number, y: number): HitRegion | null {
  // Find all regions containing the point
  const hits: HitRegion[] = [];

  for (const region of hitMap.regions) {
    if (containsPoint(region.bounds, x, y)) {
      hits.push(region);
    }
  }

  if (hits.length === 0) return null;

  // Sort by layer (highest first) and return top hit
  hits.sort((a, b) => b.layer - a.layer);
  return hits[0];
}

/**
 * Check if bounds contain a point
 */
function containsPoint(bounds: Bounds, x: number, y: number): boolean {
  if (bounds.type === 'rect') {
    return x >= bounds.x && x <= bounds.x + bounds.w &&
           y >= bounds.y && y <= bounds.y + bounds.h;
  } else if (bounds.type === 'circle') {
    const dx = x - bounds.cx;
    const dy = y - bounds.cy;
    return dx * dx + dy * dy <= bounds.r * bounds.r;
  }
  return false;
}

/**
 * Get all regions of a specific type
 */
export function getRegionsByType(hitMap: HitMap, refType: Reference['type']): HitRegion[] {
  return hitMap.regions.filter(r => r.ref.type === refType);
}

/**
 * Get region by reference
 */
export function getRegionByRef(hitMap: HitMap, ref: Reference): HitRegion | undefined {
  return hitMap.regions.find(r => {
    if (r.ref.type !== ref.type) return false;
    if (ref.type === 'entity' && r.ref.type === 'entity') {
      return r.ref.id === ref.id && r.ref.kind === ref.kind;
    }
    if (ref.type === 'row' && r.ref.type === 'row') {
      return r.ref.result === ref.result && r.ref.index === ref.index;
    }
    if (ref.type === 'cell' && r.ref.type === 'cell') {
      return r.ref.result === ref.result && r.ref.row === ref.row && r.ref.col === ref.col;
    }
    return false;
  });
}
