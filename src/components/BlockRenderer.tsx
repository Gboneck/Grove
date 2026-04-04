import type { Block } from "../lib/tauri";
import TextBlock from "./blocks/TextBlock";
import MetricCard from "./blocks/MetricCard";
import ActionList from "./blocks/ActionList";
import StatusRow from "./blocks/StatusRow";
import InsightBlock from "./blocks/InsightBlock";
import InputPrompt from "./blocks/InputPrompt";
import Divider from "./blocks/Divider";

interface BlockRendererProps {
  blocks: Block[];
  onInput: (value: string) => void;
}

export default function BlockRenderer({ blocks, onInput }: BlockRendererProps) {
  return (
    <div className="space-y-6">
      {blocks.map((block, i) => {
        switch (block.type) {
          case "text":
            return (
              <TextBlock
                key={i}
                heading={block.heading as string}
                body={block.body as string}
              />
            );
          case "metric":
            return (
              <MetricCard
                key={i}
                label={block.label as string}
                value={block.value as string}
                trend={block.trend as "up" | "down" | "flat" | null}
              />
            );
          case "actions":
            return (
              <ActionList
                key={i}
                title={block.title as string}
                items={block.items as { action: string; detail: string }[]}
                onAction={(action) =>
                  onInput(`I want to: ${action}`)
                }
              />
            );
          case "status":
            return (
              <StatusRow
                key={i}
                items={
                  block.items as {
                    name: string;
                    status: "green" | "yellow" | "red";
                    detail?: string;
                  }[]
                }
              />
            );
          case "insight":
            return (
              <InsightBlock
                key={i}
                icon={
                  block.icon as "alert" | "opportunity" | "warning" | "idea"
                }
                message={block.message as string}
              />
            );
          case "input":
            return (
              <InputPrompt
                key={i}
                prompt={block.prompt as string}
                placeholder={block.placeholder as string}
                onSubmit={onInput}
              />
            );
          case "divider":
            return <Divider key={i} />;
          default:
            return null;
        }
      })}
    </div>
  );
}
