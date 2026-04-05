interface ActionItem {
  action: string;
  detail: string;
}

interface ActionListProps {
  title: string;
  items: ActionItem[];
  onAction: (action: string) => void;
}

export default function ActionList({ title, items, onAction }: ActionListProps) {
  return (
    <div className="space-y-3">
      <h3 className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans">
        {title}
      </h3>
      <div className="space-y-2">
        {items.map((item, i) => (
          <button
            key={i}
            onClick={() => onAction(item.action)}
            className="w-full text-left bg-grove-surface/50 border border-grove-border/50 rounded-lg p-4 hover:border-grove-accent/30 hover:bg-grove-surface hover:shadow-lg hover:shadow-black/20 hover:-translate-y-0.5 active:translate-y-0 active:shadow-none transition-all duration-200 cursor-pointer group"
          >
            <div className="flex items-center justify-between">
              <div className="font-medium text-grove-text-primary group-hover:text-grove-accent transition-colors text-[15px]">
                {item.action}
              </div>
              <span className="text-grove-accent/40 group-hover:text-grove-accent group-hover:translate-x-0.5 transition-all text-sm">
                →
              </span>
            </div>
            <div className="text-sm text-grove-text-secondary mt-1">
              {item.detail}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
