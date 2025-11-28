import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Save, Trash2, Edit2, Check, X, Bookmark, Plus } from 'lucide-react';
import {
  type FilterPreset,
  STORAGE_KEY,
  DEFAULT_PRESET,
  loadPresets,
} from '@/lib/presets';

export type { FilterPreset };

interface FilterPresetsProps {
  currentFilters: {
    selectedLevels: Set<import('@/types/logs').LogLevel>;
    targetFilter: string;
    searchFilter: string;
  };
  onLoadPreset: (preset: FilterPreset) => void;
  activePresetId: string | null;
  className?: string;
}

export function FilterPresets({
  currentFilters,
  onLoadPreset,
  activePresetId,
  className
}: FilterPresetsProps) {
  const [presets, setPresets] = useState<FilterPreset[]>(() => loadPresets());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [newPresetName, setNewPresetName] = useState('');
  const [showNewPreset, setShowNewPreset] = useState(false);

  // Combine default preset with user presets
  const allPresets = [DEFAULT_PRESET, ...presets];

  const savePresets = (newPresets: FilterPreset[]) => {
    setPresets(newPresets);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(newPresets));
  };

  const handleSaveNew = () => {
    if (!newPresetName.trim()) return;

    const newPreset: FilterPreset = {
      id: Date.now().toString(),
      name: newPresetName.trim(),
      selectedLevels: Array.from(currentFilters.selectedLevels),
      targetFilter: currentFilters.targetFilter,
      searchFilter: currentFilters.searchFilter,
    };

    savePresets([...presets, newPreset]);
    setNewPresetName('');
    setShowNewPreset(false);
  };

  const handleRename = (id: string) => {
    if (!editName.trim() || id === DEFAULT_PRESET.id) return;

    const updated = presets.map((p) =>
      p.id === id ? { ...p, name: editName.trim() } : p
    );
    savePresets(updated);
    setEditingId(null);
    setEditName('');
  };

  const handleDelete = (id: string) => {
    if (id === DEFAULT_PRESET.id) return; // Cannot delete default preset
    savePresets(presets.filter((p) => p.id !== id));
  };

  const handleLoad = (preset: FilterPreset) => {
    onLoadPreset(preset);
  };

  const handleUpdatePreset = (id: string) => {
    if (id === DEFAULT_PRESET.id) return; // Cannot update default preset

    const updated = presets.map((p) =>
      p.id === id
        ? {
            ...p,
            selectedLevels: Array.from(currentFilters.selectedLevels),
            targetFilter: currentFilters.targetFilter,
            searchFilter: currentFilters.searchFilter,
          }
        : p
    );
    savePresets(updated);
  };

  return (
    <div className={className}>
      {/* Header */}
      <div className="flex items-center gap-2 mb-4">
        <Bookmark className="w-4 h-4 text-muted-foreground" />
        <h3 className="font-semibold text-foreground">Filter Presets</h3>
      </div>

      {/* Preset List */}
      <div className="space-y-2">
        {/* New Filter item */}
        {showNewPreset ? (
          <div className="border border-dashed border-primary rounded-lg p-3 space-y-2 bg-primary/5">
            <Input
              placeholder="Preset name..."
              value={newPresetName}
              onChange={(e) => setNewPresetName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSaveNew();
                if (e.key === 'Escape') {
                  setShowNewPreset(false);
                  setNewPresetName('');
                }
              }}
              autoFocus
            />
            <div className="flex gap-2">
              <Button onClick={handleSaveNew} size="sm" className="flex-1">
                Save
              </Button>
              <Button
                onClick={() => {
                  setShowNewPreset(false);
                  setNewPresetName('');
                }}
                variant="outline"
                size="sm"
              >
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <button
            onClick={() => setShowNewPreset(true)}
            className="w-full border border-dashed border-border rounded-lg p-3 hover:border-primary hover:bg-accent/50 transition-colors flex items-center gap-2 text-muted-foreground hover:text-foreground"
          >
            <Plus className="w-4 h-4" />
            <span className="text-sm font-medium">New Filter</span>
          </button>
        )}

        {allPresets.map((preset) => {
          const isDefault = preset.id === DEFAULT_PRESET.id;
          const isActive = preset.id === activePresetId;

          if (editingId === preset.id) {
            return (
              <div
                key={preset.id}
                className="border border-border rounded-lg p-3"
              >
                <div className="flex items-center gap-2">
                  <Input
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleRename(preset.id);
                      if (e.key === 'Escape') setEditingId(null);
                    }}
                    className="flex-1"
                    autoFocus
                  />
                  <Button
                    onClick={() => handleRename(preset.id)}
                    variant="ghost"
                    size="sm"
                  >
                    <Check className="w-4 h-4" />
                  </Button>
                  <Button
                    onClick={() => setEditingId(null)}
                    variant="ghost"
                    size="sm"
                  >
                    <X className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            );
          }

          return (
            <div
              key={preset.id}
              onClick={() => handleLoad(preset)}
              className={`cursor-pointer border rounded-lg p-3 transition-colors flex items-center justify-between gap-2 ${
                isActive
                  ? 'border-primary bg-primary/20'
                  : 'border-border hover:bg-accent/50'
              }`}
            >
              <span className="font-medium text-sm truncate">
                {preset.name}
              </span>

              {!isDefault && (
                <div className="flex gap-1 flex-shrink-0">
                  <Button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleUpdatePreset(preset.id);
                    }}
                    variant="ghost"
                    size="sm"
                    title="Update preset with current filters"
                  >
                    <Save className="w-3 h-3" />
                  </Button>
                  <Button
                    onClick={(e) => {
                      e.stopPropagation();
                      setEditingId(preset.id);
                      setEditName(preset.name);
                    }}
                    variant="ghost"
                    size="sm"
                    title="Rename preset"
                  >
                    <Edit2 className="w-3 h-3" />
                  </Button>
                  <Button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(preset.id);
                    }}
                    variant="ghost"
                    size="sm"
                    title="Delete preset"
                  >
                    <Trash2 className="w-3 h-3" />
                  </Button>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
