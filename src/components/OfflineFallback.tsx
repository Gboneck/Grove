import { useState, useEffect } from "react";
import { getMemoryStats, vectorSearch, MemoryStats } from "../lib/tauri";

/**
 * Shown when both models are unavailable. Displays cached data and
 * allows offline semantic search instead of a blank error state.
 */
export default function OfflineFallback() {
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<
    { content: string; category: string; score: number }[]
  >([]);

  useEffect(() => {
    getMemoryStats()
      .then(setStats)
      .catch(() => {});
  }, []);

  const handleSearch = async () => {
    if (searchQuery.length < 2) return;
    try {
      const results = await vectorSearch(searchQuery, 5);
      setSearchResults(results);
    } catch {
      setSearchResults([]);
    }
  };

  return (
    <div className="space-y-6 py-4">
      <div className="rounded-lg border border-grove-border bg-grove-surface p-6 text-center space-y-3">
        <div className="w-3 h-3 rounded-full bg-grove-model-offline mx-auto" />
        <h3 className="text-sm font-display text-grove-text-primary">
          Both models are offline
        </h3>
        <p className="text-xs text-grove-text-secondary max-w-sm mx-auto">
          Ollama isn't running and no API key is set. Grove can still search
          your memory and show cached patterns while you reconnect.
        </p>
      </div>

      {stats && (
        <div className="flex gap-3">
          <div className="flex-1 rounded-lg border border-grove-border bg-grove-surface p-3 text-center">
            <p className="text-lg font-mono text-grove-accent">{stats.total_sessions}</p>
            <p className="text-[10px] text-grove-text-secondary">sessions</p>
          </div>
          <div className="flex-1 rounded-lg border border-grove-border bg-grove-surface p-3 text-center">
            <p className="text-lg font-mono text-grove-accent">{stats.facts_count}</p>
            <p className="text-[10px] text-grove-text-secondary">facts</p>
          </div>
          <div className="flex-1 rounded-lg border border-grove-border bg-grove-surface p-3 text-center">
            <p className="text-lg font-mono text-grove-accent">{stats.patterns_count}</p>
            <p className="text-[10px] text-grove-text-secondary">patterns</p>
          </div>
        </div>
      )}

      <div className="space-y-2">
        <label className="text-xs text-grove-text-secondary">
          Search memory offline
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder="What do you remember about..."
            className="flex-1 bg-grove-surface border border-grove-border rounded-lg px-4 py-2.5 text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors"
          />
          <button
            onClick={handleSearch}
            className="bg-grove-accent/20 text-grove-accent px-4 py-2.5 rounded-lg text-sm hover:bg-grove-accent/30 transition-colors"
          >
            search
          </button>
        </div>
      </div>

      {searchResults.length > 0 && (
        <div className="space-y-2">
          {searchResults.map((r, i) => (
            <div
              key={i}
              className="rounded-lg border border-grove-border bg-grove-surface p-3"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-grove-accent/20 text-grove-accent">
                  {r.category}
                </span>
                <span className="text-[10px] text-grove-text-secondary">
                  {(r.score * 100).toFixed(0)}% match
                </span>
              </div>
              <p className="text-sm text-grove-text-primary">{r.content}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
