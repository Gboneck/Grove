import { useState, useEffect } from "react";
import Modal from "../Modal";
import { getEvolutionProposals, applyEvolution } from "../../lib/tauri";
import type { EvolutionProposal } from "../../lib/tauri";

interface EvolutionPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function EvolutionPanel({ isOpen, onClose }: EvolutionPanelProps) {
  const [proposals, setProposals] = useState<EvolutionProposal[]>([]);
  const [loading, setLoading] = useState(false);
  const [applying, setApplying] = useState<string | null>(null);
  const [applied, setApplied] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      setError(null);
      setApplied(new Set());
      getEvolutionProposals()
        .then(setProposals)
        .catch((e) => setError(String(e)))
        .finally(() => setLoading(false));
    }
  }, [isOpen]);

  const handleApply = async (proposal: EvolutionProposal) => {
    setApplying(proposal.id);
    try {
      await applyEvolution(proposal);
      setApplied((prev) => new Set(prev).add(proposal.id));
    } catch (e) {
      setError(String(e));
    } finally {
      setApplying(null);
    }
  };

  const sourceColors: Record<string, string> = {
    model_insight: "text-grove-accent",
    pattern_detection: "text-[#34d399]",
    confidence_decay: "text-grove-text-secondary",
    user_confirmation: "text-[#60a5fa]",
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Soul Evolution" maxWidth="max-w-lg">
      {loading ? (
        <div className="py-12 text-center text-sm text-grove-text-secondary animate-pulse">
          Analyzing soul for evolution proposals...
        </div>
      ) : error ? (
        <div className="py-8 text-center text-sm text-grove-status-red">{error}</div>
      ) : proposals.length === 0 ? (
        <div className="py-12 text-center space-y-2">
          <p className="text-sm text-grove-text-secondary">
            No evolution proposals right now.
          </p>
          <p className="text-xs text-gray-600">
            Proposals are generated from model insights, detected patterns, and confidence decay.
            Keep using Grove and they'll appear naturally.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          <p className="text-xs text-grove-text-secondary mb-4">
            {proposals.length} proposed change{proposals.length !== 1 ? "s" : ""} to Soul.md.
            Review and approve individually.
          </p>
          {proposals.map((p) => {
            const isApplied = applied.has(p.id);
            const isApplying = applying === p.id;
            return (
              <div
                key={p.id}
                className={`border rounded-lg p-4 transition-colors ${
                  isApplied
                    ? "border-[#34d399]/40 bg-[#34d399]/5"
                    : "border-grove-border bg-grove-surface"
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono text-grove-text-secondary">
                      {p.section}
                    </span>
                    <span
                      className={`text-[10px] ${sourceColors[p.source] || "text-grove-text-secondary"}`}
                    >
                      {p.source.replace("_", " ")}
                    </span>
                    {p.replace && (
                      <span className="text-[10px] text-grove-status-red">replace</span>
                    )}
                  </div>
                  <span className="text-[10px] text-grove-text-secondary font-mono">
                    {p.confidence_delta > 0 ? "+" : ""}
                    {(p.confidence_delta * 100).toFixed(0)}% confidence
                  </span>
                </div>

                {p.content && (
                  <p className="text-sm text-grove-text-primary mb-2 font-mono bg-grove-bg/50 rounded px-2 py-1">
                    {p.content}
                  </p>
                )}

                <p className="text-xs text-grove-text-secondary mb-3">{p.reason}</p>

                <div className="flex justify-end">
                  {isApplied ? (
                    <span className="text-xs text-[#34d399]">Applied</span>
                  ) : (
                    <button
                      onClick={() => handleApply(p)}
                      disabled={isApplying}
                      className="text-xs bg-grove-accent/20 text-grove-accent px-3 py-1.5 rounded hover:bg-grove-accent/30 transition-colors disabled:opacity-50"
                    >
                      {isApplying ? "Applying..." : "Approve & Apply"}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </Modal>
  );
}
