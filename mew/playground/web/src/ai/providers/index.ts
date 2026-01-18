import { OpenAIProvider } from './openai';
import { AnthropicProvider } from './anthropic';
import { GoogleProvider } from './google';
import type { AIProvider } from './base';
import type { ProviderType, ProviderConfig, ProviderInfo } from '../types';

export { AIProvider } from './base';
export { OpenAIProvider } from './openai';
export { AnthropicProvider } from './anthropic';
export { GoogleProvider } from './google';

export function createProvider(type: ProviderType, config: ProviderConfig): AIProvider {
  switch (type) {
    case 'openai':
      return new OpenAIProvider(config);
    case 'anthropic':
      return new AnthropicProvider(config);
    case 'google':
      return new GoogleProvider(config);
    default:
      throw new Error(`Unknown provider: ${type}`);
  }
}

export function getProviderInfo(type: ProviderType): ProviderInfo {
  const tempConfig = { apiKey: '', model: '' };
  const provider = createProvider(type, tempConfig);
  return provider.info;
}

export const PROVIDER_TYPES: ProviderType[] = ['openai', 'anthropic', 'google'];
