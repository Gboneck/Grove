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
  alert: "border-l-[#f87171]",
  opportunity: "border-l-[#4ade80]",
  warning: "border-l-[#facc15]",
  idea: "border-l-[#d4a853]",
};

export default function InsightBlock({ icon, message }: InsightBlockProps) {
  return (
    <div
      className={`bg-[#141414] border border-[#222222] ${borderColors[icon] || borderColors.idea} border-l-2 rounded-lg p-4 flex items-start gap-3`}
    >
      <span className="text-lg mt-0.5">{iconMap[icon] || "◆"}</span>
      <p className="text-[#e5e5e5] leading-relaxed">{message}</p>
    </div>
  );
}
