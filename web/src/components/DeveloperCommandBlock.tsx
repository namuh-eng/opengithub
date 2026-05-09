import { CopyButton } from "@/components/CopyButton";

type DeveloperCommandBlockProps = {
  label: string;
  value: string;
  copyLabel?: string;
};

export function DeveloperCommandBlock({
  label,
  value,
  copyLabel = "Copy",
}: DeveloperCommandBlockProps) {
  return (
    <div className="min-w-0">
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          {label}
        </p>
        <CopyButton label={copyLabel} value={value} />
      </div>
      <section
        aria-label={`${label} command`}
        className="max-w-full overflow-x-auto rounded-md"
        // biome-ignore lint/a11y/noNoninteractiveTabindex: horizontally scrollable command snippets must be keyboard-focusable for Safari/axe.
        tabIndex={0}
        style={{
          border: "1px solid var(--line)",
          background: "var(--surface-2)",
          color: "var(--ink-1)",
        }}
      >
        <pre className="t-mono-sm p-3 leading-5">{value}</pre>
      </section>
    </div>
  );
}
