// Provider identification
export type ProviderType = 'openai' | 'anthropic' | 'google';

// Configuration for each provider
export interface ProviderConfig {
  apiKey: string;
  model: string;
  maxTokens?: number;
  temperature?: number;
}

// Storage structure for all provider configs
export interface AISettings {
  activeProvider: ProviderType;
  providers: {
    openai?: ProviderConfig;
    anthropic?: ProviderConfig;
    google?: ProviderConfig;
  };
}

// Chat message structure
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  metadata?: {
    provider?: ProviderType;
    model?: string;
  };
}

// Streaming events
export interface StreamEvent {
  type: 'start' | 'delta' | 'done' | 'error' | 'tool_call';
  content?: string;
  error?: string;
  toolName?: string;
}

// Provider capability info
export interface ProviderInfo {
  name: string;
  defaultModel: string;
  models: string[];
  keyPlaceholder: string;
}

// Generation request context
export interface GenerationContext {
  currentOntology?: string;
  currentQuery?: string;
}

// Generation request
export interface GenerationRequest {
  messages: ChatMessage[];
  systemPrompt: string;
  context?: GenerationContext;
}

// Chat state
export interface ChatState {
  messages: ChatMessage[];
  isLoading: boolean;
  error: string | null;
  streamingContent: string;
  activeToolCall: string | null; // Name of tool being called, if any
}
