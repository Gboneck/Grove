import { ReactNode, useEffect, useCallback } from "react";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  /** Max width class, defaults to "max-w-2xl" */
  maxWidth?: string;
  /** Backdrop opacity: "dark" (default) or "light" */
  backdrop?: "dark" | "light";
  /** Vertical alignment: "center" (default) or "top" */
  align?: "center" | "top";
}

export default function Modal({
  isOpen,
  onClose,
  title,
  children,
  maxWidth = "max-w-2xl",
  backdrop = "dark",
  align = "center",
}: ModalProps) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    },
    [onClose]
  );

  useEffect(() => {
    if (isOpen) {
      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }
  }, [isOpen, handleKeyDown]);

  if (!isOpen) return null;

  const backdropClass =
    backdrop === "light" ? "bg-black/40" : "bg-black/60";
  const alignClass =
    align === "top" ? "items-start pt-[10vh]" : "items-center";

  return (
    <div
      className={`fixed inset-0 ${backdropClass} backdrop-blur-sm z-50 flex justify-center ${alignClass} p-4`}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <div
        className={`bg-grove-bg border border-grove-border rounded-xl ${maxWidth} w-full max-h-[80vh] flex flex-col`}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-grove-border">
          <h2 className="text-lg font-display text-grove-accent">{title}</h2>
          <button
            onClick={onClose}
            className="text-grove-text-secondary hover:text-grove-text-primary transition-colors text-xl leading-none"
            aria-label={`Close ${title}`}
          >
            &times;
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto">{children}</div>
      </div>
    </div>
  );
}
