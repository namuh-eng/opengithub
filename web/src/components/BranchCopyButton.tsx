"use client";

import { useState } from "react";

type BranchCopyButtonProps = {
  branch: string;
};

export function BranchCopyButton({ branch }: BranchCopyButtonProps) {
  const [copied, setCopied] = useState(false);

  async function copyBranch() {
    try {
      await navigator.clipboard.writeText(branch);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setCopied(false);
    }
  }

  return (
    <button
      aria-label={
        copied ? `Copied branch name ${branch}` : `Copy branch name ${branch}`
      }
      className="btn sm ghost"
      onClick={copyBranch}
      type="button"
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}
