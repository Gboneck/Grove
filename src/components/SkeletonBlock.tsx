import { memo } from "react";

/** Skeleton placeholder shown while the model is generating blocks. */
function SkeletonBlock({ variant = "text" }: { variant?: "text" | "metric" | "status" }) {
  if (variant === "metric") {
    return (
      <div className="animate-pulse rounded-lg border border-grove-border bg-grove-surface p-4">
        <div className="h-3 w-20 bg-grove-border rounded mb-3" />
        <div className="h-6 w-16 bg-grove-border rounded mb-2" />
        <div className="h-2 w-12 bg-grove-border/60 rounded" />
      </div>
    );
  }

  if (variant === "status") {
    return (
      <div className="animate-pulse flex gap-4">
        {[1, 2, 3].map((i) => (
          <div key={i} className="flex-1 rounded-lg border border-grove-border bg-grove-surface p-3">
            <div className="h-3 w-16 bg-grove-border rounded mb-2" />
            <div className="h-2 w-10 bg-grove-border/60 rounded" />
          </div>
        ))}
      </div>
    );
  }

  // Default: text skeleton
  return (
    <div className="animate-pulse space-y-3">
      <div className="h-4 w-48 bg-grove-border rounded" />
      <div className="space-y-2">
        <div className="h-3 w-full bg-grove-border/60 rounded" />
        <div className="h-3 w-5/6 bg-grove-border/60 rounded" />
        <div className="h-3 w-3/4 bg-grove-border/40 rounded" />
      </div>
    </div>
  );
}

export default memo(SkeletonBlock);
