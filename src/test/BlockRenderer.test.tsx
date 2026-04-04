import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import BlockRenderer from "../components/BlockRenderer";
import type { Block } from "../lib/tauri";

// Mock recordActionEngagement
vi.mock("../lib/tauri", async () => {
  const actual = await vi.importActual("../lib/tauri");
  return {
    ...actual,
    recordActionEngagement: vi.fn().mockResolvedValue(undefined),
  };
});

describe("BlockRenderer", () => {
  const onInput = vi.fn();

  it("renders empty state without crashing", () => {
    const { container } = render(
      <BlockRenderer blocks={[]} onInput={onInput} />
    );
    expect(container.querySelector(".space-y-6")).toBeInTheDocument();
  });

  it("renders text blocks", () => {
    const blocks: Block[] = [
      { type: "text", heading: "Hello", body: "Welcome to Grove" },
    ];
    render(<BlockRenderer blocks={blocks} onInput={onInput} />);
    expect(screen.getByText("Hello")).toBeInTheDocument();
    expect(screen.getByText("Welcome to Grove")).toBeInTheDocument();
  });

  it("renders multiple block types", () => {
    const blocks: Block[] = [
      { type: "text", heading: "Title", body: "Body text" },
      { type: "insight", icon: "idea", message: "An important insight" },
      { type: "divider" },
    ];
    render(<BlockRenderer blocks={blocks} onInput={onInput} />);
    expect(screen.getByText("Title")).toBeInTheDocument();
    expect(screen.getByText("An important insight")).toBeInTheDocument();
  });

  it("shows skeleton blocks when loading with no content", () => {
    const { container } = render(
      <BlockRenderer blocks={[]} onInput={onInput} isLoading={true} />
    );
    const skeletons = container.querySelectorAll(".animate-pulse");
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it("does not show skeletons when blocks exist", () => {
    const blocks: Block[] = [
      { type: "text", heading: "Title", body: "Content" },
    ];
    const { container } = render(
      <BlockRenderer blocks={blocks} onInput={onInput} isLoading={true} />
    );
    // Skeletons only show when blocks.length === 0
    const skeletons = container.querySelectorAll(".animate-pulse");
    // The block wrapper has transition classes but shouldn't show skeleton pulse
    expect(skeletons.length).toBe(0);
  });

  it("ignores unknown block types", () => {
    const blocks: Block[] = [{ type: "unknown_type" }];
    const { container } = render(
      <BlockRenderer blocks={blocks} onInput={onInput} />
    );
    // Should render the container but no block content
    expect(container.querySelector(".space-y-6")).toBeInTheDocument();
  });
});
