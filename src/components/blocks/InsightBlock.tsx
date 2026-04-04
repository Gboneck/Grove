interface InsightBlockProps {
  icon: "alert" | "opportunity" | "warning" | "idea";
  message: string;
}

const iconMap: Record<string, string> = {
  alert: "⚡",
  opportunity: "✦",
  warning: "⚠",
  idea: "◆",
};

const borderColors: Record<string, string> = {
  alert: "border-l-grove-status-red",
  opportunity: "border-l-grove-status-green",
  warning: "border-l-grove-status-yellow",
  idea: "border-l-grove-accent",
};

export default function InsightBlock({ icon, message }: InsightBlockProps) {
  return (
    <div
      className={`bg-grove-surface border border-grove-border ${borderColors[icon] || borderColors.idea} border-l-2 rounded-lg p-4 flex items-start gap-3`}
    >
      <span className="text-lg mt-0.5">{iconMap[icon] || "◆"}</span>
      <p className="text-grove-text-primary leading-relaxed">{message}</p>
    </div>
  );
}
