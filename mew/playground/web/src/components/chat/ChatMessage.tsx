import type { ChatMessage as ChatMessageType } from '../../ai/types';
import { CodeBlock } from './CodeBlock';
import { useEditorStore } from '../../stores';

interface ChatMessageProps {
  message: ChatMessageType;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const setOntologyContent = useEditorStore((state) => state.setOntologyContent);
  const setQueryContent = useEditorStore((state) => state.setQueryContent);

  const handleApply = (code: string, type: 'ontology' | 'query') => {
    if (type === 'ontology') {
      setOntologyContent(code);
    } else {
      setQueryContent(code);
    }
  };

  const isUser = message.role === 'user';

  return (
    <div
      className={`chat-message ${isUser ? 'chat-message--user' : 'chat-message--assistant'}`}
    >
      {isUser ? (
        <span>{message.content}</span>
      ) : (
        <MessageContent content={message.content} onApply={handleApply} />
      )}
    </div>
  );
}

interface MessageContentProps {
  content: string;
  onApply: (code: string, type: 'ontology' | 'query') => void;
}

function MessageContent({ content, onApply }: MessageContentProps) {
  const parts = parseMessageContent(content);

  return (
    <>
      {parts.map((part, index) => {
        if (part.type === 'text') {
          return <span key={index}>{part.content}</span>;
        } else {
          return (
            <CodeBlock
              key={index}
              code={part.content}
              language={part.language || ''}
              onApply={onApply}
            />
          );
        }
      })}
    </>
  );
}

interface ParsedPart {
  type: 'text' | 'code';
  content: string;
  language?: string;
}

function parseMessageContent(content: string): ParsedPart[] {
  const parts: ParsedPart[] = [];
  const codeBlockRegex = /```(\w*)\n?([\s\S]*?)```/g;

  let lastIndex = 0;
  let match;

  while ((match = codeBlockRegex.exec(content)) !== null) {
    // Add text before the code block
    if (match.index > lastIndex) {
      const text = content.slice(lastIndex, match.index);
      if (text.trim()) {
        parts.push({ type: 'text', content: text });
      }
    }

    // Add the code block
    parts.push({
      type: 'code',
      content: match[2].trim(),
      language: match[1] || undefined,
    });

    lastIndex = match.index + match[0].length;
  }

  // Add remaining text
  if (lastIndex < content.length) {
    const text = content.slice(lastIndex);
    if (text.trim()) {
      parts.push({ type: 'text', content: text });
    }
  }

  // If no parts were created, the whole content is text
  if (parts.length === 0) {
    parts.push({ type: 'text', content });
  }

  return parts;
}
