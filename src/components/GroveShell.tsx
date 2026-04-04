import { ReactNode, useEffect, useState } from "react";
import ModelIndicator from "./ModelIndicator";
import NavMenu from "./NavMenu";

interface GroveShellProps {
  children: ReactNode;
  onRefresh: () => void;
  onOpenSoul: () => void;
  onOpenMemory: () => void;
  onOpenLogs: () => void;
  onOpenPlugins: () => void;
  onOpenProfiles: () => void;
  onOpenContext: () => void;
  onOpenSearch: () => void;
  hasUpdate?: boolean;
  onAcknowledgeUpdate?: () => void;
  isLoading: boolean;
  lastUpdated: Date | null;
  modelSource: "local" | "cloud" | null;
  ambientMood: string | null;
  themeHint: string | null;
}

const THEME_CLASSES: Record<string, string> = {
  warm: "bg-gradient-to-b from-[#0a0a0a] to-[#0f0a05]",
  cool: "bg-gradient-to-b from-[#0a0a0a] to-[#050a0f]",
  dark: "bg-[#0a0a0a]",
  light: "bg-gradient-to-b from-[#0a0a0a] to-[#0f0f0f]",
};

const ACCENT_OVERRIDES: Record<string, string> = {
  urgent: "text-grove-status-red",
  calm: "text-grove-accent",
  creative: "text-[#c084fc]",
  reflective: "text-[#60a5fa]",
  focused: "text-grove-accent",
};

export default function GroveShell({
  children,
  onRefresh,
  onOpenSoul,
  onOpenMemory,
  onOpenLogs,
  onOpenPlugins,
  onOpenProfiles,
  onOpenContext,
  onOpenSearch,
  hasUpdate,
  onAcknowledgeUpdate,
  isLoading,
  lastUpdated,
  modelSource,
  ambientMood,
  themeHint,
}: GroveShellProps) {
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 60000);
    return () => clearInterval(interval);
  }, []);

  const timeStr = time.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });

  const themeClass = THEME_CLASSES[themeHint || "dark"] || THEME_CLASSES.dark;
  const accentClass =
    ACCENT_OVERRIDES[ambientMood || "focused"] || "text-grove-accent";

  return (
    <div className={`min-h-screen flex flex-col transition-colors duration-1000 ${themeClass}`}>
      {/* Top bar */}
      <header className="sticky top-0 z-10 bg-grove-bg/80 backdrop-blur-md border-b border-grove-border">
        <div className="max-w-[640px] mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span
              className={`font-semibold tracking-wide font-display transition-colors duration-1000 ${accentClass}`}
            >
              Grove
            </span>
            <span className="text-gray-600 text-sm">v0.7</span>
            {hasUpdate && (
              <button
                onClick={() => onAcknowledgeUpdate?.()}
                className="relative flex items-center gap-1 text-xs text-grove-accent animate-pulse"
                title="New periodic update arrived"
              >
                <span className="w-2 h-2 rounded-full bg-grove-accent" />
                new update
              </button>
            )}
          </div>
          <div className="flex items-center gap-3">
            <ModelIndicator lastSource={modelSource} />
            <span className="text-sm text-grove-text-secondary font-mono">
              {timeStr}
            </span>
            <button
              onClick={onOpenSearch}
              className="text-xs text-grove-text-secondary hover:text-grove-accent transition-colors px-2 py-1 rounded border border-grove-border hover:border-grove-accent/40"
              title="Search (Ctrl+/)"
            >
              search
            </button>
            <NavMenu
              items={[
                { label: "soul", action: onOpenSoul },
                { label: "context", action: onOpenContext },
                { label: "memory", action: onOpenMemory },
                { label: "logs", action: onOpenLogs },
                { label: "plugins", action: onOpenPlugins },
                { label: "profiles", action: onOpenProfiles },
              ]}
            />
            <button
              onClick={onRefresh}
              disabled={isLoading}
              className="text-sm text-grove-text-secondary hover:text-grove-accent transition-colors disabled:opacity-50"
            >
              {isLoading ? "thinking…" : "refresh"}
            </button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 max-w-[640px] mx-auto w-full px-6 py-8">
        {children}
      </main>

      {/* Footer */}
      {lastUpdated && (
        <footer className="max-w-[640px] mx-auto w-full px-6 py-4">
          <p className="text-xs text-gray-600">
            last reasoned:{" "}
            {lastUpdated.toLocaleTimeString("en-US", {
              hour: "numeric",
              minute: "2-digit",
              second: "2-digit",
              hour12: true,
            })}
            {modelSource && (
              <span className="ml-2">
                via {modelSource === "local" ? "gemma" : "claude"}
              </span>
            )}
            {ambientMood && (
              <span className="ml-2">· {ambientMood}</span>
            )}
          </p>
        </footer>
      )}
    </div>
  );
}
