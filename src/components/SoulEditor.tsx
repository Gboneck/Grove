import { useState, useEffect } from "react";
import { readSoul, writeSoul } from "../lib/tauri";

interface SoulEditorProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SoulEditor({ isOpen, onClose }: SoulEditorProps) {
  const [content, setContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (isOpen) {
      readSoul()
        .then(setContent)
        .catch((e) => console.error("Failed to read soul.md:", e));
      setSaved(false);
    }
  }, [isOpen]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await writeSoul(content);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save soul.md:", e);
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Editor */}
      <div className="relative bg-grove-bg border border-grove-border rounded-lg w-full max-w-2xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-grove-border">
          <div className="flex items-center gap-3">
            <h2 className="text-sm font-medium text-grove-text-primary">
              Soul.md
            </h2>
            <span className="text-xs text-grove-text-secondary">
              ~/.grove/soul.md
            </span>
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
              className="text-grove-text-secondary hover:text-grove-text-primary transition-colors text-lg leading-none"
            >
              ×
            </button>
          </div>
        </div>

        {/* Textarea */}
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="flex-1 bg-grove-surface m-3 rounded-lg p-4 text-grove-text-primary text-sm font-mono leading-relaxed resize-none focus:outline-none focus:ring-1 focus:ring-grove-accent/30 min-h-[400px]"
          spellCheck={false}
        />

        {/* Footer hint */}
        <div className="px-5 py-3 border-t border-grove-border">
          <p className="text-xs text-grove-text-secondary">
            This is your identity document. The more context you provide, the
            better Grove understands you. Changes take effect on the next
            reasoning cycle.
          </p>
        </div>
      </div>
    </div>
  );
}
