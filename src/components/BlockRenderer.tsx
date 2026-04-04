import { useState, useCallback, useEffect, useRef, memo } from "react";
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
  animate = false,
}: {
  blockType: string;
  children: React.ReactNode;
  dismissable?: boolean;
  animate?: boolean;
}) {
  const [dismissed, setDismissed] = useState(false);
  const [visible, setVisible] = useState(!animate);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (animate) {
      // Trigger fade-in on next frame
      requestAnimationFrame(() => setVisible(true));
    }
  }, [animate]);

  const handleDismiss = useCallback(() => {
    setDismissed(true);
    recordActionEngagement(blockType, false).catch(() => {});
  }, [blockType]);

  if (dismissed) return null;

  return (
    <div
      ref={ref}
      className={`group relative transition-all duration-500 ${
        visible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-2"
      }`}
    >
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

function BlockRenderer({ blocks, onInput }: BlockRendererProps) {
  // Track the previous block count to know which are newly streamed
  const prevCountRef = useRef(0);
  const isStreaming = blocks.length !== prevCountRef.current;
  useEffect(() => {
    prevCountRef.current = blocks.length;
  }, [blocks.length]);

  return (
    <div className="space-y-6">
      {blocks.map((block, i) => {
        const key = `${block.type}-${i}`;
        const isNew = isStreaming && i >= prevCountRef.current;
        switch (block.type) {
          case "text":
            return (
              <BlockWrapper key={key} blockType="text" animate={isNew}>
                <TextBlock
                  heading={block.heading as string}
                  body={block.body as string}
                />
              </BlockWrapper>
            );
          case "metric":
            return (
              <BlockWrapper key={key} blockType="metric" animate={isNew}>
                <MetricCard
                  label={block.label as string}
                  value={block.value as string}
                  trend={block.trend as "up" | "down" | "flat" | null}
                />
              </BlockWrapper>
            );
          case "actions":
            return (
              <BlockWrapper key={key} blockType="actions" dismissable={false} animate={isNew}>
                <ActionList
                  title={block.title as string}
                  items={block.items as { action: string; detail: string }[]}
                  onAction={(action) => onInput(`I want to: ${action}`)}
                />
              </BlockWrapper>
            );
          case "status":
            return (
              <BlockWrapper key={key} blockType="status" animate={isNew}>
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
              <BlockWrapper key={key} blockType="insight" animate={isNew}>
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
              <BlockWrapper key={key} blockType="input" dismissable={false} animate={isNew}>
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
              <BlockWrapper key={key} blockType="progress" animate={isNew}>
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
              <BlockWrapper key={key} blockType="list" animate={isNew}>
                <ListBlock
                  heading={block.heading as string | undefined}
                  items={block.items as string[]}
                  ordered={(block.ordered as boolean) ?? false}
                />
              </BlockWrapper>
            );
          case "quote":
            return (
              <BlockWrapper key={key} blockType="quote" animate={isNew}>
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

export default memo(BlockRenderer);
