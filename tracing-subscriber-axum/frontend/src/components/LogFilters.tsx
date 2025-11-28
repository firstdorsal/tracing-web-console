import type { LogLevel } from '@/types/logs';
import { Button } from '@/components/ui/button';
import { LogLevelSlider, LOG_LEVELS } from '@/components/ui/log-level-slider';
import { Filter } from 'lucide-react';

interface LogFiltersProps {
  selectedLevels: Set<LogLevel>;
  onLevelsChange: (levels: Set<LogLevel>) => void;
  onClearFilters: () => void;
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
  onClearFilters,
}: LogFiltersProps) {
  const sliderValue = levelsToSliderValue(selectedLevels);

  const handleSliderChange = (value: number) => {
    onLevelsChange(sliderValueToLevels(value));
  };

  const hasActiveFilters = sliderValue > 0;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-muted-foreground" />
          <h3 className="font-semibold text-foreground">Filters</h3>
        </div>
        {hasActiveFilters && (
          <Button onClick={onClearFilters} variant="ghost" size="sm">
            Clear
          </Button>
        )}
      </div>

      {/* Global Log Level Slider */}
      <LogLevelSlider value={sliderValue} onChange={handleSliderChange} />
    </div>
  );
}
