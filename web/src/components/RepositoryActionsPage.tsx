"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ActionsFilterOption,
  ActionsRunListItem,
  ActionsWorkflowRailItem,
  ApiErrorEnvelope,
  RepositoryActionsDashboard,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsPageProps = {
  repository: RepositoryOverview;
  dashboard: RepositoryActionsDashboard;
  query: Record<string, string | undefined>;
  validationError?: ApiErrorEnvelope | null;
};

const FILTERS = ["workflow", "event", "status", "branch", "actor"] as const;
type FilterKey = (typeof FILTERS)[number];

const FILTER_LABELS: Record<FilterKey, string> = {
  actor: "Actor",
  branch: "Branch",
  event: "Event",
  status: "Status",
  workflow: "Workflow",
};

function relativeTime(value: string | null) {
  if (!value) {
    return "not started";
  }
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) {
    return "just now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  const days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days}d ago`;
  }
  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${months}mo ago`;
  }
  return `${Math.floor(months / 12)}y ago`;
}

function durationLabel(seconds: number | null) {
  if (seconds === null) {
    return "waiting";
  }
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  if (minutes < 60) {
    return remainder ? `${minutes}m ${remainder}s` : `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}

function titleCase(value: string | null | undefined) {
  if (!value) {
    return "Unknown";
  }
  return value
    .replaceAll("_", " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

function statusTone(statusCategory: string) {
  if (["success", "completed"].includes(statusCategory)) {
    return "ok";
  }
  if (
    ["failure", "timed_out", "cancelled", "action_required"].includes(
      statusCategory,
    )
  ) {
    return "err";
  }
  if (["in_progress", "queued", "waiting"].includes(statusCategory)) {
    return "accent";
  }
  if (["skipped", "neutral", "stale"].includes(statusCategory)) {
    return "warn";
  }
  return "soft";
}

function StatusGlyph({ run }: { run: ActionsRunListItem }) {
  const tone = statusTone(run.statusCategory);
  const glyph =
    tone === "ok" ? "✓" : tone === "err" ? "!" : run.isLive ? "•" : "○";
  return (
    <span
      aria-label={`${titleCase(run.statusCategory)} run`}
      className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border text-[12px]"
      role="img"
      style={{
        borderColor:
          tone === "ok"
            ? "var(--ok)"
            : tone === "err"
              ? "var(--err)"
              : tone === "accent"
                ? "var(--accent)"
                : "var(--line-strong)",
        color:
          tone === "ok"
            ? "var(--ok)"
            : tone === "err"
              ? "var(--err)"
              : tone === "accent"
                ? "var(--accent)"
                : "var(--ink-3)",
      }}
    >
      {glyph}
    </span>
  );
}

function WorkflowRailItem({
  workflow,
  active,
  basePath,
  query,
}: {
  workflow: ActionsWorkflowRailItem;
  active: boolean;
  basePath: string;
  query: Record<string, string | undefined>;
}) {
  const href = actionsHref(basePath, query, {
    page: null,
    workflow: workflow.id,
  });
  return (
    <Link
      aria-current={active ? "page" : undefined}
      className="flex items-start justify-between gap-3 rounded-[var(--radius)] px-3 py-2 text-sm transition hover:bg-[var(--hover)]"
      href={href}
      style={{
        background: active ? "var(--accent-soft)" : "transparent",
        color: active ? "var(--ink-1)" : "var(--ink-2)",
      }}
    >
      <span className="min-w-0">
        <span className="flex items-center gap-2">
          {workflow.pinned ? (
            <span aria-label="Pinned workflow" role="img">
              ◆
            </span>
          ) : null}
          <span className="truncate font-medium">{workflow.name}</span>
        </span>
        <span className="t-xs mt-0.5 block truncate">{workflow.path}</span>
      </span>
      <span className="chip soft t-num">{workflow.runCount}</span>
    </Link>
  );
}

function actionsHref(
  basePath: string,
  query: Record<string, string | number | null | undefined>,
  updates: Record<string, string | number | null | undefined> = {},
) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries({ ...query, ...updates })) {
    if (value === undefined || value === null || value === "") {
      continue;
    }
    params.set(key, String(value));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `${basePath}/actions${suffix}`;
}

function displayFilterValue(
  key: FilterKey,
  value: string,
  dashboard: RepositoryActionsDashboard,
) {
  const options = filterOptions(key, dashboard);
  return (
    options.find((option) => option.value === value)?.label ?? titleCase(value)
  );
}

function WorkflowRail({
  dashboard,
  basePath,
  query,
}: {
  dashboard: RepositoryActionsDashboard;
  basePath: string;
  query: Record<string, string | undefined>;
}) {
  const activeWorkflow = dashboard.filters.workflow;
  const visibleWorkflows = dashboard.workflows.slice(0, 8);
  return (
    <aside className="min-w-0">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 className="t-h3">Actions</h2>
        <span className="chip soft t-num">{dashboard.workflows.length}</span>
      </div>
      <nav aria-label="Actions workflows" className="space-y-1">
        <Link
          aria-current={!activeWorkflow ? "page" : undefined}
          className="flex items-center justify-between rounded-[var(--radius)] px-3 py-2 text-sm font-medium transition hover:bg-[var(--hover)]"
          href={actionsHref(basePath, query, { page: null, workflow: null })}
          style={{
            background: !activeWorkflow ? "var(--accent-soft)" : "transparent",
          }}
        >
          <span>All workflows</span>
          <span className="chip soft t-num">{dashboard.runs.total}</span>
        </Link>
        {visibleWorkflows.map((workflow) => (
          <WorkflowRailItem
            active={activeWorkflow === workflow.id}
            basePath={basePath}
            key={workflow.id}
            query={query}
            workflow={workflow}
          />
        ))}
      </nav>
      {dashboard.workflows.length > visibleWorkflows.length ? (
        <Link
          className="btn ghost mt-3 w-full justify-center"
          href={actionsHref(basePath, query)}
        >
          Show more workflows
        </Link>
      ) : null}
      <div
        className="mt-6 border-t pt-5"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label mb-3">Management</p>
        <nav aria-label="Actions management" className="space-y-1">
          {[
            "Caches",
            "Deployments",
            "Attestations",
            "Usage metrics",
            "Performance metrics",
          ].map((label) => (
            <Link
              className="block rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--hover)]"
              href={`${basePath}/actions/${label.toLowerCase().replaceAll(" ", "-")}`}
              key={label}
            >
              {label}
            </Link>
          ))}
        </nav>
      </div>
    </aside>
  );
}

function filterOptions(
  key: FilterKey,
  dashboard: RepositoryActionsDashboard,
): ActionsFilterOption[] {
  if (key === "workflow") {
    return dashboard.filterOptions.workflows;
  }
  if (key === "event") {
    return dashboard.filterOptions.events;
  }
  if (key === "status") {
    return dashboard.filterOptions.statuses;
  }
  if (key === "branch") {
    return dashboard.filterOptions.branches;
  }
  return dashboard.filterOptions.actors;
}

function FilterPanel({
  active,
  dashboard,
  filterKey,
  onClose,
  onSelect,
}: {
  active: string | null;
  dashboard: RepositoryActionsDashboard;
  filterKey: FilterKey;
  onClose: () => void;
  onSelect: (key: FilterKey, value: string) => void;
}) {
  const [needle, setNeedle] = useState("");
  const options = filterOptions(filterKey, dashboard).filter((option) =>
    `${option.label} ${option.value}`
      .toLowerCase()
      .includes(needle.toLowerCase()),
  );

  return (
    <div
      aria-label={`${FILTER_LABELS[filterKey]} filter options`}
      className="card absolute z-20 mt-2 w-72 p-3 shadow-[var(--shadow-md)]"
      role="dialog"
    >
      <div className="mb-2 flex items-center justify-between gap-2">
        <p className="t-label">{FILTER_LABELS[filterKey]}</p>
        <button
          aria-label={`Close ${FILTER_LABELS[filterKey]} filter`}
          className="btn sm"
          onClick={onClose}
          type="button"
        >
          ×
        </button>
      </div>
      <label className="input mb-2" htmlFor={`actions-${filterKey}-search`}>
        <span aria-hidden="true">⌕</span>
        <input
          id={`actions-${filterKey}-search`}
          onChange={(event) => setNeedle(event.target.value)}
          placeholder={`Search ${FILTER_LABELS[filterKey].toLowerCase()}`}
          value={needle}
        />
      </label>
      <div className="max-h-64 overflow-auto" role="menu">
        {options.length ? (
          options.map((option) => (
            <button
              aria-checked={active === option.value}
              className="list-row w-full justify-between gap-3 px-2 py-2 text-left"
              key={option.value}
              onClick={() => onSelect(filterKey, option.value)}
              role="menuitemradio"
              type="button"
            >
              <span className="min-w-0 truncate">
                {titleCase(option.label)}
              </span>
              <span className="chip soft t-num">{option.count}</span>
            </button>
          ))
        ) : (
          <p className="t-sm px-2 py-3" style={{ color: "var(--ink-3)" }}>
            No matching options.
          </p>
        )}
      </div>
    </div>
  );
}

function FilterSummary({
  dashboard,
  basePath,
  query,
}: {
  dashboard: RepositoryActionsDashboard;
  basePath: string;
  query: Record<string, string | undefined>;
}) {
  const router = useRouter();
  const [search, setSearch] = useState(dashboard.filters.q ?? "");
  const [openFilter, setOpenFilter] = useState<FilterKey | null>(null);
  const selectedFilters = useMemo(
    () =>
      FILTERS.map((key) => [key, dashboard.filters[key]] as const).filter(
        (entry): entry is readonly [FilterKey, string] => Boolean(entry[1]),
      ),
    [dashboard.filters],
  );

  const push = useCallback(
    (updates: Record<string, string | number | null | undefined>) => {
      router.push(actionsHref(basePath, query, { ...updates, page: null }));
    },
    [basePath, query, router],
  );

  useEffect(() => {
    if (search === (dashboard.filters.q ?? "")) {
      return;
    }
    const timeout = window.setTimeout(() => {
      push({ q: search.trim() || null });
    }, 350);
    return () => window.clearTimeout(timeout);
  }, [search, dashboard.filters.q, push]);

  return (
    <div className="card p-4">
      <form
        className="flex flex-wrap gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          push({ q: search.trim() || null });
        }}
      >
        <label className="input min-w-64 flex-1" htmlFor="actions-run-filter">
          <span aria-hidden="true">⌕</span>
          <input
            id="actions-run-filter"
            name="q"
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Filter workflow runs"
            value={search}
          />
        </label>
        <button className="btn" type="submit">
          Search
        </button>
      </form>
      <div className="mt-3 flex flex-wrap gap-2">
        {FILTERS.map((filter) => (
          <span className="relative" key={filter}>
            <button
              aria-expanded={openFilter === filter}
              className="btn sm"
              onClick={() =>
                setOpenFilter(openFilter === filter ? null : filter)
              }
              type="button"
            >
              {FILTER_LABELS[filter]}
            </button>
            {openFilter === filter ? (
              <FilterPanel
                active={dashboard.filters[filter]}
                dashboard={dashboard}
                filterKey={filter}
                onClose={() => setOpenFilter(null)}
                onSelect={(key, value) => {
                  setOpenFilter(null);
                  push({ [key]: value });
                }}
              />
            ) : null}
          </span>
        ))}
        {selectedFilters.map(([key, value]) => (
          <button
            className="chip accent"
            key={key}
            onClick={() => push({ [key]: null })}
            type="button"
          >
            {FILTER_LABELS[key]}: {displayFilterValue(key, value, dashboard)} ×
          </button>
        ))}
        {dashboard.filters.q ? (
          <button
            className="chip accent"
            onClick={() => push({ q: null })}
            type="button"
          >
            Search: {dashboard.filters.q} ×
          </button>
        ) : null}
      </div>
    </div>
  );
}

