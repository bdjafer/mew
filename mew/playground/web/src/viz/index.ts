export { analyzeGraph } from './analyzer';
export { selectStrategy, getStrategyDescription, type LayoutStrategy } from './strategy';
export { computeLayout } from './layout';
export { Renderer } from './renderer';

// Result analysis
export { analyzeResult, formatCellValue, extractEntityIds, extractRefId } from './result-analyzer';
export type { RenderStrategy, ResultAnalysis } from './result-analyzer';

// HitMap system
export { buildHitMapFromLayout, buildHitMapFromTable, hitTest, getRegionsByType, getRegionByRef } from './hitmap';
export type { HitMap, HitRegion, Reference, Bounds, CursorType, Affordance, ActionHint, TooltipContent } from './hitmap';

// World canvas system
export { WorldCanvas } from './world-canvas';
export type { WorldTransform, WorldCanvasOptions } from './world-canvas';

// Content renderer
export { renderContent, hitTestWorld } from './content-renderer';
export type { ContentBounds, RenderResult, ContentRendererOptions } from './content-renderer';

// Legacy auto-renderer (for reference)
export { AutoRenderer, createRenderContext } from './auto-renderer';
export type { AutoRendererOptions, RenderContext } from './auto-renderer';
