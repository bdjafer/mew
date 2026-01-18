import React from 'react';
import { GraphCanvas } from './GraphCanvas';
import { Minimap } from './Minimap';
import { ResultsPanel } from './ResultsPanel';
import { StatusBar } from './StatusBar';
import './visualization.css';

export const Visualization: React.FC = () => {
  return (
    <div className="visualization-container">
      <div className="visualization-main">
        <div className="visualization-sidebar">
          <Minimap />
        </div>
        <div className="visualization-canvas-area">
          <GraphCanvas />
        </div>
      </div>
      <div className="visualization-results">
        <ResultsPanel />
      </div>
      <div className="visualization-status">
        <StatusBar />
      </div>
    </div>
  );
};
