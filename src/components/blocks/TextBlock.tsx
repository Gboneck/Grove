interface TextBlockProps {
  heading: string;
  body: string;
}

export default function TextBlock({ heading, body }: TextBlockProps) {
  return (
    <div className="space-y-2">
      <h2 className="text-lg font-semibold text-grove-text-primary font-serif">
        {heading}
      </h2>
      <p className="text-grove-text-secondary leading-relaxed">{body}</p>
    </div>
  );
}
