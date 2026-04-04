import { useEffect, useState } from "react";
import { getFullMemory, getMemoryStats } from "../../lib/tauri";
import Modal from "../Modal";

interface MemoryPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

interface MemoryData {
  sessions: Array<{
    id: string;
    timestamp: string;
    time_of_day: string;
    session_summary: string;
    blocks_shown: string[];
    user_inputs: Array<{ text: string }>;
  }>;
  facts: Array<{
    id: string;
    category: string;
    content: string;
    confidence: number;
    superseded_by: string | null;
  }>;
  patterns: Array<{
    id: string;
    pattern_type: string;
    description: string;
    evidence_count: number;
    effectiveness: number;
  }>;
  accumulated_insights: string[];
}

type Tab = "sessions" | "facts" | "patterns" | "stats";

export default function MemoryPanel({ isOpen, onClose }: MemoryPanelProps) {
  const [tab, setTab] = useState<Tab>("sessions");
  const [memory, setMemory] = useState<MemoryData | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [stats, setStats] = useState<any>(null);

  useEffect(() => {
    if (isOpen) {
      getFullMemory().then((m) => setMemory(m as MemoryData));
      getMemoryStats().then((s) => setStats(s));
    }
  }, [isOpen]);

  const tabs: { id: Tab; label: string }[] = [
    { id: "sessions", label: "Sessions" },
    { id: "facts", label: "Facts" },
    { id: "patterns", label: "Patterns" },
    { id: "stats", label: "Stats" },
  ];

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Memory">
      {/* Tabs */}
      <div className="flex gap-1 px-6 pt-3">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`px-3 py-1.5 text-xs rounded-md transition-colors ${
                tab === t.id
                  ? "bg-grove-accent/20 text-grove-accent"
                  : "text-grove-text-secondary hover:text-grove-text-primary"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-3">
          {!memory ? (
            <p className="text-grove-text-secondary text-sm">Loading...</p>
          ) : (
            <>
              {tab === "sessions" && (
                <div className="space-y-3">
                  {memory.sessions
                    .slice()
                    .reverse()
                    .slice(0, 20)
                    .map((s) => (
                      <div
                        key={s.id}
                        className="bg-grove-surface border border-grove-border rounded-lg p-3 space-y-1"
                      >
                        <div className="flex items-center justify-between">
                          <span className="text-xs text-grove-text-secondary font-mono">
                            {new Date(s.timestamp).toLocaleString()}
                          </span>
                          <span className="text-xs text-grove-accent">
                            {s.time_of_day}
                          </span>
                        </div>
                        <p className="text-sm text-grove-text-primary">
                          {s.session_summary}
                        </p>
                        {s.user_inputs.length > 0 && (
                          <p className="text-xs text-grove-text-secondary">
                            User: {s.user_inputs.map((i) => i.text).join(", ")}
                          </p>
                        )}
                        <div className="flex gap-1 flex-wrap">
                          {s.blocks_shown.slice(0, 5).map((b, i) => (
                            <span
                              key={i}
                              className="text-[10px] px-1.5 py-0.5 rounded bg-grove-border text-grove-text-secondary"
                            >
                              {b}
                            </span>
                          ))}
                        </div>
                      </div>
                    ))}
                  {memory.sessions.length === 0 && (
                    <p className="text-sm text-grove-text-secondary">
                      No sessions yet.
                    </p>
                  )}
                </div>
              )}

              {tab === "facts" && (
                <div className="space-y-2">
                  {memory.facts
                    .filter((f) => !f.superseded_by)
                    .sort((a, b) => b.confidence - a.confidence)
                    .map((f) => (
                      <div
                        key={f.id}
                        className="bg-grove-surface border border-grove-border rounded-lg p-3 flex items-start gap-3"
                      >
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-grove-accent/20 text-grove-accent shrink-0 mt-0.5">
                          {f.category}
                        </span>
                        <div className="flex-1 min-w-0">
                          <p className="text-sm text-grove-text-primary">
                            {f.content}
                          </p>
                        </div>
                        <span className="text-xs text-grove-text-secondary font-mono shrink-0">
                          {(f.confidence * 100).toFixed(0)}%
                        </span>
                      </div>
                    ))}
                  {memory.facts.length === 0 && (
                    <p className="text-sm text-grove-text-secondary">
                      No facts learned yet. Grove will learn about you over time.
                    </p>
                  )}
                </div>
              )}

              {tab === "patterns" && (
                <div className="space-y-2">
                  {memory.patterns.map((p) => (
                    <div
                      key={p.id}
                      className="bg-grove-surface border border-grove-border rounded-lg p-3"
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-grove-border text-grove-text-secondary">
                          {p.pattern_type}
                        </span>
                        <span className="text-xs text-grove-text-secondary font-mono">
                          {p.evidence_count} observations
                        </span>
                      </div>
                      <p className="text-sm text-grove-text-primary">
                        {p.description}
                      </p>
                    </div>
                  ))}
                  {memory.patterns.length === 0 && (
                    <p className="text-sm text-grove-text-secondary">
                      No patterns detected yet. Use Grove more for pattern learning.
                    </p>
                  )}
                </div>
              )}

              {tab === "stats" && stats && (
                <div className="grid grid-cols-2 gap-3">
                  {[
                    { label: "Total Sessions", value: stats.total_sessions },
                    { label: "Actions Clicked", value: stats.total_actions_clicked },
                    { label: "Facts Learned", value: stats.facts_count },
                    { label: "Patterns Detected", value: stats.patterns_count },
                    { label: "Insights Accumulated", value: stats.insights_count },
                  ].map((item) => (
                    <div
                      key={item.label}
                      className="bg-grove-surface border border-grove-border rounded-lg p-4"
                    >
                      <div className="text-xs text-grove-text-secondary uppercase tracking-wider mb-1">
                        {item.label}
                      </div>
                      <div className="text-xl font-mono text-grove-text-primary">
                        {String(item.value ?? 0)}
                      </div>
                    </div>
                  ))}
                  {stats.preferred_times && (
                    <div className="col-span-2 bg-grove-surface border border-grove-border rounded-lg p-4">
                      <div className="text-xs text-grove-text-secondary uppercase tracking-wider mb-2">
                        Preferred Session Times
                      </div>
                      <div className="flex gap-2 flex-wrap">
                        {(stats.preferred_times as string[]).map((t) => (
                          <span
                            key={t}
                            className="text-sm px-2 py-1 rounded bg-grove-accent/10 text-grove-accent"
                          >
                            {t}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </>
          )}
        </div>
    </Modal>
  );
}
