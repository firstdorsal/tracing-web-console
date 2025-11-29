import { Component, createRef } from 'react';
import { createPortal } from 'react-dom';
import { VariableSizeList } from 'react-window';
import InfiniteLoader from 'react-window-infinite-loader';
import AutoSizer from 'react-virtualized-auto-sizer';
import { ChevronRight, ChevronDown } from 'lucide-react';
import type { LogEvent } from '@/types/logs';

interface LogViewerProps {
  logs: LogEvent[];
  totalCount: number;
  autoScroll?: boolean;
  reverseOrder?: boolean;
  onLoadOlder?: () => void;
  hasMore?: boolean;
  isLoading?: boolean;
  onCreateFilterFromTarget?: (target: string) => void;
}

const levelStripeColors: Record<string, string> = {
  trace: `bg-gray-400`,
  debug: `bg-blue-500`,
  info: `bg-green-500`,
  warn: `bg-yellow-500`,
  error: `bg-red-500`,
};

const ROW_HEIGHT = 48;
const EXPANDED_BASE_HEIGHT = 48; // Base height when expanded
const FIELD_ROW_HEIGHT = 24; // Height per field row

interface LogItemProps {
  log: LogEvent;
  isExpanded: boolean;
  onToggleExpand: () => void;
  onTargetContextMenu: (e: React.MouseEvent, target: string) => void;
}

