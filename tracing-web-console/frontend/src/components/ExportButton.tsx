import { useState } from 'react';
import type { LogEvent, LogLevel } from '@/types/logs';
import { Button } from '@/components/ui/button';
import { Download, Loader2 } from 'lucide-react';
import { getApiBaseUrl } from '@/lib/api';

const LOG_LEVEL_ORDER: LogLevel[] = [`trace`, `debug`, `info`, `warn`, `error`];

interface ExportButtonProps {
  totalCount: number;
  selectedLevels: Set<LogLevel>;
  targetFilter: string;
  searchFilter: string;
  className?: string;
}

export function ExportButton({
  totalCount,
  selectedLevels,
  targetFilter,
  searchFilter,
  className,
}: ExportButtonProps) {
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = async () => {
    if (totalCount === 0 || isExporting) {
      return;
    }

    setIsExporting(true);

    try {
      // Find the lowest (most permissive) selected level as the global threshold
      let globalLevel = `trace`;
      if (selectedLevels.size > 0) {
        for (const level of LOG_LEVEL_ORDER) {
          if (selectedLevels.has(level)) {
            globalLevel = level;
            break;
          }
        }
      }

      // Fetch all logs from API (no limit)
      const baseUrl = getApiBaseUrl();
      const url = baseUrl ? `${baseUrl}/api/logs` : `api/logs`;

      const response = await fetch(url, {
        method: `POST`,
        headers: {
          'Content-Type': `application/json`,
        },
        body: JSON.stringify({
          global_level: globalLevel,
          target_levels: {},
          search: searchFilter || null,
          target: targetFilter || null,
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch logs: ${response.statusText}`);
      }

      const data = await response.json();
      const logs: LogEvent[] = data.logs || [];

      // Create JSON blob
      const jsonStr = JSON.stringify(logs, null, 2);
      const blob = new Blob([jsonStr], { type: `application/json` });
      const blobUrl = URL.createObjectURL(blob);

      // Create download link
      const link = document.createElement(`a`);
      link.href = blobUrl;
      link.download = `logs-${new Date().toISOString().replace(/[:.]/g, `-`)}.json`;
      document.body.appendChild(link);
      link.click();

      // Cleanup
      document.body.removeChild(link);
      URL.revokeObjectURL(blobUrl);
    } catch (err) {
      console.error(`Error exporting logs:`, err);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <Button
      onClick={handleExport}
      disabled={totalCount === 0 || isExporting}
      variant={`outline`}
      size={`icon`}
      className={className}
      title={`Export ${totalCount.toLocaleString()} logs`}
    >
      {isExporting ? (
        <Loader2 className={`w-4 h-4 animate-spin`} />
      ) : (
        <Download className={`w-4 h-4`} />
      )}
    </Button>
  );
}
