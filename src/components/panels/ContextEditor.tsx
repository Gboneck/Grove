import { useState, useEffect } from "react";
import { readContext, writeContext } from "../../lib/tauri";

interface ContextEditorProps {
  isOpen: boolean;
  onClose: () => void;
}

interface Venture {
  name: string;
  status: string;
  health: string;
  priority: number;
  nextAction: string;
  deadline?: string;
}

interface ContextData {
  ventures: Venture[];
}

export default function ContextEditor({ isOpen, onClose }: ContextEditorProps) {
  const [ventures, setVentures] = useState<Venture[]>([]);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [rawMode, setRawMode] = useState(false);
  const [rawContent, setRawContent] = useState("");

  useEffect(() => {
    if (isOpen) {
      readContext()
        .then((data) => {
          const ctx = data as ContextData;
          setVentures(ctx.ventures || []);
          setRawContent(JSON.stringify(data, null, 2));
          setSaved(false);
        })
        .catch(console.error);
    }
  }, [isOpen]);

  const handleSave = async () => {
    setSaving(true);
    try {
      if (rawMode) {
        const parsed = JSON.parse(rawContent);
        await writeContext(parsed);
        setVentures(parsed.ventures || []);
      } else {
        const data = { ventures };
        await writeContext(data);
        setRawContent(JSON.stringify(data, null, 2));
      }
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save context:", e);
    } finally {
      setSaving(false);
    }
  };

  const updateVenture = (i: number, field: keyof Venture, value: string | number) => {
    const next = [...ventures];
    (next[i] as unknown as Record<string, unknown>)[field] = value;
    setVentures(next);
  };

  const addVenture = () => {
    setVentures([
      ...ventures,
      {
        name: "",
        status: "active",
        health: "green",
        priority: ventures.length + 1,
        nextAction: "",
      },
    ]);
  };

  const removeVenture = (i: number) => {
    setVentures(ventures.filter((_, idx) => idx !== i));
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative bg-grove-bg border border-grove-border rounded-xl w-full max-w-2xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-grove-border">
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-display text-grove-accent">Context</h2>
            <button
              onClick={() => setRawMode(!rawMode)}
              className="text-[10px] px-2 py-0.5 rounded bg-grove-border text-grove-text-secondary hover:text-grove-text-primary transition-colors"
            >
              {rawMode ? "visual" : "raw"}
            </button>
          </div>
          <div className="flex items-center gap-3">
            {saved && (
              <span className="text-xs text-grove-status-green">Saved</span>
            )}
            <button
              onClick={handleSave}
              disabled={saving}
              className="text-xs bg-grove-accent text-grove-bg px-3 py-1.5 rounded font-medium hover:brightness-110 transition-all disabled:opacity-50"
            >
              {saving ? "Saving…" : "Save"}
            </button>
            <button
              onClick={onClose}
              className="text-grove-text-secondary hover:text-grove-text-primary transition-colors"
            >
              close
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {rawMode ? (
            <textarea
              value={rawContent}
              onChange={(e) => setRawContent(e.target.value)}
              className="w-full h-[400px] bg-grove-surface border border-grove-border rounded-lg p-4 text-grove-text-primary text-sm font-mono leading-relaxed resize-none focus:outline-none focus:ring-1 focus:ring-grove-accent/30"
              spellCheck={false}
            />
          ) : (
            <div className="space-y-4">
              {ventures.map((v, i) => (
                <div
                  key={i}
                  className="bg-grove-surface border border-grove-border rounded-lg p-4 space-y-3"
                >
                  <div className="flex items-start justify-between">
                    <input
                      value={v.name}
                      onChange={(e) => updateVenture(i, "name", e.target.value)}
                      placeholder="Venture name"
                      className="bg-transparent text-sm font-medium text-grove-text-primary focus:outline-none border-b border-transparent focus:border-grove-accent/40 flex-1"
                    />
                    <button
                      onClick={() => removeVenture(i)}
                      className="text-xs text-grove-text-secondary hover:text-grove-status-red transition-colors ml-2"
                    >
                      remove
                    </button>
                  </div>

                  <div className="grid grid-cols-3 gap-3">
                    <div>
                      <label className="text-[10px] text-grove-text-secondary uppercase tracking-wider">
                        Status
                      </label>
                      <select
                        value={v.status}
                        onChange={(e) => updateVenture(i, "status", e.target.value)}
                        className="w-full mt-1 bg-grove-bg border border-grove-border rounded px-2 py-1 text-xs text-grove-text-primary focus:outline-none"
                      >
                        <option value="active">Active</option>
                        <option value="paused">Paused</option>
                        <option value="planning">Planning</option>
                        <option value="completed">Completed</option>
                      </select>
                    </div>
                    <div>
                      <label className="text-[10px] text-grove-text-secondary uppercase tracking-wider">
                        Health
                      </label>
                      <select
                        value={v.health}
                        onChange={(e) => updateVenture(i, "health", e.target.value)}
                        className="w-full mt-1 bg-grove-bg border border-grove-border rounded px-2 py-1 text-xs text-grove-text-primary focus:outline-none"
                      >
                        <option value="green">Green</option>
                        <option value="yellow">Yellow</option>
                        <option value="red">Red</option>
                      </select>
                    </div>
                    <div>
                      <label className="text-[10px] text-grove-text-secondary uppercase tracking-wider">
                        Priority
                      </label>
                      <input
                        type="number"
                        min={1}
                        value={v.priority}
                        onChange={(e) => updateVenture(i, "priority", parseInt(e.target.value) || 1)}
                        className="w-full mt-1 bg-grove-bg border border-grove-border rounded px-2 py-1 text-xs text-grove-text-primary font-mono focus:outline-none"
                      />
                    </div>
                  </div>

                  <div>
                    <label className="text-[10px] text-grove-text-secondary uppercase tracking-wider">
                      Next Action
                    </label>
                    <input
                      value={v.nextAction}
                      onChange={(e) => updateVenture(i, "nextAction", e.target.value)}
                      placeholder="What's the next step?"
                      className="w-full mt-1 bg-grove-bg border border-grove-border rounded px-2 py-1.5 text-xs text-grove-text-primary focus:outline-none"
                    />
                  </div>
                </div>
              ))}

              <button
                onClick={addVenture}
                className="w-full text-xs text-grove-accent hover:text-grove-accent/80 transition-colors py-2 border border-dashed border-grove-border rounded-lg hover:border-grove-accent/40"
              >
                + Add venture
              </button>
            </div>
          )}
        </div>

        <div className="px-6 py-3 border-t border-grove-border">
          <p className="text-[10px] text-grove-text-secondary">
            Your ventures and projects. The reasoning engine uses this to
            prioritize what to show you. Changes take effect on next reasoning cycle.
          </p>
        </div>
      </div>
    </div>
  );
}
