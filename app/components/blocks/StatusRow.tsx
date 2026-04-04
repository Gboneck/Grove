interface StatusItem {
  name: string;
  status: "green" | "yellow" | "red";
  detail?: string;
}

interface StatusRowProps {
  items: StatusItem[];
}

const statusColors: Record<string, string> = {
  green: "bg-[#4ade80]",
  yellow: "bg-[#facc15]",
  red: "bg-[#f87171]",
};

export default function StatusRow({ items }: StatusRowProps) {
  return (
    <div className="space-y-2">
      {items.map((item, i) => (
        <div
          key={i}
          className="flex items-center gap-3 bg-[#141414] border border-[#222222] rounded-lg px-4 py-3"
        >
          <div className={`w-2.5 h-2.5 rounded-full ${statusColors[item.status] || statusColors.yellow}`} />
          <span className="font-medium text-[#e5e5e5] flex-1">{item.name}</span>
          {item.detail && (
            <span className="text-sm text-[#888888]">{item.detail}</span>
          )}
        </div>
      ))}
    </div>
  );
}
