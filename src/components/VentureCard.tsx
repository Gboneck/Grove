interface Venture {
  name: string;
  status?: string;
  health?: string;
  priority?: number;
  deadline?: string;
  nextAction?: string;
}

interface VentureCardProps {
  venture: Venture;
  onFocus: (name: string) => void;
}

const HEALTH_COLORS: Record<string, string> = {
  green: "bg-grove-status-green",
  yellow: "bg-grove-status-yellow",
  red: "bg-grove-status-red",
};

function daysUntil(deadline: string): number | null {
  const d = new Date(deadline);
  if (isNaN(d.getTime())) return null;
  const now = new Date();
  return Math.ceil((d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
}

export default function VentureCard({ venture, onFocus }: VentureCardProps) {
  const healthColor = HEALTH_COLORS[venture.health || "green"] || "bg-gray-500";
  const days = venture.deadline ? daysUntil(venture.deadline) : null;

  return (
    <button
      onClick={() => onFocus(venture.name)}
      className="w-full text-left p-3 rounded-lg bg-grove-surface/50 border border-grove-border/50 hover:border-grove-accent/30 hover:bg-grove-surface transition-all group"
    >
      <div className="flex items-center gap-2 mb-1">
        <span className={`w-2 h-2 rounded-full ${healthColor} flex-shrink-0`} />
        <span className="text-sm font-medium text-grove-text-primary truncate font-sans">
          {venture.name}
        </span>
        {days !== null && (
          <span className={`text-xs font-mono ml-auto flex-shrink-0 ${days <= 3 ? "text-grove-status-red" : days <= 7 ? "text-grove-status-yellow" : "text-grove-text-secondary"}`}>
            {days}d
          </span>
        )}
      </div>
      {venture.nextAction && (
        <p className="text-xs text-grove-text-secondary truncate pl-4">
          {venture.nextAction}
        </p>
      )}
      {venture.status && !venture.nextAction && (
        <p className="text-xs text-grove-text-secondary truncate pl-4 opacity-60">
          {venture.status}
        </p>
      )}
    </button>
  );
}

export type { Venture };
