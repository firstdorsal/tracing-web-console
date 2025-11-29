import type { LogLevel } from '@/types/logs';

export interface TargetLevelFilter {
  id: string;
  target: string;
  level: LogLevel;
}

export interface FilterPreset {
  id: string;
  name: string;
  selectedLevels: LogLevel[];
  targetFilter: string;
  searchFilter: string;
  targetLevelFilters?: TargetLevelFilter[];
}

export const STORAGE_KEY = `tracing-filter-presets`;
export const LAST_PRESET_KEY = `tracing-last-preset-id`;

// Default preset that cannot be deleted
export const DEFAULT_PRESET: FilterPreset = {
  id: `__default__`,
  name: `Show All`,
  selectedLevels: [`trace`, `debug`, `info`, `warn`, `error`],
  targetFilter: ``,
  searchFilter: ``,
  targetLevelFilters: [],
};

/**
 * Generate a default name for a new preset
 */
export function generateDefaultPresetName(existingPresets: FilterPreset[]): string {
  const existingNames = new Set(existingPresets.map(p => p.name));
  let counter = 1;
  let name = `New Preset ${counter}`;
  while (existingNames.has(name)) {
    counter++;
    name = `New Preset ${counter}`;
  }
  return name;
}

/**
 * Load all presets from localStorage
 */
export function loadPresets(): FilterPreset[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored ? JSON.parse(stored) : [];
}

/**
 * Load the last selected preset ID from localStorage
 */
export function loadLastPresetId(): string | null {
  return localStorage.getItem(LAST_PRESET_KEY);
}

/**
 * Find a preset by ID (checks both default and user presets)
 */
export function findPresetById(id: string, userPresets: FilterPreset[]): FilterPreset | null {
  if (id === DEFAULT_PRESET.id) {
    return DEFAULT_PRESET;
  }
  return userPresets.find(p => p.id === id) || null;
}

/**
 * Get the initial filters based on the last selected preset
 */
export function getInitialFilters(): {
  selectedLevels: Set<LogLevel>;
  targetFilter: string;
  searchFilter: string;
  targetLevelFilters: TargetLevelFilter[];
} {
  const lastPresetId = loadLastPresetId();

  if (lastPresetId) {
    const userPresets = loadPresets();
    const preset = findPresetById(lastPresetId, userPresets);

    if (preset) {
      return {
        selectedLevels: new Set(preset.selectedLevels),
        targetFilter: preset.targetFilter,
        searchFilter: preset.searchFilter,
        targetLevelFilters: preset.targetLevelFilters || [],
      };
    }
  }

  // Default to "Show All" if no preset found
  return {
    selectedLevels: new Set(DEFAULT_PRESET.selectedLevels),
    targetFilter: DEFAULT_PRESET.targetFilter,
    searchFilter: DEFAULT_PRESET.searchFilter,
    targetLevelFilters: DEFAULT_PRESET.targetLevelFilters || [],
  };
}
