import { create } from 'zustand';
import { streamChat } from '../ai/client';
import type { ChatMessage, AISettings, GenerationContext } from '../ai/types';

const STORAGE_KEY = 'mew-ai-settings';
const HISTORY_KEY = 'mew-chat-history';
const MAX_AUTO_ITERATIONS = 3;

interface ToolResult {
  tool: string;
  success: boolean;
  error?: string;
  message?: string;
  summary?: string;
  rowCount?: number;
}

interface ChatState {
  messages: ChatMessage[];
  isLoading: boolean;
  error: string | null;
  streamingContent: string;
  activeToolCall: string | null;
  settings: AISettings | null;
}

interface ChatActions {
  loadSettings: () => AISettings | null;
  saveSettings: (settings: AISettings) => void;
  loadHistory: () => void;
  clearHistory: () => void;
  sendMessage: (content: string, context?: GenerationContext, isAutoIteration?: boolean) => Promise<void>;
  sendSilentRequest: (content: string, context?: GenerationContext) => Promise<void>;
  abort: () => void;
  addToolResult: (result: ToolResult) => void;
}

interface ChatInternals {
  abortController: AbortController | null;
  pendingToolResults: ToolResult[];
  autoIterationCount: number;
  currentContext: GenerationContext | undefined;
}

const internals: ChatInternals = {
  abortController: null,
  pendingToolResults: [],
  autoIterationCount: 0,
  currentContext: undefined,
};

export const useChatStore = create<ChatState & ChatActions>((set, get) => ({
  messages: [],
  isLoading: false,
  error: null,
  streamingContent: '',
  activeToolCall: null,
  settings: null,

  loadSettings: () => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return null;
    try {
      const settings = JSON.parse(stored);
      set({ settings });
      return settings;
    } catch {
      return null;
    }
  },

  saveSettings: (settings) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    set({ settings });
  },

  loadHistory: () => {
    const stored = localStorage.getItem(HISTORY_KEY);
    if (stored) {
      try {
        set({ messages: JSON.parse(stored) });
      } catch {
        // Ignore invalid history
      }
    }
  },

  clearHistory: () => {
    set({ messages: [], error: null });
    localStorage.removeItem(HISTORY_KEY);
  },

  sendMessage: async (content, context, isAutoIteration = false) => {
    const { settings, messages } = get();
    if (!settings?.activeProvider || !settings.providers[settings.activeProvider]?.apiKey) {
      set({ error: 'Please configure an AI provider in settings (click the gear icon)' });
      return;
    }

    if (!isAutoIteration) {
      internals.autoIterationCount = 0;
      internals.currentContext = context;
    }

    internals.pendingToolResults = [];

    let updatedMessages: ChatMessage[];
    if (isAutoIteration) {
      updatedMessages = [
        ...messages,
        {
          id: crypto.randomUUID(),
          role: 'user' as const,
          content,
          timestamp: Date.now(),
        },
      ];
      set({ isLoading: true, error: null, streamingContent: '' });
    } else {
      const userMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'user',
        content,
        timestamp: Date.now(),
      };
      updatedMessages = [...messages, userMessage];
      set({
        messages: updatedMessages,
        isLoading: true,
        error: null,
        streamingContent: '',
      });
    }

    internals.abortController = new AbortController();

    try {
      let fullResponse = '';
      let toolIndicatorTimeout: ReturnType<typeof setTimeout> | null = null;

      const scheduleToolIndicator = () => {
        if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
        toolIndicatorTimeout = setTimeout(() => {
          const state = get();
          if (state.isLoading && !state.activeToolCall && !fullResponse) {
            set({ activeToolCall: 'processing' });
          }
        }, 500);
      };

      scheduleToolIndicator();

      for await (const event of streamChat(updatedMessages, settings, context)) {
        if (event.type === 'delta' && event.content) {
          if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
          fullResponse += event.content;
          set({ streamingContent: fullResponse, activeToolCall: null });
        } else if (event.type === 'tool_call' && event.toolName) {
          if (toolIndicatorTimeout) clearTimeout(toolIndicatorTimeout);
          set({ activeToolCall: event.toolName });
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
          model: settings.providers[settings.activeProvider]?.model,
        },
      };

      const newMessages = [...updatedMessages, assistantMessage];
      set({
        messages: newMessages,
        isLoading: false,
        streamingContent: '',
        activeToolCall: null,
      });

      // Save history
      const toSave = newMessages.slice(-50);
      localStorage.setItem(HISTORY_KEY, JSON.stringify(toSave));

      // Small delay to let tool results come in
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Handle tool results
      handleToolResults(get, set);
    } catch (error) {
      set({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        streamingContent: '',
        activeToolCall: null,
      });
    }
  },

  sendSilentRequest: async (content, context) => {
    const { settings } = get();
    if (!settings?.activeProvider || !settings.providers[settings.activeProvider]?.apiKey) {
      console.error('No AI provider configured');
      return;
    }

    internals.autoIterationCount = 0;
    internals.currentContext = context;
    internals.pendingToolResults = [];

    const messages: ChatMessage[] = [
      {
        id: crypto.randomUUID(),
        role: 'user' as const,
        content,
        timestamp: Date.now(),
      },
    ];

    set({
      isLoading: true,
      error: null,
      streamingContent: '',
      activeToolCall: 'processing',
    });

    internals.abortController = new AbortController();

    try {
      let fullResponse = '';

      for await (const event of streamChat(messages, settings, context)) {
        if (event.type === 'delta' && event.content) {
          fullResponse += event.content;
          set({ streamingContent: fullResponse, activeToolCall: null });
        } else if (event.type === 'tool_call' && event.toolName) {
          set({ activeToolCall: event.toolName });
        } else if (event.type === 'error') {
          throw new Error(event.error);
        }
      }

      set({
        isLoading: false,
        streamingContent: '',
        activeToolCall: null,
      });

      await new Promise((resolve) => setTimeout(resolve, 100));

      handleSilentToolResults(content, context, get, set);
    } catch (error) {
      set({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        streamingContent: '',
        activeToolCall: null,
      });
    }
  },

  abort: () => {
    internals.abortController?.abort();
    set({ isLoading: false, streamingContent: '' });
  },

  addToolResult: (result) => {
    internals.pendingToolResults.push(result);
  },
}));

