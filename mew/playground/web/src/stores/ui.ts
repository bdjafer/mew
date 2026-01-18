import { create } from 'zustand';

interface UIState {
  isChatOpen: boolean;
  isSettingsOpen: boolean;
  sidebarWidth: number;
  isGeneratingSeed: boolean;
}

interface UIActions {
  toggleChat: () => void;
  openChat: () => void;
  closeChat: () => void;
  toggleSettings: () => void;
  openSettings: () => void;
  closeSettings: () => void;
  setSidebarWidth: (width: number) => void;
  setIsGeneratingSeed: (isGenerating: boolean) => void;
}

export const useUIStore = create<UIState & UIActions>((set) => ({
  isChatOpen: false,
  isSettingsOpen: false,
  sidebarWidth: 450,
  isGeneratingSeed: false,

  toggleChat: () => set((state) => ({ isChatOpen: !state.isChatOpen })),
  openChat: () => set({ isChatOpen: true }),
  closeChat: () => set({ isChatOpen: false }),
  toggleSettings: () => set((state) => ({ isSettingsOpen: !state.isSettingsOpen })),
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  setSidebarWidth: (width) => set({ sidebarWidth: width }),
  setIsGeneratingSeed: (isGenerating) => set({ isGeneratingSeed: isGenerating }),
}));
