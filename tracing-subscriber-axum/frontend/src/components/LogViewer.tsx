import { Component, createRef } from 'react';
import { FixedSizeList } from 'react-window';
import InfiniteLoader from 'react-window-infinite-loader';
import AutoSizer from 'react-virtualized-auto-sizer';
import type { LogEvent } from '@/types/logs';

interface LogViewerProps {
  logs: LogEvent[];
  totalCount: number;
  autoScroll?: boolean;
  reverseOrder?: boolean;
  onLoadOlder?: () => void;
  hasMore?: boolean;
  isLoading?: boolean;
}

const levelStripeColors: Record<string, string> = {
  trace: 'bg-gray-400',
  debug: 'bg-blue-500',
  info: 'bg-green-500',
  warn: 'bg-yellow-500',
  error: 'bg-red-500',
};

const ROW_HEIGHT = 48;

function LogItem({ log }: { log: LogEvent }) {
  const level = log.level.toLowerCase();

  return (
    <div className="border-b border-border hover:bg-accent/50 transition-colors h-[48px] flex items-center">
      {/* Level stripe */}
      <div className={`w-1 h-full flex-shrink-0 ${levelStripeColors[level] || 'bg-gray-400'}`} />

      <div className="flex items-center gap-3 font-mono text-sm w-full px-4">
        {/* Timestamp */}
        <span className="text-xs whitespace-nowrap flex-shrink-0">
          <span className="text-foreground">
            {new Date(log.timestamp).toLocaleTimeString('en-US', {
              hour12: false,
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit',
            })}
          </span>
          <span className="text-muted-foreground">
            .{new Date(log.timestamp).getMilliseconds().toString().padStart(3, '0')}
          </span>
        </span>

        {/* Target */}
        <span className="text-muted-foreground text-xs whitespace-nowrap flex-shrink-0 w-[250px] truncate">
          {log.target}
        </span>

        {/* Message */}
        <div className="flex-1 min-w-0">
          <div className="text-foreground truncate">{log.message}</div>
        </div>
      </div>
    </div>
  );
}

export class LogViewer extends Component<LogViewerProps> {
  private listRef = createRef<FixedSizeList>();
  private lastLogsLength = 0;
  private currentScrollOffset = 0;

  static defaultProps = {
    autoScroll: false,
    reverseOrder: false,
    hasMore: false,
    isLoading: false,
  };

  componentDidUpdate(prevProps: LogViewerProps) {
    const { logs, autoScroll, reverseOrder } = this.props;
    const newItemsCount = logs.length - this.lastLogsLength;

    if (newItemsCount > 0 && this.listRef.current) {
      if (autoScroll) {
        if (reverseOrder) {
          // When reversed (oldest first), new logs appear at bottom - scroll to end
          this.listRef.current.scrollToItem(logs.length - 1, 'end');
        } else {
          // Normal order (newest first), new logs appear at top - scroll to start
          this.listRef.current.scrollToItem(0, 'start');
        }
      } else if (!reverseOrder) {
        // Keep the view stable by adjusting scroll position (only in normal order)
        // New items are added at the top, so we need to scroll down by their height
        const newScrollOffset = this.currentScrollOffset + (newItemsCount * ROW_HEIGHT);
        this.listRef.current.scrollTo(newScrollOffset);
      }
    }
    this.lastLogsLength = logs.length;
  }

  handleScroll = ({ scrollOffset }: { scrollOffset: number }) => {
    this.currentScrollOffset = scrollOffset;
  };

  // Check if an item is loaded
  isItemLoaded = (index: number): boolean => {
    return index < this.props.logs.length;
  };

  // Load more items
  loadMoreItems = (): Promise<void> => {
    const { hasMore, isLoading, onLoadOlder } = this.props;

    if (hasMore && !isLoading && onLoadOlder) {
      console.log('[LogViewer] Loading more items...');
      onLoadOlder();
    }

    return Promise.resolve();
  };

  // Render a single row
  renderRow = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const { logs, reverseOrder } = this.props;

    if (index >= logs.length) {
      // Loading placeholder
      return (
        <div style={style} className="flex items-center justify-center text-muted-foreground">
          Loading...
        </div>
      );
    }

    // When reversed, show oldest first (reverse the index)
    const logIndex = reverseOrder ? logs.length - 1 - index : index;

    return (
      <div style={style}>
        <LogItem log={logs[logIndex]} />
      </div>
    );
  };

  render() {
    const { logs, totalCount } = this.props;

    if (logs.length === 0) {
      return (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          <div className="text-center">
            <div className="text-lg mb-2">No logs yet</div>
            <div className="text-sm">Waiting for log events...</div>
          </div>
        </div>
      );
    }

    // Use totalCount from server as the item count
    const itemCount = totalCount;

    return (
      <div className="h-full w-full bg-background text-foreground">
        <AutoSizer>
          {({ height, width }) => (
            <InfiniteLoader
              isItemLoaded={this.isItemLoaded}
              itemCount={itemCount}
              loadMoreItems={this.loadMoreItems}
              threshold={10}
            >
              {({ onItemsRendered, ref }) => (
                <FixedSizeList
                  ref={(list) => {
                    // @ts-expect-error - ref types don't match perfectly
                    ref(list);
                    // @ts-expect-error - also store our own ref
                    this.listRef.current = list;
                  }}
                  height={height}
                  width={width}
                  itemCount={itemCount}
                  itemSize={ROW_HEIGHT}
                  onItemsRendered={onItemsRendered}
                  onScroll={this.handleScroll}
                  overscanCount={5}
                >
                  {this.renderRow}
                </FixedSizeList>
              )}
            </InfiniteLoader>
          )}
        </AutoSizer>
      </div>
    );
  }
}
