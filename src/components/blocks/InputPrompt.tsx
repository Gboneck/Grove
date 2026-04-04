import { useState, KeyboardEvent } from "react";

interface InputPromptProps {
  prompt: string;
  placeholder: string;
  onSubmit: (value: string) => void;
}

export default function InputPrompt({
  prompt,
  placeholder,
  onSubmit,
}: InputPromptProps) {
  const [value, setValue] = useState("");

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && value.trim()) {
      onSubmit(value.trim());
      setValue("");
    }
  };

  return (
    <div className="space-y-2">
      <label className="text-sm text-grove-text-secondary">{prompt}</label>
      <div className="flex gap-2">
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 bg-grove-surface border border-grove-border rounded-lg px-4 py-3 text-grove-text-primary placeholder-gray-600 focus:outline-none focus:border-grove-accent/60 transition-colors"
        />
        <button
          onClick={() => {
            if (value.trim()) {
              onSubmit(value.trim());
              setValue("");
            }
          }}
          className="bg-grove-accent text-grove-bg px-5 py-3 rounded-lg font-medium hover:brightness-110 transition-all"
        >
          Send
        </button>
      </div>
    </div>
  );
}