function RunRow({
  run,
  basePath,
}: {
  run: ActionsRunListItem;
  basePath: string;
}) {
  const actorLabel = run.actor?.login ?? "opengithub";
  const runHref = `${basePath}/actions/runs/${run.id}`;
  const summary = run.jobSummary;
  const statusClass = statusTone(run.statusCategory);
  return (
    <article className="list-row items-start gap-3 px-5 py-4">
      <StatusGlyph run={run} />
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link className="font-medium hover:underline" href={runHref}>
            {run.displayTitle}
          </Link>
          <span className={`chip ${statusClass}`}>
            {titleCase(run.statusCategory)}
          </span>
          {run.isLive ? (
            <span className="chip accent">
              <span className="dot live" />
              Live
            </span>
          ) : null}
        </div>
        <p className="t-xs mt-1">
          {run.workflowName} ·{" "}
          <span className="t-mono-sm">#{run.runNumber}</span> ·{" "}
          {run.event.replaceAll("_", " ")}
          {run.shortSha ? (
            <>
              {" "}
              at <span className="t-mono-sm">{run.shortSha}</span>
            </>
          ) : null}
          {run.pullRequest ? <> · PR #{run.pullRequest.number}</> : null}
        </p>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <span className="chip soft">{run.headBranch}</span>
          <span className="t-xs">
            {summary.total
              ? `${summary.success}/${summary.total} jobs passed`
              : "No jobs recorded"}
          </span>
          <span className="t-xs">
            Duration {durationLabel(run.durationSeconds)}
          </span>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <div className="hidden text-right sm:block">
          <div className="flex items-center justify-end gap-2">
            <span className="av sm">
              {actorLabel.slice(0, 2).toUpperCase()}
            </span>
            <span className="t-xs">{actorLabel}</span>
          </div>
          <p className="t-xs mt-1">{relativeTime(run.createdAt)}</p>
        </div>
        <Link
          aria-label={`Open options for run ${run.runNumber}`}
          className="btn sm"
          href={runHref}
        >
          ⋯
        </Link>
      </div>
    </article>
  );
}

function EmptyActionsState({
  dashboard,
  basePath,
}: {
  dashboard: RepositoryActionsDashboard;
  basePath: string;
}) {
  const templates = [
    ["Rust", "Build and test a Rust workspace with cargo."],
    ["Node.js", "Install dependencies, run tests, and publish artifacts."],
    ["Static site", "Publish pages from a generated build directory."],
  ] as const;
  return (
    <div className="card p-6">
      <div className="max-w-2xl">
        <p className="t-label">Workflow templates</p>
        <h2 className="t-h2 mt-2">Start automating this repository</h2>
        <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
          {dashboard.emptyState.message}
        </p>
        <Link
          className="btn accent mt-4"
          href={dashboard.emptyState.newWorkflowHref}
        >
          New workflow
        </Link>
      </div>
      <div className="mt-6 grid gap-3 md:grid-cols-3">
        {templates.map(([name, description]) => (
          <Link
            className="card block p-4 transition hover:bg-[var(--surface-2)]"
            href={dashboard.emptyState.newWorkflowHref || `${basePath}/actions`}
            key={name}
          >
            <h3 className="t-h3">{name}</h3>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {description}
            </p>
          </Link>
        ))}
      </div>
    </div>
  );
}

export function RepositoryActionsPage({
  repository,
  dashboard,
  query,
  validationError = null,
}: RepositoryActionsPageProps) {
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const pathname = usePathname();
  const telemetryPayload = useMemo(
    () =>
      JSON.stringify({
        actor: dashboard.filters.actor,
        branch: dashboard.filters.branch,
        event: dashboard.filters.event,
        q: dashboard.filters.q,
        status: dashboard.filters.status,
        workflow: dashboard.filters.workflow,
      }),
    [dashboard.filters],
  );
  useEffect(() => {
    if (validationError) {
      return;
    }
    const controller = new AbortController();
    void fetch(`${pathname}/recent-view`, {
      body: telemetryPayload,
      headers: { "content-type": "application/json" },
      method: "POST",
      signal: controller.signal,
    }).catch(() => {});
    return () => controller.abort();
  }, [pathname, telemetryPayload, validationError]);

  return (
    <RepositoryShell
      activePath={`${basePath}/actions`}
      frameClassName="grid grid-cols-[260px_minmax(0,1fr)] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <WorkflowRail basePath={basePath} dashboard={dashboard} query={query} />
      <main className="min-w-0">
        <div className="mb-5 flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label">Repository automation</p>
            <h1 className="t-h1 mt-1">All workflows</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {dashboard.runs.total} workflow runs across{" "}
              {dashboard.workflows.length} workflow files.
            </p>
          </div>
          <Link
            className="btn accent"
            href={dashboard.emptyState.newWorkflowHref}
          >
            New workflow
          </Link>
        </div>
        {validationError ? (
          <div className="chip err mb-4" role="alert">
            {validationError.error.message}
          </div>
        ) : null}
        <FilterSummary
          basePath={basePath}
          dashboard={dashboard}
          query={query}
        />
        <section
          aria-label="Workflow runs"
          className="card mt-4 overflow-hidden"
        >
          <div
            className="between border-b px-5 py-3"
            style={{ borderColor: "var(--line)" }}
          >
            <h2 className="t-h3">Recent runs</h2>
            <span className="chip soft t-num">{dashboard.runs.total}</span>
          </div>
          {dashboard.runs.items.length ? (
            dashboard.runs.items.map((run) => (
              <RunRow basePath={basePath} key={run.id} run={run} />
            ))
          ) : (
            <div className="p-5">
              <EmptyActionsState basePath={basePath} dashboard={dashboard} />
            </div>
          )}
        </section>
      </main>
    </RepositoryShell>
  );
}
