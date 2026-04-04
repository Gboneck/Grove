import { ReactNode, useEffect, useState, useRef, useCallback } from "react";
import ModelIndicator from "./ModelIndicator";
import NavMenu from "./NavMenu";
import DaemonOrb, { type OrbState } from "./DaemonOrb";

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
  onInput: (value: string) => void;
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
  onInput,
  hasUpdate,
  onAcknowledgeUpdate,
  isLoading,
  lastUpdated,
  modelSource,
  ambientMood,
  themeHint,
}: GroveShellProps) {
  const [time, setTime] = useState(new Date());
  const [inputValue, setInputValue] = useState("");
  const [inputFocused, setInputFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 60000);
    return () => clearInterval(interval);
  }, []);

  // Global Enter key — focus the input bar when nothing else is focused
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Don't capture if user is already in an input/textarea, or a panel/modal is open
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      // Don't capture if modifier keys are held (those are for shortcuts)
      if (e.metaKey || e.ctrlKey || e.altKey) return;

      if (e.key === "Enter") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleInputSubmit = useCallback(() => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    onInput(trimmed);
    setInputValue("");
    inputRef.current?.blur();
  }, [inputValue, onInput]);

  const timeStr = time.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });

  const themeClass = THEME_CLASSES[themeHint || "dark"] || THEME_CLASSES.dark;
  const accentClass =
    ACCENT_OVERRIDES[ambientMood || "focused"] || "text-grove-accent";

  // Derive orb state from app state
  const orbState: OrbState = isLoading
    ? "thinking"
    : inputFocused
      ? "listening"
      : ambientMood === "urgent"
        ? "alert"
        : ambientMood === "reflective"
          ? "reflecting"
          : modelSource === null
            ? "offline"
            : "idle";

  return (
    <div className={`min-h-screen flex flex-col transition-colors duration-1000 ${themeClass}`}>
      {/* Grain overlay */}
      <div className="grain-overlay" />

      {/* Top bar */}
      <header className="sticky top-0 z-10 bg-grove-bg/80 backdrop-blur-md border-b border-grove-border">
        <div className="max-w-[640px] mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <DaemonOrb state={orbState} size="sm" />
            <span
              className={`font-semibold tracking-wide font-display transition-colors duration-1000 ${accentClass}`}
            >
              Grove
            </span>
            <span className="text-gray-600 text-sm">v1.1</span>
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

      {/* Main content — pad bottom for sticky input bar */}
      <main className="flex-1 max-w-[640px] mx-auto w-full px-6 py-8 pb-28">
        {children}
      </main>

      {/* Sticky input bar */}
      <div className="fixed bottom-0 left-0 right-0 z-10 bg-grove-bg/90 backdrop-blur-md border-t border-grove-border">
        <div className="max-w-[640px] mx-auto px-6 py-3">
          <div className={`flex gap-2 transition-all duration-200 ${inputFocused ? "opacity-100" : "opacity-60 hover:opacity-80"}`}>
            <input
              ref={inputRef}
              type="text"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onFocus={() => setInputFocused(true)}
              onBlur={() => setInputFocused(false)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && inputValue.trim()) {
                  handleInputSubmit();
                } else if (e.key === "Escape") {
                  setInputValue("");
                  inputRef.current?.blur();
                }
              }}
              placeholder={isLoading ? "thinking..." : "press Enter to talk to Grove..."}
              disabled={isLoading}
              className="flex-1 bg-grove-surface border border-grove-border rounded-lg px-4 py-2.5 text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors disabled:opacity-50"
            />
            <button
              onClick={handleInputSubmit}
              disabled={isLoading || !inputValue.trim()}
              className="bg-grove-accent text-grove-bg px-4 py-2.5 rounded-lg text-sm font-medium hover:brightness-110 transition-all disabled:opacity-40 disabled:hover:brightness-100"
            >
              send
            </button>
          </div>
          {/* Status line */}
          {lastUpdated && (
            <p className="text-xs text-gray-600 mt-2">
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
          )}
        </div>
      </div>
    </div>
  );
}
