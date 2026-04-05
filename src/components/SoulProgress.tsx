interface SoulProgressProps {
  completeness: number; // 0-1
  phase: string;
  sessionCount: number;
  factCount: number;
}

export default function SoulProgress({ completeness, phase, sessionCount, factCount }: SoulProgressProps) {
  const pct = Math.round(completeness * 100);
  const circumference = 2 * Math.PI * 28; // radius 28
  const filled = circumference * completeness;
  const gap = circumference - filled;

  return (
    <div className="flex flex-col items-center gap-3 py-2">
      {/* Radial progress */}
      <div className="relative w-16 h-16">
        <svg viewBox="0 0 64 64" className="w-full h-full -rotate-90">
          {/* Background track */}
          <circle
            cx="32" cy="32" r="28"
            fill="none"
            stroke="rgba(255,255,255,0.06)"
            strokeWidth="3"
          />
          {/* Progress arc */}
          <circle
            cx="32" cy="32" r="28"
            fill="none"
            stroke="url(#soulGradient)"
            strokeWidth="3"
            strokeLinecap="round"
            strokeDasharray={`${filled} ${gap}`}
            className="transition-all duration-1000"
          />
          <defs>
            <linearGradient id="soulGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#d4a853" />
              <stop offset="100%" stopColor="#c084fc" />
            </linearGradient>
          </defs>
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs font-mono text-grove-text-primary">{pct}%</span>
        </div>
      </div>

      {/* Phase label */}
      <span className="text-xs text-grove-accent font-sans tracking-wide">{phase}</span>

      {/* Stats */}
      <div className="flex gap-4 text-xs text-grove-text-secondary font-mono">
        <span>{sessionCount} sessions</span>
        <span>{factCount} facts</span>
      </div>
    </div>
  );
}
