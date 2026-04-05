import { useState } from "react";

interface ArtifactBlockProps {
  id: string;
  name: string;
  artifactType: string;
  summary?: string;
  blocks: Array<Record<string, unknown>>;
  updatedAt: string;
  updateCount: number;
  onRemove?: (id: string) => void;
  isDragging?: boolean;
  isDragOver?: boolean;
  dragHandleProps?: {
    draggable: boolean;
    onDragStart: (e: React.DragEvent) => void;
    onDragEnd: (e: React.DragEvent) => void;
  };
  onDragOver?: (e: React.DragEvent) => void;
  onDrop?: (e: React.DragEvent) => void;
  onPointerDownDrag?: (e: React.PointerEvent) => void;
}

const TYPE_ICONS: Record<string, string> = {
  dashboard: "◧",
  journal: "◉",
  brief: "◆",
  tracker: "◫",
  map: "◈",
  custom: "◇",
};

const TYPE_COLORS: Record<string, string> = {
  dashboard: "border-l-grove-accent",
  journal: "border-l-[#60a5fa]",
  brief: "border-l-[#c084fc]",
  tracker: "border-l-grove-status-green",
  map: "border-l-[#f59e0b]",
  custom: "border-l-grove-text-secondary",
};

function relativeTime(isoString: string): string {
  const ms = Date.now() - new Date(isoString).getTime();
  const mins = Math.floor(ms / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export default function ArtifactBlock({
  id,
  name,
  artifactType,
  summary,
  blocks,
  updatedAt,
  updateCount,
  onRemove,
  isDragging,
  isDragOver,
  dragHandleProps,
  onDragOver,
  onDrop,
  onPointerDownDrag,
}: ArtifactBlockProps) {
  const [expanded, setExpanded] = useState(true);
  const icon = TYPE_ICONS[artifactType] || TYPE_ICONS.custom;
  const borderColor = TYPE_COLORS[artifactType] || TYPE_COLORS.custom;

  return (
    <div
      className={`rounded-lg border border-grove-border/30 ${borderColor} border-l-2 bg-grove-surface/30 overflow-hidden transition-all duration-200 ${
        isDragging ? "opacity-40 scale-[0.98]" : ""
      } ${
        isDragOver ? "border-grove-accent/60 shadow-[0_0_12px_rgba(212,168,83,0.15)]" : ""
      }`}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      {/* Header — always visible */}
      <div
        className="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-grove-surface/50 transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center gap-2.5">
          {/* Drag handle */}
          <span
            className="text-grove-text-secondary/30 hover:text-grove-text-secondary cursor-grab active:cursor-grabbing text-xs select-none"
            onClick={(e) => e.stopPropagation()}
            onPointerDown={(e) => {
              if (onPointerDownDrag) {
                onPointerDownDrag(e);
              }
            }}
            {...(onPointerDownDrag ? {} : dragHandleProps)}
          >
            ⠿
          </span>
          <span className="text-grove-accent text-sm">{icon}</span>
          <span className="font-serif text-grove-text-primary text-[15px]">{name}</span>
          <span className="text-[9px] uppercase tracking-widest text-grove-text-secondary font-sans">
            {artifactType}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[10px] text-grove-text-secondary font-mono">
            v{updateCount} · {relativeTime(updatedAt)}
          </span>
          {onRemove && (
            <button
              onClick={(e) => { e.stopPropagation(); onRemove(id); }}
              className="text-grove-text-secondary hover:text-grove-status-red transition-colors text-xs"
              title="Remove from workspace"
            >
              ×
            </button>
          )}
          <span className={`text-grove-text-secondary text-xs transition-transform duration-200 ${expanded ? "rotate-0" : "-rotate-90"}`}>
            ▾
          </span>
        </div>
      </div>

      {/* Summary line */}
      {summary && !expanded && (
        <div className="px-4 pb-2 text-xs text-grove-text-secondary italic">
          {summary}
        </div>
      )}

      {/* Content blocks */}
      {expanded && (
        <div className="px-4 pb-4 space-y-3 border-t border-grove-border/20 pt-3">
          {blocks.map((block, i) => (
            <ArtifactContentBlock key={i} block={block} />
          ))}
        </div>
      )}
    </div>
  );
}

/** Render a single block inside an artifact — simplified, no wrapper chrome */
function ArtifactContentBlock({ block }: { block: Record<string, unknown> }) {
  switch (block.type as string) {
    case "text":
      return (
        <div className="space-y-1">
          {block.heading && (
            <h3 className="text-sm font-semibold text-grove-text-primary font-serif">{block.heading as string}</h3>
          )}
          {block.body && (
            <p className="text-sm text-grove-text-secondary leading-relaxed">{block.body as string}</p>
          )}
        </div>
      );
    case "metric":
      return (
        <div className="flex items-baseline gap-2">
          <span className="text-[10px] uppercase tracking-widest text-grove-text-secondary">{block.label as string}</span>
          <span className="text-lg font-mono font-semibold text-grove-text-primary">{block.value as string}</span>
          {block.trend && <span className="text-xs text-grove-text-secondary">{block.trend === "up" ? "↑" : block.trend === "down" ? "↓" : "→"}</span>}
        </div>
      );
    case "status":
      return (
        <div className="flex flex-wrap gap-2">
          {(block.items as Array<{ name: string; status: string; detail?: string }>)?.map((item, i) => (
            <span key={i} className={`text-xs px-2 py-0.5 rounded-full ${
              item.status === "green" ? "bg-grove-status-green/10 text-grove-status-green" :
              item.status === "red" ? "bg-grove-status-red/10 text-grove-status-red" :
              "bg-grove-accent/10 text-grove-accent"
            }`}>
              {item.name}{item.detail ? ` — ${item.detail}` : ""}
            </span>
          ))}
        </div>
      );
    case "list":
      return (
        <div className="space-y-1">
          {block.heading && <span className="text-xs text-grove-text-secondary">{block.heading as string}</span>}
          <ul className="text-sm text-grove-text-secondary space-y-0.5">
            {(block.items as string[])?.map((item, i) => (
              <li key={i} className="flex items-start gap-1.5">
                <span className="text-grove-accent text-[8px] mt-1.5">●</span>
                {item}
              </li>
            ))}
          </ul>
        </div>
      );
    case "progress":
      return (
        <div className="space-y-1">
          <div className="flex justify-between text-xs text-grove-text-secondary">
            <span>{block.label as string}</span>
            <span className="font-mono">{block.value as number}/{(block.max as number) || 100}</span>
          </div>
          <div className="h-1 bg-grove-border/30 rounded-full overflow-hidden">
            <div className="h-full bg-grove-accent/70 rounded-full" style={{ width: `${Math.min(100, ((block.value as number) / ((block.max as number) || 100)) * 100)}%` }} />
          </div>
        </div>
      );
    case "timeline":
      return (
        <div className="space-y-2">
          {block.heading && <span className="text-xs text-grove-text-secondary">{block.heading as string}</span>}
          <div className="space-y-1.5 border-l border-grove-border/30 pl-3">
            {(block.events as Array<{ time: string; label: string; detail?: string; type?: string }>)?.map((event, i) => (
              <div key={i} className="relative">
                <div className="absolute -left-[15px] top-1 w-1.5 h-1.5 rounded-full bg-grove-accent/60" />
                <div className="flex items-baseline gap-2">
                  <span className="text-[10px] text-grove-text-secondary font-mono shrink-0">{event.time}</span>
                  <span className="text-sm text-grove-text-primary">{event.label}</span>
                </div>
                {event.detail && <p className="text-xs text-grove-text-secondary ml-0">{event.detail}</p>}
              </div>
            ))}
          </div>
        </div>
      );
    case "insight":
      return (
        <div className="flex items-start gap-2 text-sm">
          <span className="text-grove-accent text-xs mt-0.5">
            {block.icon === "warning" ? "⚠" : block.icon === "alert" ? "!" : block.icon === "opportunity" ? "★" : "💡"}
          </span>
          <span className="text-grove-text-secondary">{block.message as string}</span>
        </div>
      );
    case "actions":
      return (
        <div className="space-y-1">
          {block.title && <span className="text-xs text-grove-text-secondary">{block.title as string}</span>}
          <div className="flex flex-wrap gap-1.5">
            {(block.items as Array<{ action: string; detail: string }>)?.map((item, i) => (
              <span key={i} className="text-xs px-2 py-1 rounded bg-grove-accent/10 text-grove-accent">
                {item.action}
              </span>
            ))}
          </div>
        </div>
      );
    default:
      return (
        <p className="text-sm text-grove-text-secondary">
          {(block.body as string) || (block.message as string) || JSON.stringify(block)}
        </p>
      );
  }
}
