import { useState, useEffect } from 'react';
import { useChatStore, useUIStore } from '../../stores';
import { ProviderSection } from './ProviderSection';
import type { ProviderType, ProviderConfig, AISettings } from '../../ai/types';

const PROVIDER_TYPES: ProviderType[] = ['openai', 'anthropic', 'google'];

const DEFAULT_MODELS: Record<ProviderType, string> = {
  openai: 'gpt-4o',
  anthropic: 'claude-sonnet-4-20250514',
  google: 'gemini-1.5-pro',
};

function getDefaultConfig(provider: ProviderType): ProviderConfig {
  return {
    apiKey: '',
    model: DEFAULT_MODELS[provider],
    temperature: 0.7,
  };
}

export function SettingsModal() {
  const isSettingsOpen = useUIStore((state) => state.isSettingsOpen);
  const closeSettings = useUIStore((state) => state.closeSettings);
  const settings = useChatStore((state) => state.settings);
  const saveSettings = useChatStore((state) => state.saveSettings);

  const [activeProvider, setActiveProvider] = useState<ProviderType>('openai');
  const [providerConfigs, setProviderConfigs] = useState<
    Record<ProviderType, ProviderConfig>
  >({
    openai: getDefaultConfig('openai'),
    anthropic: getDefaultConfig('anthropic'),
    google: getDefaultConfig('google'),
  });
  const [temperature, setTemperature] = useState(0.7);

  // Load settings when modal opens
  useEffect(() => {
    if (isSettingsOpen && settings) {
      setActiveProvider(settings.activeProvider);
      setProviderConfigs({
        openai: settings.providers.openai || getDefaultConfig('openai'),
        anthropic: settings.providers.anthropic || getDefaultConfig('anthropic'),
        google: settings.providers.google || getDefaultConfig('google'),
      });
      // Get temperature from active provider or default
      const activeConfig = settings.providers[settings.activeProvider];
      setTemperature(activeConfig?.temperature ?? 0.7);
    }
  }, [isSettingsOpen, settings]);

  if (!isSettingsOpen) {
    return null;
  }

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.target === e.currentTarget) {
      closeSettings();
    }
  };

  const handleProviderChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setActiveProvider(e.target.value as ProviderType);
  };

  const handleProviderConfigChange = (
    provider: ProviderType,
    config: ProviderConfig
  ) => {
    setProviderConfigs((prev) => ({
      ...prev,
      [provider]: config,
    }));
  };

  const handleTemperatureChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTemperature(parseFloat(e.target.value));
  };

  const handleSave = () => {
    const newSettings: AISettings = {
      activeProvider,
      providers: {},
    };

    PROVIDER_TYPES.forEach((provider) => {
      const config = providerConfigs[provider];
      if (config.apiKey) {
        newSettings.providers[provider] = {
          ...config,
          temperature,
        };
      }
    });

    saveSettings(newSettings);
    closeSettings();
  };

  const handleCancel = () => {
    closeSettings();
  };

  return (
    <div className="modal" onClick={handleBackdropClick}>
      <div className="modal__backdrop" />
      <div className="modal__content">
        <div className="modal__header">
          <h2>AI Settings</h2>
          <button className="modal__close" onClick={closeSettings}>
            &times;
          </button>
        </div>
        <div className="modal__body">
          <div className="settings-section">
            <label className="settings-label">Active Provider</label>
            <select
              className="settings-select"
              value={activeProvider}
              onChange={handleProviderChange}
            >
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="google">Google</option>
            </select>
          </div>

          <div className="settings-divider" />

          {PROVIDER_TYPES.map((provider) => (
            <ProviderSection
              key={provider}
              provider={provider}
              config={providerConfigs[provider]}
              isActive={activeProvider === provider}
              onChange={(config) => handleProviderConfigChange(provider, config)}
            />
          ))}

          <div className="settings-divider" />

          <div className="settings-section">
            <h3 className="settings-subtitle">Advanced</h3>
            <div className="settings-field">
              <label className="settings-label">
                Temperature (creativity): <span>{temperature.toFixed(1)}</span>
              </label>
              <input
                type="range"
                className="settings-range"
                min="0"
                max="1"
                step="0.1"
                value={temperature}
                onChange={handleTemperatureChange}
              />
            </div>
          </div>
        </div>
        <div className="modal__footer">
          <button
            className="settings-btn settings-btn--cancel"
            onClick={handleCancel}
          >
            Cancel
          </button>
          <button className="settings-btn settings-btn--save" onClick={handleSave}>
            Save Settings
          </button>
        </div>
      </div>
    </div>
  );
}
