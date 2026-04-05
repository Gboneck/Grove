import { useState, useEffect } from "react";
import VentureCard from "./VentureCard";
import type { Venture } from "./VentureCard";
import SoulProgress from "./SoulProgress";
import AnimatedValue from "./AnimatedValue";
import { readContext, getMemoryStats } from "../lib/tauri";

interface SidebarProps {
  onFocusVenture: (name: string) => void;
  isCollapsed: boolean;
  onToggle: () => void;
}

interface SidebarData {
  ventures: Venture[];
  stats: { sessions: number; facts: number; patterns: number; insights: number };
  soulCompleteness: number;
  phase: string;
}

export default function Sidebar({ onFocusVenture, isCollapsed, onToggle }: SidebarProps) {
  const [data, setData] = useState<SidebarData | null>(null);

  useEffect(() => {
    loadSidebarData().then(setData).catch(() => {});

    // Refresh every 30s
    const interval = setInterval(() => {
      loadSidebarData().then(setData).catch(() => {});
    }, 30000);
    return () => clearInterval(interval);
  }, []);

  if (isCollapsed) {
    return (
      <button
        onClick={onToggle}
        className="fixed left-0 top-1/2 -translate-y-1/2 z-20 bg-grove-surface/80 backdrop-blur-sm border border-grove-border/50 rounded-r-lg px-1.5 py-4 text-grove-text-secondary hover:text-grove-accent transition-colors"
        title="Show sidebar"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M4 2l4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/></svg>
      </button>
    );
  }

  return (
    <aside className="w-56 flex-shrink-0 border-r border-grove-border/50 bg-grove-bg/50 overflow-y-auto">
      <div className="p-4 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <span className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans">
            Your World
          </span>
          <button
            onClick={onToggle}
            className="text-grove-text-secondary hover:text-grove-accent transition-colors"
            title="Hide sidebar"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M8 2l-4 4 4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/></svg>
          </button>
        </div>

        {/* Soul Progress */}
        {data && (
          <SoulProgress
            completeness={data.soulCompleteness}
            phase={data.phase}
            sessionCount={data.stats.sessions}
            factCount={data.stats.facts}
          />
        )}

        {/* Ventures */}
        {data && data.ventures.length > 0 && (
          <div className="space-y-2">
            <span className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans">
              Ventures
            </span>
            <div className="space-y-1.5">
              {data.ventures.map((v) => (
                <VentureCard key={v.name} venture={v} onFocus={onFocusVenture} />
              ))}
            </div>
          </div>
        )}

        {data && data.ventures.length === 0 && (
          <div className="text-xs text-grove-text-secondary/60 text-center py-4">
            <p>No ventures yet.</p>
            <p className="mt-1">Tell Grove what you're building.</p>
          </div>
        )}

        {/* Quick stats */}
        {data && (
          <div className="space-y-2">
            <span className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans">
              Memory
            </span>
            <div className="grid grid-cols-2 gap-2">
              <StatPill label="sessions" value={data.stats.sessions} />
              <StatPill label="facts" value={data.stats.facts} />
              <StatPill label="patterns" value={data.stats.patterns} />
              <StatPill label="insights" value={data.stats.insights} />
            </div>
          </div>
        )}
      </div>
    </aside>
  );
}

function StatPill({ label, value }: { label: string; value: number }) {
  return (
    <div className="bg-grove-surface/40 rounded px-2 py-1.5 text-center hover:bg-grove-surface/60 transition-colors">
      <AnimatedValue value={value} className="text-sm font-mono text-grove-text-primary" />
      <div className="text-[9px] uppercase tracking-wider text-grove-text-secondary">{label}</div>
    </div>
  );
}

// Derive relationship phase from session count (mirrors Rust logic)
function derivePhase(sessions: number, facts: number): { name: string; completeness: number } {
  const total = sessions + facts;
  if (total < 3) return { name: "Awakening", completeness: 0.05 };
  if (total < 8) return { name: "Discovery", completeness: 0.15 };
  if (total < 15) return { name: "Attunement", completeness: 0.3 };
  if (total < 25) return { name: "Synchrony", completeness: 0.45 };
  if (total < 40) return { name: "Resonance", completeness: 0.6 };
  if (total < 60) return { name: "Symbiosis", completeness: 0.75 };
  return { name: "Deep Trust", completeness: 0.9 };
}

async function loadSidebarData(): Promise<SidebarData> {
  const [context, stats] = await Promise.all([
    readContext().catch(() => ({ ventures: [] })),
    getMemoryStats().catch(() => ({ total_sessions: 0, facts_count: 0, patterns_count: 0, insights_count: 0 })),
  ]);

  const ctx = context as { ventures?: Venture[] };
  const memStats = stats as { total_sessions: number; facts_count: number; patterns_count: number; insights_count: number };
  const phase = derivePhase(memStats.total_sessions, memStats.facts_count);

  return {
    ventures: ctx.ventures || [],
    stats: {
      sessions: memStats.total_sessions,
      facts: memStats.facts_count,
      patterns: memStats.patterns_count,
      insights: memStats.insights_count,
    },
    soulCompleteness: phase.completeness,
    phase: phase.name,
  };
}
