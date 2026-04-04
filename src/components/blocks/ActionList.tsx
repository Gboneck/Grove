interface ActionItem {
  action: string;
  detail: string;
}

interface ActionListProps {
  title: string;
  items: ActionItem[];
}

export default function ActionList({ title, items }: ActionListProps) {
  return (
    <div className="space-y-3">
      <h3 className="text-sm uppercase tracking-wider text-grove-text-secondary">
        {title}
      </h3>
      <div className="space-y-2">
        {items.map((item, i) => (
          <div
            key={i}
            className="bg-grove-surface border border-grove-border rounded-lg p-4 hover:border-grove-accent/40 hover:bg-grove-surface-hover transition-colors cursor-pointer"
          >
            <div className="font-medium text-grove-text-primary">
              {item.action}
            </div>
            <div className="text-sm text-grove-text-secondary mt-1">
              {item.detail}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
