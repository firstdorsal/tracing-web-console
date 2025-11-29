import { useState, useEffect, useRef, useCallback } from 'react';
import { LogViewer } from '@/components/LogViewer';
import { LogFilters, type TargetFilterValue } from '@/components/LogFilters';
import { ExportButton } from '@/components/ExportButton';
import { ThemeToggle } from '@/components/ThemeToggle';
import { FilterPresets, type FilterPreset } from '@/components/FilterPresets';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';
import { RefreshCw, ArrowDown, ArrowUp, Github } from 'lucide-react';
import { useLogs } from '@/hooks/useLogs';
import { useTargets } from '@/hooks/useTargets';
import type { LogLevel } from '@/types/logs';
import { Toaster, toast } from 'sonner';
import { getInitialFilters, LAST_PRESET_KEY, loadLastPresetId, DEFAULT_PRESET } from '@/lib/presets';
import { loadSettings, updateSetting } from '@/lib/settings';

export default function App() {
  // Initialize filters from the last saved preset (if any)
  const initialFilters = getInitialFilters();
  const initialSettings = loadSettings();

  const [selectedLevels, setSelectedLevels] = useState<Set<LogLevel>>(initialFilters.selectedLevels);
  const [targetFilter, setTargetFilter] = useState(initialFilters.targetFilter);
  const [searchFilter, setSearchFilter] = useState(initialFilters.searchFilter);
  const [targetLevelFilters, setTargetLevelFilters] = useState<TargetFilterValue[]>(
    initialFilters.targetLevelFilters
  );
  const [autoScroll, setAutoScroll] = useState(initialSettings.autoScroll);
  const [reverseOrder, setReverseOrder] = useState(initialSettings.reverseOrder);

  const { targets: availableTargets } = useTargets();

  const { logs, connected, error, loadOlder, refresh, hasMore, isLoading, totalCount } = useLogs({
    selectedLevels,
    targetFilter,
    searchFilter,
    reverseOrder,
    targetLevelFilters,
  });

  const [lastPresetId, setLastPresetId] = useState<string | null>(() => {
    return loadLastPresetId();
  });
  const prevConnected = useRef<boolean | null>(null);
  const hasEverConnected = useRef(false);

  // Check if default/read-only preset is active (filters should be disabled)
  const isDefaultPresetActive = lastPresetId === DEFAULT_PRESET.id || lastPresetId === null;

  const handleLoadPreset = (preset: FilterPreset) => {
    setSelectedLevels(new Set(preset.selectedLevels));
    setTargetFilter(preset.targetFilter);
    setSearchFilter(preset.searchFilter);
    setTargetLevelFilters(preset.targetLevelFilters || []);

    // Save the last selected preset ID
    localStorage.setItem(LAST_PRESET_KEY, preset.id);
    setLastPresetId(preset.id);
  };

  // Show toast notifications for connection status changes (skip initial connection)
  useEffect(() => {
    if (connected && !hasEverConnected.current) {
      // First connection - no toast
      hasEverConnected.current = true;
    } else if (prevConnected.current !== null && prevConnected.current !== connected) {
      // Status change after first connection
      if (connected) {
        toast.success(`Reconnected to server`, {
          description: `Real-time log streaming active`,
        });
      } else {
        toast.error(`Disconnected from server`, {
          description: `Attempting to reconnect...`,
        });
      }
    }
    prevConnected.current = connected;
  }, [connected]);

  // Show toast for WebSocket errors
  useEffect(() => {
    if (error) {
      toast.error(`WebSocket Error`, {
        description: error,
      });
    }
  }, [error]);

  // Handle creating a filter from a target (right-click context menu)
  const handleCreateFilterFromTarget = useCallback((target: string) => {
    // Check if a filter for this target already exists
    const existingFilter = targetLevelFilters.find(f => f.target === target);
    if (existingFilter) {
      toast.info(`Filter already exists`, {
        description: `A filter for "${target}" already exists`,
      });
      return;
    }

    // Create a new target filter with trace level (most permissive)
    const newFilter: TargetFilterValue = {
      id: crypto.randomUUID(),
      target,
      level: `trace`,
    };
    setTargetLevelFilters(prev => [...prev, newFilter]);
    toast.success(`Filter created`, {
      description: `Added filter for "${target}"`,
    });
  }, [targetLevelFilters]);

  return (
    <div className={`h-screen flex flex-col bg-background`}>
      <Toaster position={`top-center`} richColors />

      {/* Header */}
      <header className={`bg-card border-b border-border px-6 py-4`}>
        <div className={`flex items-center justify-between`}>
          <div className={`flex items-center gap-4`}>
            <h1 className={`text-2xl font-bold text-foreground`}>Tracing Web Console</h1>
            <div className={`flex items-center gap-2`}>
              <span
                className={`w-2 h-2 rounded-full ${connected ? `bg-green-500` : `bg-red-500`}`}
              />
              <span className={`text-sm text-muted-foreground`}>
                {connected ? `Connected` : `Disconnected`}
              </span>
            </div>
          </div>

          <div className={`flex items-center gap-3`}>
            {/* GitHub link */}
            <a
              href={`https://github.com/firstdorsal/tracing-web-console`}
              target={`_blank`}
              rel={`noopener noreferrer`}
              className={`text-muted-foreground hover:text-foreground transition-colors`}
              title={`View on GitHub`}
            >
              <Github className={`w-5 h-5`} />
            </a>
            {/* Theme toggle */}
            <ThemeToggle />
          </div>
        </div>
      </header>

      {/* Main content area */}
      <div className={`flex-1 flex overflow-hidden`}>
        {/* Left sidebar - Presets and Filters */}
        <div className={`w-96 border-r border-border bg-background flex flex-col`}>
          {/* Filter Presets - 1/3 height */}
          <div className={`h-1/3 p-6 border-b border-border overflow-y-auto`}>
            <FilterPresets
              onLoadPreset={handleLoadPreset}
              activePresetId={lastPresetId}
              currentFilters={{
                selectedLevels,
                targetFilter,
                searchFilter,
                targetLevelFilters,
              }}
            />
          </div>

          {/* Filters - 2/3 height */}
          <div className={`flex-1 p-6 overflow-y-auto`}>
            <LogFilters
              selectedLevels={selectedLevels}
              onLevelsChange={setSelectedLevels}
              targetFilters={targetLevelFilters}
              onTargetFiltersChange={setTargetLevelFilters}
              availableTargets={availableTargets}
              disabled={isDefaultPresetActive}
            />
          </div>
        </div>

        {/* Right side - Log viewer */}
        <div className={`flex-1 flex flex-col min-w-0 relative`}>
          {/* Search and Stats bar */}
          <div className={`px-4 py-3 bg-muted/30 border-b border-border space-y-2`}>
            {/* Search Input and Controls */}
            <div className={`flex items-center gap-3`}>
              <Input
                type={`text`}
                placeholder={isDefaultPresetActive ? `Create a preset to enable search...` : `Search logs...`}
                value={searchFilter}
                onChange={(e) => setSearchFilter(e.target.value)}
                className={`flex-1`}
                disabled={isDefaultPresetActive}
              />

              {/* Refresh button */}
              <Button
                onClick={refresh}
                variant={`outline`}
                size={`icon`}
                title={`Refresh logs`}
              >
                <RefreshCw className={`w-4 h-4`} />
              </Button>

              {/* Reverse order button */}
              <Button
                onClick={() => {
                  setReverseOrder(!reverseOrder);
                  updateSetting(`reverseOrder`, !reverseOrder);
                }}
                variant={`outline`}
                size={`icon`}
                title={reverseOrder ? `Showing oldest first - click for newest first` : `Showing newest first - click for oldest first`}
              >
                {reverseOrder ? <ArrowUp className={`w-4 h-4`} /> : <ArrowDown className={`w-4 h-4`} />}
              </Button>

              {/* Auto-scroll toggle */}
              <label className={`flex items-center gap-2 text-sm text-foreground cursor-pointer whitespace-nowrap`}>
                <Checkbox
                  checked={autoScroll}
                  onCheckedChange={(checked) => {
                    const value = checked === true;
                    setAutoScroll(value);
                    updateSetting(`autoScroll`, value);
                  }}
                />
                Auto-scroll
              </label>

              {/* Export button */}
              <ExportButton
                totalCount={Math.max(logs.length, totalCount)}
                selectedLevels={selectedLevels}
                targetFilter={targetFilter}
                searchFilter={searchFilter}
              />
            </div>
          </div>

          {/* Log viewer */}
          <div className={`flex-1 overflow-hidden`}>
            <LogViewer
              logs={logs}
              totalCount={totalCount}
              autoScroll={autoScroll}
              reverseOrder={reverseOrder}
              onLoadOlder={loadOlder}
              hasMore={hasMore}
              isLoading={isLoading}
              onCreateFilterFromTarget={handleCreateFilterFromTarget}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
