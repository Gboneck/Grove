interface StatusItem {
  name: string;
  status: "green" | "yellow" | "red";
  detail?: string;
}

interface StatusRowProps {
  items: StatusItem[];
}

const statusColors: Record<string, string> = {
  green: "bg-grove-status-green",
  yellow: "bg-grove-status-yellow",
  red: "bg-grove-status-red",
};

export default function StatusRow({ items }: StatusRowProps) {
  return (
    <div className="space-y-2">
      {items.map((item, i) => (
        <div
          key={i}
          className="flex items-center gap-3 bg-grove-surface border border-grove-border rounded-lg px-4 py-3"
        >
          <div
            className={`w-2.5 h-2.5 rounded-full ${statusColors[item.status] || statusColors.yellow}`}
          />
          <span className="font-medium text-grove-text-primary flex-1">
            {item.name}
          </span>
          {item.detail && (
            <span className="text-sm text-grove-text-secondary">
              {item.detail}
            </span>
          )}
        </div>
      ))}
    </div>
  );
}
