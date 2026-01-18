import type { ProviderConfig, StreamEvent, GenerationRequest, ProviderInfo } from '../types';

export abstract class AIProvider {
  protected config: ProviderConfig;

  constructor(config: ProviderConfig) {
    this.config = config;
  }

  abstract get info(): ProviderInfo;

  // Validate API key format (basic check)
  abstract validateApiKey(key: string): boolean;

  // Build the request body for the provider's API
  abstract buildRequestBody(request: GenerationRequest): unknown;

  // Build headers including auth
  abstract buildHeaders(): Record<string, string>;

  // Get the API endpoint URL
  abstract getEndpoint(): string;

  // Parse streaming response chunks
  abstract parseStreamChunk(chunk: string): StreamEvent | null;

  // Main streaming generation method
  async *generate(request: GenerationRequest): AsyncGenerator<StreamEvent> {
    const response = await fetch(this.getEndpoint(), {
      method: 'POST',
      headers: this.buildHeaders(),
      body: JSON.stringify(this.buildRequestBody(request)),
    });

    if (!response.ok) {
      const errorText = await response.text();
      let errorMessage = `API error: ${response.status}`;
      try {
        const errorJson = JSON.parse(errorText);
        errorMessage = errorJson.error?.message || errorJson.message || errorMessage;
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
            const event = this.parseStreamChunk(line);
            if (event) yield event;
          }
        }
      }

      // Process any remaining buffer
      if (buffer.trim()) {
        const event = this.parseStreamChunk(buffer);
        if (event) yield event;
      }
    } finally {
      reader.releaseLock();
    }

    yield { type: 'done' };
  }
}
