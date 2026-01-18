import { useState } from 'react';
import type { ProviderConfig, ProviderType } from '../../ai/types';

interface ProviderSectionProps {
  provider: ProviderType;
  config: ProviderConfig;
  isActive: boolean;
  onChange: (config: ProviderConfig) => void;
}

const PROVIDER_MODELS: Record<ProviderType, string[]> = {
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'],
  anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-haiku-20241022'],
  google: ['gemini-1.5-pro', 'gemini-1.5-flash'],
};

const PROVIDER_NAMES: Record<ProviderType, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  google: 'Google',
};

const KEY_PLACEHOLDERS: Record<ProviderType, string> = {
  openai: 'sk-...',
  anthropic: 'sk-ant-...',
  google: 'AIza...',
};

export function ProviderSection({
  provider,
  config,
  isActive,
  onChange,
}: ProviderSectionProps) {
  const [showPassword, setShowPassword] = useState(false);

  const handleApiKeyChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...config, apiKey: e.target.value });
  };

  const handleModelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onChange({ ...config, model: e.target.value });
  };

  const togglePasswordVisibility = () => {
    setShowPassword((prev) => !prev);
  };

  return (
    <div
      className={`settings-section provider-section ${isActive ? 'provider-section--active' : ''}`}
      data-provider={provider}
    >
      <h3 className="settings-subtitle">{PROVIDER_NAMES[provider]}</h3>
      <div className="settings-field">
        <label className="settings-label">API Key</label>
        <div className="settings-input-group">
          <input
            type={showPassword ? 'text' : 'password'}
            className="settings-input"
            placeholder={KEY_PLACEHOLDERS[provider]}
            value={config.apiKey}
            onChange={handleApiKeyChange}
          />
          <button
            type="button"
            className="settings-btn settings-btn--toggle"
            onClick={togglePasswordVisibility}
          >
            {showPassword ? 'Hide' : 'Show'}
          </button>
        </div>
      </div>
      <div className="settings-field">
        <label className="settings-label">Model</label>
        <select
          className="settings-select"
          value={config.model}
          onChange={handleModelChange}
        >
          {PROVIDER_MODELS[provider].map((model) => (
            <option key={model} value={model}>
              {model}
              {model === PROVIDER_MODELS[provider][0] ? ' (default)' : ''}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
