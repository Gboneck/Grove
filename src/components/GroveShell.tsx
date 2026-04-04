import { ReactNode, useEffect, useState } from "react";

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

  return (
    <div className="min-h-screen flex flex-col">
      {/* Top bar */}
      <header className="sticky top-0 z-10 bg-grove-bg/90 backdrop-blur-sm border-b border-grove-border">
        <div className="max-w-[640px] mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-grove-accent font-semibold tracking-wide">
              Grove
            </span>
            <span className="text-gray-600 text-sm">v0.1</span>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-grove-text-secondary font-mono">
              {timeStr}
            </span>
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
          </p>
        </footer>
      )}
    </div>
  );
}
