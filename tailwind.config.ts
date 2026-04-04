/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        grove: {
          bg: "#0a0a0a",
          surface: "#141414",
          "surface-hover": "#1a1a1a",
          border: "#222222",
          "text-primary": "#e5e5e5",
          "text-secondary": "#888888",
          accent: "#d4a853",
          "accent-dim": "rgba(212, 168, 83, 0.2)",
          "status-green": "#4ade80",
          "status-yellow": "#facc15",
          "status-red": "#f87171",
          "model-local": "#4ade80",
          "model-cloud": "#60a5fa",
          "model-offline": "#6b7280",
        },
      },
      fontFamily: {
        display: ["Instrument Serif", "ui-serif", "Georgia", "serif"],
        sans: ["Syne", "Inter", "ui-sans-serif", "system-ui", "-apple-system", "sans-serif"],
        serif: ["Instrument Serif", "ui-serif", "Georgia", "Times New Roman", "serif"],
        mono: ["JetBrains Mono", "ui-monospace", "SF Mono", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
