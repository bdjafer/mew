import React, { useRef, useEffect, useCallback } from 'react';
import { useSessionStore, useViewStore } from '../../stores';
import { Renderer } from '../../viz/renderer';
import { analyzeGraph, selectStrategy, computeLayout } from '../../viz';
import type { NodeData, Layout } from '../../types';

export const GraphCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<Renderer | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const graph = useSessionStore((state) => state.graph);
  const schema = useSessionStore((state) => state.schema);
  const focal = useViewStore((state) => state.focal);
  const selected = useViewStore((state) => state.selected);
  const query = useViewStore((state) => state.query);
  const viewport = useViewStore((state) => state.viewport);
  const zoom = useViewStore((state) => state.zoom);
  const setZoom = useViewStore((state) => state.setZoom);
  const setSelected = useViewStore((state) => state.setSelected);
  const setFocal = useViewStore((state) => state.setFocal);

  const handleNodeClick = useCallback((node: NodeData) => {
    setSelected(node.id);
    setFocal(new Set([node.id]));
  }, [setSelected, setFocal]);

  const handleNodeHover = useCallback((_node: NodeData | null) => {
    // Could be used for tooltips or highlighting
  }, []);

  const handleZoomChange = useCallback((zoom: number) => {
    setZoom(zoom);
  }, [setZoom]);

  // Initialize renderer
  useEffect(() => {
    if (!canvasRef.current) return;

    const renderer = new Renderer(canvasRef.current);
    renderer.onNodeClick = handleNodeClick;
    renderer.onNodeHover = handleNodeHover;
    renderer.onZoomChange = handleZoomChange;
    rendererRef.current = renderer;

    return () => {
      rendererRef.current = null;
    };
  }, [handleNodeClick, handleNodeHover, handleZoomChange]);

  // Handle resize
  useEffect(() => {
    if (!containerRef.current || !rendererRef.current) return;

    const resizeObserver = new ResizeObserver(() => {
      rendererRef.current?.resize();
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  // Render when graph, focal, or selected changes
  useEffect(() => {
    if (!rendererRef.current || !graph) return;

    const viewState = { query, viewport, zoom, focal, selected };
    const analysis = analyzeGraph(graph, schema);
    const strategy = selectStrategy(analysis, viewState);
    const layout: Layout = computeLayout(graph, strategy, analysis);

    rendererRef.current.render(graph, layout, focal, selected);
  }, [graph, schema, focal, selected, query, viewport, zoom]);

  // Fit to view on initial load
  useEffect(() => {
    if (graph && graph.nodes.length > 0 && rendererRef.current) {
      // Small delay to ensure layout is calculated
      setTimeout(() => {
        rendererRef.current?.fitToView();
      }, 100);
    }
  }, [graph]);

  return (
    <div ref={containerRef} className="graph-canvas-container">
      <canvas ref={canvasRef} className="graph-canvas" />
      {!graph && (
        <div className="graph-canvas-placeholder">
          Load an ontology and run a query to visualize data
        </div>
      )}
    </div>
  );
};

// Export methods for external control
export const useGraphCanvasControls = () => {
  const rendererRef = useRef<Renderer | null>(null);

  return {
    zoomIn: () => rendererRef.current?.zoomIn(),
    zoomOut: () => rendererRef.current?.zoomOut(),
    fitToView: () => rendererRef.current?.fitToView(),
  };
};
