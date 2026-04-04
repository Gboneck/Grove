import { useState, useEffect, useCallback } from "react";
import GroveShell from "./components/GroveShell";
import BlockRenderer from "./components/BlockRenderer";
import LoadingState from "./components/LoadingState";
import { reason as invokeReason, Block } from "./lib/tauri";

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
      "If this persists, check that your ANTHROPIC_API_KEY is set in ~/.grove/.env",
  },
];

export default function App() {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [error, setError] = useState(false);

  const reason = useCallback(async (userInput?: string) => {
    setIsLoading(true);
    setError(false);
    try {
      const response = await invokeReason(userInput);
      if (response.blocks && Array.isArray(response.blocks)) {
        setBlocks(response.blocks);
        setLastUpdated(new Date());
      } else {
        throw new Error("Invalid response structure");
      }
    } catch (err) {
      console.error("Reasoning failed:", err);
      setError(true);
      setBlocks(FALLBACK_BLOCKS);
      setLastUpdated(new Date());
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    reason();
  }, [reason]);

  const handleInput = (value: string) => {
    reason(value);
  };

  return (
    <GroveShell
      onRefresh={() => reason()}
      isLoading={isLoading}
      lastUpdated={lastUpdated}
    >
      {isLoading ? (
        <LoadingState />
      ) : (
        <div className={error ? "opacity-70" : ""}>
          <BlockRenderer blocks={blocks} onInput={handleInput} />
        </div>
      )}
    </GroveShell>
  );
}
