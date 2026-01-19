import { useState } from 'react';
import { useViewStore, useSessionStore, useChatStore, useUIStore } from '../../stores';
import { ChatMessages } from '../chat/ChatMessages';
import { ChatInput } from '../chat/ChatInput';
import type { NodeData } from '../../types';

/**
 * RightPanel - Combined panel with Node Details (top) and AI Chat (bottom)
 */
export function RightPanel() {
  const [detailHeight, setDetailHeight] = useState(200);
  const [isResizing, setIsResizing] = useState(false);

  // Node details state
  const selected = useViewStore((state) => state.selected);
  const graph = useSessionStore((state) => state.graph);

  // Chat state
  const openSettings = useUIStore((state) => state.openSettings);
  const clearHistory = useChatStore((state) => state.clearHistory);

  const selectedNode: NodeData | null =
    selected !== null && graph
      ? graph.nodes.find((n) => n.id === selected) ?? null
      : null;

  // Handle resize
  const handleResizeStart = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);

    const startY = e.clientY;
    const startHeight = detailHeight;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientY - startY;
      const newHeight = Math.max(100, Math.min(400, startHeight + delta));
      setDetailHeight(newHeight);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  return (
    <aside className="right-panel">
      {/* Node Details Section */}
      <div className="right-panel__section right-panel__details" style={{ height: detailHeight }}>
        <div className="right-panel__header">
          <span className="right-panel__title">Node Details</span>
        </div>
        <div className="right-panel__content">
          {selectedNode ? (
            <div className="node-detail">
              <div className="node-type">{selectedNode.type}</div>
              <div className="node-id">ID: {selectedNode.id}</div>
              {Object.entries(selectedNode.attrs).length > 0 && (
                <div className="node-attrs">
                  <h3 className="node-attrs__title">Attributes</h3>
                  {Object.entries(selectedNode.attrs).map(([key, value]) => (
                    <div key={key} className="node-attr">
                      <span className="node-attr__key">{key}:</span>
                      <span className="node-attr__value">{formatValue(value)}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <div className="right-panel__empty">
              Select a node to view details
            </div>
          )}
        </div>
      </div>

      {/* Resize Handle */}
      <div
        className={`right-panel__resize ${isResizing ? 'right-panel__resize--active' : ''}`}
        onMouseDown={handleResizeStart}
      />

      {/* AI Chat Section */}
      <div className="right-panel__section right-panel__chat">
        <div className="right-panel__header">
          <span className="right-panel__title right-panel__title--accent">AI Assistant</span>
          <div className="right-panel__actions">
            <button
              className="right-panel__btn"
              onClick={openSettings}
              title="Settings"
            >
              <SettingsIcon />
            </button>
            <button
              className="right-panel__btn"
              onClick={clearHistory}
              title="Clear chat"
            >
              <TrashIcon />
            </button>
          </div>
        </div>
        <div className="right-panel__content right-panel__chat-content">
          <ChatMessages />
          <ChatInput />
        </div>
      </div>
    </aside>
  );
}

function formatValue(value: unknown): string {
  if (value === null) return 'null';
  if (value === undefined) return 'undefined';
  if (typeof value === 'string') return `"${value}"`;
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

function SettingsIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}
