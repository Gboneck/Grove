import { useState, useCallback } from "react";
import type { OrbState } from "../components/DaemonOrb";

type AmbientMood =
  | "focused"
  | "calm"
  | "urgent"
  | "creative"
  | "reflective"
  | null;

type ThemeHint = "warm" | "cool" | "dark" | "light" | null;

interface AmbientState {
  mood: AmbientMood;
  theme: ThemeHint;
  orbState: OrbState;
  isLoading: boolean;
  modelSource: "local" | "cloud" | null;
  inputFocused: boolean;
}

/**
 * Manages the ambient state of the Grove UI — mood, theme, and orb state.
 * Derives the Daemon Orb state from a combination of loading state,
 * input focus, mood, and model availability.
 */
export function useAmbient() {
  const [state, setState] = useState<AmbientState>({
    mood: null,
    theme: null,
    orbState: "idle",
    isLoading: false,
    modelSource: null,
    inputFocused: false,
  });

  // Derive orb state from ambient conditions
  const deriveOrbState = useCallback(
    (overrides: Partial<AmbientState> = {}): OrbState => {
      const merged = { ...state, ...overrides };
      if (merged.isLoading) return "thinking";
      if (merged.inputFocused) return "listening";
      if (merged.mood === "urgent") return "alert";
      if (merged.mood === "reflective") return "reflecting";
      if (merged.modelSource === null) return "offline";
      return "idle";
    },
    [state]
  );

  const setMood = useCallback((mood: AmbientMood) => {
    setState((prev) => {
      const next = { ...prev, mood };
      return { ...next, orbState: deriveOrbState(next) };
    });
  }, [deriveOrbState]);

  const setTheme = useCallback((theme: ThemeHint) => {
    setState((prev) => ({ ...prev, theme }));
  }, []);

  const setLoading = useCallback((isLoading: boolean) => {
    setState((prev) => {
      const next = { ...prev, isLoading };
      return { ...next, orbState: deriveOrbState(next) };
    });
  }, [deriveOrbState]);

  const setModelSource = useCallback(
    (modelSource: "local" | "cloud" | null) => {
      setState((prev) => {
        const next = { ...prev, modelSource };
        return { ...next, orbState: deriveOrbState(next) };
      });
    },
    [deriveOrbState]
  );

  const setInputFocused = useCallback((inputFocused: boolean) => {
    setState((prev) => {
      const next = { ...prev, inputFocused };
      return { ...next, orbState: deriveOrbState(next) };
    });
  }, [deriveOrbState]);

  // Update from model response
  const updateFromResponse = useCallback(
    (response: {
      ambient_mood?: string | null;
      theme_hint?: string | null;
      model_source?: string | null;
    }) => {
      setState((prev) => {
        const next = {
          ...prev,
          mood: (response.ambient_mood as AmbientMood) ?? prev.mood,
          theme: (response.theme_hint as ThemeHint) ?? prev.theme,
          modelSource:
            (response.model_source as "local" | "cloud" | null) ??
            prev.modelSource,
          isLoading: false,
        };
        return { ...next, orbState: deriveOrbState(next) };
      });
    },
    [deriveOrbState]
  );

  return {
    ...state,
    setMood,
    setTheme,
    setLoading,
    setModelSource,
    setInputFocused,
    updateFromResponse,
  };
}
