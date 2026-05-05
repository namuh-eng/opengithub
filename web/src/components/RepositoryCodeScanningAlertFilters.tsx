"use client";

import Link from "next/link";
import { useEffect, useId, useRef, useState } from "react";
import type {
  RepositoryCodeScanningBranchFilter,
  RepositoryCodeScanningFilters,
  RepositoryCodeScanningToolStatus,
} from "@/lib/api";
import { repositoryCodeScanningAlertsHref } from "@/lib/navigation";

const SEVERITIES = [
  { key: "all", label: "All severities" },
  { key: "error", label: "Error" },
  { key: "warning", label: "Warning" },
  { key: "note", label: "Note" },
] as const;

const SECURITY_SEVERITIES = [
  { key: "all", label: "All security severities" },
  { key: "critical", label: "Critical" },
  { key: "high", label: "High" },
  { key: "medium", label: "Medium" },
  { key: "low", label: "Low" },
] as const;

const APPLICATION_CODE = [
  { key: "all", label: "All code" },
  { key: "true", label: "Application code" },
  { key: "false", label: "Generated and vendor code" },
] as const;

const SORTS = [
  { key: "most_important", label: "Most important" },
  { key: "recently_detected", label: "Recently detected" },
  { key: "recently_updated", label: "Recently updated" },
  { key: "path", label: "File path" },
] as const;

type Option = {
  key: string;
  label: string;
  count?: number | null;
};

type RepositoryCodeScanningAlertFiltersProps = {
  owner: string;
  repo: string;
  filters: RepositoryCodeScanningFilters;
  tools: RepositoryCodeScanningToolStatus[];
  branches: RepositoryCodeScanningBranchFilter[];
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

export function RepositoryCodeScanningAlertFilters({
  owner,
  repo,
  filters,
  tools,
  branches,
}: RepositoryCodeScanningAlertFiltersProps) {
  const [draftQuery, setDraftQuery] = useState(filters.query ?? "");
  const toolOptions: Option[] = [
    { key: "all", label: "All tools" },
    ...tools.map((tool) => ({
      key: tool.name,
      label: tool.version ? `${tool.name} ${tool.version}` : tool.name,
      count: tool.alertCount,
    })),
  ];
  const branchOptions: Option[] = [
    { key: "all", label: "All branches" },
    ...branches.map((branch) => ({
      key: branch.name,
      label: branch.name,
      count: branch.openCount,
    })),
  ];

  const href = (
    next: Partial<{
      state: string | null;
      query: string | null;
      severity: string | null;
      securitySeverity: string | null;
      tool: string | null;
      branch: string | null;
      ref: string | null;
      tag: string | null;
      applicationCode: string | null;
      sort: string | null;
    }>,
  ) =>
    repositoryCodeScanningAlertsHref(owner, repo, {
      state: next.state !== undefined ? filterValue(next.state) : filters.state,
      query: next.query !== undefined ? filterValue(next.query) : filters.query,
      severity:
        next.severity !== undefined
          ? filterValue(next.severity)
          : filters.severity,
      securitySeverity:
        next.securitySeverity !== undefined
          ? filterValue(next.securitySeverity)
          : filters.securitySeverity,
      tool: next.tool !== undefined ? filterValue(next.tool) : filters.tool,
      branch:
        next.branch !== undefined ? filterValue(next.branch) : filters.branch,
      ref: next.ref !== undefined ? filterValue(next.ref) : filters.ref,
      tag: next.tag !== undefined ? filterValue(next.tag) : filters.tag,
      applicationCode:
        next.applicationCode !== undefined
          ? filterValue(next.applicationCode)
          : filters.applicationCode,
      sort: next.sort !== undefined ? filterValue(next.sort) : filters.sort,
    });

  return (
    <div className="grid gap-3">
      <form
        action={href({ query: draftQuery })}
        className="flex flex-wrap gap-2"
      >
        <label className="input min-w-64 flex-1" htmlFor="code-scanning-search">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search alerts
          </span>
          <input
            id="code-scanning-search"
            name="q"
            onChange={(event) => setDraftQuery(event.target.value)}
            placeholder="Search rule, message, file, or tool"
            value={draftQuery}
          />
        </label>
        <button className="btn primary" type="submit">
          Apply
        </button>
        <Link
          className="btn"
          href={repositoryCodeScanningAlertsHref(owner, repo)}
        >
          Clear filters
        </Link>
      </form>

      <div className="flex flex-wrap gap-2">
        <FilterMenu
          hrefFor={(key) =>
            href({ severity: key === "all" ? null : key, query: draftQuery })
          }
          label="Severity"
          options={SEVERITIES}
          selected={filters.severity}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({
              securitySeverity: key === "all" ? null : key,
              query: draftQuery,
            })
          }
          label="Security severity"
          options={SECURITY_SEVERITIES}
          selected={filters.securitySeverity}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ tool: key === "all" ? null : key, query: draftQuery })
          }
          label="Tool"
          options={toolOptions}
          selected={filters.tool}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({ branch: key === "all" ? null : key, query: draftQuery })
          }
          label="Branch"
          options={branchOptions}
          selected={filters.branch}
        />
        <FilterMenu
          hrefFor={(key) =>
            href({
              applicationCode: key === "all" ? null : key,
              query: draftQuery,
            })
          }
          label="Application code"
          options={APPLICATION_CODE}
          selected={filters.applicationCode}
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
