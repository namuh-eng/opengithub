import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
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

const FILTERS = ["Workflow", "Event", "Status", "Branch", "Actor"] as const;

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
}: {
  workflow: ActionsWorkflowRailItem;
  active: boolean;
  basePath: string;
}) {
  const href = `${basePath}/actions?workflow=${encodeURIComponent(workflow.id)}`;
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

function WorkflowRail({
  dashboard,
  basePath,
}: {
  dashboard: RepositoryActionsDashboard;
  basePath: string;
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
          href={`${basePath}/actions`}
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
            workflow={workflow}
          />
        ))}
      </nav>
      {dashboard.workflows.length > visibleWorkflows.length ? (
        <Link
          className="btn ghost mt-3 w-full justify-center"
          href={`${basePath}/actions`}
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

function FilterSummary({
  dashboard,
  basePath,
  query,
}: {
  dashboard: RepositoryActionsDashboard;
  basePath: string;
  query: Record<string, string | undefined>;
}) {
  const queryString = new URLSearchParams(
    Object.entries(query).filter((entry): entry is [string, string] =>
      Boolean(entry[1]),
    ),
  );
  const searchHref = queryString.size
    ? `${basePath}/actions?${queryString.toString()}`
    : `${basePath}/actions`;

  return (
    <div className="card p-4">
      <form action={`${basePath}/actions`} className="flex flex-wrap gap-2">
        <label className="input min-w-64 flex-1" htmlFor="actions-run-filter">
          <span aria-hidden="true">⌕</span>
          <input
            defaultValue={dashboard.filters.q ?? ""}
            id="actions-run-filter"
            name="q"
            placeholder="Filter workflow runs"
          />
        </label>
        <button className="btn" type="submit">
          Search
        </button>
        {dashboard.filters.workflow ? (
          <input
            name="workflow"
            type="hidden"
            value={dashboard.filters.workflow}
          />
        ) : null}
      </form>
      <div className="mt-3 flex flex-wrap gap-2">
        {FILTERS.map((filter) => (
          <Link className="btn sm" href={searchHref} key={filter}>
            {filter}
          </Link>
        ))}
        {Object.entries(dashboard.filters)
          .filter(
            ([key, value]) =>
              Boolean(value) && !["page", "pageSize"].includes(key),
          )
          .map(([key, value]) => (
            <Link
              className="chip accent"
              href={`${basePath}/actions`}
              key={key}
            >
              {key}:{String(value)}
            </Link>
          ))}
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
  return (
    <RepositoryShell
      activePath={`${basePath}/actions`}
      frameClassName="grid grid-cols-[260px_minmax(0,1fr)] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <WorkflowRail basePath={basePath} dashboard={dashboard} />
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
