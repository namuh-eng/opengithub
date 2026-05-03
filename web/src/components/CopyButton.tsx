"use client";

import { useState } from "react";

type CopyButtonProps = {
  value: string;
  label?: string;
  copiedLabel?: string;
  className?: string;
};

export function CopyButton({
  value,
  label = "Copy",
  copiedLabel = "Copied",
  className,
}: CopyButtonProps) {
  const [status, setStatus] = useState<string | null>(null);

  function copyWithSelectionFallback() {
    const textarea = document.createElement("textarea");
    textarea.value = value;
    textarea.setAttribute("readonly", "true");
    textarea.style.left = "-9999px";
    textarea.style.position = "fixed";
    document.body.append(textarea);
    textarea.select();
    const copied = document.execCommand("copy");
    textarea.remove();
    if (!copied) {
      throw new Error("copy_failed");
    }
  }

  async function copy() {
    try {
      if (navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(value);
        } catch {
          copyWithSelectionFallback();
        }
      } else {
        copyWithSelectionFallback();
      }
      setStatus(copiedLabel);
    } catch {
      setStatus("Copy unavailable");
    }
  }

  return (
    <div className="flex items-center gap-2">
      <button
        className={className ?? "btn ghost sm"}
        onClick={copy}
        type="button"
      >
        {label}
      </button>
      {status ? (
        <span
          className="t-xs font-medium"
          style={{ color: "var(--ok)" }}
          role="status"
        >
          {status}
        </span>
      ) : null}
    </div>
  );
}
