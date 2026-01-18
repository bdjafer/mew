import type { ChatMessage, AISettings, StreamEvent, GenerationContext } from './types';
import { buildSystemPrompt } from './prompts';

/**
 * Stream chat messages from the AI API server.
 * The server uses the Vercel AI SDK with tool calling support.
 */
export async function* streamChat(
  messages: ChatMessage[],
  settings: AISettings,
  context?: GenerationContext
): AsyncGenerator<StreamEvent> {
  const providerConfig = settings.providers[settings.activeProvider];

  if (!providerConfig?.apiKey) {
    yield { type: 'error', error: 'No API key configured' };
    return;
  }

  const response = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      messages: messages.map((m) => ({
        role: m.role,
        content: m.content,
      })),
      provider: settings.activeProvider,
      model: providerConfig.model,
      apiKey: providerConfig.apiKey,
      systemPrompt: buildSystemPrompt(context),
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    let errorMessage = `API error: ${response.status}`;
    try {
      const errorJson = JSON.parse(errorText);
      errorMessage = errorJson.error || errorMessage;
    } catch {
      // Use status code message
    }
    yield { type: 'error', error: errorMessage };
    return;
  }

  yield { type: 'start' };

  const reader = response.body?.getReader();
  if (!reader) {
    yield { type: 'error', error: 'No response body' };
    return;
  }

  const decoder = new TextDecoder();
  let buffer = '';

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (line.trim()) {
          const event = parseDataStreamLine(line);
          if (event) yield event;
        }
      }
    }

    // Process remaining buffer
    if (buffer.trim()) {
      const event = parseDataStreamLine(buffer);
      if (event) yield event;
    }
  } finally {
    reader.releaseLock();
  }

  yield { type: 'done' };
}

/**
 * Parse a line from the AI SDK data stream format.
 * Format: <type>:<json-data>
 *
 * Types:
 * - 0: text delta
 * - 1: tool call streaming start
 * - 2: tool call delta
 * - 3: tool call result
 * - 9: tool result
 * - a: tool call
 * - b: tool result
 * - d: finish
 * - e: error
 */
function parseDataStreamLine(line: string): StreamEvent | null {
  // Skip empty lines
  if (!line.trim()) return null;

  // Data stream format: <type>:<json>
  const colonIndex = line.indexOf(':');
  if (colonIndex === -1) return null;

  const type = line.slice(0, colonIndex);
  const data = line.slice(colonIndex + 1);

  try {
    switch (type) {
      case '0': {
        // Text delta
        const text = JSON.parse(data);
        if (typeof text === 'string') {
          return { type: 'delta', content: text };
        }
        break;
      }

      case '3': {
        // Error
        const error = JSON.parse(data);
        console.error('Stream error:', error);
        return { type: 'error', error: error.message || error.error || JSON.stringify(error) || 'Unknown error' };
      }

      case 'd': {
        // Finish message - final response complete
        return { type: 'done' };
      }

      case 'e': {
        // Finish step - a step (including tool use) completed
        const stepInfo = JSON.parse(data);
        console.log('Step finished:', stepInfo);

        // If this step finished because of tool calls, emit tool_call event
        if (stepInfo.finishReason === 'tool-calls') {
          // Generic indicator that tools are being called
          return { type: 'tool_call' as const, toolName: 'processing' };
        }
        break;
      }

      case '9':
      case 'c': {
        // Tool result - the args contain the tool input (content, explanation)
        const result = JSON.parse(data);
        console.log('Tool result:', result.toolName);

        // Handle editor tools - the content is in args, not result
        if (result.toolName === 'edit_ontology' && result.args?.content) {
          dispatchToolAction({
            action: 'edit_ontology',
            content: result.args.content,
            explanation: result.args.explanation,
          });
        } else if (result.toolName === 'edit_query' && result.args?.content) {
          dispatchToolAction({
            action: 'edit_query',
            content: result.args.content,
            explanation: result.args.explanation,
          });
        } else if (result.toolName === 'execute_query') {
          dispatchToolAction({
            action: 'execute_query',
            waitForResults: result.args?.waitForResults,
          });
        } else if (result.toolName === 'generate_seed' && result.args?.content) {
          dispatchToolAction({
            action: 'generate_seed',
            content: result.args.content,
            explanation: result.args.explanation,
          });
        }

        // Also emit tool_call event for UI (tool result means tool was called)
        if (result.toolName) {
          return { type: 'tool_call' as const, toolName: result.toolName };
        }
        break;
      }

      case 'a': {
        // Tool call - structure varies, could be {toolName, args} or {toolCallId, result: {toolName, args}}
        const toolCall = JSON.parse(data);
        console.log('Parsed tool call (type a):', toolCall);

        // Try different possible locations for toolName
        const toolName = toolCall.toolName || toolCall.result?.toolName;

        if (toolName) {
          // Emit tool call event for UI feedback
          return { type: 'tool_call' as const, toolName };
        }
        break;
      }

      case 'b': {
        // Tool call delta - not used since args come complete
        break;
      }
    }
  } catch (e) {
    // Ignore parse errors
    console.debug('Failed to parse stream line:', line, e);
  }

  return null;
}

/**
 * Dispatch tool action events to the window.
 * The main app listens for these and executes the appropriate actions.
 */
function dispatchToolAction(result: { action: string; content?: string; explanation?: string; waitForResults?: boolean }) {
  console.log('Dispatching tool action:', result.action);

  switch (result.action) {
    case 'edit_ontology':
      window.dispatchEvent(
        new CustomEvent('ai-tool-action', {
          detail: { type: 'edit_ontology', content: result.content, explanation: result.explanation },
        })
      );
      break;

    case 'edit_query':
      window.dispatchEvent(
        new CustomEvent('ai-tool-action', {
          detail: { type: 'edit_query', content: result.content, explanation: result.explanation },
        })
      );
      break;

    case 'execute_query':
      window.dispatchEvent(
        new CustomEvent('ai-tool-action', {
          detail: { type: 'execute_query', waitForResults: result.waitForResults },
        })
      );
      break;

    case 'generate_seed':
      window.dispatchEvent(
        new CustomEvent('ai-tool-action', {
          detail: { type: 'generate_seed', content: result.content, explanation: result.explanation },
        })
      );
      break;
  }
}
