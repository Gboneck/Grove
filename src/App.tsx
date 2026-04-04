import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import GroveShell from "./components/GroveShell";
import BlockRenderer from "./components/BlockRenderer";
import LoadingState from "./components/LoadingState";
import SetupScreen from "./components/SetupScreen";
import SoulEditor from "./components/SoulEditor";
import MemoryPanel from "./components/panels/MemoryPanel";
import LogsPanel from "./components/panels/LogsPanel";
import PluginPanel from "./components/panels/PluginPanel";
import ProfilePanel from "./components/panels/ProfilePanel";
import ContextEditor from "./components/panels/ContextEditor";
import SearchPanel from "./components/panels/SearchPanel";
import CommandPalette from "./components/CommandPalette";
import {
  reason as invokeReason,
  checkSetup,
  getFileStamps,
  clearConversation,
  Block,
  SetupStatus,
  FileStamps,
} from "./lib/tauri";
import { invoke } from "@tauri-apps/api/core";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";

const FALLBACK_BLOCKS: Block[] = [
  {
    type: "text",
    heading: "Grove",
    body: "The reasoning engine is warming up or unavailable. Hit refresh to try again.",
  },
  {
    type: "insight",
    icon: "warning",
    message:
      "Make sure Ollama is running (for local reasoning) or ANTHROPIC_API_KEY is set in ~/.grove/.env (for cloud reasoning).",
  },
];

type AppPhase = "checking" | "setup" | "running";

