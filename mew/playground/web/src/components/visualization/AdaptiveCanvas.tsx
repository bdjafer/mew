import React, { useRef, useEffect, useCallback, useState } from 'react';
import { useSessionStore, useViewStore, useResultsStore } from '../../stores';
import { WorldCanvas, type WorldTransform } from '../../viz/world-canvas';
import { renderContent, hitTestWorld, type RenderResult } from '../../viz/content-renderer';
import { analyzeGraph, selectStrategy, computeLayout, analyzeResult } from '../../viz';
import type { Layout, ExecuteResult } from '../../types';
import type { HitMap } from '../../viz/hitmap';

/**
 * AdaptiveCanvas - World canvas with adaptive content rendering
 *
 * Two-layer model:
 * 1. WorldCanvas - Infinite navigable space (pan/zoom always works)
 * 2. Content - Rendered views placed within world space
 */
export const AdaptiveCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const worldCanvasRef = useRef<WorldCanvas | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const hitMapRef = useRef<HitMap | null>(null);
  const renderResultRef = useRef<RenderResult | null>(null);

  // Session state
  const graph = useSessionStore((state) => state.graph);
  const schema = useSessionStore((state) => state.schema);

  // View state
  const focal = useViewStore((state) => state.focal);
  const selected = useViewStore((state) => state.selected);
  const query = useViewStore((state) => state.query);
  const setSelected = useViewStore((state) => state.setSelected);
  const setFocal = useViewStore((state) => state.setFocal);
  const setZoom = useViewStore((state) => state.setZoom);

  // Results state
  const lastResult = useResultsStore((state) => state.lastResult);

  // Local UI state
  const [strategy, setStrategy] = useState<string>('empty');
  const [transform, setTransform] = useState<WorldTransform>({ panX: 0, panY: 0, zoom: 1 });

  // Compute content data (memoized computation)
  const getContentData = useCallback(() => {
    let result: ExecuteResult;
    let layout: Layout | undefined;

    if (lastResult) {
      result = lastResult;
      const analysis = analyzeResult(lastResult);

      if (analysis.strategy === 'graph' && graph) {
        const viewState = { query, viewport: { x: 0, y: 0 }, zoom: 1, focal, selected };
        const graphAnalysis = analyzeGraph(graph, schema);
        const layoutStrategy = selectStrategy(graphAnalysis, viewState);
        layout = computeLayout(graph, layoutStrategy, graphAnalysis);
      }
    } else if (graph) {
      // Synthetic result for graph display
      result = {
        success: true,
        result_type: 'query',
        columns: ['node'],
        rows: graph.nodes.map(n => [{ _type: 'node', _id: n.id }]),
      };
      const viewState = { query, viewport: { x: 0, y: 0 }, zoom: 1, focal, selected };
      const graphAnalysis = analyzeGraph(graph, schema);
      const layoutStrategy = selectStrategy(graphAnalysis, viewState);
      layout = computeLayout(graph, layoutStrategy, graphAnalysis);
    } else {
      result = { success: true, result_type: 'empty' };
    }

    return { result, layout };
  }, [lastResult, graph, schema, focal, selected, query]);

  // Content renderer callback for WorldCanvas
  const contentRenderer = useCallback((ctx: CanvasRenderingContext2D, _transform: WorldTransform) => {
    const { result, layout } = getContentData();

    const renderResult = renderContent(ctx, result, {
      graphData: graph ?? undefined,
      layout,
      focal,
      selected,
    });

    // Store for hit testing and bounds
    renderResultRef.current = renderResult;
    hitMapRef.current = renderResult.hitMap;
    setStrategy(renderResult.strategy);
  }, [getContentData, graph, focal, selected]);

  // Handle world clicks
  const handleWorldClick = useCallback((worldX: number, worldY: number) => {
    if (!hitMapRef.current) return;

    const hit = hitTestWorld(hitMapRef.current, worldX, worldY);
    if (!hit) return;

    if (hit.ref.type === 'entity' && hit.ref.kind === 'node') {
      setSelected(hit.ref.id);
      setFocal(new Set([hit.ref.id]));
    }
  }, [setSelected, setFocal]);

  // Handle transform changes
  const handleTransformChange = useCallback((newTransform: WorldTransform) => {
    setTransform(newTransform);
    setZoom(newTransform.zoom);
  }, [setZoom]);

  // Initialize WorldCanvas
  useEffect(() => {
    if (!canvasRef.current) return;

    const worldCanvas = new WorldCanvas({
      canvas: canvasRef.current,
      backgroundColor: '#0a0a0a',
      grid: {
        size: 50,
        color: '#1a1a1a',
        opacity: 0.5,
      },
      onTransformChange: handleTransformChange,
      onWorldClick: handleWorldClick,
    });

    worldCanvas.setContentRenderer(contentRenderer);
    worldCanvasRef.current = worldCanvas;

    // Initial render
    worldCanvas.resize();
    worldCanvas.render();

    return () => {
      worldCanvas.destroy();
      worldCanvasRef.current = null;
    };
  }, [contentRenderer, handleTransformChange, handleWorldClick]);

  // Update content renderer when data changes
  useEffect(() => {
    if (worldCanvasRef.current) {
      worldCanvasRef.current.setContentRenderer(contentRenderer);
      worldCanvasRef.current.render();
    }
  }, [contentRenderer]);

  // Handle resize
  useEffect(() => {
    if (!containerRef.current || !worldCanvasRef.current) return;

    const resizeObserver = new ResizeObserver(() => {
      worldCanvasRef.current?.resize();
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  // Fit content on initial load or when content changes significantly
  useEffect(() => {
    if (!worldCanvasRef.current) return;

    // Wait for render to complete
    const timeout = setTimeout(() => {
      if (renderResultRef.current && renderResultRef.current.bounds.width > 0) {
        const { x, y, width, height } = renderResultRef.current.bounds;
        worldCanvasRef.current?.fitBounds(x, y, width, height, 80);
      } else {
        // Center on origin if no content
        worldCanvasRef.current?.centerOn(0, 0);
      }
    }, 100);

    return () => clearTimeout(timeout);
  }, [lastResult, graph]);

  // Determine placeholder
  const showPlaceholder = !graph && !lastResult;

  return (
    <div ref={containerRef} className="graph-canvas-container">
      <canvas ref={canvasRef} className="graph-canvas" />
      {showPlaceholder && (
        <div className="graph-canvas-placeholder">
          Load an ontology and run a query to visualize data
        </div>
      )}
      <div className="render-strategy-badge">
        {strategy}
      </div>
      <div className="world-canvas-info">
        <span>Zoom: {Math.round(transform.zoom * 100)}%</span>
      </div>
    </div>
  );
};

// Export controls for external use
export const useAdaptiveCanvasControls = () => {
  return {
    // Controls will be implemented via refs
  };
};
