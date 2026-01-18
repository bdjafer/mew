import { useViewStore, useSessionStore } from '../../stores';
import type { NodeData } from '../../types';

export function DetailPanel() {
  const selected = useViewStore((state) => state.selected);
  const graph = useSessionStore((state) => state.graph);

  const selectedNode: NodeData | null =
    selected !== null && graph
      ? graph.nodes.find((n) => n.id === selected) ?? null
      : null;

  return (
    <aside id="detail-panel">
      <h2>Node Details</h2>
      {selectedNode ? (
        <div className="node-detail">
          <div className="node-type">{selectedNode.type}</div>
          <div className="node-id">ID: {selectedNode.id}</div>
          {Object.entries(selectedNode.attrs).length > 0 && (
            <div className="node-attrs">
              <h3
                style={{
                  fontSize: '0.7rem',
                  fontWeight: 600,
                  textTransform: 'uppercase',
                  letterSpacing: '0.05em',
                  color: 'var(--text-secondary)',
                  marginTop: '0.75rem',
                  marginBottom: '0.5rem',
                }}
              >
                Attributes
              </h3>
              {Object.entries(selectedNode.attrs).map(([key, value]) => (
                <div key={key}>
                  <strong>{key}:</strong> {formatValue(value)}
                </div>
              ))}
            </div>
          )}
        </div>
      ) : (
        <div style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
          Select a node to view details
        </div>
      )}
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
