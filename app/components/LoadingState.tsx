export default function LoadingState() {
  return (
    <div className="flex flex-col items-center justify-center py-32 space-y-6">
      {/* Pulsing amber orb */}
      <div className="relative">
        <div className="w-4 h-4 rounded-full bg-[#d4a853] animate-pulse" />
        <div className="absolute inset-0 w-4 h-4 rounded-full bg-[#d4a853]/30 animate-ping" />
      </div>
      <p className="text-[#888888] text-sm tracking-wide">thinking…</p>
    </div>
  );
}
