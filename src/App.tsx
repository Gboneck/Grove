import { useState, useEffect, useCallback, useRef } from "react";
import GroveShell from "./components/GroveShell";
import BlockRenderer from "./components/BlockRenderer";
import LoadingState from "./components/LoadingState";
import SetupScreen from "./components/SetupScreen";
import SoulEditor from "./components/SoulEditor";
import {
  reason as invokeReason,
  checkSetup,
  getFileStamps,
  Block,
  SetupStatus,
  FileStamps,
} from "./lib/tauri";

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
        onRefresh={() => reason()}
        onOpenSoul={() => setSoulEditorOpen(true)}
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
    </>
  );
}
