import AnimatedValue from "../AnimatedValue";

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

const borderAccents: Record<string, string> = {
  up: "border-l-grove-status-green",
  down: "border-l-grove-status-red",
  flat: "border-l-grove-border",
};

export default function MetricCard({ label, value, trend }: MetricCardProps) {
  const borderClass = trend && trend in borderAccents
    ? borderAccents[trend]
    : "border-l-grove-accent/30";

  // Try to parse numeric value for animation
  const numericValue = parseFloat(value.replace(/[^0-9.-]/g, ""));
  const isNumeric = !isNaN(numericValue) && isFinite(numericValue);
  const prefix = isNumeric ? value.replace(/[0-9.-]/g, "").split(/[0-9]/)[0] || "" : "";
  const suffix = isNumeric ? value.replace(/^[^0-9]*[0-9.-]+/, "") : "";

  return (
    <div className={`bg-grove-surface border border-grove-border/50 ${borderClass} border-l-2 rounded-lg p-5 hover:shadow-lg hover:shadow-black/20 hover:-translate-y-0.5 transition-all duration-200`}>
      <div className="text-[10px] uppercase tracking-widest text-grove-text-secondary mb-2 font-sans">
        {label}
      </div>
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-mono font-semibold text-grove-text-primary">
          {isNumeric ? (
            <>
              {prefix}
              <AnimatedValue
                value={numericValue}
                format={(n) => Number.isInteger(numericValue) ? Math.round(n).toString() : n.toFixed(1)}
              />
              {suffix}
            </>
          ) : (
            value
          )}
        </span>
        {trend && trend in trendIcons && (
          <span className={`text-sm font-medium ${trendColors[trend]} animate-trend-pop`}>
            {trendIcons[trend]}
          </span>
        )}
      </div>
    </div>
  );
}
