import { useEffect, useState } from "react";
import { getReasoningLogs } from "../../lib/tauri";

interface LogsPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

interface LogEntry {
  timestamp: string;
  model_source: string;
  intent: string;
  confidence: number;
  escalated: boolean;
  escalation_reason: string | null;
  blocks_count: number;
  user_input: string | null;
  duration_ms: number;
}

export default function LogsPanel({ isOpen, onClose }: LogsPanelProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);

  useEffect(() => {
    if (isOpen) {
      getReasoningLogs().then((l) => setLogs((l as LogEntry[]).reverse()));
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-grove-bg border border-grove-border rounded-xl max-w-2xl w-full max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-grove-border">
          <h2 className="text-lg font-display text-grove-accent">
            Reasoning Logs
          </h2>
          <button
            onClick={onClose}
            className="text-grove-text-secondary hover:text-grove-text-primary transition-colors"
          >
            close
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-2">
          {logs.length === 0 ? (
            <p className="text-sm text-grove-text-secondary">
              No reasoning logs for today.
            </p>
          ) : (
            logs.map((log, i) => (
              <div
                key={i}
                className="bg-grove-surface border border-grove-border rounded-lg p-3 space-y-2"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span
                      className={`w-2 h-2 rounded-full ${
                        log.model_source === "local"
                          ? "bg-grove-model-local"
                          : "bg-grove-model-cloud"
                      }`}
                    />
                    <span className="text-xs font-mono text-grove-text-secondary">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-grove-border text-grove-text-secondary">
                      {log.intent}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono text-grove-text-secondary">
                      {log.duration_ms}ms
                    </span>
                    <span
                      className={`text-xs font-mono ${
                        log.confidence >= 0.8
                          ? "text-grove-status-green"
                          : log.confidence >= 0.6
                            ? "text-grove-status-yellow"
                            : "text-grove-status-red"
                      }`}
                    >
                      {(log.confidence * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
                {log.user_input && (
                  <p className="text-xs text-grove-text-secondary truncate">
                    Input: {log.user_input}
                  </p>
                )}
                <div className="flex items-center gap-3 text-xs text-grove-text-secondary">
                  <span>{log.blocks_count} blocks</span>
                  <span>via {log.model_source}</span>
                  {log.escalated && (
                    <span className="text-grove-status-yellow">
                      escalated
                      {log.escalation_reason && `: ${log.escalation_reason}`}
                    </span>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
