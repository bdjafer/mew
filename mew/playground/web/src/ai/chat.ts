import type { ChatMessage, AISettings, ChatState, GenerationContext } from './types';
import { streamChat } from './client';

const STORAGE_KEY = 'mew-ai-settings';
const HISTORY_KEY = 'mew-chat-history';
const MAX_AUTO_ITERATIONS = 3; // Prevent infinite loops

interface ToolResult {
  tool: string;
  success: boolean;
  error?: string;
  message?: string;
  summary?: string;
  rowCount?: number;
}

type ChatStateListener = (state: ChatState) => void;

class ChatManager {
  private state: ChatState = {
    messages: [],
    isLoading: false,
    error: null,
    streamingContent: '',
    activeToolCall: null,
  };

  private listeners: Set<ChatStateListener> = new Set();
  private abortController: AbortController | null = null;
  private pendingToolResults: ToolResult[] = [];
  private autoIterationCount = 0;
  private currentContext: GenerationContext | undefined;

  constructor() {
    // Listen for tool results from the main app
    window.addEventListener('ai-tool-result', ((e: CustomEvent<ToolResult>) => {
      this.pendingToolResults.push(e.detail);
    }) as EventListener);
  }

  subscribe(listener: ChatStateListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => this.listeners.delete(listener);
  }

  getState(): ChatState {
    return this.state;
  }

  private emit() {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  private update(partial: Partial<ChatState>) {
    this.state = { ...this.state, ...partial };
    this.emit();
  }

  getSettings(): AISettings | null {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return null;
    try {
      return JSON.parse(stored);
    } catch {
      return null;
    }
  }

  saveSettings(settings: AISettings): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }

  loadHistory(): void {
    const stored = localStorage.getItem(HISTORY_KEY);
    if (stored) {
      try {
        this.update({ messages: JSON.parse(stored) });
      } catch {
        // Ignore invalid history
      }
    }
  }

  private saveHistory(): void {
    // Keep last 50 messages to avoid localStorage bloat
    const toSave = this.state.messages.slice(-50);
    localStorage.setItem(HISTORY_KEY, JSON.stringify(toSave));
  }

  clearHistory(): void {
    this.update({ messages: [], error: null });
    localStorage.removeItem(HISTORY_KEY);
  }

  async sendMessage(content: string, context?: GenerationContext, isAutoIteration = false): Promise<void> {
    const settings = this.getSettings();
    if (!settings?.activeProvider || !settings.providers[settings.activeProvider]?.apiKey) {
      this.update({ error: 'Please configure an AI provider in settings (click the gear icon)' });
      return;
    }

    // Reset iteration count for new user messages
    if (!isAutoIteration) {
      this.autoIterationCount = 0;
      this.currentContext = context;
    }

    // Clear pending tool results at the start
    this.pendingToolResults = [];

    // For auto-iterations, don't show the message in chat - it's internal
    let updatedMessages: ChatMessage[];
    if (isAutoIteration) {
      // Add as a hidden system message (not displayed but sent to AI)
      updatedMessages = [...this.state.messages, {
        id: crypto.randomUUID(),
        role: 'user' as const,
        content,
        timestamp: Date.now(),
      }];
      // Don't update visible messages, just set loading state
      this.update({
        isLoading: true,
        error: null,
        streamingContent: '',
      });
    } else {
      const userMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'user',
        content,
        timestamp: Date.now(),
      };
      updatedMessages = [...this.state.messages, userMessage];
      this.update({
        messages: updatedMessages,
        isLoading: true,
        error: null,
        streamingContent: '',
      });
    }

    const providerConfig = settings.providers[settings.activeProvider]!;
    this.abortController = new AbortController();

    try {
      let fullResponse = '';
      let toolIndicatorTimeout: ReturnType<typeof setTimeout> | null = null;

      // Show "using tools" if no content for 500ms while still loading
      const scheduleToolIndicator = () => {
        if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
        toolIndicatorTimeout = setTimeout(() => {
          if (this.state.isLoading && !this.state.activeToolCall && !fullResponse) {
            this.update({ activeToolCall: 'processing' });
          }
        }, 500);
      };

      scheduleToolIndicator();

      // Use the new API client with tool support
      for await (const event of streamChat(updatedMessages, settings, context)) {
        if (event.type === 'delta' && event.content) {
          if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
          fullResponse += event.content;
          this.update({ streamingContent: fullResponse, activeToolCall: null });
        } else if (event.type === 'tool_call' && event.toolName) {
          if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
          this.update({ activeToolCall: event.toolName });
        } else if (event.type === 'error') {
          throw new Error(event.error);
        }
      }

      if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);

