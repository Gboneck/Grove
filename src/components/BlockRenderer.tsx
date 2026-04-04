import { useState, useCallback } from "react";
import type { Block } from "../lib/tauri";
import { recordActionEngagement } from "../lib/tauri";
import TextBlock from "./blocks/TextBlock";
import MetricCard from "./blocks/MetricCard";
import ActionList from "./blocks/ActionList";
import StatusRow from "./blocks/StatusRow";
import InsightBlock from "./blocks/InsightBlock";
import InputPrompt from "./blocks/InputPrompt";
import Divider from "./blocks/Divider";
import ProgressBlock from "./blocks/ProgressBlock";
import ListBlock from "./blocks/ListBlock";
import QuoteBlock from "./blocks/QuoteBlock";

interface BlockRendererProps {
  blocks: Block[];
  onInput: (value: string) => void;
}

function BlockWrapper({
  blockType,
  children,
  dismissable = true,
}: {
  blockType: string;
  children: React.ReactNode;
  dismissable?: boolean;
}) {
  const [dismissed, setDismissed] = useState(false);

  const handleDismiss = useCallback(() => {
    setDismissed(true);
    // Track that user dismissed this block type
    recordActionEngagement(blockType, false).catch(() => {});
  }, [blockType]);

  if (dismissed) return null;

  return (
    <div className="group relative">
      {children}
      {dismissable && (
        <button
          onClick={handleDismiss}
          className="absolute -top-1.5 -right-1.5 w-5 h-5 rounded-full bg-grove-border text-grove-text-secondary text-xs flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-grove-status-red hover:text-white"
          title="Dismiss"
        >
          ×
        </button>
      )}
    </div>
  );
}

export default function BlockRenderer({ blocks, onInput }: BlockRendererProps) {
  return (
    <div className="space-y-6">
      {blocks.map((block, i) => {
        const key = `${block.type}-${i}`;
        switch (block.type) {
          case "text":
            return (
              <BlockWrapper key={key} blockType="text">
                <TextBlock
                  heading={block.heading as string}
                  body={block.body as string}
                />
              </BlockWrapper>
            );
          case "metric":
            return (
              <BlockWrapper key={key} blockType="metric">
                <MetricCard
                  label={block.label as string}
                  value={block.value as string}
                  trend={block.trend as "up" | "down" | "flat" | null}
                />
              </BlockWrapper>
            );
          case "actions":
            return (
              <BlockWrapper key={key} blockType="actions" dismissable={false}>
                <ActionList
                  title={block.title as string}
                  items={block.items as { action: string; detail: string }[]}
                  onAction={(action) => onInput(`I want to: ${action}`)}
                />
              </BlockWrapper>
            );
          case "status":
            return (
              <BlockWrapper key={key} blockType="status">
                <StatusRow
                  items={
                    block.items as {
                      name: string;
                      status: "green" | "yellow" | "red";
                      detail?: string;
                    }[]
                  }
                />
              </BlockWrapper>
            );
          case "insight":
            return (
              <BlockWrapper key={key} blockType="insight">
                <InsightBlock
                  icon={
                    block.icon as "alert" | "opportunity" | "warning" | "idea"
                  }
                  message={block.message as string}
                />
              </BlockWrapper>
            );
          case "input":
            return (
              <BlockWrapper key={key} blockType="input" dismissable={false}>
                <InputPrompt
                  prompt={block.prompt as string}
                  placeholder={block.placeholder as string}
                  onSubmit={onInput}
                />
              </BlockWrapper>
            );
          case "divider":
            return <Divider key={key} />;
          case "progress":
            return (
              <BlockWrapper key={key} blockType="progress">
                <ProgressBlock
                  label={block.label as string}
                  value={block.value as number}
                  max={(block.max as number) ?? 100}
                  detail={block.detail as string | undefined}
                />
              </BlockWrapper>
            );
          case "list":
            return (
              <BlockWrapper key={key} blockType="list">
                <ListBlock
                  heading={block.heading as string | undefined}
                  items={block.items as string[]}
                  ordered={(block.ordered as boolean) ?? false}
                />
              </BlockWrapper>
            );
          case "quote":
            return (
              <BlockWrapper key={key} blockType="quote">
                <QuoteBlock
                  text={block.text as string}
                  attribution={block.attribution as string | undefined}
                />
              </BlockWrapper>
            );
          default:
            return null;
        }
      })}
    </div>
  );
}
