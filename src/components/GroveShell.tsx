import { ReactNode, useEffect, useState, useRef, useCallback } from "react";
import ModelIndicator from "./ModelIndicator";
import NavMenu from "./NavMenu";
import DaemonOrb, { type OrbState } from "./DaemonOrb";
import { RoleSwitcher } from "./RoleSwitcher";
import Sidebar from "./Sidebar";

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

// Time-of-day adjusts the background subtly
function timeOfDayTheme(hour: number): string {
  if (hour >= 5 && hour < 8) return "bg-gradient-to-b from-[#0a0a0a] to-[#0f0b08]"; // Early morning — warm amber
  if (hour >= 8 && hour < 12) return "bg-gradient-to-b from-[#0a0a0a] to-[#0c0a08]"; // Morning — light warm
  if (hour >= 12 && hour < 17) return "bg-gradient-to-b from-[#0a0a0a] to-[#0a0a0c]"; // Afternoon — neutral
  if (hour >= 17 && hour < 21) return "bg-gradient-to-b from-[#0a0a0a] to-[#0a080c]"; // Evening — cool purple
  return "bg-gradient-to-b from-[#0a0a0a] to-[#080808]"; // Night — deep dark
}

const ACCENT_OVERRIDES: Record<string, string> = {
  urgent: "text-grove-status-red",
  calm: "text-grove-accent",
  creative: "text-[#c084fc]",
  reflective: "text-[#60a5fa]",
  focused: "text-grove-accent",
};

// Ambient glow colors that bleed into header/footer borders
const MOOD_GLOW: Record<string, string> = {
  urgent: "rgba(248, 113, 113, 0.15)",
  calm: "rgba(212, 168, 83, 0.08)",
  creative: "rgba(192, 132, 252, 0.12)",
  reflective: "rgba(96, 165, 250, 0.1)",
  focused: "rgba(212, 168, 83, 0.1)",
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
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 60000);
    return () => clearInterval(interval);
  }, []);

  // Global Enter key — focus the input bar when nothing else is focused
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
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

  const handleFocusVenture = useCallback((name: string) => {
    onInput(`Focus on ${name} — what's the status and what should I do next?`);
  }, [onInput]);

  const timeStr = time.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });

  const dayStr = time.toLocaleDateString("en-US", { weekday: "long" });

  const themeClass = themeHint
    ? THEME_CLASSES[themeHint] || THEME_CLASSES.dark
    : timeOfDayTheme(time.getHours());
  const moodGlow = MOOD_GLOW[ambientMood || "focused"] || MOOD_GLOW.focused;
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
    <div className={`h-screen flex flex-col transition-colors duration-1000 ${themeClass}`}>
      {/* Grain overlay */}
      <div className="grain-overlay" />

      {/* Top bar */}
      <header
        className="sticky top-0 z-10 bg-grove-bg/80 backdrop-blur-xl border-b border-grove-border/30 transition-shadow duration-1000"
        style={{ boxShadow: `0 1px 20px ${moodGlow}, 0 1px 8px rgba(0,0,0,0.4)` }}
      >
        <div className="px-6 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <DaemonOrb state={orbState} size="md" />
            <span
              className={`font-semibold tracking-wide font-display text-lg transition-colors duration-1000 ${accentClass}`}
            >
              Grove
            </span>
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
          <div className="flex items-center gap-4">
            <RoleSwitcher />
            <ModelIndicator lastSource={modelSource} />
            <div className="text-sm text-grove-text-secondary font-mono">
              <span>{timeStr}</span>
              <span className="mx-1.5 opacity-40">·</span>
              <span className="opacity-60">{dayStr}</span>
            </div>
            <button
              onClick={onOpenSearch}
              className="text-xs text-grove-text-secondary hover:text-grove-accent transition-colors px-2 py-1 rounded border border-grove-border/50 hover:border-grove-accent/40"
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
              {isLoading ? "thinking..." : "refresh"}
            </button>
          </div>
        </div>
      </header>

      {/* Body: Sidebar + Main */}
      <div className="flex-1 flex overflow-hidden">
        <div className={`transition-opacity duration-500 ${isLoading ? "opacity-50" : "opacity-100"}`}>
          <Sidebar
            onFocusVenture={handleFocusVenture}
            isCollapsed={sidebarCollapsed}
            onToggle={() => setSidebarCollapsed(!sidebarCollapsed)}
          />
        </div>

        {/* Main content area */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <main className="flex-1 overflow-auto px-8 py-8 pb-28">
            {children}
          </main>

          {/* Sticky input bar */}
          <div
            className="border-t border-grove-border/30 bg-grove-bg/90 backdrop-blur-xl transition-shadow duration-1000 px-8 py-3"
            style={{ boxShadow: `0 -1px 20px ${moodGlow}, 0 -1px 8px rgba(0,0,0,0.4)` }}
          >
            <div className="max-w-[720px] mx-auto">
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
                  placeholder={isLoading ? "thinking..." : "talk to Grove or ask it to build something..."}
                  disabled={isLoading}
                  className={`flex-1 bg-grove-surface border rounded-lg px-4 py-2.5 text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none transition-all disabled:opacity-50 disabled:grayscale font-sans ${
                    inputValue.length > 0
                      ? "border-grove-accent/60 shadow-[0_0_16px_rgba(212,168,83,0.2)]"
                      : inputFocused
                        ? "border-grove-accent/40 shadow-[0_0_12px_rgba(212,168,83,0.1)]"
                        : "border-grove-border/50"
                  }`}
                />
                <button
                  onClick={handleInputSubmit}
                  disabled={isLoading || !inputValue.trim()}
                  className="bg-grove-accent text-grove-bg px-5 py-2.5 rounded-lg text-sm font-medium hover:brightness-110 transition-all disabled:opacity-40 disabled:hover:brightness-100 font-sans"
                >
                  send
                </button>
              </div>
              {/* Status line */}
              {lastUpdated && (
                <p className="text-xs text-gray-600 mt-1.5 font-mono">
                  {lastUpdated.toLocaleTimeString("en-US", {
                    hour: "numeric",
                    minute: "2-digit",
                    hour12: true,
                  })}
                  {modelSource && (
                    <span className="ml-2">
                      via {modelSource === "local" ? "gemma" : "claude"}
                    </span>
                  )}
                  {ambientMood && (
                    <span className="ml-2 opacity-60">· {ambientMood}</span>
                  )}
                </p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