function LogItem({ log, isExpanded, onToggleExpand, onTargetContextMenu }: LogItemProps) {
  const level = log.level.toLowerCase();

  // Format file path to show only the filename or last part of the path
  const formatFile = (file: string | undefined, line: number | undefined) => {
    if (!file) return null;
    const filename = file.split(`/`).pop() || file;
    return line ? `${filename}:${line}` : filename;
  };

  const location = formatFile(log.file, log.line);

  // Check if there are any fields or span to show
  const hasExpandableContent =
    (log.fields && Object.keys(log.fields).length > 0) ||
    (log.span && (log.span.name || Object.keys(log.span.fields).length > 0));

  const handleTargetContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    onTargetContextMenu(e, log.target);
  };

  return (
    <div className={`border-b border-border hover:bg-accent/50 transition-colors`}>
      {/* Main row - always visible */}
      <div
        className={`h-[48px] flex items-center ${hasExpandableContent ? `cursor-pointer` : ``}`}
        onClick={hasExpandableContent ? onToggleExpand : undefined}
      >
        {/* Level stripe */}
        <div className={`w-1 h-full flex-shrink-0 ${levelStripeColors[level] || `bg-gray-400`}`} />

        <div className={`flex items-center gap-3 font-mono text-sm w-full px-4`}>
          {/* Expand/collapse indicator */}
          <div className={`w-4 flex-shrink-0`}>
            {hasExpandableContent && (
              isExpanded ? (
                <ChevronDown className={`w-4 h-4 text-muted-foreground`} />
              ) : (
                <ChevronRight className={`w-4 h-4 text-muted-foreground`} />
              )
            )}
          </div>

          {/* Timestamp */}
          <span className={`text-xs whitespace-nowrap flex-shrink-0`}>
            <span className={`text-foreground`}>
              {new Date(log.timestamp).toLocaleTimeString(`en-US`, {
                hour12: false,
                hour: `2-digit`,
                minute: `2-digit`,
                second: `2-digit`,
              })}
            </span>
            <span className={`text-muted-foreground`}>
              .{new Date(log.timestamp).getMilliseconds().toString().padStart(3, `0`)}
            </span>
          </span>

          {/* Target */}
          <span
            className={`text-muted-foreground text-xs whitespace-nowrap flex-shrink-0 w-[200px] truncate hover:text-foreground cursor-pointer`}
            title={log.target}
            onContextMenu={handleTargetContextMenu}
          >
            {log.target}
          </span>

          {/* File:Line */}
          {location && (
            <span className={`text-muted-foreground/70 text-xs whitespace-nowrap flex-shrink-0 w-[120px] truncate`} title={log.file}>
              {location}
            </span>
          )}

          {/* Message */}
          <div className={`flex-1 min-w-0`}>
            <div className={`text-foreground truncate`}>{log.message}</div>
          </div>
        </div>
      </div>

      {/* Expanded content */}
      {isExpanded && hasExpandableContent && (
        <div className={`flex bg-accent/30`}>
          {/* Level stripe */}
          <div className={`w-1 flex-shrink-0 ${levelStripeColors[level] || `bg-gray-400`}`} />

          <div className={`pl-8 pr-4 pt-2 pb-2 flex-1`}>
            {/* Span information */}
            {log.span && (
              <div className={`mb-2`}>
                <div className={`text-xs font-semibold text-muted-foreground mb-1`}>Span: {log.span.name}</div>
                {Object.entries(log.span.fields).map(([key, value]) => (
                  <div key={key} className={`flex gap-2 text-xs font-mono h-[24px] items-center`}>
                    <span className={`text-blue-400`}>{key}:</span>
                    <span className={`text-foreground`}>{value}</span>
                  </div>
                ))}
              </div>
            )}

            {/* Event fields */}
            {log.fields && Object.keys(log.fields).length > 0 && (
              <div>
                <div className={`text-xs font-semibold text-muted-foreground mb-1`}>Fields</div>
                {Object.entries(log.fields).map(([key, value]) => (
                  <div key={key} className={`flex gap-2 text-xs font-mono h-[24px] items-center`}>
                    <span className={`text-green-400`}>{key}:</span>
                    <span className={`text-foreground`}>{value}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// Calculate the height of a log item based on whether it's expanded
function calculateItemHeight(log: LogEvent, isExpanded: boolean): number {
  if (!isExpanded) {
    return ROW_HEIGHT;
  }

  let height = EXPANDED_BASE_HEIGHT;

  // Add height for span section
  if (log.span) {
    height += FIELD_ROW_HEIGHT; // "Span: name" header
    height += Object.keys(log.span.fields).length * FIELD_ROW_HEIGHT;
  }

  // Add height for fields section
  if (log.fields && Object.keys(log.fields).length > 0) {
    height += FIELD_ROW_HEIGHT; // "Fields" header
    height += Object.keys(log.fields).length * FIELD_ROW_HEIGHT;
  }

  // Add padding (pt-2 + pb-2)
  height += 16;

  return height;
}

interface ContextMenuState {
  x: number;
  y: number;
  target: string;
}

interface LogViewerState {
  expandedItems: Set<number>;
  contextMenu: ContextMenuState | null;
}

export class LogViewer extends Component<LogViewerProps, LogViewerState> {
  private listRef = createRef<VariableSizeList>();
  private lastLogsLength = 0;
  private currentScrollOffset = 0;

  static defaultProps = {
    autoScroll: false,
    reverseOrder: false,
    hasMore: false,
    isLoading: false,
  };

  state: LogViewerState = {
    expandedItems: new Set(),
    contextMenu: null,
  };

  componentDidMount() {
    document.addEventListener(`click`, this.handleDocumentClick);
    document.addEventListener(`keydown`, this.handleKeyDown);
  }

  componentWillUnmount() {
    document.removeEventListener(`click`, this.handleDocumentClick);
    document.removeEventListener(`keydown`, this.handleKeyDown);
  }

  handleDocumentClick = () => {
    if (this.state.contextMenu) {
      this.setState({ contextMenu: null });
    }
  };

  handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === `Escape` && this.state.contextMenu) {
      this.setState({ contextMenu: null });
    }
  };

  handleTargetContextMenu = (e: React.MouseEvent, target: string) => {
    this.setState({
      contextMenu: {
        x: e.clientX,
        y: e.clientY,
        target,
      },
    });
  };

  handleCreateFilter = () => {
    const { onCreateFilterFromTarget } = this.props;
    const { contextMenu } = this.state;
    if (onCreateFilterFromTarget && contextMenu) {
      onCreateFilterFromTarget(contextMenu.target);
    }
    this.setState({ contextMenu: null });
  };

  componentDidUpdate(_prevProps: LogViewerProps) {
    const { logs, autoScroll, reverseOrder } = this.props;
    const newItemsCount = logs.length - this.lastLogsLength;

    if (newItemsCount > 0 && this.listRef.current) {
      // Reset cached sizes when new items arrive
      this.listRef.current.resetAfterIndex(0);

      if (autoScroll) {
        if (reverseOrder) {
          // When reversed (oldest first), new logs appear at bottom - scroll to end
          this.listRef.current.scrollToItem(logs.length - 1, `end`);
        } else {
          // Normal order (newest first), new logs appear at top - scroll to start
          this.listRef.current.scrollToItem(0, `start`);
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

  // Toggle expanded state for an item
  toggleExpanded = (index: number) => {
    this.setState((prevState) => {
      const newExpanded = new Set(prevState.expandedItems);
      if (newExpanded.has(index)) {
        newExpanded.delete(index);
      } else {
        newExpanded.add(index);
      }
      return { expandedItems: newExpanded };
    }, () => {
      // Reset cached size for this item after state update
      if (this.listRef.current) {
        this.listRef.current.resetAfterIndex(index);
      }
    });
  };

  // Get item size based on expanded state
  getItemSize = (index: number): number => {
    const { logs } = this.props;
    if (index >= logs.length) {
      return ROW_HEIGHT; // Loading placeholder
    }
    const isExpanded = this.state.expandedItems.has(index);
    return calculateItemHeight(logs[index], isExpanded);
  };

  // Check if an item is loaded
  isItemLoaded = (index: number): boolean => {
    return index < this.props.logs.length;
  };

  // Load more items
  loadMoreItems = (): Promise<void> => {
    const { hasMore, isLoading, onLoadOlder } = this.props;

    if (hasMore && !isLoading && onLoadOlder) {
      console.log(`[LogViewer] Loading more items...`);
      onLoadOlder();
    }

    return Promise.resolve();
  };

  // Render a single row
  renderRow = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const { logs } = this.props;

    if (index >= logs.length) {
      // Loading placeholder
      return (
        <div style={style} className={`flex items-center justify-center text-muted-foreground`}>
          Loading...
        </div>
      );
    }

    const isExpanded = this.state.expandedItems.has(index);

    // Data is pre-sorted by the backend, so index maps directly to logs array
    return (
      <div style={style}>
        <LogItem
          log={logs[index]}
          isExpanded={isExpanded}
          onToggleExpand={() => this.toggleExpanded(index)}
          onTargetContextMenu={this.handleTargetContextMenu}
        />
      </div>
    );
  };

  renderContextMenu() {
    const { contextMenu } = this.state;
    if (!contextMenu) return null;

    return createPortal(
      <div
        className={`fixed z-50 bg-popover border border-border rounded-md shadow-md py-1 min-w-[180px]`}
        style={{ left: contextMenu.x, top: contextMenu.y }}
        onClick={(e) => e.stopPropagation()}
      >
        <button
          className={`w-full px-3 py-1.5 text-sm text-left hover:bg-accent transition-colors`}
          onClick={this.handleCreateFilter}
        >
          Create filter for "{contextMenu.target}"
        </button>
      </div>,
      document.body
    );
  }

  render() {
    const { logs, totalCount } = this.props;

    if (logs.length === 0) {
      return (
        <div className={`flex items-center justify-center h-full text-muted-foreground`}>
          <div className={`text-center`}>
            <div className={`text-lg mb-2`}>No logs yet</div>
            <div className={`text-sm`}>Waiting for log events...</div>
          </div>
        </div>
      );
    }

    // Use totalCount from server as the item count
    const itemCount = totalCount;

    return (
      <div className={`h-full w-full bg-background text-foreground`}>
        <AutoSizer>
          {({ height, width }) => (
            <InfiniteLoader
              isItemLoaded={this.isItemLoaded}
              itemCount={itemCount}
              loadMoreItems={this.loadMoreItems}
              threshold={10}
            >
              {({ onItemsRendered, ref }) => (
                <VariableSizeList
                  ref={(list) => {
                    // @ts-expect-error - ref types don't match perfectly
                    ref(list);
                    // @ts-expect-error - also store our own ref
                    this.listRef.current = list;
                  }}
                  height={height}
                  width={width}
                  itemCount={itemCount}
                  itemSize={this.getItemSize}
                  onItemsRendered={onItemsRendered}
                  onScroll={this.handleScroll}
                  overscanCount={5}
                >
                  {this.renderRow}
                </VariableSizeList>
              )}
            </InfiniteLoader>
          )}
        </AutoSizer>
        {this.renderContextMenu()}
      </div>
    );
  }
}
