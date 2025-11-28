export interface LogEvent {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  fields: Record<string, string>;
  span?: {
    name: string;
    fields: Record<string, string>;
  };
}

export interface WSMessage {
  type: 'log' | 'historical';
  data: LogEvent | LogEvent[];
}

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface LogFilters {
  levels: Set<LogLevel>;
  target: string;
  search: string;
}
