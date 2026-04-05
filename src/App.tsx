import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import GroveShell from "./components/GroveShell";
import BlockRenderer from "./components/BlockRenderer";
import LoadingState from "./components/LoadingState";
import useWorkspace from "./hooks/useWorkspace";
import WorkspaceCanvas from "./components/WorkspaceCanvas";
import SetupScreen from "./components/SetupScreen";
import SoulEditor from "./components/SoulEditor";
import MemoryPanel from "./components/panels/MemoryPanel";
import LogsPanel from "./components/panels/LogsPanel";
import PluginPanel from "./components/panels/PluginPanel";
import ProfilePanel from "./components/panels/ProfilePanel";
import ContextEditor from "./components/panels/ContextEditor";
import SearchPanel from "./components/panels/SearchPanel";
import CommandPalette from "./components/CommandPalette";
import ActionLog from "./components/ActionLog";
import DigestPanel from "./components/panels/DigestPanel";
import EvolutionPanel from "./components/panels/EvolutionPanel";
import OfflineFallback from "./components/OfflineFallback";
import {
  reasonStream,
  checkSetup,
  getFileStamps,
  clearConversation,
  getEnrichmentPrompts,
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
  const workspace = useWorkspace();
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [reasonPhase, setReasonPhase] = useState<string | null>(null);
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
  const [digestOpen, setDigestOpen] = useState(false);
  const [evolutionOpen, setEvolutionOpen] = useState(false);
  const [autoActions, setAutoActions] = useState<string[]>([]);
  const [ventureUpdates, setVentureUpdates] = useState<string[]>([]);
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

    // Notifications disabled — they interrupt the user
  }, []);

  // Workspace artifacts load automatically via useWorkspace hook

  const streamingBlocksRef = useRef<Block[]>([]);
  const reasoningRef = useRef(false); // Guard against concurrent reasoning
  const lastReasonedRef = useRef(0); // Timestamp of last completed reasoning
  const REASON_COOLDOWN_MS = 60000; // 60s minimum between auto-triggered reasoning

  const reason = useCallback(async (userInput?: string) => {
    // Prevent concurrent reasoning calls
    if (reasoningRef.current) return;
    reasoningRef.current = true;

    // User-initiated input replaces blocks; auto-triggered adds to them
    const isUserInput = !!userInput;

    setIsLoading(true);
    setError(false);
    setReasonPhase(null);

    if (isUserInput) {
      // User asked something — clear canvas for fresh response
      setBlocks([]);
      streamingBlocksRef.current = [];
    } else {
      // Auto-triggered — keep existing blocks, new ones will append
      streamingBlocksRef.current = [];
    }

    // Listen for individual blocks and progress events as they stream in
    let blockUnsub: (() => void) | null = null;
    let progressUnsub: (() => void) | null = null;
    try {
      blockUnsub = await listen<Block>("reason-block", (event) => {
        // Tag each block with a unique ID for removal tracking
        const tagged = { ...event.payload, _id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}` };
        streamingBlocksRef.current = [...streamingBlocksRef.current, tagged];
        if (isUserInput) {
          setBlocks([...streamingBlocksRef.current]);
        } else {
          // Additive — append to existing
          setBlocks(prev => [...prev, tagged]);
        }
      }).then(fn => { return fn; });

      progressUnsub = await listen<string>("reason-progress", (event) => {
        setReasonPhase(event.payload);
      }).then(fn => { return fn; });

      const response = await reasonStream(userInput);

      if (response.blocks && Array.isArray(response.blocks)) {
        try {
          const enrichment = await getEnrichmentPrompts();
          if (enrichment.length > 0) {
            response.blocks.push(...enrichment);
          }
        } catch {
          // Enrichment is optional
        }

        // Tag final blocks with IDs
        const taggedBlocks = response.blocks.map((b: Block) => ({
          ...b,
          _id: (b as Record<string, unknown>)._id || `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        }));

        if (isUserInput) {
          setBlocks(taggedBlocks);
        } else {
          // Additive: append final blocks
          setBlocks(prev => {
            const streamIds = new Set(streamingBlocksRef.current.map((b: Record<string, unknown>) => b._id));
            const kept = prev.filter((b: Record<string, unknown>) => !streamIds.has(b._id));
            return [...kept, ...taggedBlocks];
          });
        }

        setLastUpdated(new Date());
        setModelSource(response.model_source);
        setAmbientMood(response.ambient_mood);
        setThemeHint(response.theme_hint);
        setAutoActions(response.auto_action_results || []);
        setVentureUpdates(response.venture_update_results || []);
        // Refresh artifacts — model may have created/updated them via auto-actions
        refreshArtifactsAfterReason();
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
      if (blockUnsub) blockUnsub();
      if (progressUnsub) progressUnsub();
      setIsLoading(false);
      setReasonPhase(null);
      reasoningRef.current = false;
      lastReasonedRef.current = Date.now();
    }
  }, []);

  // Auto-reason when entering running phase
  useEffect(() => {
    if (phase === "running") {
      reason();
    }
  }, [phase, reason]);

  // Listen for heartbeat-triggered reasoning (proactive intelligence)
  // Only auto-reasons if cooldown has elapsed — don't interrupt the user
  useEffect(() => {
    if (phase !== "running") return;
    const unlisten = listen<{
      observation_count: number;
      summary: string;
      has_deadline: boolean;
      has_time_shift: boolean;
    }>("heartbeat-reason-trigger", (event) => {
      const elapsed = Date.now() - lastReasonedRef.current;
      if (elapsed < REASON_COOLDOWN_MS) return; // Cooldown — don't interrupt
      const data = event.payload;
      reason(data.has_time_shift
        ? "Time has shifted. Re-evaluate priorities for this part of the day."
        : undefined
      );
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [phase, reason]);

  // Listen for periodic reasoning events from the background timer
  // Don't replace current blocks — just set the flag so user can pull when ready
  useEffect(() => {
    const unlisten = listen<{
      blocks: Block[];
      timestamp: string;
      model_source: string;
      ambient_mood: string | null;
      theme_hint: string | null;
      has_urgent: boolean;
    }>("periodic-reasoning", () => {
      setHasPeriodicUpdate(true);
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
    { id: "digest", label: "Weekly Digest", action: () => setDigestOpen(true) },
    { id: "evolution", label: "Soul Evolution", action: () => setEvolutionOpen(true) },
  ];

  // Poll ~/.grove/ files for external changes every 30s (not 10s — avoid thrashing)
  useEffect(() => {
    if (phase !== "running") return;

    const pollInterval = setInterval(async () => {
      // Don't trigger if already reasoning or in cooldown
      if (reasoningRef.current) return;
      if (Date.now() - lastReasonedRef.current < REASON_COOLDOWN_MS) return;

      try {
        const stamps = await getFileStamps();
        const prev = lastStampsRef.current;
        lastStampsRef.current = stamps;

        if (!prev) return; // First poll, just save baseline

        // Check if soul.md or context.json changed (ignore memory/log files)
        for (const [file, mtime] of Object.entries(stamps.files)) {
          if (!file.endsWith("soul.md") && !file.endsWith("context.json")) continue;
          const prevTime = prev.files[file];
          if (prevTime !== undefined && prevTime !== mtime) {
            console.log(`[grove] ${file} changed externally, re-reasoning`);
            invoke("notify_file_change").catch(() => {});
            reason();
            return;
          }
        }
      } catch {
        // Ignore polling errors
      }
    }, 30000);

    return () => clearInterval(pollInterval);
  }, [phase, reason]);

  const handleInput = (value: string) => {
    reason(value);
  };

  const handleDismissBlock = useCallback((id: string) => {
    setBlocks(prev => prev.filter((b: Record<string, unknown>) => b._id !== id));
  }, []);

  // After reasoning completes, refresh artifacts from disk (model may have created/updated them)
  const refreshArtifactsAfterReason = useCallback(() => {
    workspace.refresh();
  }, [workspace]);

  // Workspace canvas handles drag state internally

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
        onInput={handleInput}
        hasUpdate={hasPeriodicUpdate}
        onAcknowledgeUpdate={() => setHasPeriodicUpdate(false)}
        isLoading={isLoading}
        lastUpdated={lastUpdated}
        modelSource={modelSource}
        ambientMood={ambientMood}
        themeHint={themeHint}
      >
        <div className={error ? "opacity-70" : ""}>
          {error && blocks === FALLBACK_BLOCKS ? (
            <OfflineFallback />
          ) : (
            <>
              {workspace.hasArtifacts && (
                <WorkspaceCanvas
                  artifacts={workspace.artifacts}
                  onMove={workspace.moveArtifact}
                  onResize={workspace.resizeArtifact}
                  onRemove={workspace.removeArtifact}
                  onCollapse={workspace.collapseArtifact}
                />
              )}
              <div className="max-w-[720px] mx-auto">
                <BlockRenderer blocks={blocks} onInput={handleInput} onDismissBlock={handleDismissBlock} isLoading={isLoading} />
              </div>
            </>
          )}
        </div>
        {isLoading && blocks.length === 0 && <LoadingState phase={reasonPhase} />}
        {isLoading && blocks.length > 0 && (
          <div className="flex items-center gap-2 mt-4 text-sm text-grove-text-secondary">
            <span className="w-1.5 h-1.5 rounded-full bg-grove-accent animate-pulse" />
            <span className="streaming-dots font-mono">
              <span>.</span><span>.</span><span>.</span>
            </span>
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
      <DigestPanel
        isOpen={digestOpen}
        onClose={() => setDigestOpen(false)}
      />
      <EvolutionPanel
        isOpen={evolutionOpen}
        onClose={() => setEvolutionOpen(false)}
      />
      <CommandPalette
        isOpen={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        commands={paletteCommands}
      />
      <ActionLog
        autoActions={autoActions}
        ventureUpdates={ventureUpdates}
      />
    </>
  );
}
