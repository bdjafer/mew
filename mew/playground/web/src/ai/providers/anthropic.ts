import { AIProvider } from './base';
import type { ProviderInfo, GenerationRequest, StreamEvent } from '../types';

export class AnthropicProvider extends AIProvider {
  get info(): ProviderInfo {
    return {
      name: 'Anthropic',
      defaultModel: 'claude-sonnet-4-5-20250929',
      models: [
        'claude-sonnet-4-5-20250929',
        'claude-opus-4-5-20251101',
      ],
      keyPlaceholder: 'sk-ant-...',
    };
  }

  validateApiKey(key: string): boolean {
    return key.startsWith('sk-ant-') && key.length > 20;
  }

  getEndpoint(): string {
    return 'https://api.anthropic.com/v1/messages';
  }

  buildHeaders(): Record<string, string> {
    return {
      'Content-Type': 'application/json',
      'x-api-key': this.config.apiKey,
      'anthropic-version': '2023-06-01',
      'anthropic-dangerous-direct-browser-access': 'true',
    };
  }

  buildRequestBody(request: GenerationRequest): unknown {
    return {
      model: this.config.model || 'claude-sonnet-4-5-20250929',
      max_tokens: this.config.maxTokens || 4096,
      system: request.systemPrompt,
      messages: request.messages.map((m) => ({
        role: m.role === 'assistant' ? 'assistant' : 'user',
        content: m.content,
      })),
      stream: true,
    };
  }

  parseStreamChunk(line: string): StreamEvent | null {
    if (!line.startsWith('data: ')) return null;
    try {
      const json = JSON.parse(line.slice(6));
      if (json.type === 'content_block_delta') {
        return { type: 'delta', content: json.delta?.text || '' };
      }
      if (json.type === 'message_stop') {
        return { type: 'done' };
      }
      if (json.type === 'error') {
        return { type: 'error', error: json.error?.message || 'Unknown error' };
      }
    } catch {
      // Ignore parse errors
    }
    return null;
  }
}
