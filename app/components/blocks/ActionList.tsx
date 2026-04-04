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
      <h3 className="text-sm uppercase tracking-wider text-[#888888]">{title}</h3>
      <div className="space-y-2">
        {items.map((item, i) => (
          <div
            key={i}
            className="bg-[#141414] border border-[#222222] rounded-lg p-4 hover:border-[#d4a853]/40 transition-colors cursor-pointer"
          >
            <div className="font-medium text-[#e5e5e5]">{item.action}</div>
            <div className="text-sm text-[#888888] mt-1">{item.detail}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
