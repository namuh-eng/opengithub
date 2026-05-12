"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ActionsFilterOption,
  ActionsRunListItem,
  ActionsWorkflowRailItem,
  ActionsWorkflowRef,
  ApiErrorEnvelope,
  RepositoryActionsWorkflowDetail,
  RepositoryOverview,
  WorkflowDispatchInput,
} from "@/lib/api";

type RepositoryActionsWorkflowPageProps = {
  repository: RepositoryOverview;
  detail: RepositoryActionsWorkflowDetail;
  query: Record<string, string | undefined>;
  validationError?: ApiErrorEnvelope | null;
};

const FILTERS = ["event", "status", "branch", "actor"] as const;
type FilterKey = (typeof FILTERS)[number];

const FILTER_LABELS: Record<FilterKey, string> = {
  actor: "Actor",
  branch: "Branch",
  event: "Event",
  status: "Status",
};

function workflowHref(basePath: string, workflowPath: string) {
  return `${basePath}/actions/workflows/${workflowPath
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/")}`;
}

function actionsHref(
  basePath: string,
  workflowPath: string,
  query: Record<string, string | number | null | undefined>,
  updates: Record<string, string | number | null | undefined> = {},
) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries({ ...query, ...updates })) {
    if (
      key === "workflow" ||
      value === undefined ||
      value === null ||
      value === ""
    ) {
      continue;
    }
    params.set(key, String(value));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `${workflowHref(basePath, workflowPath)}${suffix}`;
}

function titleCase(value: string | null | undefined) {
  if (!value) {
    return "Unknown";
  }
  return value
    .replaceAll("_", " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

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

function dateTimeLabel(value: string | null | undefined) {
  if (!value) {
    return "Not recorded";
  }
  const timestamp = new Date(value);
  if (!Number.isFinite(timestamp.getTime())) {
    return "Not recorded";
  }
  return timestamp.toLocaleString("en", {
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    month: "short",
    year: "numeric",
    timeZone: "UTC",
  });
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
  return (
    <Link
      aria-current={active ? "page" : undefined}
      className="flex items-start justify-between gap-3 rounded-[var(--radius)] px-3 py-2 text-sm transition hover:bg-[var(--hover)]"
      href={workflowHref(basePath, workflow.path)}
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
  detail,
  basePath,
}: {
  detail: RepositoryActionsWorkflowDetail;
  basePath: string;
}) {
  return (
    <aside className="min-w-0">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 className="t-h3">Actions</h2>
        <span className="chip soft t-num">{detail.workflows.length}</span>
      </div>
      <nav aria-label="Actions workflows" className="space-y-1">
        <Link
          className="flex items-center justify-between rounded-[var(--radius)] px-3 py-2 text-sm font-medium transition hover:bg-[var(--hover)]"
          href={`${basePath}/actions`}
        >
          <span>All workflows</span>
          <span className="chip soft t-num">{detail.workflows.length}</span>
        </Link>
        {detail.workflows.map((workflow) => (
          <WorkflowRailItem
            active={workflow.id === detail.workflow.id}
            basePath={basePath}
            key={workflow.id}
            workflow={workflow}
          />
        ))}
      </nav>
      <div
        className="mt-6 border-t pt-5"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label mb-3">Management</p>
        <nav aria-label="Actions management" className="space-y-1">
          <Link
            className="block rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--hover)]"
            href={`${basePath}/actions/caches`}
          >
            Caches
          </Link>
          <Link
            className="block rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--hover)]"
            href={`${basePath}/settings/actions`}
          >
            Actions policy
          </Link>
          <Link
            className="block rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--hover)]"
            href="/docs/api#actions-workflow-detail"
          >
            API docs
          </Link>
        </nav>
      </div>
    </aside>
  );
}

function filterOptions(
  key: FilterKey,
  detail: RepositoryActionsWorkflowDetail,
): ActionsFilterOption[] {
  if (key === "event") {
    return detail.filterOptions.events;
  }
  if (key === "status") {
    return detail.filterOptions.statuses;
  }
  if (key === "branch") {
    return detail.filterOptions.branches;
  }
  return detail.filterOptions.actors;
}

function displayFilterValue(
  key: FilterKey,
  value: string,
  detail: RepositoryActionsWorkflowDetail,
) {
  const options = filterOptions(key, detail);
  return (
    options.find((option) => option.value === value)?.label ?? titleCase(value)
  );
}

function FilterPanel({
  active,
  detail,
  filterKey,
  onClose,
  onSelect,
}: {
  active: string | null;
  detail: RepositoryActionsWorkflowDetail;
  filterKey: FilterKey;
  onClose: () => void;
  onSelect: (key: FilterKey, value: string) => void;
}) {
  const [needle, setNeedle] = useState("");
  const options = filterOptions(filterKey, detail).filter((option) =>
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
      <label
        className="input mb-2"
        htmlFor={`workflow-actions-${filterKey}-search`}
      >
        <span aria-hidden="true">⌕</span>
        <input
          id={`workflow-actions-${filterKey}-search`}
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
  detail,
  basePath,
  query,
}: {
  detail: RepositoryActionsWorkflowDetail;
  basePath: string;
  query: Record<string, string | undefined>;
}) {
  const router = useRouter();
  const [search, setSearch] = useState(detail.filters.q ?? "");
  const [openFilter, setOpenFilter] = useState<FilterKey | null>(null);
  const selectedFilters = useMemo(
    () =>
      FILTERS.map((key) => [key, detail.filters[key]] as const).filter(
        (entry): entry is readonly [FilterKey, string] => Boolean(entry[1]),
      ),
    [detail.filters],
  );

  const push = useCallback(
    (updates: Record<string, string | number | null | undefined>) => {
      router.push(actionsHref(basePath, detail.workflow.path, query, updates));
    },
    [basePath, detail.workflow.path, query, router],
  );

  return (
    <div className="card p-4">
      <form
        className="flex flex-wrap gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          push({ page: null, q: search.trim() || null });
        }}
      >
        <label className="input min-w-64 flex-1" htmlFor="workflow-run-filter">
          <span aria-hidden="true">⌕</span>
          <input
            id="workflow-run-filter"
            name="q"
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Filter this workflow's runs"
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
                active={detail.filters[filter]}
                detail={detail}
                filterKey={filter}
                onClose={() => setOpenFilter(null)}
                onSelect={(key, value) => {
                  setOpenFilter(null);
                  push({ [key]: value, page: null });
                }}
              />
            ) : null}
          </span>
        ))}
        {selectedFilters.map(([key, value]) => (
          <button
            className="chip accent"
            key={key}
            onClick={() => push({ [key]: null, page: null })}
            type="button"
          >
            {FILTER_LABELS[key]}: {displayFilterValue(key, value, detail)} ×
          </button>
        ))}
        {detail.filters.q ? (
          <button
            className="chip accent"
            onClick={() => push({ page: null, q: null })}
            type="button"
          >
            Search: {detail.filters.q} ×
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
          aria-label={`Open run ${run.runNumber} details and options`}
          className="btn sm"
          href={`${runHref}?panel=options`}
        >
          ⋯
        </Link>
      </div>
    </article>
  );
}

