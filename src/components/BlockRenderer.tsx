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
import TimelineBlock from "./blocks/TimelineBlock";
import PromptBlock from "./blocks/PromptBlock";
import SkeletonBlock from "./SkeletonBlock";

interface BlockRendererProps {
  blocks: Block[];
  onInput: (value: string) => void;
  onDismissBlock?: (id: string) => void;
  isLoading?: boolean;
}

function BlockWrapper({
  blockType,
  blockId,
  blockState,
  children,
  dismissable = true,
  animate = false,
  staggerDelay,
  onDismiss,
}: {
  blockType: string;
  blockId?: string;
  blockState?: string;
  children: React.ReactNode;
  dismissable?: boolean;
  animate?: boolean;
  staggerDelay?: string;
  onDismiss?: (id: string) => void;
}) {
  const [dismissing, setDismissing] = useState(false);
  const [visible, setVisible] = useState(!animate);
  const ref = useRef<HTMLDivElement>(null);

  // Type-specific entrance animation
  const entranceClass = animate && visible ? (() => {
    switch (blockType) {
      case "metric": return "animate-[block-enter-scale_400ms_ease-out]";
      case "insight": return "animate-[block-enter-glow_600ms_ease-out] animate-insight-pulse";
      case "actions":
      case "prompt": return "animate-[block-enter-slide_400ms_ease-out]";
      default: return "animate-[block-enter-slide_350ms_ease-out]";
    }
  })() : "";

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
    setDismissing(true);
    recordActionEngagement(blockType, false).catch(() => {});
    if (blockId && onDismiss) {
      setTimeout(() => onDismiss(blockId), 300);
    }
  }, [blockType, blockId, onDismiss]);

  return (
    <div
      ref={ref}
      className={`group relative ${
        dismissing ? "animate-block-dismiss" : ""
      } ${
        visible ? `opacity-100 ${entranceClass}` : "opacity-0"
      }`}
      style={!visible && !dismissing ? { transform: "translateY(8px)" } : undefined}
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

function BlockRenderer({ blocks, onInput, onDismissBlock, isLoading }: BlockRendererProps) {
  // Track the previous block count to know which are newly streamed
  const prevCountRef = useRef(0);
  const isStreaming = blocks.length !== prevCountRef.current;
  useEffect(() => {
    prevCountRef.current = blocks.length;
  }, [blocks.length]);

  const bid = (block: Block) => (block as Record<string, unknown>)._id as string | undefined;

  return (
    <div className="space-y-6">
      {blocks.map((block, i) => {
        const key = (block as Record<string, unknown>)._id as string || `${block.type}-${i}`;
        const isNew = isStreaming && i >= prevCountRef.current;
        const staggerDelay = isNew ? `${(i - prevCountRef.current) * 80}ms` : undefined;
        const id = bid(block);
        const dismiss = onDismissBlock;
        const bstate = undefined;
        switch (block.type) {
          case "text":
            return (
              <BlockWrapper key={key} blockType="text" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <TextBlock
                  heading={block.heading as string}
                  body={block.body as string}
                />
              </BlockWrapper>
            );
          case "metric":
            return (
              <BlockWrapper key={key} blockType="metric" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <MetricCard
                  label={block.label as string}
                  value={block.value as string}
                  trend={block.trend as "up" | "down" | "flat" | null}
                />
              </BlockWrapper>
            );
          case "actions":
            return (
              <BlockWrapper key={key} blockType="actions" blockId={id} blockState={bstate} onDismiss={dismiss} dismissable={false} animate={isNew} staggerDelay={staggerDelay}>
                <ActionList
                  title={block.title as string}
                  items={block.items as { action: string; detail: string }[]}
                  onAction={(action) => {
                    onInput(`I want to: ${action}`);
                    // Remove the action block after user picks
                    if (id && dismiss) setTimeout(() => dismiss(id), 100);
                  }}
                />
              </BlockWrapper>
            );
          case "status":
            return (
              <BlockWrapper key={key} blockType="status" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="insight" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="input" blockId={id} blockState={bstate} onDismiss={dismiss} dismissable={false} animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="progress" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
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
              <BlockWrapper key={key} blockType="list" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <ListBlock
                  heading={block.heading as string | undefined}
                  items={block.items as string[]}
                  ordered={(block.ordered as boolean) ?? false}
                />
              </BlockWrapper>
            );
          case "quote":
            return (
              <BlockWrapper key={key} blockType="quote" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <QuoteBlock
                  text={block.text as string}
                  attribution={block.attribution as string | undefined}
                />
              </BlockWrapper>
            );
          case "timeline":
            return (
              <BlockWrapper key={key} blockType="timeline" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <TimelineBlock
                  heading={block.heading as string | undefined}
                  events={block.events as { time: string; label: string; detail?: string; type?: string }[]}
                />
              </BlockWrapper>
            );
          case "prompt":
            return (
              <BlockWrapper key={key} blockType="prompt" blockId={id} blockState={bstate} onDismiss={dismiss} animate={isNew} staggerDelay={staggerDelay}>
                <PromptBlock
                  title={block.title as string}
                  prompt={block.prompt as string}
                  context={block.context as string | undefined}
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
