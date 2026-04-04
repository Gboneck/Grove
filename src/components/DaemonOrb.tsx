import { useEffect, useState } from "react";

export type OrbState =
  | "idle"
  | "thinking"
  | "listening"
  | "acting"
  | "alert"
  | "reflecting"
  | "offline";

interface DaemonOrbProps {
  state: OrbState;
  size?: "sm" | "md" | "lg";
  onClick?: () => void;
}

const stateConfig: Record<
  OrbState,
  { animation: string; glowColor: string; glowOpacity: string; label: string }
> = {
  idle: {
    animation: "animate-orb-breathe",
    glowColor: "rgba(212, 168, 83, 0.3)",
    glowOpacity: "0.7",
    label: "Idle",
  },
  thinking: {
    animation: "animate-orb-think",
    glowColor: "rgba(212, 168, 83, 0.5)",
    glowOpacity: "0.9",
    label: "Thinking",
  },
  listening: {
    animation: "animate-orb-listen",
    glowColor: "rgba(212, 168, 83, 0.35)",
    glowOpacity: "0.8",
    label: "Listening",
  },
  acting: {
    animation: "animate-orb-act",
    glowColor: "rgba(74, 222, 128, 0.4)",
    glowOpacity: "0.9",
    label: "Acting",
  },
  alert: {
    animation: "animate-orb-alert",
    glowColor: "rgba(250, 204, 21, 0.5)",
    glowOpacity: "1",
    label: "Alert",
  },
  reflecting: {
    animation: "animate-orb-reflect",
    glowColor: "rgba(212, 168, 83, 0.2)",
    glowOpacity: "0.6",
    label: "Reflecting",
  },
  offline: {
    animation: "",
    glowColor: "rgba(107, 114, 128, 0.3)",
    glowOpacity: "0.4",
    label: "Offline",
  },
};

const sizeMap = {
  sm: { outer: 20, inner: 10 },
  md: { outer: 28, inner: 14 },
  lg: { outer: 36, inner: 18 },
};

export default function DaemonOrb({
  state,
  size = "md",
  onClick,
}: DaemonOrbProps) {
  const config = stateConfig[state];
  const dims = sizeMap[size];
  const [prevState, setPrevState] = useState(state);

  useEffect(() => {
    setPrevState(state);
  }, [state]);

  const isTransitioning = prevState !== state;

  return (
    <button
      onClick={onClick}
      className={`relative flex items-center justify-center transition-all duration-300 ${
        onClick ? "cursor-pointer" : "cursor-default"
      }`}
      style={{ width: dims.outer, height: dims.outer }}
      aria-label={`Grove status: ${config.label}`}
      title={config.label}
    >
      {/* Outer glow */}
      <div
        className={`absolute inset-0 rounded-full ${config.animation} ${
          isTransitioning ? "transition-all duration-300" : ""
        }`}
        style={{
          boxShadow: `0 0 ${dims.outer / 2}px ${dims.outer / 4}px ${config.glowColor}`,
          opacity: config.glowOpacity,
        }}
      />

      {/* Inner core */}
      <div
        className={`relative rounded-full ${config.animation}`}
        style={{
          width: dims.inner,
          height: dims.inner,
          backgroundColor:
            state === "offline" ? "#6b7280" : "#d4a853",
          boxShadow:
            state === "offline"
              ? "none"
              : `0 0 ${dims.inner / 2}px ${dims.inner / 4}px rgba(212, 168, 83, 0.4)`,
        }}
      />
    </button>
  );
}