      const assistantMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: fullResponse,
        timestamp: Date.now(),
        metadata: {
          provider: settings.activeProvider,
          model: providerConfig.model,
        },
      };

      this.update({
        messages: [...updatedMessages, assistantMessage],
        isLoading: false,
        streamingContent: '',
        activeToolCall: null,
      });
      this.saveHistory();

      // Small delay to let tool results come in
      await new Promise(resolve => setTimeout(resolve, 100));

      // Check for tool errors and auto-iterate if needed
      this.handleToolResults();
    } catch (error) {
      this.update({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        streamingContent: '',
        activeToolCall: null,
      });
    }
  }

  private handleToolResults(): void {
    if (this.pendingToolResults.length === 0) return;

    // Collect errors
    const errors = this.pendingToolResults.filter(r => !r.success);

    if (errors.length > 0 && this.autoIterationCount < MAX_AUTO_ITERATIONS) {
      this.autoIterationCount++;

      // Format error message for AI
      const errorMessages = errors.map(e => {
        if (e.tool === 'edit_ontology') {
          return `Ontology validation failed:\n${e.error}`;
        } else if (e.tool === 'edit_query' || e.tool === 'execute_query') {
          return `Query execution failed:\n${e.error}`;
        }
        return `${e.tool} failed: ${e.error}`;
      }).join('\n\n');

      const feedbackMessage = `[System] The previous tool execution had errors. Please fix and try again:\n\n${errorMessages}`;

      // Send error feedback to AI automatically
      console.log(`Auto-iteration ${this.autoIterationCount}/${MAX_AUTO_ITERATIONS}:`, feedbackMessage);
      this.sendMessage(feedbackMessage, this.currentContext, true);
    }
  }

  abort(): void {
    this.abortController?.abort();
    this.update({ isLoading: false, streamingContent: '' });
  }

  /**
   * Send a silent request that doesn't appear in chat history.
   * Used for automated actions like "Generate Seed" button.
   */
  async sendSilentRequest(content: string, context?: GenerationContext): Promise<void> {
    const settings = this.getSettings();
    if (!settings?.activeProvider || !settings.providers[settings.activeProvider]?.apiKey) {
      console.error('No AI provider configured');
      return;
    }

    this.autoIterationCount = 0;
    this.currentContext = context;
    this.pendingToolResults = [];

    // Create a temporary message list with just this request
    const messages: ChatMessage[] = [{
      id: crypto.randomUUID(),
      role: 'user' as const,
      content,
      timestamp: Date.now(),
    }];

    this.update({
      isLoading: true,
      error: null,
      streamingContent: '',
      activeToolCall: 'processing',
    });

    this.abortController = new AbortController();

    try {
      let fullResponse = '';

      for await (const event of streamChat(messages, settings, context)) {
        if (event.type === 'delta' && event.content) {
          fullResponse += event.content;
          this.update({ streamingContent: fullResponse, activeToolCall: null });
        } else if (event.type === 'tool_call' && event.toolName) {
          this.update({ activeToolCall: event.toolName });
        } else if (event.type === 'error') {
          throw new Error(event.error);
        }
      }

      // Silent request completed - no need to save to history
      this.update({
        isLoading: false,
        streamingContent: '',
        activeToolCall: null,
      });

      // Small delay to let tool results come in
      await new Promise(resolve => setTimeout(resolve, 100));

      // Handle tool errors and auto-iterate if needed
      this.handleSilentToolResults(content, context);
    } catch (error) {
      this.update({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        streamingContent: '',
        activeToolCall: null,
      });
    }
  }

  private handleSilentToolResults(originalContent: string, context?: GenerationContext): void {
    if (this.pendingToolResults.length === 0) return;

    const errors = this.pendingToolResults.filter(r => !r.success);

    if (errors.length > 0 && this.autoIterationCount < MAX_AUTO_ITERATIONS) {
      this.autoIterationCount++;

      const errorMessages = errors.map(e => {
        if (e.tool === 'generate_seed') {
          return `Seed generation failed:\n${e.error}`;
        }
        return `${e.tool} failed: ${e.error}`;
      }).join('\n\n');

      const feedbackMessage = `[System] The previous action failed. Please fix and try again:\n\n${errorMessages}\n\nOriginal request: ${originalContent}`;

      console.log(`Silent auto-iteration ${this.autoIterationCount}/${MAX_AUTO_ITERATIONS}:`, feedbackMessage);
      this.sendSilentRequest(feedbackMessage, context);
    }
  }
}

export const chatManager = new ChatManager();
export type { ChatState };
