import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Modal from "../Modal";

interface ProfileInfo {
  name: string;
  description: string;
  is_active: boolean;
}

interface ProfilePanelProps {
  isOpen: boolean;
  onClose: () => void;
  onSwitch: () => void;
}

export default function ProfilePanel({ isOpen, onClose, onSwitch }: ProfilePanelProps) {
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [error, setError] = useState("");

  const refresh = () => {
    invoke<ProfileInfo[]>("list_profiles").then(setProfiles);
  };

  useEffect(() => {
    if (isOpen) refresh();
  }, [isOpen]);

  const handleSwitch = async (name: string) => {
    setError("");
    try {
      await invoke("switch_profile", { name });
      refresh();
      onSwitch();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    setError("");
    try {
      await invoke("create_profile", {
        name: newName.trim(),
        description: newDesc.trim(),
      });
      setCreating(false);
      setNewName("");
      setNewDesc("");
      refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (name: string) => {
    setError("");
    try {
      await invoke("delete_profile", { name });
      refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Profiles" maxWidth="max-w-lg">
      <div className="px-6 py-4 space-y-3">
        {profiles.map((p) => (
          <div
            key={p.name}
            className={`bg-grove-surface border rounded-lg p-4 flex items-center justify-between ${
              p.is_active ? "border-grove-accent/50" : "border-grove-border"
            }`}
          >
            <div className="space-y-0.5">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium text-grove-text-primary">{p.name}</span>
                {p.is_active && (
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-grove-accent/20 text-grove-accent">
                    active
                  </span>
                )}
              </div>
              {p.description && (
                <p className="text-xs text-grove-text-secondary">{p.description}</p>
              )}
            </div>
            <div className="flex items-center gap-2">
              {!p.is_active && (
                <>
                  <button
                    onClick={() => handleSwitch(p.name)}
                    className="text-xs px-3 py-1.5 rounded bg-grove-accent text-grove-bg hover:brightness-110 transition-all"
                  >
                    switch
                  </button>
                  {p.name !== "default" && (
                    <button
                      onClick={() => handleDelete(p.name)}
                      className="text-xs px-2 py-1.5 rounded text-grove-status-red hover:bg-grove-status-red/10 transition-colors"
                    >
                      delete
                    </button>
                  )}
                </>
              )}
            </div>
          </div>
        ))}

        {creating ? (
          <div className="bg-grove-surface border border-grove-border rounded-lg p-4 space-y-3">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Profile name"
              className="w-full bg-grove-bg border border-grove-border rounded px-3 py-2 text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60"
            />
            <input
              value={newDesc}
              onChange={(e) => setNewDesc(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
              placeholder="Description (optional)"
              className="w-full bg-grove-bg border border-grove-border rounded px-3 py-2 text-sm text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60"
            />
            <div className="flex gap-2">
              <button
                onClick={handleCreate}
                disabled={!newName.trim()}
                className="px-4 py-1.5 text-xs rounded bg-grove-accent text-grove-bg hover:brightness-110 disabled:opacity-50"
              >
                Create
              </button>
              <button
                onClick={() => { setCreating(false); setNewName(""); setNewDesc(""); }}
                className="px-4 py-1.5 text-xs rounded text-grove-text-secondary hover:text-grove-text-primary"
              >
                Cancel
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={() => setCreating(true)}
            className="w-full text-xs text-grove-accent hover:text-grove-accent/80 transition-colors py-2"
          >
            + Create new profile
          </button>
        )}

        {error && <p className="text-xs text-grove-status-red" role="alert">{error}</p>}
      </div>

      <div className="px-6 py-3 border-t border-grove-border">
        <p className="text-[10px] text-grove-text-secondary">
          Profiles let you switch between different contexts. Each profile has its own context.json.
        </p>
      </div>
    </Modal>
  );
}
