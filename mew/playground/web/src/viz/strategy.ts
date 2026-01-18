import type { Analysis, ViewState } from '../types';

export type LayoutStrategy = 'detail' | 'hierarchical' | 'layered' | 'bipartite' | 'force' | 'cluster';

export function selectStrategy(analysis: Analysis, viewState: ViewState): LayoutStrategy {
  if (analysis.cardinality === 0) return 'detail';
  if (analysis.cardinality === 1) return 'detail';
  if (viewState.focal.size === 1) return 'detail';
  switch (analysis.structure) {
    case 'single':
      return 'detail';
    case 'tree':
      return 'hierarchical';
    case 'dag':
      return 'layered';
    case 'bipartite':
      return 'bipartite';
    case 'cyclic':
      return 'force';
    case 'cluster':
      return 'cluster';
    default:
      return 'force';
  }
}

export function getStrategyDescription(strategy: LayoutStrategy): string {
  switch (strategy) {
    case 'detail': return 'Single entity detail view';
    case 'hierarchical': return 'Tree structure from root';
    case 'layered': return 'DAG with horizontal layers';
    case 'bipartite': return 'Two-column type layout';
    case 'force': return 'Force-directed graph';
    case 'cluster': return 'Clustered by type';
  }
}
