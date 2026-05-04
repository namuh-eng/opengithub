"use client";

import Link from "next/link";
import type { KeyboardEvent } from "react";
import { useEffect, useId, useRef, useState } from "react";

type BranchActionsMenuProps = {
  activityHref: string;
  treeHref: string;
  commitsHref: string;
  rulesHref: string;
  canViewRules: boolean;
  canDelete: boolean;
  deleteDisabledReason: string | null;
  restoreDisabledReason: string | null;
};

export function BranchActionsMenu({
  activityHref,
  treeHref,
  commitsHref,
  rulesHref,
  canViewRules,
  canDelete,
  deleteDisabledReason,
  restoreDisabledReason,
}: BranchActionsMenuProps) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    function closeOnOutsideClick(event: MouseEvent) {
      if (!wrapperRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", closeOnOutsideClick);
    return () => document.removeEventListener("mousedown", closeOnOutsideClick);
  }, [open]);

  function onKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "Escape") {
      setOpen(false);
    }
  }

  return (
    <div
      className="relative justify-self-start lg:justify-self-end"
      ref={wrapperRef}
    >
      <button
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        aria-haspopup="menu"
        className="btn sm ghost"
        onKeyDown={onKeyDown}
        onClick={() => setOpen((value) => !value)}
        type="button"
      >
        Actions
      </button>
      {open ? (
        <div
          className="card absolute right-0 z-10 mt-2 grid min-w-52 gap-1 p-2"
          id={menuId}
          style={{ background: "var(--surface)" }}
        >
          <Link className="btn sm ghost justify-start" href={activityHref}>
            Activity
          </Link>
          {canViewRules ? (
            <Link className="btn sm ghost justify-start" href={rulesHref}>
              View rules
            </Link>
          ) : (
            <span className="chip soft justify-start">No visible rules</span>
          )}
          <Link className="btn sm ghost justify-start" href={treeHref}>
            Open tree
          </Link>
          <Link className="btn sm ghost justify-start" href={commitsHref}>
            Open commits
          </Link>
          <button
            className="btn sm ghost justify-start"
            disabled
            title={
              deleteDisabledReason ??
              (canDelete
                ? "Branch deletion is handled by a later mutation phase."
                : "Branch deletion is not available for this branch.")
            }
            type="button"
          >
            Delete branch
          </button>
          {restoreDisabledReason ? (
            <button
              className="btn sm ghost justify-start"
              disabled
              title={restoreDisabledReason}
              type="button"
            >
              Restore branch
            </button>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
