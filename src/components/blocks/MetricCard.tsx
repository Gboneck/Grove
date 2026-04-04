interface MetricCardProps {
  label: string;
  value: string;
  trend?: "up" | "down" | "flat" | null;
}

const trendIcons: Record<string, string> = {
  up: "↑",
  down: "↓",
  flat: "→",
};

const trendColors: Record<string, string> = {
  up: "text-grove-status-green",
  down: "text-grove-status-red",
  flat: "text-grove-text-secondary",
};

export default function MetricCard({ label, value, trend }: MetricCardProps) {
  return (
    <div className="bg-grove-surface border border-grove-border rounded-lg p-5">
      <div className="text-xs uppercase tracking-wider text-grove-text-secondary mb-2">
        {label}
      </div>
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-mono font-semibold text-grove-text-primary">
          {value}
        </span>
        {trend && trend in trendIcons && (
          <span className={`text-sm ${trendColors[trend]}`}>
            {trendIcons[trend]}
          </span>
        )}
      </div>
    </div>
  );
}