function handleToolResults(
  get: () => ChatState & ChatActions,
  _set: (state: Partial<ChatState>) => void
) {
  if (internals.pendingToolResults.length === 0) return;

  const errors = internals.pendingToolResults.filter((r) => !r.success);

  if (errors.length > 0 && internals.autoIterationCount < MAX_AUTO_ITERATIONS) {
    internals.autoIterationCount++;

    const errorMessages = errors
      .map((e) => {
        if (e.tool === 'edit_ontology') {
          return `Ontology validation failed:\n${e.error}`;
        } else if (e.tool === 'edit_query' || e.tool === 'execute_query') {
          return `Query execution failed:\n${e.error}`;
        }
        return `${e.tool} failed: ${e.error}`;
      })
      .join('\n\n');

    const feedbackMessage = `[System] The previous tool execution had errors. Please fix and try again:\n\n${errorMessages}`;

    console.log(
      `Auto-iteration ${internals.autoIterationCount}/${MAX_AUTO_ITERATIONS}:`,
      feedbackMessage
    );
    get().sendMessage(feedbackMessage, internals.currentContext, true);
  }
}

function handleSilentToolResults(
  originalContent: string,
  context: GenerationContext | undefined,
  get: () => ChatState & ChatActions,
  _set: (state: Partial<ChatState>) => void
) {
  if (internals.pendingToolResults.length === 0) return;

  const errors = internals.pendingToolResults.filter((r) => !r.success);

  if (errors.length > 0 && internals.autoIterationCount < MAX_AUTO_ITERATIONS) {
    internals.autoIterationCount++;

    const errorMessages = errors
      .map((e) => {
        if (e.tool === 'generate_seed') {
          return `Seed generation failed:\n${e.error}`;
        }
        return `${e.tool} failed: ${e.error}`;
      })
      .join('\n\n');

    const feedbackMessage = `[System] The previous action failed. Please fix and try again:\n\n${errorMessages}\n\nOriginal request: ${originalContent}`;

    console.log(
      `Silent auto-iteration ${internals.autoIterationCount}/${MAX_AUTO_ITERATIONS}:`,
      feedbackMessage
    );
    get().sendSilentRequest(feedbackMessage, context);
  }
}
