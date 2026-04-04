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
  up: "text-[#4ade80]",
  down: "text-[#f87171]",
  flat: "text-[#888888]",
};

export default function MetricCard({ label, value, trend }: MetricCardProps) {
  return (
    <div className="bg-[#141414] border border-[#222222] rounded-lg p-5">
      <div className="text-xs uppercase tracking-wider text-[#888888] mb-2">{label}</div>
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-mono font-semibold text-[#e5e5e5]">{value}</span>
        {trend && trend in trendIcons && (
          <span className={`text-sm ${trendColors[trend]}`}>{trendIcons[trend]}</span>
        )}
      </div>
    </div>
  );
}
