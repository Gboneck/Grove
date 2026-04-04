interface ListBlockProps {
  heading?: string;
  items: string[];
  ordered?: boolean;
}

export default function ListBlock({
  heading,
  items,
  ordered = false,
}: ListBlockProps) {
  const Tag = ordered ? "ol" : "ul";

  return (
    <div className="space-y-2">
      {heading && (
        <h3 className="text-sm uppercase tracking-wider text-grove-text-secondary">
          {heading}
        </h3>
      )}
      <Tag
        className={`space-y-1.5 ${ordered ? "list-decimal" : "list-disc"} list-inside text-grove-text-primary`}
      >
        {items.map((item, i) => (
          <li key={i} className="leading-relaxed text-sm">
            <span className="text-grove-text-secondary">{item}</span>
          </li>
        ))}
      </Tag>
    </div>
  );
}
