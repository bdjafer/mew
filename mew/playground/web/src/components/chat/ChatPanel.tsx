import { useState, useRef, useCallback, useEffect } from 'react';
import { useChatStore, useUIStore } from '../../stores';
import { ChatMessages } from './ChatMessages';
import { ChatInput } from './ChatInput';

interface Position {
  x: number;
  y: number;
}

export function ChatPanel() {
  const isChatOpen = useUIStore((state) => state.isChatOpen);
  const toggleChat = useUIStore((state) => state.toggleChat);
  const openSettings = useUIStore((state) => state.openSettings);
  const clearHistory = useChatStore((state) => state.clearHistory);

  // Draggable state - default position: middle-right area
  const [position, setPosition] = useState<Position>(() => {
    // Default: center of right half, above results panel
    const defaultX = typeof window !== 'undefined' ? window.innerWidth * 0.55 : 700;
    const defaultY = typeof window !== 'undefined' ? window.innerHeight * 0.25 : 150;
    return { x: defaultX, y: defaultY };
  });
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef<{ mouseX: number; mouseY: number; posX: number; posY: number } | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  // Handle drag start
  const handleDragStart = useCallback((e: React.MouseEvent) => {
    // Only drag from header, not from buttons
    if ((e.target as HTMLElement).closest('.chat-panel__btn')) return;

    e.preventDefault();
    setIsDragging(true);
    dragStartRef.current = {
      mouseX: e.clientX,
      mouseY: e.clientY,
      posX: position.x,
      posY: position.y,
    };
  }, [position]);

  // Handle drag move
  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!dragStartRef.current) return;

      const dx = e.clientX - dragStartRef.current.mouseX;
      const dy = e.clientY - dragStartRef.current.mouseY;

      // Calculate new position with bounds checking
      const panelWidth = 320;
      const panelHeight = isChatOpen ? 400 : 44;
      const maxX = window.innerWidth - panelWidth - 10;
      const maxY = window.innerHeight - panelHeight - 10;

      const newX = Math.max(10, Math.min(maxX, dragStartRef.current.posX + dx));
      const newY = Math.max(10, Math.min(maxY, dragStartRef.current.posY + dy));

      setPosition({ x: newX, y: newY });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      dragStartRef.current = null;
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, isChatOpen]);

  // Adjust position when window resizes
  useEffect(() => {
    const handleResize = () => {
      setPosition((prev) => {
        const panelWidth = 320;
        const panelHeight = 400;
        const maxX = window.innerWidth - panelWidth - 10;
        const maxY = window.innerHeight - panelHeight - 10;
        return {
          x: Math.min(prev.x, maxX),
          y: Math.min(prev.y, maxY),
        };
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return (
    <div
      ref={panelRef}
      className={`chat-panel ${!isChatOpen ? 'chat-panel--collapsed' : ''} ${isDragging ? 'chat-panel--dragging' : ''}`}
      style={{
        left: position.x,
        top: position.y,
        right: 'auto',
        bottom: 'auto',
      }}
    >
      <div
        className="chat-panel__header"
        onMouseDown={handleDragStart}
        style={{ cursor: isDragging ? 'grabbing' : 'grab' }}
      >
        <div className="chat-panel__drag-handle">
          <DragIcon />
        </div>
        <span className="chat-panel__title">AI Assistant</span>
        <div className="chat-panel__actions" onClick={(e) => e.stopPropagation()}>
          <button
            className="chat-panel__btn"
            onClick={openSettings}
            title="Settings"
          >
            <SettingsIcon />
          </button>
          <button
            className="chat-panel__btn"
            onClick={clearHistory}
            title="Clear chat history"
          >
            <TrashIcon />
          </button>
          <button
            className="chat-panel__btn"
            onClick={toggleChat}
            title={isChatOpen ? 'Collapse' : 'Expand'}
          >
            {isChatOpen ? <ChevronDownIcon /> : <ChevronUpIcon />}
          </button>
        </div>
      </div>
      <div className="chat-panel__body">
        <ChatMessages />
        <ChatInput />
      </div>
    </div>
  );
}

function DragIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="currentColor"
      opacity="0.5"
    >
      <circle cx="9" cy="6" r="2" />
      <circle cx="15" cy="6" r="2" />
      <circle cx="9" cy="12" r="2" />
      <circle cx="15" cy="12" r="2" />
      <circle cx="9" cy="18" r="2" />
      <circle cx="15" cy="18" r="2" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}

function ChevronDownIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

function ChevronUpIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="18 15 12 9 6 15" />
    </svg>
  );
}
