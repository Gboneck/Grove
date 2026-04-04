"use client";

import { ReactNode } from "react";

interface GroveShellProps {
  children: ReactNode;
  onRefresh: () => void;
  isLoading: boolean;
  lastUpdated: Date | null;
}

export default function GroveShell({
  children,
  onRefresh,
  isLoading,
  lastUpdated,
}: GroveShellProps) {
  const now = new Date();
  const timeStr = now.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });

  return (
    <div className="min-h-screen flex flex-col">
      {/* Top bar */}
      <header className="sticky top-0 z-10 bg-[#0a0a0a]/90 backdrop-blur-sm border-b border-[#222222]">
        <div className="max-w-2xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-[#d4a853] font-semibold tracking-wide">Grove OS</span>
            <span className="text-[#555555] text-sm">v0.0.1</span>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-[#888888] font-mono">{timeStr}</span>
            <button
              onClick={onRefresh}
              disabled={isLoading}
              className="text-sm text-[#888888] hover:text-[#d4a853] transition-colors disabled:opacity-50"
            >
              {isLoading ? "thinking…" : "refresh"}
            </button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 max-w-2xl mx-auto w-full px-6 py-8">
        {children}
      </main>

      {/* Footer */}
      {lastUpdated && (
        <footer className="max-w-2xl mx-auto w-full px-6 py-4">
          <p className="text-xs text-[#555555]">
            last updated{" "}
            {lastUpdated.toLocaleTimeString("en-US", {
              hour: "numeric",
              minute: "2-digit",
              second: "2-digit",
              hour12: true,
            })}
          </p>
        </footer>
      )}
    </div>
  );
}
