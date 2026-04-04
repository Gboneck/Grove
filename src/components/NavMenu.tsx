import { useState, useRef, useEffect } from "react";

interface NavItem {
  label: string;
  action: () => void;
}

interface NavMenuProps {
  items: NavItem[];
}

export default function NavMenu({ items }: NavMenuProps) {
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open]);

  return (
    <div className="relative" ref={menuRef}>
      <button
        onClick={() => setOpen((v) => !v)}
        className="text-xs text-grove-text-secondary hover:text-grove-accent transition-colors px-2 py-1 rounded border border-grove-border hover:border-grove-accent/40"
      >
        menu
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-1 bg-grove-bg border border-grove-border rounded-lg shadow-xl overflow-hidden min-w-[140px] z-20">
          {items.map((item) => (
            <button
              key={item.label}
              onClick={() => {
                setOpen(false);
                item.action();
              }}
              className="w-full text-left px-4 py-2 text-sm text-grove-text-secondary hover:text-grove-accent hover:bg-grove-surface transition-colors"
            >
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
