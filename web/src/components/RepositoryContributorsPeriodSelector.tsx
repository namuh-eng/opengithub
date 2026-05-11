"use client";

import { useEffect, useId, useRef, useState } from "react";
import { repositoryContributorsHref } from "@/lib/navigation";

const PERIODS = [
  { key: "24h", label: "Last 24 hours" },
  { key: "3d", label: "Last 3 days" },
  { key: "1w", label: "Last week" },
  { key: "1m", label: "Last month" },
] as const;

type RepositoryContributorsPeriodSelectorProps = {
  owner: string;
  repo: string;
  activePeriod: string;
  start?: string | null;
  end?: string | null;
};

export function RepositoryContributorsPeriodSelector({
  owner,
  repo,
  activePeriod,
  start,
  end,
}: RepositoryContributorsPeriodSelectorProps) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const wrapperRef = useRef<HTMLDivElement>(null);
  const active =
    PERIODS.find((period) => period.key === activePeriod) ?? PERIODS[2];

  useEffect(() => {
    if (!open) return;

    function closeOnOutsideClick(event: MouseEvent) {
      if (!wrapperRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", closeOnOutsideClick);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("mousedown", closeOnOutsideClick);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [open]);

  return (
    <div className="relative" ref={wrapperRef}>
      <button
        aria-controls={open ? menuId : undefined}
        aria-expanded={open}
        aria-haspopup="menu"
        className="btn"
        onClick={() => setOpen((value) => !value)}
        type="button"
      >
        Period: {active.label}
      </button>
      {open ? (
        <div
          aria-label="Contributors period"
          className="card absolute right-0 z-20 mt-2 grid min-w-56 gap-1 p-2"
          id={menuId}
          role="menu"
          style={{ background: "var(--surface)" }}
        >
          {PERIODS.map((period) => {
            const selected = period.key === active.key;
            return (
              <a
                aria-current={selected ? "page" : undefined}
                className={`btn sm ghost justify-start ${selected ? "active" : ""}`}
                href={repositoryContributorsHref(owner, repo, {
                  period: period.key,
                  start,
                  end,
                })}
                key={period.key}
                onClick={() => setOpen(false)}
                role="menuitem"
                style={{
                  background: selected ? "var(--surface-2)" : undefined,
                  color: selected ? "var(--ink-1)" : undefined,
                }}
              >
                {period.label}
                {selected ? (
                  <span className="chip active ml-auto">Selected</span>
                ) : null}
              </a>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
