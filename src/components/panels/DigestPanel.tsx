import { useEffect, useState } from "react";
import { getWeeklyDigest } from "../../lib/tauri";
import Modal from "../Modal";

interface DigestPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

interface Digest {
  week_start: string;
  week_end: string;
  session_count: number;
  active_days: string[];
  top_topics: string[];
  mood_trend: string;
  key_insights: string[];
  stuck_ventures: string[];
  momentum_ventures: string[];
  behavioral_patterns: string[];
  recommendation: string;
}

export default function DigestPanel({ isOpen, onClose }: DigestPanelProps) {
  const [digest, setDigest] = useState<Digest | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      getWeeklyDigest()
        .then((d) => setDigest(d as Digest))
        .catch(() => setDigest(null))
        .finally(() => setLoading(false));
    }
  }, [isOpen]);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Weekly Digest" maxWidth="max-w-lg">
      <div className="px-6 py-4 space-y-5">
        {loading && (
          <p className="text-grove-text-secondary text-sm animate-pulse">
            Generating digest...
          </p>
        )}

        {digest && !loading && (
          <>
            <div className="text-xs text-grove-text-secondary">
              {digest.week_start} — {digest.week_end}
            </div>

            <div className="grid grid-cols-3 gap-3">
              <div className="bg-grove-surface rounded-lg p-3 text-center">
                <div className="text-2xl text-grove-accent font-mono">
                  {digest.session_count}
                </div>
                <div className="text-xs text-grove-text-secondary">sessions</div>
              </div>
              <div className="bg-grove-surface rounded-lg p-3 text-center">
                <div className="text-2xl text-grove-accent font-mono">
                  {digest.active_days.length}
                </div>
                <div className="text-xs text-grove-text-secondary">active days</div>
              </div>
              <div className="bg-grove-surface rounded-lg p-3 text-center">
                <div className="text-2xl text-grove-accent font-mono">
                  {digest.top_topics.length}
                </div>
                <div className="text-xs text-grove-text-secondary">topics</div>
              </div>
            </div>

            <div>
              <h3 className="text-sm text-grove-text-primary font-medium mb-1">Mood</h3>
              <p className="text-sm text-grove-text-secondary">{digest.mood_trend}</p>
            </div>

            {digest.momentum_ventures.length > 0 && (
              <div>
                <h3 className="text-sm text-grove-text-primary font-medium mb-1">Momentum</h3>
                <div className="flex flex-wrap gap-2">
                  {digest.momentum_ventures.map((v) => (
                    <span key={v} className="text-xs bg-grove-status-green/20 text-grove-status-green px-2 py-1 rounded">
                      {v}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {digest.stuck_ventures.length > 0 && (
              <div>
                <h3 className="text-sm text-grove-text-primary font-medium mb-1">Stuck</h3>
                <div className="flex flex-wrap gap-2">
                  {digest.stuck_ventures.map((v) => (
                    <span key={v} className="text-xs bg-grove-status-red/20 text-grove-status-red px-2 py-1 rounded">
                      {v}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {digest.key_insights.length > 0 && (
              <div>
                <h3 className="text-sm text-grove-text-primary font-medium mb-1">Key Insights</h3>
                <ul className="space-y-1">
                  {digest.key_insights.map((insight, i) => (
                    <li key={i} className="text-sm text-grove-text-secondary pl-3 border-l-2 border-grove-accent/30">
                      {insight}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {digest.behavioral_patterns.length > 0 && (
              <div>
                <h3 className="text-sm text-grove-text-primary font-medium mb-1">Patterns</h3>
                <ul className="space-y-1">
                  {digest.behavioral_patterns.map((p, i) => (
                    <li key={i} className="text-sm text-grove-text-secondary">{p}</li>
                  ))}
                </ul>
              </div>
            )}

            <div className="bg-grove-surface rounded-lg p-4 border-l-2 border-grove-accent">
              <h3 className="text-sm text-grove-accent font-medium mb-1">Recommendation</h3>
              <p className="text-sm text-grove-text-primary">{digest.recommendation}</p>
            </div>
          </>
        )}

        {!digest && !loading && (
          <p className="text-grove-text-secondary text-sm">
            No digest data available yet. Use Grove for a few sessions first.
          </p>
        )}
      </div>
    </Modal>
  );
}