export default function App() {
  const [phase, setPhase] = useState<AppPhase>("checking");
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null);
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [error, setError] = useState(false);
  const [modelSource, setModelSource] = useState<"local" | "cloud" | null>(
    null
  );
  const [ambientMood, setAmbientMood] = useState<string | null>(null);
  const [themeHint, setThemeHint] = useState<string | null>(null);
  const [soulEditorOpen, setSoulEditorOpen] = useState(false);
  const [memoryOpen, setMemoryOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  const [pluginsOpen, setPluginsOpen] = useState(false);
  const [profilesOpen, setProfilesOpen] = useState(false);
  const [contextOpen, setContextOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [hasPeriodicUpdate, setHasPeriodicUpdate] = useState(false);
  const lastStampsRef = useRef<FileStamps | null>(null);

  // Check setup on mount
  useEffect(() => {
    checkSetup()
      .then((status) => {
        setSetupStatus(status);
        if (status.needs_setup) {
          setPhase("setup");
        } else {
          setPhase("running");
        }
      })
      .catch(() => {
        // Can't check setup — just try to run
        setPhase("running");
      });

    // Request notification permission on startup
    isPermissionGranted().then((granted) => {
      if (!granted) {
        requestPermission().catch(() => {});
      }
    }).catch(() => {});
  }, []);

  const reason = useCallback(async (userInput?: string) => {
    setIsLoading(true);
    setError(false);
    try {
      const response = await invokeReason(userInput);
      if (response.blocks && Array.isArray(response.blocks)) {
        setBlocks(response.blocks);
        setLastUpdated(new Date());
        setModelSource(response.model_source);
        setAmbientMood(response.ambient_mood);
        setThemeHint(response.theme_hint);
      } else {
        throw new Error("Invalid response structure");
      }
    } catch (err) {
      console.error("Reasoning failed:", err);
      setError(true);
      setBlocks(FALLBACK_BLOCKS);
      setLastUpdated(new Date());
      setModelSource(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Auto-reason when entering running phase
  useEffect(() => {
    if (phase === "running") {
      reason();
    }
  }, [phase, reason]);

  // Listen for periodic reasoning events from the background timer
  useEffect(() => {
    const unlisten = listen<{
      blocks: Block[];
      timestamp: string;
      model_source: string;
      ambient_mood: string | null;
      theme_hint: string | null;
      has_urgent: boolean;
    }>("periodic-reasoning", (event) => {
      const data = event.payload;
      if (data.blocks && Array.isArray(data.blocks)) {
        setBlocks(data.blocks);
        setLastUpdated(new Date());
        setModelSource(data.model_source as "local" | "cloud");
        setAmbientMood(data.ambient_mood);
        setThemeHint(data.theme_hint);
        setHasPeriodicUpdate(true);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Global keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      if (meta && e.key === "k") {
        e.preventDefault();
        setPaletteOpen((v) => !v);
      } else if (meta && e.key === "/") {
        e.preventDefault();
        setSearchOpen((v) => !v);
      } else if (e.key === "Escape") {
        // Close any open panel
        setPaletteOpen(false);
        setSearchOpen(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  // Command palette commands
  const paletteCommands = [
    { id: "reason", label: "Refresh / Re-reason", shortcut: "R", action: () => { clearConversation().catch(() => {}); reason(); } },
    { id: "search", label: "Search memory & logs", shortcut: "Ctrl+/", action: () => setSearchOpen(true) },
    { id: "soul", label: "Edit Soul.md", action: () => setSoulEditorOpen(true) },
    { id: "context", label: "Edit Context / Ventures", action: () => setContextOpen(true) },
    { id: "memory", label: "View Memory", action: () => setMemoryOpen(true) },
    { id: "logs", label: "View Reasoning Logs", action: () => setLogsOpen(true) },
    { id: "plugins", label: "Manage Plugins", action: () => setPluginsOpen(true) },
    { id: "profiles", label: "Switch Profile", action: () => setProfilesOpen(true) },
  ];

  // Poll ~/.grove/ files for external changes every 10s
  useEffect(() => {
    if (phase !== "running") return;

    const pollInterval = setInterval(async () => {
      try {
        const stamps = await getFileStamps();
        const prev = lastStampsRef.current;
        lastStampsRef.current = stamps;

        if (!prev) return; // First poll, just save baseline

        // Check if any file changed
        for (const [file, mtime] of Object.entries(stamps.files)) {
          const prevTime = prev.files[file];
          if (prevTime !== undefined && prevTime !== mtime) {
            console.log(`[grove] ${file} changed externally, re-reasoning`);
            // Notify backend for on_file_change hooks
            invoke("notify_file_change").catch(() => {});
            reason();
            return;
          }
        }
      } catch {
        // Ignore polling errors
      }
    }, 10000);

    return () => clearInterval(pollInterval);
  }, [phase, reason]);

  const handleInput = (value: string) => {
    reason(value);
  };

  const handleSetupComplete = () => {
    setPhase("running");
  };

  // Phase: checking setup
  if (phase === "checking") {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingState />
      </div>
    );
  }

  // Phase: setup needed
  if (phase === "setup" && setupStatus) {
    return (
      <SetupScreen status={setupStatus} onComplete={handleSetupComplete} />
    );
  }

  // Phase: running
  return (
    <>
      <GroveShell
        onRefresh={() => {
          clearConversation().catch(() => {});
          reason();
        }}
        onOpenSoul={() => setSoulEditorOpen(true)}
        onOpenMemory={() => setMemoryOpen(true)}
        onOpenLogs={() => setLogsOpen(true)}
        onOpenPlugins={() => setPluginsOpen(true)}
        onOpenProfiles={() => setProfilesOpen(true)}
        onOpenContext={() => setContextOpen(true)}
        onOpenSearch={() => setSearchOpen(true)}
        hasUpdate={hasPeriodicUpdate}
        onAcknowledgeUpdate={() => setHasPeriodicUpdate(false)}
        isLoading={isLoading}
        lastUpdated={lastUpdated}
        modelSource={modelSource}
        ambientMood={ambientMood}
        themeHint={themeHint}
      >
        {isLoading ? (
          <LoadingState />
        ) : (
          <div className={error ? "opacity-70" : ""}>
            <BlockRenderer blocks={blocks} onInput={handleInput} />
          </div>
        )}
      </GroveShell>

      <SoulEditor
        isOpen={soulEditorOpen}
        onClose={() => setSoulEditorOpen(false)}
      />
      <MemoryPanel
        isOpen={memoryOpen}
        onClose={() => setMemoryOpen(false)}
      />
      <LogsPanel
        isOpen={logsOpen}
        onClose={() => setLogsOpen(false)}
      />
      <PluginPanel
        isOpen={pluginsOpen}
        onClose={() => setPluginsOpen(false)}
      />
      <ProfilePanel
        isOpen={profilesOpen}
        onClose={() => setProfilesOpen(false)}
        onSwitch={() => {
          clearConversation().catch(() => {});
          reason();
        }}
      />
      <ContextEditor
        isOpen={contextOpen}
        onClose={() => setContextOpen(false)}
      />
      <SearchPanel
        isOpen={searchOpen}
        onClose={() => setSearchOpen(false)}
      />
      <CommandPalette
        isOpen={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        commands={paletteCommands}
      />
    </>
  );
}
