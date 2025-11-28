import * as React from 'react';
import * as SliderPrimitive from '@radix-ui/react-slider';
import { cn } from '@/lib/utils';

const LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error'] as const;
type LogLevel = (typeof LOG_LEVELS)[number];

const levelColors: Record<LogLevel, string> = {
  trace: 'bg-gray-400',
  debug: 'bg-blue-500',
  info: 'bg-green-500',
  warn: 'bg-yellow-500',
  error: 'bg-red-500',
};

interface LogLevelSliderProps {
  value: number;
  onChange: (value: number) => void;
  className?: string;
}

export function LogLevelSlider({ value, onChange, className }: LogLevelSliderProps) {
  const currentLevel = LOG_LEVELS[value] || 'trace';

  return (
    <div className={cn('space-y-2', className)}>
      <div className="flex justify-between items-center">
        <span className="text-xs text-muted-foreground font-medium">Global Log Level</span>
        <span className={cn(
          'text-xs font-semibold uppercase px-2 py-0.5 rounded',
          levelColors[currentLevel],
          currentLevel === 'trace' ? 'text-gray-900' : 'text-white'
        )}>
          {currentLevel}
        </span>
      </div>

      <SliderPrimitive.Root
        className="relative flex items-center select-none touch-none w-full h-5"
        value={[value]}
        onValueChange={([v]) => onChange(v)}
        max={4}
        step={1}
      >
        {/* Track with current level color */}
        <SliderPrimitive.Track className={cn(
          "relative h-2 w-full grow overflow-hidden rounded-full transition-colors",
          levelColors[currentLevel]
        )}>
          <SliderPrimitive.Range className="absolute h-full" />
        </SliderPrimitive.Track>

        <SliderPrimitive.Thumb
          className="block h-4 w-4 rounded-full border-2 border-primary bg-background shadow transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
        />
      </SliderPrimitive.Root>

      {/* Level labels */}
      <div className="flex justify-between text-[10px] text-muted-foreground uppercase">
        {LOG_LEVELS.map((level) => (
          <span key={level} className="w-0 text-center">
            {level.charAt(0)}
          </span>
        ))}
      </div>
    </div>
  );
}

export { LOG_LEVELS, type LogLevel };
