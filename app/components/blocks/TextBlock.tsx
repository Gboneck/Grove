interface TextBlockProps {
  heading: string;
  body: string;
}

export default function TextBlock({ heading, body }: TextBlockProps) {
  return (
    <div className="space-y-2">
      <h2 className="text-lg font-semibold text-[#e5e5e5] font-serif">{heading}</h2>
      <p className="text-[#888888] leading-relaxed">{body}</p>
    </div>
  );
}
