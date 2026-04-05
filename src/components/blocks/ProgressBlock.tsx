import AnimatedValue from "../AnimatedValue";

interface ProgressBlockProps {
  label: string;
  value: number;
  max?: number;
  detail?: string;
}

export default function ProgressBlock({
  label,
  value,
  max = 100,
  detail,
}: ProgressBlockProps) {
  const pct = Math.min(100, Math.max(0, (value / max) * 100));

  return (
    <div className="bg-grove-surface border border-grove-border/50 rounded-lg p-5 space-y-3 hover:shadow-lg hover:shadow-black/20 transition-shadow duration-200">
      <div className="flex items-baseline justify-between">
        <span className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans">
          {label}
        </span>
        <span className="text-sm font-mono text-grove-text-primary">
          <AnimatedValue value={value} /><span className="text-grove-text-secondary">/{max}</span>
        </span>
      </div>
      <div className="h-1.5 bg-grove-border/50 rounded-full overflow-hidden">
        <div
          className="h-full rounded-full bg-gradient-to-r from-grove-accent to-grove-accent/70 animate-progress-fill"
          style={{ width: `${pct}%` }}
        />
      </div>
      {detail && (
        <p className="text-xs text-grove-text-secondary">{detail}</p>
      )}
    </div>
  );
}
