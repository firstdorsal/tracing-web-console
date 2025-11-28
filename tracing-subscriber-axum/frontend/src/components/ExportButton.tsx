import type { LogEvent } from '@/types/logs';
import { Button } from '@/components/ui/button';
import { Download } from 'lucide-react';

interface ExportButtonProps {
  logs: LogEvent[];
  className?: string;
}

export function ExportButton({ logs, className }: ExportButtonProps) {
  const handleExport = () => {
    if (logs.length === 0) {
      return;
    }

    try {
      // Create JSON blob
      const jsonStr = JSON.stringify(logs, null, 2);
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);

      // Create download link
      const link = document.createElement('a');
      link.href = url;
      link.download = `logs-${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
      document.body.appendChild(link);
      link.click();

      // Cleanup
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Error exporting logs:', err);
    }
  };

  return (
    <Button
      onClick={handleExport}
      disabled={logs.length === 0}
      variant="outline"
      size="icon"
      className={className}
      title={`Export ${logs.length} logs`}
    >
      <Download className="w-4 h-4" />
    </Button>
  );
}
