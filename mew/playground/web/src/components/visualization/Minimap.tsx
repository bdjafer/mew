import React, { useMemo } from 'react';
import { useSessionStore, useEditorStore } from '../../stores';

export const Minimap: React.FC = () => {
  const schema = useSessionStore((state) => state.schema);
  const graph = useSessionStore((state) => state.graph);
  const executeQuery = useSessionStore((state) => state.executeQuery);
  const setQueryContent = useEditorStore((state) => state.setQueryContent);

  // Calculate node type counts from graph
  const nodeTypeCounts = useMemo(() => {
    if (!graph) return new Map<string, number>();
    const counts = new Map<string, number>();
    for (const node of graph.nodes) {
      counts.set(node.type, (counts.get(node.type) ?? 0) + 1);
    }
    return counts;
  }, [graph]);

  // Calculate edge type counts from graph
  const edgeTypeCounts = useMemo(() => {
    if (!graph) return new Map<string, number>();
    const counts = new Map<string, number>();
    for (const edge of graph.edges) {
      counts.set(edge.type, (counts.get(edge.type) ?? 0) + 1);
    }
    return counts;
  }, [graph]);

  // Get node types from schema
  const nodeTypes = useMemo(() => {
    if (!schema) return [];
    return schema.types.map((t) => ({
      name: t.name,
      count: nodeTypeCounts.get(t.name) ?? 0,
    }));
  }, [schema, nodeTypeCounts]);

  // Get edge types from schema
  const edgeTypes = useMemo(() => {
    if (!schema) return [];
    return schema.edge_types.map((t) => ({
      name: t.name,
      count: edgeTypeCounts.get(t.name) ?? 0,
    }));
  }, [schema, edgeTypeCounts]);

  const handleNodeTypeClick = (typeName: string) => {
    const query = `MATCH n: ${typeName} RETURN n`;
    setQueryContent(query);
    executeQuery(query);
  };

  const handleEdgeTypeClick = (typeName: string) => {
    const query = `MATCH ()-[e: ${typeName}]->() RETURN e`;
    setQueryContent(query);
    executeQuery(query);
  };

  if (!schema) {
    return (
      <div className="minimap">
        <div className="minimap-header">Types</div>
        <div className="minimap-empty">Load an ontology to see types</div>
      </div>
    );
  }

  return (
    <div className="minimap">
      <div className="minimap-header">Types</div>
      <div className="minimap-section">
        <div className="minimap-section-title">Nodes</div>
        <ul className="minimap-list">
          {nodeTypes.map((type) => (
            <li
              key={type.name}
              className="minimap-item minimap-node-type"
              onClick={() => handleNodeTypeClick(type.name)}
            >
              <span className="minimap-type-name">{type.name}</span>
              <span className="minimap-type-count">{type.count}</span>
            </li>
          ))}
        </ul>
      </div>
      {edgeTypes.length > 0 && (
        <div className="minimap-section">
          <div className="minimap-section-title">Edges</div>
          <ul className="minimap-list">
            {edgeTypes.map((type) => (
              <li
                key={type.name}
                className="minimap-item minimap-edge-type"
                onClick={() => handleEdgeTypeClick(type.name)}
              >
                <span className="minimap-type-name">{type.name}</span>
                <span className="minimap-type-count">{type.count}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
