"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type { RepositorySecurityAdvisoryFilters as RepositorySecurityAdvisoryFilterState } from "@/lib/api";
import { repositorySecurityAdvisoriesHref } from "@/lib/navigation";

const SEVERITIES = [
  { key: "all", label: "All severities" },
  { key: "critical", label: "Critical" },
  { key: "high", label: "High" },
  { key: "medium", label: "Medium" },
  { key: "low", label: "Low" },
] as const;

const SORTS = [
  { key: "recently_updated", label: "Recently updated" },
  { key: "recently_published", label: "Recently published" },
  { key: "severity", label: "Severity" },
  { key: "identifier", label: "GHSA identifier" },
] as const;

type Option = {
  key: string;
  label: string;
  count?: number | null;
};

function clean(value: string | null | undefined) {
  return value?.trim() || null;
}

function labelFor(options: readonly Option[], selected: string | null) {
  const key = selected ?? "all";
  return (
    options.find((option) => option.key === key)?.label ?? selected ?? "All"
  );
}

function FilterMenu({
  label,
  options,
  selected,
  hrefFor,
  align = "left",
}: {
  label: string;
  options: readonly Option[];
  selected: string | null;
  hrefFor: (key: string) => string;
  align?: "left" | "right";
}) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const ref = useRef<HTMLDivElement>(null);
  const selectedKey = selected ?? "all";

  useEffect(() => {
    if (!open) return;
    function onMouseDown(event: MouseEvent) {
      if (!ref.current?.contains(event.target as Node)) setOpen(false);
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
    }
    document.addEventListener("mousedown", onMouseDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onMouseDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  return (
    <div className="relative" ref={ref}>
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
          className={`card absolute z-20 mt-2 grid max-h-80 min-w-72 gap-1 overflow-auto p-2 ${
            align === "right" ? "right-0" : "left-0"
          }`}
          id={menuId}
          role="menu"
          style={{ background: "var(--surface)" }}
        >
          {options.map((option) => {
            const active = option.key === selectedKey;
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
                <span className="min-w-0 break-words">{option.label}</span>
                {typeof option.count === "number" ? (
                  <span className="chip soft ml-auto">
                    <span className="t-num">{option.count}</span>
                  </span>
                ) : null}
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

export function RepositorySecurityAdvisoryFilters({
  owner,
  repo,
  filters,
}: {
  owner: string;
  repo: string;
  filters: RepositorySecurityAdvisoryFilterState;
}) {
  const [query, setQuery] = useState(filters.query ?? "");
  const href = (
    next: Partial<{
      state: string | null;
      query: string | null;
      severity: string | null;
      sort: string | null;
      page: string | number | null;
    }>,
  ) =>
    repositorySecurityAdvisoriesHref(owner, repo, {
      state: next.state !== undefined ? clean(next.state) : filters.state,
      query: next.query !== undefined ? clean(next.query) : filters.query,
      severity:
        next.severity !== undefined ? clean(next.severity) : filters.severity,
      sort: next.sort !== undefined ? clean(next.sort) : filters.sort,
      page: next.page !== undefined ? next.page : 1,
      pageSize: filters.pageSize,
    });

  return (
    <div className="grid gap-3">
      <form action={href({ query })} className="flex flex-wrap gap-2">
        <label className="input min-w-64 flex-1" htmlFor="advisory-search">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search advisories
          </span>
          <input
            id="advisory-search"
            name="q"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search title, GHSA, CVE, package, or author"
            value={query}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply
        </button>
        <Link
          className="btn"
          href={repositorySecurityAdvisoriesHref(owner, repo)}
        >
          Clear filters
        </Link>
      </form>

      <div className="flex flex-wrap gap-2">
        <FilterMenu
          hrefFor={(key) =>
            href({ severity: key === "all" ? null : key, query })
          }
          label="Severity"
          options={SEVERITIES}
          selected={filters.severity}
        />
        <FilterMenu
          align="right"
          hrefFor={(key) => href({ sort: key, query })}
          label="Sort"
          options={SORTS}
          selected={filters.sort}
        />
      </div>

      <div className="flex flex-wrap gap-2">
        <span className="chip soft">
          Query {clean(filters.query) ? `"${filters.query}"` : "all advisories"}
        </span>
        <span className="chip soft">State {filters.state}</span>
        <span className="chip soft">Sort {labelFor(SORTS, filters.sort)}</span>
      </div>
    </div>
  );
}
