import type { LogLevel } from '@/types/logs';
import { Button } from '@/components/ui/button';
import { LogLevelSlider, LOG_LEVELS } from '@/components/ui/log-level-slider';
import { TargetFilter, type TargetFilterValue } from '@/components/TargetFilter';
import { Filter, Plus } from 'lucide-react';

export type { TargetFilterValue };

interface LogFiltersProps {
  selectedLevels: Set<LogLevel>;
  onLevelsChange: (levels: Set<LogLevel>) => void;
  targetFilters: TargetFilterValue[];
  onTargetFiltersChange: (filters: TargetFilterValue[]) => void;
  availableTargets: string[];
  disabled?: boolean;
}

// Convert slider value (0-4) to a Set of levels at or above that threshold
function sliderValueToLevels(value: number): Set<LogLevel> {
  const levels = new Set<LogLevel>();
  for (let i = value; i < LOG_LEVELS.length; i++) {
    levels.add(LOG_LEVELS[i]);
  }
  return levels;
}

// Convert a Set of levels to slider value (find the lowest selected level)
function levelsToSliderValue(levels: Set<LogLevel>): number {
  for (let i = 0; i < LOG_LEVELS.length; i++) {
    if (levels.has(LOG_LEVELS[i])) {
      return i;
    }
  }
  return 0; // Default to trace if nothing selected
}

export function LogFilters({
  selectedLevels,
  onLevelsChange,
  targetFilters,
  onTargetFiltersChange,
  availableTargets,
  disabled,
}: LogFiltersProps) {
  const sliderValue = levelsToSliderValue(selectedLevels);

  const handleSliderChange = (value: number) => {
    onLevelsChange(sliderValueToLevels(value));
  };

  const handleAddFilter = () => {
    const newFilter: TargetFilterValue = {
      id: crypto.randomUUID(),
      target: ``,
      level: `trace`,
    };
    onTargetFiltersChange([...targetFilters, newFilter]);
  };

  const handleUpdateFilter = (updated: TargetFilterValue) => {
    onTargetFiltersChange(
      targetFilters.map((f) => (f.id === updated.id ? updated : f))
    );
  };

  const handleRemoveFilter = (id: string) => {
    onTargetFiltersChange(targetFilters.filter((f) => f.id !== id));
  };

  return (
    <div className={`space-y-4`}>
      {/* Header */}
      <div className={`flex items-center gap-2`}>
        <Filter className={`w-4 h-4 text-muted-foreground`} />
        <h3 className={`font-semibold text-foreground`}>Filters</h3>
      </div>

      {/* Global Log Level */}
      <div>
        <span className={`text-xs text-muted-foreground mb-2 block`}>Global Level</span>
        <LogLevelSlider value={sliderValue} onChange={handleSliderChange} disabled={disabled} />
      </div>

      {/* Target Filters */}
      <div className={`space-y-2`}>
        <div className={`flex items-center justify-between`}>
          <span className={`text-xs text-muted-foreground`}>Target Filters</span>
          <Button
            onClick={handleAddFilter}
            variant={`ghost`}
            size={`sm`}
            disabled={disabled}
            title={`Add target filter`}
            className={`h-6 w-6 p-0`}
          >
            <Plus className={`w-4 h-4`} />
          </Button>
        </div>
        {targetFilters.map((filter) => (
          <TargetFilter
            key={filter.id}
            filter={filter}
            availableTargets={availableTargets}
            onChange={handleUpdateFilter}
            onRemove={() => handleRemoveFilter(filter.id)}
            disabled={disabled}
          />
        ))}
      </div>
    </div>
  );
}
