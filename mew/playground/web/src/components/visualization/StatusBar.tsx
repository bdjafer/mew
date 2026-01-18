import React, { useMemo, useCallback, useRef, useEffect } from 'react';
import { useSessionStore, useViewStore } from '../../stores';
import { Renderer } from '../../viz/renderer';

interface StatusBarProps {
  rendererRef?: React.MutableRefObject<Renderer | null>;
}

export const StatusBar: React.FC<StatusBarProps> = ({ rendererRef: externalRendererRef }) => {
  const graph = useSessionStore((state) => state.graph);
  const status = useSessionStore((state) => state.status);
  const zoom = useViewStore((state) => state.zoom);
  const selected = useViewStore((state) => state.selected);
  const setZoom = useViewStore((state) => state.setZoom);

  // Internal renderer ref for zoom control access
  const internalRendererRef = useRef<Renderer | null>(null);
  const rendererRef = externalRendererRef ?? internalRendererRef;

  // Calculate graph stats
  const stats = useMemo(() => {
    if (!graph) {
      return { nodeCount: 0, edgeCount: 0 };
    }
    return {
      nodeCount: graph.nodes.length,
      edgeCount: graph.edges.length,
    };
  }, [graph]);

  // Get selected node info
  const selectedInfo = useMemo(() => {
    if (!graph || selected === null) return null;
    const node = graph.nodes.find((n) => n.id === selected);
    if (!node) return null;
    return {
      id: node.id,
      type: node.type,
    };
  }, [graph, selected]);

  const handleZoomIn = useCallback(() => {
    if (rendererRef.current) {
      rendererRef.current.zoomIn();
    } else {
      setZoom(Math.min(5, zoom * 1.2));
    }
  }, [rendererRef, setZoom, zoom]);

  const handleZoomOut = useCallback(() => {
    if (rendererRef.current) {
      rendererRef.current.zoomOut();
    } else {
      setZoom(Math.max(0.1, zoom / 1.2));
    }
  }, [rendererRef, setZoom, zoom]);

  const handleFitToView = useCallback(() => {
    if (rendererRef.current) {
      rendererRef.current.fitToView();
    }
  }, [rendererRef]);

  // Access renderer from global for zoom controls
  useEffect(() => {
    // Get renderer reference from GraphCanvas via window
    const checkRenderer = () => {
      const canvas = document.querySelector('.graph-canvas') as HTMLCanvasElement | null;
      if (canvas && (canvas as unknown as { __renderer?: Renderer }).__renderer) {
        internalRendererRef.current = (canvas as unknown as { __renderer: Renderer }).__renderer;
      }
    };
    checkRenderer();
    const interval = setInterval(checkRenderer, 1000);
    return () => clearInterval(interval);
  }, []);

  const zoomPercentage = Math.round(zoom * 100);

  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <span className="status-bar-item">
          Nodes: <strong>{stats.nodeCount}</strong>
        </span>
        <span className="status-bar-separator">|</span>
        <span className="status-bar-item">
          Edges: <strong>{stats.edgeCount}</strong>
        </span>
        {selectedInfo && (
          <>
            <span className="status-bar-separator">|</span>
            <span className="status-bar-item status-bar-selection">
              Selected: <strong>{selectedInfo.type} #{selectedInfo.id}</strong>
            </span>
          </>
        )}
      </div>
      <div className="status-bar-center">
        <span className={`status-bar-status${status.toLowerCase().includes('error') ? ' status-bar-error' : ''}`}>
          {status}
        </span>
      </div>
      <div className="status-bar-right">
        <div className="zoom-controls">
          <button
            className="zoom-button"
            onClick={handleZoomOut}
            title="Zoom out"
            aria-label="Zoom out"
          >
            -
          </button>
          <span className="zoom-percentage">{zoomPercentage}%</span>
          <button
            className="zoom-button"
            onClick={handleZoomIn}
            title="Zoom in"
            aria-label="Zoom in"
          >
            +
          </button>
          <button
            className="zoom-button zoom-fit"
            onClick={handleFitToView}
            title="Fit to view"
            aria-label="Fit to view"
          >
            Fit
          </button>
        </div>
      </div>
    </div>
  );
};
