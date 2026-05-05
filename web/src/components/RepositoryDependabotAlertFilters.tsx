"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type {
  RepositoryDependabotAlertFilters as RepositoryDependabotAlertFilterState,
  RepositoryDependabotManifestFilter,
  RepositoryDependabotPackageFilter,
} from "@/lib/api";
import { repositoryDependabotAlertsHref } from "@/lib/navigation";

const ECOSYSTEMS = [
  { key: "all", label: "All ecosystems" },
  { key: "npm", label: "npm" },
  { key: "cargo", label: "Cargo" },
  { key: "pip", label: "pip" },
] as const;

const SCOPES = [
  { key: "all", label: "All scopes" },
  { key: "production", label: "Production" },
  { key: "development", label: "Development" },
] as const;

const SEVERITIES = [
  { key: "all", label: "All severities" },
  { key: "critical", label: "Critical" },
  { key: "high", label: "High" },
  { key: "moderate", label: "Moderate" },
  { key: "low", label: "Low" },
] as const;

const SORTS = [
  { key: "most_important", label: "Most important" },
  { key: "recently_detected", label: "Recently detected" },
  { key: "package", label: "Package" },
  { key: "manifest", label: "Manifest" },
] as const;

type Option = {
  key: string;
  label: string;
  count?: number | null;
};

type RepositoryDependabotAlertFiltersProps = {
  owner: string;
  repo: string;
  filters: RepositoryDependabotAlertFilterState;
  packages: RepositoryDependabotPackageFilter[];
  manifests: RepositoryDependabotManifestFilter[];
};

function filterValue(value: string | null | undefined) {
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
  const wrapperRef = useRef<HTMLDivElement>(null);
  const selectedKey = selected ?? "all";

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

export function RepositoryDependabotAlertFilters({
  owner,
  repo,
  filters,
  packages,
  manifests,
}: RepositoryDependabotAlertFiltersProps) {
  const [draftQuery, setDraftQuery] = useState(filters.query ?? "");
  const packageOptions: Option[] = [
    { key: "all", label: "All packages" },
    ...packages.map((item) => ({
      key: `${item.package.ecosystem}:${item.package.name}`,
      label: `${item.package.ecosystem}:${item.package.name}`,
      count: item.openCount,
    })),
  ];
  const manifestOptions: Option[] = [
    { key: "all", label: "All manifests" },
    ...manifests.map((item) => ({
      key: item.path,
      label: `${item.ecosystem}:${item.path}`,
      count: item.openCount,
    })),
  ];

  const href = (
    next: Partial<{
      state: string | null;
      query: string | null;
      package: string | null;
      ecosystem: string | null;
      manifest: string | null;
      scope: string | null;
      severity: string | null;
      sort: string | null;
    }>,
  ) =>
    repositoryDependabotAlertsHref(owner, repo, {
      state: next.state !== undefined ? filterValue(next.state) : filters.state,
      query: next.query !== undefined ? filterValue(next.query) : filters.query,
      package:
        next.package !== undefined
          ? filterValue(next.package)
          : filters.package,
      ecosystem:
        next.ecosystem !== undefined
          ? filterValue(next.ecosystem)
          : filters.ecosystem,
      manifest:
        next.manifest !== undefined
          ? filterValue(next.manifest)
          : filters.manifest,
      scope: next.scope !== undefined ? filterValue(next.scope) : filters.scope,
      severity:
        next.severity !== undefined
          ? filterValue(next.severity)
          : filters.severity,
      sort: next.sort !== undefined ? filterValue(next.sort) : filters.sort,
    });

  return (
    <div className="grid gap-3">
      <form
        action={href({ query: draftQuery })}
        className="flex flex-wrap gap-2"
      >
        <label className="input min-w-64 flex-1" htmlFor="dependabot-search">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search alerts
          </span>
          <input
            id="dependabot-search"
            name="q"
            onChange={(event) => setDraftQuery(event.target.value)}
            placeholder="Search package, advisory, or manifest"
            value={draftQuery}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply
        </button>
        <Link
          className="btn"
          href={repositoryDependabotAlertsHref(owner, repo)}
        >
          Clear filters
        </Link>
      </form>

      <div className="flex flex-wrap gap-2">
        <FilterMenu
          hrefFor={(key) =>
            href({ package: key === "all" ? null : key, query: draftQuery })
          }
          label="Package"
          options={packageOptions}
          selected={filters.package}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ ecosystem: key === "all" ? null : key, query: draftQuery })
          }
          label="Ecosystem"
          options={ECOSYSTEMS}
          selected={filters.ecosystem}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ manifest: key === "all" ? null : key, query: draftQuery })
          }
          label="Manifest"
          options={manifestOptions}
          selected={filters.manifest}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ scope: key === "all" ? null : key, query: draftQuery })
          }
          label="Scope"
          options={SCOPES}
          selected={filters.scope}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ severity: key === "all" ? null : key, query: draftQuery })
          }
          label="Severity"
          options={SEVERITIES}
          selected={filters.severity}
        />
        <FilterMenu
          align="right"
          hrefFor={(key) => href({ query: draftQuery, sort: key })}
          label="Sort"
          options={SORTS}
          selected={filters.sort}
        />
      </div>

      <div className="flex flex-wrap gap-2">
        <span className="chip soft">
          Query{" "}
          {filterValue(filters.query) ? `"${filters.query}"` : "all alerts"}
        </span>
        <span className="chip soft">
          State {filters.state === "closed" ? "Closed" : "Open"}
        </span>
        <span className="chip soft">Sort {labelFor(SORTS, filters.sort)}</span>
      </div>
    </div>
  );
}
