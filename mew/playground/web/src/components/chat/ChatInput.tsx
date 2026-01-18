import { useState, useRef, useEffect, KeyboardEvent } from 'react';
import { useChatStore, useEditorStore } from '../../stores';

export function ChatInput() {
  const [input, setInput] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const isLoading = useChatStore((state) => state.isLoading);
  const sendMessage = useChatStore((state) => state.sendMessage);
  const ontologyContent = useEditorStore((state) => state.ontologyContent);
  const queryContent = useEditorStore((state) => state.queryContent);

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = `${Math.min(textarea.scrollHeight, 120)}px`;
    }
  }, [input]);

  const handleSend = () => {
    if (!input.trim() || isLoading) return;

    const context = {
      currentOntology: ontologyContent,
      currentQuery: queryContent,
    };

    sendMessage(input.trim(), context);
    setInput('');
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="chat-panel__input-area">
      <textarea
        ref={textareaRef}
        className="chat-panel__input"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Ask about MEW ontologies, queries..."
        disabled={isLoading}
        rows={1}
      />
      <button
        className={`chat-panel__send ${isLoading ? 'chat-panel__send--disabled' : ''}`}
        onClick={handleSend}
        disabled={isLoading || !input.trim()}
        title="Send message"
      >
        <SendIcon />
      </button>
    </div>
  );
}

function SendIcon() {
  return (
    <svg
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <line x1="22" y1="2" x2="11" y2="13" />
      <polygon points="22 2 15 22 11 13 2 9 22 2" />
    </svg>
  );
}
