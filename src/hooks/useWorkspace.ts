import { useState, useCallback, useEffect } from "react";
import type { Artifact, WorkspaceData } from "../lib/tauri";
import { loadWorkspace, saveWorkspace, removeArtifact as removeArtifactCmd } from "../lib/tauri";

export default function useWorkspace() {
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [loaded, setLoaded] = useState(false);

  // Load on mount
  useEffect(() => {
    loadWorkspace()
      .then((data: WorkspaceData) => {
        setArtifacts(data.artifacts || []);
        setLoaded(true);
      })
      .catch(() => setLoaded(true));
  }, []);

  // Refresh from disk (called after reasoning, which may have created/updated artifacts)
  const refresh = useCallback(() => {
    loadWorkspace()
      .then((data: WorkspaceData) => {
        setArtifacts(data.artifacts || []);
      })
      .catch(() => {});
  }, []);

  // Remove an artifact
  const removeArtifact = useCallback((id: string) => {
    setArtifacts(prev => prev.filter(a => a.id !== id));
    removeArtifactCmd(id).catch(() => {});
  }, []);

  // Move artifact to new position
  const moveArtifact = useCallback((id: string, x: number, y: number) => {
    setArtifacts(prev => {
      const next = prev.map(a => a.id === id ? { ...a, x, y } : a);
      saveWorkspace(next).catch(() => {});
      return next;
    });
  }, []);

  // Resize artifact width
  const resizeArtifact = useCallback((id: string, width: number) => {
    setArtifacts(prev => {
      const next = prev.map(a => a.id === id ? { ...a, width } : a);
      saveWorkspace(next).catch(() => {});
      return next;
    });
  }, []);

  // Toggle collapse
  const collapseArtifact = useCallback((id: string, collapsed: boolean) => {
    setArtifacts(prev => {
      const next = prev.map(a => a.id === id ? { ...a, collapsed } : a);
      saveWorkspace(next).catch(() => {});
      return next;
    });
  }, []);

  return {
    artifacts,
    loaded,
    refresh,
    removeArtifact,
    moveArtifact,
    resizeArtifact,
    collapseArtifact,
    hasArtifacts: artifacts.length > 0,
  };
}
