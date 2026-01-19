import React, { useState } from 'react';
import { GraphCanvas } from './GraphCanvas';
import { AdaptiveCanvas } from './AdaptiveCanvas';
import { Minimap } from './Minimap';
import { ResultsPanel } from './ResultsPanel';
import './visualization.css';

export const Visualization: React.FC = () => {
  // Toggle between old and new renderer for testing
  const [useAdaptive, setUseAdaptive] = useState(true);

  return (
    <div className="visualization-container">
      <div className="visualization-main">
        <div className="visualization-sidebar">
          <Minimap />
          <div className="renderer-toggle" style={{ padding: '8px 12px', borderTop: '1px solid #333' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '11px', color: '#888', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={useAdaptive}
                onChange={(e) => setUseAdaptive(e.target.checked)}
              />
              Use adaptive renderer
            </label>
          </div>
        </div>
        <div className="visualization-canvas-area">
          {useAdaptive ? <AdaptiveCanvas /> : <GraphCanvas />}
        </div>
      </div>
      <div className="visualization-results">
        <ResultsPanel />
      </div>
    </div>
  );
};
