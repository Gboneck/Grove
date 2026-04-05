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
  { animation: string; coreColor: string; glowColor: string; glowIntensity: number; ringColor: string; label: string }
> = {
  idle: {
    animation: "animate-orb-breathe",
    coreColor: "#d4a853",
    glowColor: "rgba(212, 168, 83, 0.25)",
    glowIntensity: 1,
    ringColor: "rgba(212, 168, 83, 0.08)",
    label: "Idle — breathing",
  },
  thinking: {
    animation: "animate-orb-think",
    coreColor: "#d4a853",
    glowColor: "rgba(212, 168, 83, 0.5)",
    glowIntensity: 2,
    ringColor: "rgba(212, 168, 83, 0.15)",
    label: "Thinking...",
  },
  listening: {
    animation: "animate-orb-listen",
    coreColor: "#d4a853",
    glowColor: "rgba(212, 168, 83, 0.35)",
    glowIntensity: 1.3,
    ringColor: "rgba(212, 168, 83, 0.1)",
    label: "Listening",
  },
  acting: {
    animation: "animate-orb-act",
    coreColor: "#4ade80",
    glowColor: "rgba(74, 222, 128, 0.4)",
    glowIntensity: 1.5,
    ringColor: "rgba(74, 222, 128, 0.12)",
    label: "Taking action",
  },
  alert: {
    animation: "animate-orb-alert",
    coreColor: "#facc15",
    glowColor: "rgba(250, 204, 21, 0.5)",
    glowIntensity: 2.5,
    ringColor: "rgba(250, 204, 21, 0.15)",
    label: "Needs attention",
  },
  reflecting: {
    animation: "animate-orb-reflect",
    coreColor: "#60a5fa",
    glowColor: "rgba(96, 165, 250, 0.2)",
    glowIntensity: 0.8,
    ringColor: "rgba(96, 165, 250, 0.06)",
    label: "Reflecting",
  },
  offline: {
    animation: "",
    coreColor: "#6b7280",
    glowColor: "rgba(107, 114, 128, 0.15)",
    glowIntensity: 0.3,
    ringColor: "transparent",
    label: "Offline",
  },
};

const sizeMap = {
  sm: { outer: 24, inner: 10, ring: 20 },
  md: { outer: 36, inner: 14, ring: 28 },
  lg: { outer: 48, inner: 20, ring: 40 },
};

export default function DaemonOrb({
  state,
  size = "md",
  onClick,
}: DaemonOrbProps) {
  const config = stateConfig[state];
  const dims = sizeMap[size];
  const [prevState, setPrevState] = useState(state);
  const showParticles = state === "thinking" || state === "acting";

  useEffect(() => {
    setPrevState(state);
  }, [state]);

  const glowRadius = dims.outer * config.glowIntensity;

  return (
    <button
      onClick={onClick}
      className={`relative flex items-center justify-center group ${
        onClick ? "cursor-pointer" : "cursor-default"
      }`}
      style={{ width: dims.outer, height: dims.outer }}
      aria-label={`Grove status: ${config.label}`}
      title={config.label}
    >
      {/* Ambient glow — large soft bloom */}
      <div
        className="absolute inset-0 rounded-full transition-all duration-500"
        style={{
          boxShadow: `0 0 ${glowRadius}px ${glowRadius / 2}px ${config.glowColor}`,
        }}
      />

      {/* Orbital ring */}
      <div
        className={`absolute rounded-full border transition-all duration-500 ${showParticles ? "animate-spin-slow" : ""}`}
        style={{
          width: dims.ring,
          height: dims.ring,
          top: (dims.outer - dims.ring) / 2,
          left: (dims.outer - dims.ring) / 2,
          borderColor: config.ringColor,
          borderWidth: 1,
        }}
      />

      {/* Particle dots (visible during thinking/acting) */}
      {showParticles && (
        <>
          <div
            className="absolute rounded-full animate-orbit-1"
            style={{
              width: 3, height: 3,
              backgroundColor: config.coreColor,
              opacity: 0.7,
              top: 0, left: "50%", marginLeft: -1.5,
            }}
          />
          <div
            className="absolute rounded-full animate-orbit-2"
            style={{
              width: 2, height: 2,
              backgroundColor: config.coreColor,
              opacity: 0.5,
              bottom: 0, left: "50%", marginLeft: -1,
            }}
          />
          <div
            className="absolute rounded-full animate-orbit-3"
            style={{
              width: 2, height: 2,
              backgroundColor: config.coreColor,
              opacity: 0.4,
              top: "50%", right: 0, marginTop: -1,
            }}
          />
        </>
      )}

      {/* Inner core */}
      <div
        className={`relative rounded-full ${config.animation} transition-colors duration-500`}
        style={{
          width: dims.inner,
          height: dims.inner,
          backgroundColor: config.coreColor,
          boxShadow: state === "offline"
            ? "none"
            : `0 0 ${dims.inner}px ${dims.inner / 2}px ${config.glowColor}`,
        }}
      />

      {/* Hover tooltip */}
      <span className="absolute -bottom-6 left-1/2 -translate-x-1/2 text-[9px] text-grove-text-secondary opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap font-mono">
        {config.label}
      </span>
    </button>
  );
}
