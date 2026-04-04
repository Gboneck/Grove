import { render } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import SkeletonBlock from "../components/SkeletonBlock";

describe("SkeletonBlock", () => {
  it("renders text skeleton by default", () => {
    const { container } = render(<SkeletonBlock />);
    const pulseElements = container.querySelectorAll(".animate-pulse");
    expect(pulseElements.length).toBeGreaterThan(0);
  });

  it("renders metric skeleton variant", () => {
    const { container } = render(<SkeletonBlock variant="metric" />);
    const pulseElements = container.querySelectorAll(".animate-pulse");
    expect(pulseElements.length).toBeGreaterThan(0);
    // Metric skeleton has a border
    const bordered = container.querySelector(".border");
    expect(bordered).toBeInTheDocument();
  });

  it("renders status skeleton variant with 3 columns", () => {
    const { container } = render(<SkeletonBlock variant="status" />);
    const items = container.querySelectorAll(".flex-1");
    expect(items.length).toBe(3);
  });
});
