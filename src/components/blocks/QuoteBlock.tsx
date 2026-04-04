interface QuoteBlockProps {
  text: string;
  attribution?: string;
}

export default function QuoteBlock({ text, attribution }: QuoteBlockProps) {
  return (
    <blockquote className="border-l-2 border-grove-accent/50 pl-4 py-1 space-y-2">
      <p className="text-grove-text-primary italic leading-relaxed">{text}</p>
      {attribution && (
        <cite className="text-xs text-grove-text-secondary not-italic">
          — {attribution}
        </cite>
      )}
    </blockquote>
  );
}
