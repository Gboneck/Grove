import { useState } from "react";
import { saveApiKey, SetupStatus } from "../lib/tauri";

interface SetupScreenProps {
  status: SetupStatus;
  onComplete: () => void;
}

export default function SetupScreen({ status, onComplete }: SetupScreenProps) {
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const handleSave = async () => {
    if (!apiKey.trim()) {
      setError("Please enter an API key");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await saveApiKey(apiKey.trim());
      onComplete();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-8">
      <div className="max-w-md w-full space-y-8">
        {/* Header */}
        <div className="text-center space-y-3">
          <h1 className="text-3xl font-display text-grove-accent">Grove</h1>
          <p className="text-grove-text-secondary text-sm">
            Your personal operating system
          </p>
        </div>

        {/* Status checks */}
        <div className="space-y-3">
          <div className="flex items-center gap-3 text-sm">
            <span className="w-2 h-2 rounded-full bg-grove-status-green" />
            <span className="text-grove-text-secondary">
              ~/.grove/ directory initialized
            </span>
          </div>

          <div className="flex items-center gap-3 text-sm">
            <span
              className={`w-2 h-2 rounded-full ${status.ollama_available ? "bg-grove-status-green" : "bg-grove-status-yellow"}`}
            />
            <span className="text-grove-text-secondary">
              {status.ollama_available
                ? `Ollama detected — local reasoning ready`
                : "Ollama not detected — install for offline reasoning"}
            </span>
          </div>

          <div className="flex items-center gap-3 text-sm">
            <span
              className={`w-2 h-2 rounded-full ${status.api_key_set ? "bg-grove-status-green" : "bg-grove-status-red"}`}
            />
            <span className="text-grove-text-secondary">
              {status.api_key_set
                ? "Anthropic API key configured"
                : "Anthropic API key needed"}
            </span>
          </div>
        </div>

        {/* System info */}
        {!status.ollama_available && (
          <div className="bg-grove-surface border border-grove-border rounded-lg p-4 space-y-2">
            <p className="text-sm text-grove-text-primary font-medium">
              Recommended local model
            </p>
            <p className="text-xs text-grove-text-secondary">
              System RAM: {status.system_ram_gb}GB → We recommend{" "}
              <span className="font-mono text-grove-accent">
                {status.recommended_model}
              </span>
            </p>
            <p className="text-xs text-grove-text-secondary mt-2">
              Install Ollama, then run:{" "}
              <code className="font-mono text-grove-text-primary">
                ollama pull {status.recommended_model}
              </code>
            </p>
          </div>
        )}

        {/* API key input */}
        {!status.api_key_set && (
          <div className="space-y-3">
            <label className="text-sm text-grove-text-secondary block">
              Enter your Anthropic API key to connect the cloud reasoning
              engine.
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
              placeholder="sk-ant-..."
              className="w-full bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors font-mono text-sm"
            />
            {error && (
              <p className="text-xs text-grove-status-red">{error}</p>
            )}
            <button
              onClick={handleSave}
              disabled={saving}
              className="w-full bg-grove-accent text-grove-bg py-3 rounded-lg font-medium hover:brightness-110 transition-all disabled:opacity-50"
            >
              {saving ? "Saving…" : "Save & Launch Grove"}
            </button>
          </div>
        )}

        {/* Skip option if Ollama is available */}
        {status.ollama_available && !status.api_key_set && (
          <button
            onClick={onComplete}
            className="w-full text-sm text-grove-text-secondary hover:text-grove-text-primary transition-colors py-2"
          >
            Skip — use local reasoning only
          </button>
        )}

        {/* Already set up, just continue */}
        {status.api_key_set && (
          <button
            onClick={onComplete}
            className="w-full bg-grove-accent text-grove-bg py-3 rounded-lg font-medium hover:brightness-110 transition-all"
          >
            Launch Grove
          </button>
        )}
      </div>
    </div>
  );
}
