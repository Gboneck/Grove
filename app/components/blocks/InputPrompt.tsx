"use client";

import { useState, KeyboardEvent } from "react";

interface InputPromptProps {
  prompt: string;
  placeholder: string;
  onSubmit: (value: string) => void;
}

export default function InputPrompt({ prompt, placeholder, onSubmit }: InputPromptProps) {
  const [value, setValue] = useState("");

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && value.trim()) {
      onSubmit(value.trim());
      setValue("");
    }
  };

  return (
    <div className="space-y-2">
      <label className="text-sm text-[#888888]">{prompt}</label>
      <div className="flex gap-2">
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 bg-[#141414] border border-[#222222] rounded-lg px-4 py-3 text-[#e5e5e5] placeholder-[#555555] focus:outline-none focus:border-[#d4a853]/60 transition-colors"
        />
        <button
          onClick={() => {
            if (value.trim()) {
              onSubmit(value.trim());
              setValue("");
            }
          }}
          className="bg-[#d4a853] text-[#0a0a0a] px-5 py-3 rounded-lg font-medium hover:bg-[#e0b965] transition-colors"
        >
          Send
        </button>
      </div>
    </div>
  );
}
