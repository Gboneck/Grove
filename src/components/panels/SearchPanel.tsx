import { useState, useRef, useEffect } from "react";
import { getFullMemory, getReasoningLogs } from "../../lib/tauri";

interface SearchResult {
  type: "session" | "fact" | "insight" | "log";
  title: string;
  detail: string;
  timestamp?: string;
  meta?: string;
}

interface SearchPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SearchPanel({ isOpen, onClose }: SearchPanelProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setResults([]);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const search = async (q: string) => {
    if (q.length < 2) {
      setResults([]);
      return;
    }

    setSearching(true);
    const lower = q.toLowerCase();
    const found: SearchResult[] = [];

    try {
      // Search memory
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const memory = (await getFullMemory()) as any;

      // Search sessions
      if (memory.sessions) {
        for (const s of memory.sessions) {
          if (
            s.session_summary?.toLowerCase().includes(lower) ||
            s.user_inputs?.some((i: { text: string }) =>
              i.text.toLowerCase().includes(lower)
            )
          ) {
            found.push({
              type: "session",
              title: s.session_summary || "Session",
              detail:
                s.user_inputs
                  ?.map((i: { text: string }) => i.text)
                  .join(", ") || "",
              timestamp: s.timestamp,
              meta: s.time_of_day,
            });
          }
        }
      }

      // Search facts
      if (memory.facts) {
        for (const f of memory.facts) {
          if (f.content?.toLowerCase().includes(lower)) {
            found.push({
              type: "fact",
              title: f.content,
              detail: f.category,
              meta: `${(f.confidence * 100).toFixed(0)}% confidence`,
            });
          }
        }
      }

      // Search insights
      if (memory.accumulated_insights) {
        for (const insight of memory.accumulated_insights) {
          if (insight.toLowerCase().includes(lower)) {
            found.push({
              type: "insight",
              title: insight,
              detail: "Accumulated insight",
            });
          }
        }
      }

      // Search today's logs
      const logs = (await getReasoningLogs()) as Array<{
        timestamp: string;
        user_input: string | null;
        intent: string;
        model_source: string;
      }>;
      for (const log of logs) {
        if (log.user_input?.toLowerCase().includes(lower)) {
          found.push({
            type: "log",
            title: log.user_input || "",
            detail: `${log.intent} via ${log.model_source}`,
            timestamp: log.timestamp,
          });
        }
      }
    } catch (e) {
      console.error("Search error:", e);
    }

    setResults(found);
    setSearching(false);
  };

  const handleInput = (value: string) => {
    setQuery(value);
    // Debounce
    const timer = setTimeout(() => search(value), 300);
    return () => clearTimeout(timer);
  };

  if (!isOpen) return null;

  const typeColors: Record<string, string> = {
    session: "bg-grove-model-local/20 text-grove-model-local",
    fact: "bg-grove-accent/20 text-grove-accent",
    insight: "bg-[#c084fc]/20 text-[#c084fc]",
    log: "bg-grove-model-cloud/20 text-grove-model-cloud",
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[10vh]">
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={onClose}
      />
      <div className="relative bg-grove-bg border border-grove-border rounded-xl w-full max-w-lg shadow-2xl overflow-hidden">
        {/* Search input */}
        <div className="px-4 py-3 border-b border-grove-border flex items-center gap-3">
          <span className="text-grove-text-secondary text-sm">Search</span>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => handleInput(e.target.value)}
            onKeyDown={(e) => e.key === "Escape" && onClose()}
            placeholder="Search memory, facts, logs..."
            className="flex-1 bg-transparent text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none"
          />
          {searching && (
            <span className="text-[10px] text-grove-text-secondary">
              searching...
            </span>
          )}
        </div>

        {/* Results */}
        <div className="max-h-[400px] overflow-y-auto">
          {results.length === 0 && query.length >= 2 && !searching ? (
            <div className="px-4 py-8 text-center text-xs text-grove-text-secondary">
              No results found
            </div>
          ) : results.length === 0 ? (
            <div className="px-4 py-8 text-center text-xs text-grove-text-secondary">
              Type at least 2 characters to search
            </div>
          ) : (
            <div className="py-1">
              {results.map((r, i) => (
                <div
                  key={i}
                  className="px-4 py-3 hover:bg-grove-surface transition-colors border-b border-grove-border/50 last:border-0"
                >
                  <div className="flex items-center gap-2 mb-1">
                    <span
                      className={`text-[10px] px-1.5 py-0.5 rounded ${typeColors[r.type] || ""}`}
                    >
                      {r.type}
                    </span>
                    {r.timestamp && (
                      <span className="text-[10px] text-grove-text-secondary font-mono">
                        {new Date(r.timestamp).toLocaleDateString()}
                      </span>
                    )}
                    {r.meta && (
                      <span className="text-[10px] text-grove-text-secondary">
                        {r.meta}
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-grove-text-primary truncate">
                    {r.title}
                  </p>
                  {r.detail && (
                    <p className="text-xs text-grove-text-secondary truncate mt-0.5">
                      {r.detail}
                    </p>
                  )}
                </div>
              ))}
              <div className="px-4 py-2 text-[10px] text-grove-text-secondary">
                {results.length} result{results.length !== 1 ? "s" : ""}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
