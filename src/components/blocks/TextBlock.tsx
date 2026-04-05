interface TextBlockProps {
  heading: string;
  body: string;
}

export default function TextBlock({ heading, body }: TextBlockProps) {
  return (
    <div className="space-y-3">
      <h2 className="text-xl font-semibold text-grove-text-primary font-serif tracking-tight">
        {heading}
      </h2>
      <p className="text-grove-text-secondary leading-relaxed text-[15px]">{body}</p>
    </div>
  );
}