function EmptyWorkflowState({
  detail,
  basePath,
}: {
  detail: RepositoryActionsWorkflowDetail;
  basePath: string;
}) {
  return (
    <div className="card p-6">
      <p className="t-label">Workflow history</p>
      <h2 className="t-h2 mt-2">No runs for this workflow yet</h2>
      <p className="t-body mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {detail.emptyState.message}
      </p>
      <div className="mt-4 flex flex-wrap gap-2">
        <Link className="btn" href={detail.workflow.sourceHref}>
          Open workflow file
        </Link>
        <Link className="btn ghost" href={`${basePath}/actions`}>
          Back to all workflows
        </Link>
      </div>
    </div>
  );
}

function defaultInputValue(input: WorkflowDispatchInput) {
  if (input.type === "boolean") {
    return input.default ?? "false";
  }
  return input.default ?? "";
}

function DispatchField({
  input,
  value,
  onChange,
}: {
  input: WorkflowDispatchInput;
  value: string;
  onChange: (value: string) => void;
}) {
  const id = `dispatch-input-${input.name}`;
  return (
    <label className="block" htmlFor={id}>
      <span className="t-sm font-medium">
        {input.label}
        {input.required ? " *" : ""}
      </span>
      {input.description ? (
        <span className="t-xs mt-1 block">{input.description}</span>
      ) : null}
      {input.type === "choice" ? (
        <select
          className="input mt-2 w-full"
          id={id}
          onChange={(event) => onChange(event.target.value)}
          value={value}
        >
          {input.options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      ) : input.type === "boolean" ? (
        <span className="mt-2 flex items-center gap-2">
          <input
            checked={value === "true"}
            id={id}
            onChange={(event) =>
              onChange(event.target.checked ? "true" : "false")
            }
            type="checkbox"
          />
          <span className="t-sm">Enabled</span>
        </span>
      ) : (
        <input
          className="input mt-2 w-full"
          id={id}
          onChange={(event) => onChange(event.target.value)}
          type={input.type === "number" ? "number" : "text"}
          value={value}
        />
      )}
    </label>
  );
}

function DispatchDialog({
  basePath,
  detail,
  onCancel,
  onDispatched,
}: {
  basePath: string;
  detail: RepositoryActionsWorkflowDetail;
  onCancel: () => void;
  onDispatched: (run: ActionsRunListItem) => void;
}) {
  const defaultRef =
    detail.refs.find(
      (ref) =>
        ref.kind === "branch" &&
        ref.shortName === detail.repository.defaultBranch,
    ) ??
    detail.refs.find((ref) => ref.kind === "branch") ??
    detail.refs[0];
  const [selectedRef, setSelectedRef] = useState(defaultRef?.shortName ?? "");
  const [inputs, setInputs] = useState<Record<string, string>>(() =>
    Object.fromEntries(
      detail.workflow.dispatch.inputs.map((input) => [
        input.name,
        defaultInputValue(input),
      ]),
    ),
  );
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setSubmitting(true);
    setError(null);
    try {
      const response = await fetch(`${basePath}/actions/workflows/dispatches`, {
        body: JSON.stringify({
          workflowFile: detail.workflow.path,
          ref: selectedRef,
          inputs,
        }),
        headers: { "content-type": "application/json" },
        method: "POST",
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Workflow dispatch could not be queued.",
        );
      }
      onDispatched(body as ActionsRunListItem);
    } catch (dispatchError) {
      setError(
        dispatchError instanceof Error
          ? dispatchError.message
          : "Workflow dispatch could not be queued.",
      );
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div
      aria-label="Run workflow"
      aria-modal="true"
      className="fixed inset-0 z-40 flex items-start justify-center px-4 py-16"
      role="dialog"
      style={{
        background: "color-mix(in oklch, var(--ink-1) 24%, transparent)",
      }}
    >
      <div className="card w-full max-w-xl p-5 shadow-[var(--shadow-lg)]">
        <div className="between gap-4">
          <div>
            <p className="t-label">Manual dispatch</p>
            <h2 className="t-h2 mt-1">Run {detail.workflow.name}</h2>
          </div>
          <button
            aria-label="Close run workflow dialog"
            className="btn sm"
            onClick={onCancel}
            type="button"
          >
            ×
          </button>
        </div>
        <div className="mt-4 space-y-4">
          <label className="block" htmlFor="dispatch-ref">
            <span className="t-sm font-medium">Branch or tag</span>
            <select
              className="input mt-2 w-full"
              id="dispatch-ref"
              onChange={(event) => setSelectedRef(event.target.value)}
              value={selectedRef}
            >
              {detail.refs.map((ref: ActionsWorkflowRef) => (
                <option key={ref.name} value={ref.shortName}>
                  {ref.shortName} · {ref.kind}
                </option>
              ))}
            </select>
          </label>
          {detail.workflow.dispatch.inputs.map((input) => (
            <DispatchField
              input={input}
              key={input.name}
              onChange={(value) =>
                setInputs((current) => ({ ...current, [input.name]: value }))
              }
              value={inputs[input.name] ?? ""}
            />
          ))}
        </div>
        {error ? (
          <div className="chip err mt-4" role="alert">
            {error}
          </div>
        ) : null}
        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <button className="btn" onClick={onCancel} type="button">
            Cancel
          </button>
          <button
            className="btn accent"
            disabled={submitting || !selectedRef}
            onClick={submit}
            type="button"
          >
            {submitting ? "Queuing..." : "Run workflow"}
          </button>
        </div>
      </div>
    </div>
  );
}

export function RepositoryActionsWorkflowPage({
  repository,
  detail,
  query,
  validationError = null,
}: RepositoryActionsWorkflowPageProps) {
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const latestRef = detail.refs[0];
  const [dispatchOpen, setDispatchOpen] = useState(false);
  const [queuedRuns, setQueuedRuns] = useState<ActionsRunListItem[]>([]);
  const runs = [...queuedRuns, ...detail.runs.items];
  return (
    <RepositoryShell
      activePath={`${basePath}/actions`}
      frameClassName="grid grid-cols-[260px_minmax(0,1fr)] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <WorkflowRail basePath={basePath} detail={detail} />
      <main className="min-w-0">
        <div className="mb-5 flex flex-wrap items-end justify-between gap-4">
          <div className="min-w-0">
            <p className="t-label">Workflow</p>
            <h1 className="t-h1 mt-1 break-words">{detail.workflow.name}</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {detail.runs.total} scoped runs ·{" "}
              <span className="t-mono-sm">{detail.workflow.path}</span>
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn" href={detail.workflow.sourceHref}>
              Workflow file
            </Link>
            <Link className="btn ghost" href={`${basePath}/settings/actions`}>
              Workflow options
            </Link>
            <Link
              className="btn ghost"
              href="/docs/api#actions-workflow-dispatch"
            >
              Dispatch API
            </Link>
            {detail.workflow.dispatch.enabled ? (
              <button
                className="btn accent"
                onClick={() => setDispatchOpen(true)}
                type="button"
              >
                Run workflow
              </button>
            ) : null}
          </div>
        </div>
        {validationError ? (
          <div className="chip err mb-4" role="alert">
            {validationError.error.message}
          </div>
        ) : null}
        {!detail.workflow.valid ? (
          <div className="card mb-4 p-4" role="alert">
            <p className="t-label" style={{ color: "var(--err)" }}>
              Invalid workflow file
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              {detail.workflow.yamlParseError ??
                "The workflow YAML could not be parsed."}
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              <Link className="btn sm" href={detail.workflow.sourceHref}>
                Open YAML
              </Link>
              <Link
                className="btn sm ghost"
                href="/docs/api#actions-workflow-detail"
              >
                Read workflow API docs
              </Link>
            </div>
          </div>
        ) : null}
        <div className="mb-4 grid gap-3 md:grid-cols-3">
          <div className="card p-4">
            <p className="t-label">Source</p>
            <Link
              className="t-sm mt-2 block hover:underline"
              href={detail.workflow.sourceHref}
            >
              {detail.workflow.sourceBranch}
            </Link>
            <p className="t-xs mt-1">
              {detail.workflow.sourceSha ?? "No source SHA recorded"}
            </p>
            <p className="t-xs mt-1">
              Parsed {dateTimeLabel(detail.workflow.yamlParsedAt)}
            </p>
          </div>
          <div className="card p-4">
            <p className="t-label">Triggers</p>
            <div className="mt-2 flex flex-wrap gap-2">
              {detail.workflow.triggerEvents.length ? (
                detail.workflow.triggerEvents.map((event) => (
                  <span className="chip soft" key={event}>
                    {event.replaceAll("_", " ")}
                  </span>
                ))
              ) : (
                <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No triggers recorded
                </span>
              )}
            </div>
          </div>
          <div className="card p-4">
            <p className="t-label">Refs</p>
            <p className="t-sm mt-2">
              {latestRef
                ? `${latestRef.shortName} · ${latestRef.kind}`
                : "No branches or tags recorded"}
            </p>
            <p className="t-xs mt-1">
              {detail.refs.length} refs available for dispatch
            </p>
          </div>
        </div>
        <FilterSummary basePath={basePath} detail={detail} query={query} />
        <section
          aria-label="Workflow runs"
          className="card mt-4 overflow-hidden"
        >
          <div
            className="between border-b px-5 py-3"
            style={{ borderColor: "var(--line)" }}
          >
            <h2 className="t-h3">Recent runs</h2>
            <span className="chip soft t-num">
              {detail.runs.total + queuedRuns.length}
            </span>
          </div>
          {runs.length ? (
            runs.map((run) => (
              <RunRow basePath={basePath} key={run.id} run={run} />
            ))
          ) : (
            <div className="p-5">
              <EmptyWorkflowState basePath={basePath} detail={detail} />
            </div>
          )}
        </section>
        {dispatchOpen ? (
          <DispatchDialog
            basePath={basePath}
            detail={detail}
            onCancel={() => setDispatchOpen(false)}
            onDispatched={(run) => {
              setQueuedRuns((current) => [run, ...current]);
              setDispatchOpen(false);
            }}
          />
        ) : null}
      </main>
    </RepositoryShell>
  );
}
