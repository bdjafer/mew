import { useEffect, useRef } from 'react';
import { useChatStore } from '../../stores';
import { ChatMessage } from './ChatMessage';

export function ChatMessages() {
  const messages = useChatStore((state) => state.messages);
  const isLoading = useChatStore((state) => state.isLoading);
  const error = useChatStore((state) => state.error);
  const streamingContent = useChatStore((state) => state.streamingContent);
  const activeToolCall = useChatStore((state) => state.activeToolCall);

  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent, activeToolCall, isLoading]);

  return (
    <div className="chat-panel__messages">
      {messages.length === 0 && !isLoading && (
        <div className="chat-panel__welcome">
          <p>Welcome! I can help you with:</p>
          <ul>
            <li>Writing MEW ontologies</li>
            <li>Creating queries</li>
            <li>Seeding data</li>
          </ul>
        </div>
      )}

      {messages.map((message) => (
        <ChatMessage key={message.id} message={message} />
      ))}

      {/* Streaming content */}
      {isLoading && streamingContent && (
        <div className="chat-message chat-message--assistant">
          <ChatMessageContent content={streamingContent} />
        </div>
      )}

      {/* Tool call indicator */}
      {isLoading && activeToolCall && (
        <div className="chat-message chat-message--tool-call">
          <div className="chat-message__tool-status">
            <span className="tool-icon">
              <LoadingIcon />
            </span>
            <span>
              {activeToolCall === 'processing'
                ? 'Processing...'
                : `Calling ${formatToolName(activeToolCall)}...`}
            </span>
          </div>
        </div>
      )}

      {/* Typing indicator */}
      {isLoading && !streamingContent && !activeToolCall && (
        <div className="chat-message chat-message--loading">
          <div className="chat-message__typing">
            <span />
            <span />
            <span />
          </div>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="chat-message chat-message--error">{error}</div>
      )}

      <div ref={messagesEndRef} />
    </div>
  );
}

function ChatMessageContent({ content }: { content: string }) {
  // Simple text rendering without code block parsing for streaming
  return <span>{content}</span>;
}

function LoadingIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="12" r="10" />
      <path d="M12 6v6l4 2" />
    </svg>
  );
}

function formatToolName(toolName: string): string {
  return toolName
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}
