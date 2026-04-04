"use client";

import { useState, useEffect, useCallback } from "react";
import GroveShell from "./components/GroveShell";
import BlockRenderer, { Block } from "./components/BlockRenderer";
import LoadingState from "./components/LoadingState";

const FALLBACK_BLOCKS: Block[] = [
  {
    type: "text",
    heading: "Grove OS",
    body: "The reasoning engine is warming up or unavailable. Hit refresh to try again.",
  },
  {
    type: "insight",
    icon: "warning",
    message:
      "If this persists, check that the ANTHROPIC_API_KEY environment variable is set.",
  },
];

export default function Home() {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [error, setError] = useState(false);

  const reason = useCallback(async (userInput?: string) => {
    setIsLoading(true);
    setError(false);
    try {
      const res = await fetch("/api/reason", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ userInput: userInput || null }),
      });

      if (!res.ok) {
        throw new Error(`API returned ${res.status}`);
      }

      const data = await res.json();

      if (data.blocks && Array.isArray(data.blocks)) {
        setBlocks(data.blocks);
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
