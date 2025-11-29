const SETTINGS_KEY = `tracing-dashboard-settings`;

export interface Settings {
  autoScroll: boolean;
  reverseOrder: boolean;
}

const DEFAULT_SETTINGS: Settings = {
  autoScroll: false,
  reverseOrder: false,
};

export function loadSettings(): Settings {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_SETTINGS, ...parsed };
    }
  } catch {
    // Ignore parse errors
  }
  return DEFAULT_SETTINGS;
}

export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // Ignore storage errors
  }
}

export function updateSetting<K extends keyof Settings>(key: K, value: Settings[K]): void {
  const settings = loadSettings();
  settings[key] = value;
  saveSettings(settings);
}
