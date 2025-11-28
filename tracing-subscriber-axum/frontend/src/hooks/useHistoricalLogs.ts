import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import type { LogEvent, LogLevel } from '@/types/logs';

const BATCH_SIZE = 100; // Fetch 100 logs at a time

// Log level hierarchy: TRACE < DEBUG < INFO < WARN < ERROR
const LOG_LEVEL_ORDER: LogLevel[] = ['trace', 'debug', 'info', 'warn', 'error'];

interface UseLogPollingProps {
  selectedLevels: Set<LogLevel>;
  targetFilter: string;
  searchFilter: string;
}

interface UseLogPollingReturn {
  logs: LogEvent[];
  connected: boolean;
  error: string | null;
  loadOlder: () => void;
  refresh: () => void;
  hasMore: boolean;
  isLoading: boolean;
  totalCount: number;
}

/**
 * Get the API base URL
 */
function getApiBaseUrl(): string {
  // In development mode, use the backend URL from environment
  if (import.meta.env.DEV && import.meta.env.VITE_BACKEND_URL) {
    return import.meta.env.VITE_BACKEND_URL;
  }
  // In production, use relative paths (will use current host)
  return '';
}

/**
 * Get WebSocket URL from API base URL
 */
function getWebSocketUrl(): string {
  const baseUrl = getApiBaseUrl();

  if (baseUrl) {
    // Convert http to ws
    const wsUrl = baseUrl.replace(/^http/, 'ws');
    return `${wsUrl}/api/ws`;
  }

  // In production, construct WebSocket URL from current location
  // The app is served from /tracing, so WebSocket is at /tracing/api/ws
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/tracing/api/ws`;
}

/**
 * Client-side filter to apply exact level filtering on logs
 */
function filterLogsByLevel(logs: LogEvent[], selectedLevels: Set<LogLevel>): LogEvent[] {
  if (selectedLevels.size === 0 || selectedLevels.size === 5) {
    // No filtering needed if all levels selected or none specified
    return logs;
  }
  return logs.filter((log) => selectedLevels.has(log.level.toLowerCase() as LogLevel));
}

/**
 * Check if a single log matches search and target filters
 */
function matchesFilters(log: LogEvent, targetFilter: string, searchFilter: string): boolean {
  if (targetFilter && !log.target.toLowerCase().includes(targetFilter.toLowerCase())) {
    return false;
  }
  if (searchFilter && !log.message.toLowerCase().includes(searchFilter.toLowerCase())) {
    return false;
  }
  return true;
}

export function useLogPolling({
  selectedLevels,
  targetFilter,
  searchFilter,
}: UseLogPollingProps): UseLogPollingReturn {
  const [rawLogs, setRawLogs] = useState<LogEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [totalCount, setTotalCount] = useState(0);
  const wsRef = useRef<WebSocket | null>(null);
  const lastTimestampRef = useRef<string | null>(null);
  const offsetRef = useRef(0);
  // Refs to access current filter values in WebSocket handler
  const targetFilterRef = useRef(targetFilter);
  const searchFilterRef = useRef(searchFilter);
  targetFilterRef.current = targetFilter;
  searchFilterRef.current = searchFilter;

  const fetchLogs = useCallback(
    async (isLoadOlder = false) => {
      try {
        if (isLoadOlder) {
          setIsLoading(true);
        }

        const baseUrl = getApiBaseUrl();
        const limit = BATCH_SIZE;
        const offset = isLoadOlder ? offsetRef.current : 0;

        // Find the lowest (most permissive) selected level as the global threshold
        // If no levels are selected, default to 'trace' to show everything
        let globalLevel = 'trace';
        if (selectedLevels.size > 0) {
          for (const level of LOG_LEVEL_ORDER) {
            if (selectedLevels.has(level)) {
              globalLevel = level;
              break;
            }
          }
        }

        // Build request body with all filters
        const requestBody = {
          limit,
          offset,
          global_level: globalLevel,
          target_levels: {}, // No per-target overrides for now
          search: searchFilter || null,
          target: targetFilter || null,
        };

        const url = baseUrl ? `${baseUrl}/api/logs` : `api/logs`;
        console.log('[useLogPolling] Fetching logs from:', url, 'isLoadOlder:', isLoadOlder, 'offset:', offset);

        const response = await fetch(url, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(requestBody),
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch logs: ${response.statusText}`);
        }

        const data = await response.json();
        const newLogs: LogEvent[] = data.logs || [];
        const serverTotal = data.total || 0;
        console.log('[useLogPolling] Received', newLogs.length, 'logs, total in storage:', serverTotal);

        // Update total count from server
        setTotalCount(serverTotal);

        if (isLoadOlder) {
          // Append older logs to the end
          setRawLogs((prev) => {
            console.log('[useLogPolling] Appending', newLogs.length, 'older logs to', prev.length, 'existing logs');
            return [...prev, ...newLogs];
          });
          offsetRef.current += newLogs.length;
          setHasMore(newLogs.length === limit);
        } else {
          // This is a fresh fetch due to filter change or initial load
          setRawLogs(newLogs);
          offsetRef.current = newLogs.length;
          setHasMore(newLogs.length === limit);
          if (newLogs.length > 0) {
            lastTimestampRef.current = newLogs[0].timestamp;
          }
        }

        setConnected(true);
        setError(null);
      } catch (err) {
        console.error('Error fetching logs:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
        setConnected(false);
      } finally {
        if (isLoadOlder) {
          setIsLoading(false);
        }
      }
    },
    [selectedLevels, targetFilter, searchFilter]
  );

  const loadOlder = useCallback(() => {
    if (!isLoading && hasMore) {
      fetchLogs(true);
    }
  }, [isLoading, hasMore, fetchLogs]);

  const refresh = useCallback(() => {
    setRawLogs([]);
    offsetRef.current = 0;
    lastTimestampRef.current = null;
    fetchLogs(false);
  }, [fetchLogs]);

  // Effect for WebSocket connection to receive real-time logs with auto-reconnect
  useEffect(() => {
    let ws: WebSocket | null = null;
    let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
    let reconnectAttempts = 0;
    let isCleaningUp = false;
    const MAX_RECONNECT_DELAY = 30000; // 30 seconds max

    const connect = () => {
      if (isCleaningUp) return;

      const wsUrl = getWebSocketUrl();
      console.log('[useLogPolling] Connecting to WebSocket:', wsUrl, 'attempt:', reconnectAttempts + 1);

      ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('[useLogPolling] WebSocket connected');
        reconnectAttempts = 0; // Reset on successful connection
        setConnected(true);
        setError(null);
      };

      ws.onmessage = (event) => {
        try {
          const newLog: LogEvent = JSON.parse(event.data);
          // Only add log if it matches current filters
          if (matchesFilters(newLog, targetFilterRef.current, searchFilterRef.current)) {
            // Prepend new log to the list (newest first)
            setRawLogs((prev) => [newLog, ...prev]);
            // Also increment totalCount since we have a new log
            setTotalCount((prev) => prev + 1);
          }
        } catch (err) {
          console.error('[useLogPolling] Failed to parse WebSocket message:', err);
        }
      };

      ws.onerror = (err) => {
        console.error('[useLogPolling] WebSocket error:', err);
        setError('WebSocket connection failed');
      };

      ws.onclose = (event) => {
        console.log('[useLogPolling] WebSocket closed, code:', event.code, 'reason:', event.reason);
        setConnected(false);
        wsRef.current = null;

        // Auto-reconnect unless we're cleaning up or it was a normal close
        if (!isCleaningUp && event.code !== 1000) {
          reconnectAttempts++;
          // Exponential backoff: 1s, 2s, 4s, 8s... up to MAX_RECONNECT_DELAY
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts - 1), MAX_RECONNECT_DELAY);
          console.log(`[useLogPolling] Reconnecting in ${delay}ms...`);
          reconnectTimeout = setTimeout(connect, delay);
        }
      };
    };

    connect();

    return () => {
      console.log('[useLogPolling] Cleanup: closing WebSocket');
      isCleaningUp = true;
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
      }
      if (ws) {
        ws.close(1000, 'Component unmounting');
      }
    };
  }, []); // Empty deps - connect once on mount

  // Effect for initial load and filter changes
  useEffect(() => {
    // Reset on filter change
    setRawLogs([]);
    offsetRef.current = 0;
    lastTimestampRef.current = null;
    fetchLogs(false);
  }, [fetchLogs]);

  // Apply client-side filtering for exact log level matching
  // (search and target filtering is done server-side)
  const filteredLogs = useMemo(() => {
    return filterLogsByLevel(rawLogs, selectedLevels);
  }, [rawLogs, selectedLevels]);

  return { logs: filteredLogs, connected, error, loadOlder, refresh, hasMore, isLoading, totalCount };
}
