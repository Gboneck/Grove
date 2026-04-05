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

const bgTints: Record<string, string> = {
  alert: "bg-grove-status-red/[0.04]",
  opportunity: "bg-grove-status-green/[0.04]",
  warning: "bg-grove-status-yellow/[0.04]",
  idea: "bg-grove-accent/[0.04]",
};

const borderGradients: Record<string, string> = {
  alert: "from-grove-status-red to-grove-status-red/20",
  opportunity: "from-grove-status-green to-grove-status-green/20",
  warning: "from-grove-status-yellow to-grove-status-yellow/20",
  idea: "from-grove-accent to-[#c084fc]/40",
};

export default function InsightBlock({ icon, message }: InsightBlockProps) {
  const gradient = borderGradients[icon] || borderGradients.idea;
  const tint = bgTints[icon] || bgTints.idea;

  return (
    <div className={`relative rounded-lg overflow-hidden ${tint}`}>
      {/* Gradient left border */}
      <div className={`absolute left-0 top-0 bottom-0 w-0.5 bg-gradient-to-b ${gradient}`} />
      <div className="border border-grove-border/50 border-l-0 rounded-lg p-4 flex items-start gap-3 ml-0.5">
        <span className="text-lg mt-0.5 flex-shrink-0">{iconMap[icon] || "◆"}</span>
        <p className="text-grove-text-primary leading-relaxed text-sm">{message}</p>
      </div>
    </div>
  );
}
