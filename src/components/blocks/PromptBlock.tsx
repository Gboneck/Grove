import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PromptBlockProps {
  title: string;
  prompt: string;
  context?: string;
  onCopied?: (title: string, prompt: string) => void;
}

export default function PromptBlock({ title, prompt, context, onCopied }: PromptBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(prompt);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = prompt;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);

    // Record that the user copied this prompt
    if (onCopied) onCopied(title, prompt);

    // Also log to backend
    invoke("record_prompt_copied", {
      title,
      promptPreview: prompt.slice(0, 200),
    }).catch(() => {});
  }, [prompt, title, onCopied]);

  return (
    <div className="rounded-lg overflow-hidden border border-grove-border/50">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-grove-surface/80 border-b border-grove-border/30">
        <div className="flex items-center gap-2">
          <span className="text-[10px] uppercase tracking-widest text-grove-accent font-sans">
            Claude Code Prompt
          </span>
          <span className="text-xs text-grove-text-secondary font-sans">
            {title}
          </span>
        </div>
        <button
          onClick={handleCopy}
          className={`text-xs px-3 py-1 rounded font-mono transition-all duration-200 ${
            copied
              ? "bg-grove-status-green/20 text-grove-status-green"
              : "bg-grove-accent/10 text-grove-accent hover:bg-grove-accent/20"
          }`}
        >
          {copied ? "copied — paste in Claude Code" : "copy"}
        </button>
      </div>

      {/* Context line (optional) */}
      {context && (
        <div className="px-4 py-2 bg-grove-surface/40 border-b border-grove-border/20">
          <p className="text-xs text-grove-text-secondary italic">{context}</p>
        </div>
      )}

      {/* Prompt body */}
      <div className="px-4 py-3 bg-grove-bg/50">
        <pre className="text-sm text-grove-text-primary font-mono whitespace-pre-wrap leading-relaxed">
          {prompt}
        </pre>
      </div>
    </div>
  );
}
