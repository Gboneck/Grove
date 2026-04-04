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
      <h3 className="text-sm uppercase tracking-wider text-grove-text-secondary">
        {title}
      </h3>
      <div className="space-y-2">
        {items.map((item, i) => (
          <button
            key={i}
            onClick={() => onAction(item.action)}
            className="w-full text-left bg-grove-surface border border-grove-border rounded-lg p-4 hover:border-grove-accent/40 hover:bg-grove-surface-hover transition-colors cursor-pointer group"
          >
            <div className="flex items-center justify-between">
              <div className="font-medium text-grove-text-primary group-hover:text-grove-accent transition-colors">
                {item.action}
              </div>
              <span className="text-grove-text-secondary opacity-0 group-hover:opacity-100 transition-opacity text-xs">
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
