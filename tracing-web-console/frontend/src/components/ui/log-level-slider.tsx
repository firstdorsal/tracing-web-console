import { cn } from '@/lib/utils';

const LOG_LEVELS = [`trace`, `debug`, `info`, `warn`, `error`] as const;
type LogLevel = (typeof LOG_LEVELS)[number];

const levelColors: Record<LogLevel, string> = {
  trace: `bg-gray-400`,
  debug: `bg-blue-500`,
  info: `bg-green-500`,
  warn: `bg-yellow-500`,
  error: `bg-red-500`,
};

interface LogLevelSliderProps {
  value: number;
  onChange: (value: number) => void;
  className?: string;
  disabled?: boolean;
}

export function LogLevelSlider({ value, onChange, className, disabled }: LogLevelSliderProps) {
  const handleLevelClick = (index: number) => {
    if (!disabled) {
      onChange(index);
    }
  };

  return (
    <div className={cn(`flex gap-1`, className, disabled && `opacity-50`)}>
      {LOG_LEVELS.map((level, index) => {
        const isSelected = index >= value;
        return (
          <button
            key={level}
            onClick={() => handleLevelClick(index)}
            disabled={disabled}
            className={cn(
              `flex-1 py-1.5 text-xs font-medium uppercase rounded transition-colors`,
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
  );
}

export { LOG_LEVELS, type LogLevel };
