"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import { repositoryForksHref } from "@/lib/navigation";

const PERIODS = [
  { key: "24h", label: "Last 24 hours" },
  { key: "3d", label: "Last 3 days" },
  { key: "1w", label: "Last week" },
  { key: "1m", label: "Last month" },
  { key: "all", label: "All time" },
] as const;

const TYPES = [
  { key: "all", label: "All repositories" },
  { key: "active", label: "Active" },
  { key: "inactive", label: "Inactive" },
  { key: "archived", label: "Archived" },
  { key: "starred", label: "Starred by you" },
] as const;

const SORTS = [
  { key: "most_starred", label: "Most starred" },
  { key: "recently_pushed", label: "Recently pushed" },
  { key: "recently_created", label: "Recently created" },
  { key: "recently_updated", label: "Recently updated" },
  { key: "name", label: "Name" },
] as const;

type RepositoryForkFiltersProps = {
  owner: string;
  repo: string;
  period: string;
  repositoryType: string;
  sort: string;
  defaultsMatch: boolean;
  defaultsSaved: boolean;
};

function labelFor(
  options: readonly { key: string; label: string }[],
  key: string,
) {
  return options.find((option) => option.key === key)?.label ?? key;
}

function FilterMenu({
  label,
  options,
  selected,
  hrefFor,
}: {
  label: string;
  options: readonly { key: string; label: string }[];
  selected: string;
  hrefFor: (key: string) => string;
}) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;

    function closeOnOutsideClick(event: MouseEvent) {
      if (!wrapperRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
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
        {label}: {labelFor(options, selected)}
      </button>
      {open ? (
        <div
          aria-label={`${label} options`}
          className="card absolute right-0 z-20 mt-2 grid min-w-60 gap-1 p-2"
          id={menuId}
          role="menu"
          style={{ background: "var(--surface)" }}
        >
          {options.map((option) => {
            const active = option.key === selected;
            return (
              <Link
                aria-current={active ? "page" : undefined}
                className="btn sm ghost justify-start"
                href={hrefFor(option.key)}
                key={option.key}
                role="menuitem"
                style={{
                  background: active ? "var(--surface-2)" : undefined,
                  color: active ? "var(--ink-1)" : undefined,
                }}
              >
                {option.label}
                {active ? (
                  <span className="chip active ml-auto">Selected</span>
                ) : null}
              </Link>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}

export function RepositoryForkFilters({
  owner,
  repo,
  period,
  repositoryType,
  sort,
  defaultsMatch,
  defaultsSaved,
}: RepositoryForkFiltersProps) {
  const [status, setStatus] = useState<"idle" | "saving" | "saved" | "error">(
    "idle",
  );

  async function saveDefaults() {
    setStatus("saving");
    const response = await fetch(
      `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/forks/defaults`,
      {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ period, repositoryType, sort }),
      },
    );
    if (response.ok) {
      setStatus("saved");
    } else {
      setStatus("error");
    }
  }

  const href = (
    next: Partial<{ period: string; repositoryType: string; sort: string }>,
  ) =>
    repositoryForksHref(owner, repo, {
      period: next.period ?? period,
      repositoryType: next.repositoryType ?? repositoryType,
      sort: next.sort ?? sort,
    });

  const saveDisabled =
    defaultsMatch || status === "saving" || status === "saved";

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap gap-2">
        <Link className="btn" href={repositoryForksHref(owner, repo)}>
          Clear filters
        </Link>
        <FilterMenu
          hrefFor={(key) => href({ period: key })}
          label="Period"
          options={PERIODS}
          selected={period}
        />
        <FilterMenu
          hrefFor={(key) => href({ repositoryType: key })}
          label="Repository type"
          options={TYPES}
          selected={repositoryType}
        />
        <FilterMenu
          hrefFor={(key) => href({ sort: key })}
          label="Sort"
          options={SORTS}
          selected={sort}
        />
        <button
          aria-disabled={saveDisabled}
          className={`btn ${defaultsMatch ? "" : "primary"}`}
          disabled={saveDisabled}
          onClick={saveDefaults}
          type="button"
        >
          {defaultsMatch || status === "saved"
            ? "Defaults Saved"
            : status === "saving"
              ? "Saving defaults"
              : "Save defaults"}
        </button>
      </div>
      <div className="flex flex-wrap gap-2">
        <span className="chip soft">Period {labelFor(PERIODS, period)}</span>
        <span className="chip soft">
          Type {labelFor(TYPES, repositoryType)}
        </span>
        <span className="chip soft">Sort {labelFor(SORTS, sort)}</span>
        <span className={defaultsSaved ? "chip ok" : "chip warn"}>
          {defaultsSaved ? "Defaults saved" : "Default filters"}
        </span>
        {status === "error" ? (
          <span className="chip err" role="status">
            Defaults could not be saved
          </span>
        ) : null}
        {status === "saved" ? (
          <span className="chip ok" role="status">
            Saved for this repository
          </span>
        ) : null}
      </div>
    </div>
  );
}
