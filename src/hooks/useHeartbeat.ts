import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";

interface HeartbeatObservation {
  kind: string;
  detail: string;
  timestamp: string;
}

interface HeartbeatStatus {
  total_ticks: number;
  queue_size: number;
  observations: HeartbeatObservation[];
  patterns: HeartbeatPattern[];
}

interface HeartbeatPattern {
  description: string;
  confidence: number;
  occurrences: number;
  pattern_type: string;
}

interface HeartbeatEvent {
  observations: HeartbeatObservation[];
  triggered_reasoning: boolean;
  queue_size: number;
}

/**
 * Subscribe to heartbeat events from the Rust backend.
 * The heartbeat runs on a timer and emits observations about
 * file changes, time shifts, and approaching deadlines.
 */
export function useHeartbeat() {
  const [status, setStatus] = useState<HeartbeatStatus>({
    total_ticks: 0,
    queue_size: 0,
    observations: [],
    patterns: [],
  });
  const [lastEvent, setLastEvent] = useState<HeartbeatEvent | null>(null);

  useEffect(() => {
    const unlisten = listen<HeartbeatEvent>("heartbeat-tick", (event) => {
      const data = event.payload;
      setLastEvent(data);
      setStatus((prev) => ({
        ...prev,
        total_ticks: prev.total_ticks + 1,
        queue_size: data.queue_size,
        observations: [
          ...data.observations,
          ...prev.observations,
        ].slice(0, 50), // Keep last 50 observations
      }));
    });

    const unlistenPatterns = listen<HeartbeatPattern[]>(
      "heartbeat-patterns",
      (event) => {
        setStatus((prev) => ({
          ...prev,
          patterns: event.payload,
        }));
      }
    );

    return () => {
      unlisten.then((fn) => fn());
      unlistenPatterns.then((fn) => fn());
    };
  }, []);

  const clearObservations = useCallback(() => {
    setStatus((prev) => ({
      ...prev,
      observations: [],
    }));
  }, []);

  return {
    status,
    lastEvent,
    clearObservations,
    hasObservations: status.observations.length > 0,
    hasPatterns: status.patterns.length > 0,
  };
}
