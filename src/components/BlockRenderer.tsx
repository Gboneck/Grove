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
import SkeletonBlock from "./SkeletonBlock";

interface BlockRendererProps {
  blocks: Block[];
  onInput: (value: string) => void;
  isLoading?: boolean;
}

function BlockWrapper({
  blockType,
  children,
  dismissable = true,
  animate = false,
  staggerDelay,
}: {
  blockType: string;
  children: React.ReactNode;
  dismissable?: boolean;
  animate?: boolean;
  staggerDelay?: string;
}) {
  const [dismissed, setDismissed] = useState(false);
  const [visible, setVisible] = useState(!animate);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (animate) {
      if (staggerDelay) {
        const ms = parseInt(staggerDelay, 10) || 0;
        const timer = setTimeout(() => setVisible(true), ms);
        return () => clearTimeout(timer);
      }
      requestAnimationFrame(() => setVisible(true));
    }
  }, [animate, staggerDelay]);

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

function BlockRenderer({ blocks, onInput, isLoading }: BlockRendererProps) {
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
        // Stagger entrance animation: each new block delays by 80ms
        const staggerDelay = isNew ? `${(i - prevCountRef.current) * 80}ms` : undefined;
        switch (block.type) {
          case "text":
            return (
              <BlockWrapper key={key} blockType="text" animate={isNew} staggerDelay={staggerDelay}>
                <TextBlock
                  heading={block.heading as string}
                  body={block.body as string}
                />
              </BlockWrapper>
            );
          case "metric":
            return (
              <BlockWrapper key={key} blockType="metric" animate={isNew} staggerDelay={staggerDelay}>
                <MetricCard
                  label={block.label as string}
                  value={block.value as string}
                  trend={block.trend as "up" | "down" | "flat" | null}
                />
              </BlockWrapper>
            );
          case "actions":
            return (
              <BlockWrapper key={key} blockType="actions" dismissable={false} animate={isNew} staggerDelay={staggerDelay}>
                <ActionList
                  title={block.title as string}
                  items={block.items as { action: string; detail: string }[]}
                  onAction={(action) => onInput(`I want to: ${action}`)}
                />
              </BlockWrapper>
            );
          case "status":
            return (
              <BlockWrapper key={key} blockType="status" animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="insight" animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="input" dismissable={false} animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="progress" animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="list" animate={isNew} staggerDelay={staggerDelay}>
                <ListBlock
                  heading={block.heading as string | undefined}
                  items={block.items as string[]}
                  ordered={(block.ordered as boolean) ?? false}
                />
              </BlockWrapper>
            );
          case "quote":
            return (
              <BlockWrapper key={key} blockType="quote" animate={isNew} staggerDelay={staggerDelay}>
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
      {/* Skeleton placeholders while model is generating */}
      {isLoading && blocks.length === 0 && (
        <>
          <SkeletonBlock variant="text" />
          <SkeletonBlock variant="metric" />
          <SkeletonBlock variant="status" />
        </>
      )}
    </div>
  );
}

export default memo(BlockRenderer);
