"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type {
  RepositorySecretScanningFilters,
  RepositorySecretScanningProviderFilter,
  RepositorySecretScanningSecretTypeFilter,
} from "@/lib/api";
import { repositorySecretScanningAlertsHref } from "@/lib/navigation";

const VALIDITY = [
  { key: "all", label: "All validity states" },
  { key: "active", label: "Active" },
  { key: "inactive", label: "Inactive" },
  { key: "unknown", label: "Unknown" },
] as const;

const RESOLUTIONS = [
  { key: "all", label: "All resolutions" },
  { key: "revoked", label: "Revoked" },
  { key: "false_positive", label: "False positive" },
  { key: "used_in_tests", label: "Used in tests" },
  { key: "wont_fix", label: "Will not fix" },
] as const;

const BYPASSED = [
  { key: "all", label: "All bypass states" },
  { key: "true", label: "Bypassed push protection" },
  { key: "false", label: "No bypass" },
] as const;

const SORTS = [
  { key: "recently_detected", label: "Recently detected" },
  { key: "recently_updated", label: "Recently updated" },
  { key: "provider", label: "Provider" },
  { key: "secret_type", label: "Secret type" },
] as const;

type Option = {
  key: string;
  label: string;
  count?: number | null;
};

type RepositorySecretScanningAlertFiltersProps = {
  owner: string;
  repo: string;
  filters: RepositorySecretScanningFilters;
  providers: RepositorySecretScanningProviderFilter[];
  secretTypes: RepositorySecretScanningSecretTypeFilter[];
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
      if (!wrapperRef.current?.contains(event.target as Node)) setOpen(false);
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

export function RepositorySecretScanningAlertFilters({
  owner,
  repo,
  filters,
  providers,
  secretTypes,
}: RepositorySecretScanningAlertFiltersProps) {
  const [draftQuery, setDraftQuery] = useState(filters.query ?? "");
  const providerOptions: Option[] = [
    { key: "all", label: "All providers" },
    ...providers.map((provider) => ({
      key: provider.provider,
      label: provider.provider,
      count: provider.openCount,
    })),
  ];
  const secretTypeOptions: Option[] = [
    { key: "all", label: "All secret types" },
    ...secretTypes.map((secretType) => ({
      key: secretType.secretType,
      label: `${secretType.displayName} · ${secretType.provider}`,
      count: secretType.openCount,
    })),
  ];

  const href = (
    next: Partial<{
      state: string | null;
      query: string | null;
      provider: string | null;
      secretType: string | null;
      validity: string | null;
      resolution: string | null;
      bypassed: string | null;
      team: string | null;
      topic: string | null;
      sort: string | null;
    }>,
  ) =>
    repositorySecretScanningAlertsHref(owner, repo, {
      state: next.state !== undefined ? filterValue(next.state) : filters.state,
      query: next.query !== undefined ? filterValue(next.query) : filters.query,
      provider:
        next.provider !== undefined
          ? filterValue(next.provider)
          : filters.provider,
      secretType:
        next.secretType !== undefined
          ? filterValue(next.secretType)
          : filters.secretType,
      validity:
        next.validity !== undefined
          ? filterValue(next.validity)
          : filters.validity,
      resolution:
        next.resolution !== undefined
          ? filterValue(next.resolution)
          : filters.resolution,
      bypassed:
        next.bypassed !== undefined
          ? filterValue(next.bypassed)
          : filters.bypassed,
      team: next.team !== undefined ? filterValue(next.team) : filters.team,
      topic: next.topic !== undefined ? filterValue(next.topic) : filters.topic,
      sort: next.sort !== undefined ? filterValue(next.sort) : filters.sort,
    });

  return (
    <div className="grid gap-3">
      <form
        action={href({ query: draftQuery })}
        className="flex flex-wrap gap-2"
      >
        <label
          className="input min-w-64 flex-1"
          htmlFor="secret-scanning-search"
        >
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search alerts
          </span>
          <input
            id="secret-scanning-search"
            name="q"
            onChange={(event) => setDraftQuery(event.target.value)}
            placeholder="Search provider, type, path, or redacted evidence"
            value={draftQuery}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply
        </button>
        <Link
          className="btn"
          href={repositorySecretScanningAlertsHref(owner, repo)}
        >
          Clear filters
        </Link>
      </form>

      <div className="flex flex-wrap gap-2">
        <FilterMenu
          hrefFor={(key) =>
            href({ provider: key === "all" ? null : key, query: draftQuery })
          }
          label="Provider"
          options={providerOptions}
          selected={filters.provider}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ secretType: key === "all" ? null : key, query: draftQuery })
          }
          label="Secret type"
          options={secretTypeOptions}
          selected={filters.secretType}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ validity: key === "all" ? null : key, query: draftQuery })
          }
          label="Validity"
          options={VALIDITY}
          selected={filters.validity}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ resolution: key === "all" ? null : key, query: draftQuery })
          }
          label="Resolution"
          options={RESOLUTIONS}
          selected={filters.resolution}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ bypassed: key === "all" ? null : key, query: draftQuery })
          }
          label="Bypassed"
          options={BYPASSED}
          selected={filters.bypassed}
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
          State {filters.state === "resolved" ? "Resolved" : "Open"}
        </span>
        <span className="chip soft">Sort {labelFor(SORTS, filters.sort)}</span>
      </div>
    </div>
  );
}
