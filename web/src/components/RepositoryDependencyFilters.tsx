"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import { repositoryDependenciesHref } from "@/lib/navigation";

const ECOSYSTEMS = [
  { key: "all", label: "All ecosystems" },
  { key: "npm", label: "npm" },
  { key: "cargo", label: "Cargo" },
  { key: "pip", label: "pip" },
] as const;

const RELATIONSHIPS = [
  { key: "all", label: "All" },
  { key: "direct", label: "Direct" },
  { key: "transitive", label: "Transitive" },
] as const;

type RepositoryDependencyFiltersProps = {
  owner: string;
  repo: string;
  query: string | null;
  ecosystem: string | null;
  relationship: string | null;
  supportedEcosystems: string[];
};

function filterValue(value: string | null | undefined) {
  return value?.trim() || null;
}

function labelFor(
  options: readonly { key: string; label: string }[],
  selected: string | null,
) {
  const key = selected ?? "all";
  return options.find((option) => option.key === key)?.label ?? key;
}

function FilterMenu({
  label,
  options,
  selected,
  hrefFor,
}: {
  label: string;
  options: readonly { key: string; label: string; disabled?: boolean }[];
  selected: string | null;
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

  const selectedKey = selected ?? "all";

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
            const active = option.key === selectedKey;
            if (option.disabled) {
              return (
                <button
                  className="btn sm ghost justify-start"
                  disabled
                  key={option.key}
                  role="menuitem"
                  style={{ color: "var(--ink-4)" }}
                  type="button"
                >
                  {option.label}
                  <span className="chip soft ml-auto">Unavailable</span>
                </button>
              );
            }
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

export function RepositoryDependencyFilters({
  owner,
  repo,
  query,
  ecosystem,
  relationship,
  supportedEcosystems,
}: RepositoryDependencyFiltersProps) {
  const [draftQuery, setDraftQuery] = useState(query ?? "");
  const supported = new Set(supportedEcosystems);
  const ecosystemOptions = ECOSYSTEMS.map((option) => ({
    ...option,
    disabled: option.key !== "all" && !supported.has(option.key),
  }));

  const href = (
    next: Partial<{
      query: string | null;
      ecosystem: string | null;
      relationship: string | null;
    }>,
  ) =>
    repositoryDependenciesHref(owner, repo, {
      query: next.query !== undefined ? filterValue(next.query) : query,
      ecosystem:
        next.ecosystem !== undefined ? filterValue(next.ecosystem) : ecosystem,
      relationship:
        next.relationship !== undefined
          ? filterValue(next.relationship)
          : relationship,
    });

  return (
    <div className="grid gap-3">
      <form
        action={href({ query: draftQuery })}
        className="flex flex-wrap gap-2"
      >
        <label className="input min-w-64 flex-1" htmlFor="dependency-search">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search
          </span>
          <input
            id="dependency-search"
            name="q"
            onChange={(event) => setDraftQuery(event.target.value)}
            placeholder="Search all dependencies"
            value={draftQuery}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply
        </button>
        <Link className="btn" href={repositoryDependenciesHref(owner, repo)}>
          Clear filters
        </Link>
      </form>

      <div className="flex flex-wrap gap-2">
        <FilterMenu
          hrefFor={(key) =>
            href({ ecosystem: key === "all" ? null : key, query: draftQuery })
          }
          label="Ecosystem"
          options={ecosystemOptions}
          selected={ecosystem}
        />
        {RELATIONSHIPS.map((option) => {
          const active = (relationship ?? "all") === option.key;
          return (
            <Link
              aria-current={active ? "page" : undefined}
              className={active ? "chip active" : "chip soft"}
              href={href({
                query: draftQuery,
                relationship: option.key === "all" ? null : option.key,
              })}
              key={option.key}
            >
              {option.label}
            </Link>
          );
        })}
      </div>

      <div className="flex flex-wrap gap-2">
        <span className="chip soft">
          Query {filterValue(query) ? `"${query}"` : "all packages"}
        </span>
        <span className="chip soft">
          Ecosystem {labelFor(ECOSYSTEMS, ecosystem)}
        </span>
        <span className="chip soft">
          Relationship {labelFor(RELATIONSHIPS, relationship)}
        </span>
      </div>
    </div>
  );
}
