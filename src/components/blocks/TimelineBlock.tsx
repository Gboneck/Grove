interface TimelineEvent {
  time: string;
  label: string;
  detail?: string;
  type?: "action" | "observation" | "insight" | "milestone";
}

interface TimelineBlockProps {
  heading?: string;
  events: TimelineEvent[];
}

const TYPE_COLORS: Record<string, string> = {
  action: "bg-grove-accent",
  observation: "bg-grove-text-secondary",
  insight: "bg-[#c084fc]",
  milestone: "bg-grove-status-green",
};

export default function TimelineBlock({ heading, events }: TimelineBlockProps) {
  return (
    <div>
      {heading && (
        <h3 className="text-[10px] uppercase tracking-widest text-grove-text-secondary font-sans mb-3">
          {heading}
        </h3>
      )}
      <div className="relative pl-4 border-l border-grove-border/40">
        {events.map((event, i) => {
          const dotColor = TYPE_COLORS[event.type || "observation"] || TYPE_COLORS.observation;
          return (
            <div key={i} className="relative pb-4 last:pb-0">
              {/* Dot on the line */}
              <div
                className={`absolute -left-[calc(1rem+3px)] top-1.5 w-1.5 h-1.5 rounded-full ${dotColor}`}
              />
              <div className="flex items-baseline gap-2">
                <span className="text-[10px] font-mono text-grove-text-secondary flex-shrink-0">
                  {event.time}
                </span>
                <span className="text-sm text-grove-text-primary font-sans">
                  {event.label}
                </span>
              </div>
              {event.detail && (
                <p className="text-xs text-grove-text-secondary mt-0.5 ml-[calc(3ch+0.5rem)]">
                  {event.detail}
                </p>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
