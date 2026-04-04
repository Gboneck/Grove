import { useEffect, useState } from "react";

interface ActionLogProps {
  autoActions: string[];
  ventureUpdates: string[];
}

export default function ActionLog({ autoActions, ventureUpdates }: ActionLogProps) {
  const [visible, setVisible] = useState(true);
  const items = [...autoActions, ...ventureUpdates];

  useEffect(() => {
    if (items.length === 0) return;
    setVisible(true);
    const timer = setTimeout(() => setVisible(false), 8000);
    return () => clearTimeout(timer);
  }, [autoActions, ventureUpdates]);

  if (items.length === 0 || !visible) return null;

  return (
    <div className="fixed top-16 right-4 z-20 max-w-sm space-y-2 animate-in">
      {items.map((item, i) => (
        <div
          key={i}
          className="bg-grove-surface border border-grove-accent/30 rounded-lg px-4 py-2.5 text-sm text-grove-text-secondary shadow-lg backdrop-blur-md"
        >
          <span className="text-grove-accent mr-2">~</span>
          {item}
        </div>
      ))}
    </div>
  );
}
