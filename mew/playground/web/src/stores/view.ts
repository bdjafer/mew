import { create } from 'zustand';

interface ViewState {
  query: string;
  viewport: { x: number; y: number };
  zoom: number;
  focal: Set<number>;
  selected: number | null;
}

interface ViewActions {
  setQuery: (query: string) => void;
  setViewport: (viewport: { x: number; y: number }) => void;
  setZoom: (zoom: number) => void;
  setFocal: (focal: Set<number>) => void;
  setSelected: (selected: number | null) => void;
  clearFocal: () => void;
}

export const useViewStore = create<ViewState & ViewActions>((set) => ({
  query: '',
  viewport: { x: 0, y: 0 },
  zoom: 1,
  focal: new Set(),
  selected: null,

  setQuery: (query) => set({ query }),
  setViewport: (viewport) => set({ viewport }),
  setZoom: (zoom) => set({ zoom }),
  setFocal: (focal) => set({ focal }),
  setSelected: (selected) => set({ selected }),
  clearFocal: () => set({ focal: new Set() }),
}));
