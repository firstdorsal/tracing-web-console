import { useState, useRef, useEffect } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { LogLevel } from '@/types/logs';

const LOG_LEVELS: LogLevel[] = [`trace`, `debug`, `info`, `warn`, `error`];

const levelColors: Record<LogLevel, string> = {
  trace: `bg-gray-400`,
  debug: `bg-blue-500`,
  info: `bg-green-500`,
  warn: `bg-yellow-500`,
  error: `bg-red-500`,
};

export interface TargetFilterValue {
  id: string;
  target: string;
  level: LogLevel;
}

interface TargetFilterProps {
  filter: TargetFilterValue;
  availableTargets: string[];
  onChange: (filter: TargetFilterValue) => void;
  onRemove: () => void;
  disabled?: boolean;
}

export function TargetFilter({
  filter,
  availableTargets,
  onChange,
  onRemove,
  disabled,
}: TargetFilterProps) {
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [inputValue, setInputValue] = useState(filter.target);
  const inputRef = useRef<HTMLInputElement>(null);
  const suggestionsRef = useRef<HTMLDivElement>(null);

  // Filter suggestions based on input
  const suggestions = availableTargets.filter(
    (t) => t.toLowerCase().includes(inputValue.toLowerCase()) && t !== inputValue
  );

  // Handle click outside to close suggestions
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        suggestionsRef.current &&
        !suggestionsRef.current.contains(e.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(e.target as Node)
      ) {
        setShowSuggestions(false);
      }
    };

    document.addEventListener(`mousedown`, handleClickOutside);
    return () => document.removeEventListener(`mousedown`, handleClickOutside);
  }, []);

  const handleInputChange = (value: string) => {
    setInputValue(value);
    onChange({ ...filter, target: value });
    setShowSuggestions(true);
  };

  const handleSelectSuggestion = (target: string) => {
    setInputValue(target);
    onChange({ ...filter, target });
    setShowSuggestions(false);
  };

  const handleLevelClick = (level: LogLevel) => {
    if (!disabled) {
      onChange({ ...filter, level });
    }
  };

  const currentLevelIndex = LOG_LEVELS.indexOf(filter.level);

  return (
    <div className={cn(`space-y-2 p-3 border rounded-lg bg-muted/30`, disabled && `opacity-50`)}>
      <div className={`flex items-center gap-2`}>
        {/* Target input with autocomplete */}
        <div className={`relative flex-1`}>
          <Input
            ref={inputRef}
            value={inputValue}
            onChange={(e) => handleInputChange(e.target.value)}
            onFocus={() => setShowSuggestions(true)}
            placeholder={`Target name...`}
            className={`h-8 text-sm`}
            disabled={disabled}
          />
          {showSuggestions && suggestions.length > 0 && (
            <div
              ref={suggestionsRef}
              className={`absolute z-10 top-full left-0 right-0 mt-1 max-h-40 overflow-y-auto bg-popover border rounded-md shadow-md`}
            >
              {suggestions.map((target) => (
                <div
                  key={target}
                  onClick={() => handleSelectSuggestion(target)}
                  className={`px-3 py-2 text-sm cursor-pointer hover:bg-accent truncate`}
                >
                  {target}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Remove button */}
        <Button
          onClick={onRemove}
          variant={`ghost`}
          size={`sm`}
          className={`h-8 w-8 p-0`}
          disabled={disabled}
        >
          <X className={`w-4 h-4`} />
        </Button>
      </div>

      {/* Level selector */}
      <div className={`flex gap-1`}>
        {LOG_LEVELS.map((level, index) => {
          const isSelected = index >= currentLevelIndex;
          return (
            <button
              key={level}
              onClick={() => handleLevelClick(level)}
              disabled={disabled}
              className={cn(
                `flex-1 py-1 text-xs font-medium uppercase rounded transition-colors`,
                isSelected
                  ? cn(levelColors[level], level === `trace` ? `text-gray-900` : `text-white`)
                  : `bg-muted text-muted-foreground hover:bg-muted/80`,
                disabled ? `cursor-not-allowed` : `cursor-pointer`
              )}
            >
              {level}
            </button>
          );
        })}
      </div>
    </div>
  );
}
