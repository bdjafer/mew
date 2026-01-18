import { AIProvider } from './base';
import type { ProviderInfo, GenerationRequest, StreamEvent } from '../types';

export class GoogleProvider extends AIProvider {
  get info(): ProviderInfo {
    return {
      name: 'Google',
      defaultModel: 'gemini-3.0-flash',
      models: ['gemini-3.0-flash', 'gemini-3.0-pro'],
      keyPlaceholder: 'AIza...',
    };
  }

  validateApiKey(key: string): boolean {
    return key.length > 20;
  }

  getEndpoint(): string {
    const model = this.config.model || 'gemini-3.0-flash';
    return `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?key=${this.config.apiKey}&alt=sse`;
  }

  buildHeaders(): Record<string, string> {
    return {
      'Content-Type': 'application/json',
    };
  }

  buildRequestBody(request: GenerationRequest): unknown {
    // Build contents from messages
    const contents = request.messages.map((m) => ({
      role: m.role === 'assistant' ? 'model' : 'user',
      parts: [{ text: m.content }],
    }));

    return {
      contents,
      systemInstruction: { parts: [{ text: request.systemPrompt }] },
      generationConfig: {
        maxOutputTokens: this.config.maxTokens || 4096,
        temperature: this.config.temperature ?? 0.7,
      },
    };
  }

  parseStreamChunk(line: string): StreamEvent | null {
    if (!line.startsWith('data: ')) return null;
    try {
      const json = JSON.parse(line.slice(6));
      const text = json.candidates?.[0]?.content?.parts?.[0]?.text;
      if (text) return { type: 'delta', content: text };
      if (json.error) {
        return { type: 'error', error: json.error.message || 'Unknown error' };
      }
    } catch {
      // Ignore parse errors
    }
    return null;
  }
}
