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
    <div className="bg-grove-surface border border-grove-border rounded-lg p-5 space-y-3">
      <div className="flex items-baseline justify-between">
        <span className="text-xs uppercase tracking-wider text-grove-text-secondary">
          {label}
        </span>
        <span className="text-sm font-mono text-grove-text-primary">
          {value}/{max}
        </span>
      </div>
      <div className="h-2 bg-grove-border rounded-full overflow-hidden">
        <div
          className="h-full rounded-full bg-grove-accent transition-all duration-500"
          style={{ width: `${pct}%` }}
        />
      </div>
      {detail && (
        <p className="text-xs text-grove-text-secondary">{detail}</p>
      )}
    </div>
  );
}
