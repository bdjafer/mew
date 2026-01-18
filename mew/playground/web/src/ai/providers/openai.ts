import { AIProvider } from './base';
import type { ProviderInfo, GenerationRequest, StreamEvent } from '../types';

export class OpenAIProvider extends AIProvider {
  get info(): ProviderInfo {
    return {
      name: 'OpenAI',
      defaultModel: 'gpt-4.1',
      models: ['o3', 'o4-mini', 'gpt-4.1', 'gpt-4.1-mini', 'gpt-4.1-nano', 'codex-1'],
      keyPlaceholder: 'sk-...',
    };
  }

  validateApiKey(key: string): boolean {
    return key.startsWith('sk-') && key.length > 20;
  }

  getEndpoint(): string {
    return 'https://api.openai.com/v1/chat/completions';
  }

  buildHeaders(): Record<string, string> {
    return {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.config.apiKey}`,
    };
  }

  buildRequestBody(request: GenerationRequest): unknown {
    const messages = [
      { role: 'system', content: request.systemPrompt },
      ...request.messages.map((m) => ({
        role: m.role,
        content: m.content,
      })),
    ];

    return {
      model: this.config.model || 'gpt-4.1',
      messages,
      max_tokens: this.config.maxTokens || 4096,
      temperature: this.config.temperature ?? 0.7,
      stream: true,
    };
  }

  parseStreamChunk(line: string): StreamEvent | null {
    if (!line.startsWith('data: ')) return null;
    const data = line.slice(6);
    if (data === '[DONE]') return { type: 'done' };

    try {
      const json = JSON.parse(data);
      const content = json.choices?.[0]?.delta?.content;
      if (content) return { type: 'delta', content };
    } catch {
      // Ignore parse errors
    }
    return null;
  }
}
