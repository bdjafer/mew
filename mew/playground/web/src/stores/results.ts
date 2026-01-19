import { create } from 'zustand';
import type { ExecuteResult } from '../types';

interface ResultsState {
  /** The last query result */
  lastResult: ExecuteResult | null;
  /** History of query results (capped at 10) */
  resultHistory: ExecuteResult[];
  /** The index of the currently viewed result in history */
  currentIndex: number;
}

interface ResultsActions {
  /** Set the current result (adds to history) */
  setResult: (result: ExecuteResult) => void;
  /** Clear all results */
  clearResults: () => void;
  /** Navigate to a specific result in history */
  goToResult: (index: number) => void;
  /** Navigate to previous result */
  prevResult: () => void;
  /** Navigate to next result */
  nextResult: () => void;
}

const MAX_HISTORY = 10;

export const useResultsStore = create<ResultsState & ResultsActions>((set, get) => ({
  lastResult: null,
  resultHistory: [],
  currentIndex: -1,

  setResult: (result) => {
    const { resultHistory } = get();
    const newHistory = [...resultHistory, result].slice(-MAX_HISTORY);
    set({
      lastResult: result,
      resultHistory: newHistory,
      currentIndex: newHistory.length - 1,
    });
  },

  clearResults: () => set({
    lastResult: null,
    resultHistory: [],
    currentIndex: -1,
  }),

  goToResult: (index) => {
    const { resultHistory } = get();
    if (index >= 0 && index < resultHistory.length) {
      set({
        lastResult: resultHistory[index],
        currentIndex: index,
      });
    }
  },

  prevResult: () => {
    const { resultHistory, currentIndex } = get();
    if (currentIndex > 0) {
      const newIndex = currentIndex - 1;
      set({
        lastResult: resultHistory[newIndex],
        currentIndex: newIndex,
      });
    }
  },

  nextResult: () => {
    const { resultHistory, currentIndex } = get();
    if (currentIndex < resultHistory.length - 1) {
      const newIndex = currentIndex + 1;
      set({
        lastResult: resultHistory[newIndex],
        currentIndex: newIndex,
      });
    }
  },
}));
