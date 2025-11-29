import { useState, useEffect, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Trash2, Bookmark, Plus, Copy, Pencil, Check } from 'lucide-react';
import {
  type FilterPreset,
  type TargetLevelFilter,
  STORAGE_KEY,
  DEFAULT_PRESET,
  loadPresets,
  generateDefaultPresetName,
} from '@/lib/presets';

export type { FilterPreset };

interface FilterPresetsProps {
  onLoadPreset: (preset: FilterPreset) => void;
  activePresetId: string | null;
  currentFilters: {
    selectedLevels: Set<import("@/types/logs").LogLevel>;
    targetFilter: string;
    searchFilter: string;
    targetLevelFilters: TargetLevelFilter[];
  };
  className?: string;
}

export function FilterPresets({
  onLoadPreset,
  activePresetId,
  currentFilters,
  className
}: FilterPresetsProps) {
  const [presets, setPresets] = useState<FilterPreset[]>(() => loadPresets());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState(``);
  const isDefaultActive = activePresetId === DEFAULT_PRESET.id || activePresetId === null;

  const savePresets = (newPresets: FilterPreset[]) => {
    setPresets(newPresets);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(newPresets));
  };

  // Auto-save filter changes to the active preset (if it's a user preset)
  const prevFiltersRef = useRef(currentFilters);
  useEffect(() => {
    // Skip if default preset is active or no preset is active
    if (isDefaultActive) return;

    // Check if filters actually changed
    const prev = prevFiltersRef.current;
    const targetLevelFiltersChanged =
      prev.targetLevelFilters.length !== currentFilters.targetLevelFilters.length ||
      !prev.targetLevelFilters.every((f, i) => {
        const curr = currentFilters.targetLevelFilters[i];
        return curr && f.id === curr.id && f.target === curr.target && f.level === curr.level;
      });

    const filtersChanged =
      prev.targetFilter !== currentFilters.targetFilter ||
      prev.searchFilter !== currentFilters.searchFilter ||
      prev.selectedLevels.size !== currentFilters.selectedLevels.size ||
      ![...prev.selectedLevels].every(l => currentFilters.selectedLevels.has(l)) ||
      targetLevelFiltersChanged;

    if (filtersChanged) {
      // Update the active preset with new filter values
      const updated = presets.map((p) =>
        p.id === activePresetId
          ? {
              ...p,
              selectedLevels: Array.from(currentFilters.selectedLevels),
              targetFilter: currentFilters.targetFilter,
              searchFilter: currentFilters.searchFilter,
              targetLevelFilters: currentFilters.targetLevelFilters,
            }
          : p
      );
      savePresets(updated);
    }
    prevFiltersRef.current = currentFilters;
  }, [currentFilters, activePresetId, isDefaultActive, presets]);

  const handleAddPreset = () => {
    const newPreset: FilterPreset = {
      id: crypto.randomUUID(),
      name: generateDefaultPresetName(presets),
      selectedLevels: [`trace`, `debug`, `info`, `warn`, `error`],
      targetFilter: ``,
      searchFilter: ``,
      targetLevelFilters: [],
    };

    savePresets([...presets, newPreset]);
    // Load the new preset to make it active and editable
    onLoadPreset(newPreset);
  };

  const handleDuplicate = (preset: FilterPreset) => {
    const newPreset: FilterPreset = {
      id: crypto.randomUUID(),
      name: `${preset.name} (copy)`,
      selectedLevels: [...preset.selectedLevels],
      targetFilter: preset.targetFilter,
      searchFilter: preset.searchFilter,
      targetLevelFilters: preset.targetLevelFilters ? [...preset.targetLevelFilters] : [],
    };

    savePresets([...presets, newPreset]);
    onLoadPreset(newPreset);
  };

  const handleStartEdit = (preset: FilterPreset) => {
    setEditingId(preset.id);
    setEditName(preset.name);
  };

  const handleConfirmRename = (id: string) => {
    if (!editName.trim()) {
      setEditingId(null);
      return;
    }

    const updated = presets.map((p) =>
      p.id === id ? { ...p, name: editName.trim() } : p
    );
    savePresets(updated);
    setEditingId(null);
  };

  const handleDelete = (id: string) => {
    const wasActive = id === activePresetId;
    savePresets(presets.filter((p) => p.id !== id));
    // If we deleted the active preset, switch to default
    if (wasActive) {
      onLoadPreset(DEFAULT_PRESET);
    }
  };

  const handleLoad = (preset: FilterPreset) => {
    onLoadPreset(preset);
  };

  return (
    <div className={className}>
      {/* Header */}
      <div className={`flex items-center justify-between mb-4`}>
        <div className={`flex items-center gap-2`}>
          <Bookmark className={`w-4 h-4 text-muted-foreground`} />
          <h3 className={`font-semibold text-foreground`}>Filter Presets</h3>
        </div>
        <Button
          onClick={handleAddPreset}
          variant={`ghost`}
          size={`sm`}
          title={`Add new preset`}
        >
          <Plus className={`w-4 h-4`} />
        </Button>
      </div>

      {/* Preset List */}
      <div className={`space-y-2`}>
        {/* Show All (Default) Preset */}
        <div
          onClick={() => handleLoad(DEFAULT_PRESET)}
          className={`cursor-pointer border rounded-lg p-3 transition-colors ${
            isDefaultActive
              ? `border-primary bg-primary/20`
              : `border-border hover:bg-accent/50`
          }`}
        >
          <div className={`flex items-center justify-between gap-2 h-7`}>
            <span className={`font-medium text-sm truncate px-3`}>
              {DEFAULT_PRESET.name}
            </span>
          </div>
        </div>

        {/* User Presets */}
        {presets.map((preset) => {
          const isActive = preset.id === activePresetId;
          const isEditing = editingId === preset.id;

          return (
            <div
              key={preset.id}
              onClick={() => !isEditing && handleLoad(preset)}
              className={`cursor-pointer border rounded-lg p-3 transition-colors ${
                isActive
                  ? `border-primary bg-primary/20`
                  : `border-border hover:bg-accent/50`
              }`}
            >
              <div className={`flex items-center justify-between gap-2 h-7`}>
                {isEditing ? (
                  <Input
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    onClick={(e) => e.stopPropagation()}
                    onKeyDown={(e) => {
                      if (e.key === `Enter`) handleConfirmRename(preset.id);
                      if (e.key === `Escape`) setEditingId(null);
                    }}
                    className={`flex-1 h-7 text-sm font-medium`}
                    autoFocus
                  />
                ) : (
                  <span className={`flex-1 font-medium text-sm truncate px-3`}>
                    {preset.name}
                  </span>
                )}

                <div className={`flex gap-1 flex-shrink-0`}>
                  {isEditing ? (
                    <Button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleConfirmRename(preset.id);
                      }}
                      variant={`ghost`}
                      size={`sm`}
                      title={`Confirm rename`}
                    >
                      <Check className={`w-3 h-3`} />
                    </Button>
                  ) : (
                    <Button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleStartEdit(preset);
                      }}
                      variant={`ghost`}
                      size={`sm`}
                      title={`Rename preset`}
                    >
                      <Pencil className={`w-3 h-3`} />
                    </Button>
                  )}
                  <Button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDuplicate(preset);
                    }}
                    variant={`ghost`}
                    size={`sm`}
                    title={`Duplicate preset`}
                  >
                    <Copy className={`w-3 h-3`} />
                  </Button>
                  <Button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(preset.id);
                    }}
                    variant={`ghost`}
                    size={`sm`}
                    title={`Delete preset`}
                  >
                    <Trash2 className={`w-3 h-3`} />
                  </Button>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
