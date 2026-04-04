export default function LoadingState() {
  return (
    <div className="flex flex-col items-center justify-center py-32 space-y-6">
      <div className="relative">
        <div className="w-4 h-4 rounded-full bg-grove-accent animate-pulse" />
        <div className="absolute inset-0 w-4 h-4 rounded-full bg-grove-accent/30 animate-ping" />
      </div>
      <p className="text-grove-text-secondary text-sm tracking-wide">
        reasoning…
      </p>
    </div>
  );
}
