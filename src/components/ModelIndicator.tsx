import { useState, useEffect, useRef } from "react";
import { getModelStatus, setModelMode, ModelStatus } from "../lib/tauri";

interface ModelIndicatorProps {
  lastSource: "local" | "cloud" | null;
}

const MODE_CYCLE = ["auto", "local_only", "cloud_only"] as const;
const MODE_LABELS: Record<string, string> = {
  auto: "Auto",
  local_only: "Local",
  cloud_only: "Cloud",
};

export default function ModelIndicator({ lastSource }: ModelIndicatorProps) {
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [modeIndex, setModeIndex] = useState(0);
  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    getModelStatus()
      .then(setStatus)
      .catch(() => {});

    const interval = setInterval(() => {
      getModelStatus()
        .then(setStatus)
        .catch(() => {});
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (status) {
      const idx = MODE_CYCLE.indexOf(
        status.mode as (typeof MODE_CYCLE)[number]
      );
      if (idx >= 0) setModeIndex(idx);
    }
  }, [status]);

  const cycleMode = async () => {
    const nextIndex = (modeIndex + 1) % MODE_CYCLE.length;
    const nextMode = MODE_CYCLE[nextIndex];
    setModeIndex(nextIndex);
    await setModelMode(nextMode);
    const newStatus = await getModelStatus();
    setStatus(newStatus);
  };

  const handleMouseDown = () => {
    longPressTimer.current = setTimeout(() => {
      setModelMode("cloud_only").then(() => {
        setModeIndex(2);
        getModelStatus().then(setStatus);
      });
    }, 800);
  };

  const handleMouseUp = () => {
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  };

  // Determine dot color based on last source and availability
  let dotColor = "bg-gray-500"; // offline
  let label = "Offline";

  if (lastSource === "local") {
    dotColor = "bg-[#4ade80]";
    label = "Local";
  } else if (lastSource === "cloud") {
    dotColor = "bg-[#60a5fa]";
    label = "Cloud";
  } else if (status?.gemma_available) {
    dotColor = "bg-[#4ade80]";
    label = "Local";
  } else if (status?.claude_available) {
    dotColor = "bg-[#60a5fa]";
    label = "Cloud";
  }

  const currentMode = MODE_LABELS[MODE_CYCLE[modeIndex]] || "Auto";

  return (
    <button
      onClick={cycleMode}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      className="flex items-center gap-1.5 text-xs text-grove-text-secondary hover:text-grove-text-primary transition-colors"
      title={`Mode: ${currentMode}. Click to cycle. Long-press for Cloud.`}
    >
      <span className={`w-2 h-2 rounded-full ${dotColor}`} />
      <span>{label}</span>
    </button>
  );
}
