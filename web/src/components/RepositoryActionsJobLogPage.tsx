"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ActionsJobLogStep,
  ActionsRunJobDetail,
  ApiErrorEnvelope,
  RepositoryActionsJobLogDetail,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsJobLogPageProps = {
  repository: RepositoryOverview;
  detail: RepositoryActionsJobLogDetail;
  validationError?: ApiErrorEnvelope | null;
};

function titleCase(value: string | null | undefined) {
  if (!value) {
    return "Unknown";
  }
  return value
    .replaceAll("_", " ")
    .replaceAll("-", " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
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
  });
}

function statusTone(status: string | null | undefined) {
  const normalized = status ?? "";
  if (["success", "completed"].includes(normalized)) {
    return "ok";
  }
  if (
    ["failure", "timed_out", "cancelled", "action_required"].includes(
      normalized,
    )
  ) {
    return "err";
  }
  if (["in_progress", "queued", "waiting"].includes(normalized)) {
    return "accent";
  }
  if (["skipped", "neutral", "stale"].includes(normalized)) {
    return "warn";
  }
  return "soft";
}

function statusLabel(status: string | null, conclusion: string | null) {
  return titleCase(conclusion ?? status);
}

function statusGlyph(status: string | null, conclusion: string | null) {
  const tone = statusTone(conclusion ?? status);
  if (tone === "ok") {
    return "✓";
  }
  if (tone === "err") {
    return "!";
  }
  if (tone === "accent") {
    return "•";
  }
  return "○";
}

function jobLogHref(basePath: string, runId: string, jobId: string) {
  return `${basePath}/actions/runs/${runId}/jobs/${jobId}`;
}

function jobDownloadHref(basePath: string, jobId: string) {
  return `${basePath}/actions/jobs/${jobId}/logs/download`;
}

function groupedJobs(jobs: ActionsRunJobDetail[]) {
  const groups = new Map<string, ActionsRunJobDetail[]>();
  for (const job of jobs) {
    const group = job.groupName ?? "All jobs";
    groups.set(group, [...(groups.get(group) ?? []), job]);
  }
  return [...groups.entries()];
}

export function RepositoryActionsJobLogPage({
  repository,
  detail,
  validationError,
}: RepositoryActionsJobLogPageProps) {
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(
    () => new Set(detail.steps.map((step) => step.id ?? "job-log")),
  );
  const [annotationsVisible, setAnnotationsVisible] = useState(true);
  const [optionsOpen, setOptionsOpen] = useState(false);
  const groups = useMemo(() => groupedJobs(detail.jobs), [detail.jobs]);

  function toggleStep(stepKey: string) {
    setExpandedSteps((current) => {
      const next = new Set(current);
      if (next.has(stepKey)) {
        next.delete(stepKey);
      } else {
        next.add(stepKey);
      }
      return next;
    });
  }

  return (
    <RepositoryShell
      activePath={`${basePath}/actions/runs/${detail.run.id}/jobs/${detail.job.id}`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="space-y-6">
        {validationError ? (
          <div className="card p-4" role="status">
            <p className="t-label" style={{ color: "var(--err)" }}>
              Job log unavailable
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              {validationError.error.message}
            </p>
          </div>
        ) : null}

        <header className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Link className="t-sm hover:underline" href={`${basePath}/actions`}>
              Actions
            </Link>
            <span className="t-xs">/</span>
            <Link
              className="t-sm hover:underline"
              href={`${basePath}/actions/runs/${detail.run.id}`}
            >
              Run #{detail.run.runNumber}
            </Link>
          </div>
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0 flex-1">
              <div className="flex items-start gap-3">
                <span
                  aria-label={`${statusLabel(detail.job.status, detail.job.conclusion)} job`}
                  className={`chip ${statusTone(detail.job.conclusion ?? detail.job.status)}`}
                  role="img"
                >
                  {statusGlyph(detail.job.status, detail.job.conclusion)}
                </span>
                <div className="min-w-0">
                  <h1 className="t-h1 break-words">{detail.job.name}</h1>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    {detail.workflow.name} · {detail.run.displayTitle} ·{" "}
                    <span className="t-mono-sm">{detail.run.headBranch}</span>
                    {detail.run.shortSha ? (
                      <>
                        {" "}
                        at{" "}
                        <span className="t-mono-sm">{detail.run.shortSha}</span>
                      </>
                    ) : null}
                  </p>
                </div>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <button
                className="btn"
                onClick={() => setAnnotationsVisible((visible) => !visible)}
                type="button"
              >
                {annotationsVisible ? "Hide" : "Show"} annotations
              </button>
              <div className="relative">
                <button
                  aria-expanded={optionsOpen}
                  className="btn"
                  onClick={() => setOptionsOpen((open) => !open)}
                  type="button"
                >
                  Log options
                </button>
                {optionsOpen ? (
                  <div
                    className="card absolute right-0 z-10 mt-2 w-64 p-3"
                    role="menu"
                  >
                    <p className="t-label mb-2">Display</p>
                    <p className="t-sm" style={{ color: "var(--ink-2)" }}>
                      Timestamps{" "}
                      {detail.options.showTimestamps ? "shown" : "hidden"} ·{" "}
                      {detail.options.rawLogs ? "raw" : "rendered"} logs
                    </p>
                    <p className="t-xs mt-2">
                      Preference writes and archive actions land in the next
                      phase.
                    </p>
                  </div>
                ) : null}
              </div>
              {detail.logState.available ? (
                <a
                  className="btn primary"
                  href={jobDownloadHref(basePath, detail.job.id)}
                >
                  Download log
                </a>
              ) : (
                <button className="btn primary" disabled type="button">
                  Download log
                </button>
              )}
            </div>
          </div>
        </header>

        <div className="grid grid-cols-[300px_minmax(0,1fr)] gap-6 max-lg:grid-cols-1">
          <aside className="space-y-4">
            <nav aria-label="Workflow run jobs" className="card p-3">
              <div className="mb-2 flex items-center justify-between gap-3">
                <p className="t-label">Jobs</p>
                <span className="chip soft t-num">{detail.jobs.length}</span>
              </div>
              {groups.map(([group, jobs]) => (
                <div className="mt-3" key={group}>
                  <p className="t-xs mb-1 px-3">{group}</p>
                  <div className="space-y-1">
                    {jobs.map((job) => {
                      const active = job.id === detail.job.id;
                      return (
                        <Link
                          aria-current={active ? "page" : undefined}
                          className="flex w-full items-center gap-2 rounded-[var(--radius)] px-3 py-2 text-left text-sm hover:bg-[var(--hover)]"
                          href={jobLogHref(basePath, detail.run.id, job.id)}
                          key={job.id}
                          style={{
                            background: active
                              ? "var(--accent-soft)"
                              : "transparent",
                          }}
                        >
                          <span
                            className={`chip ${statusTone(job.conclusion ?? job.status)} h-6 w-6 justify-center px-0`}
                          >
                            {statusGlyph(job.status, job.conclusion)}
                          </span>
                          <span className="min-w-0 flex-1 truncate">
                            {job.name}
                          </span>
                          <span className="t-xs t-num">
                            {durationLabel(job.durationSeconds)}
                          </span>
                        </Link>
                      );
                    })}
                  </div>
                </div>
              ))}
            </nav>

            <section className="card p-4">
              <p className="t-label">Run</p>
              <h2 className="t-h3 mt-2">{detail.run.displayTitle}</h2>
              <dl className="mt-3 space-y-2">
                <MetadataRow
                  label="Status"
                  value={statusLabel(detail.run.status, detail.run.conclusion)}
                />
                <MetadataRow
                  label="Duration"
                  value={durationLabel(detail.job.durationSeconds)}
                />
                <MetadataRow
                  label="Started"
                  value={dateTimeLabel(detail.job.startedAt)}
                />
              </dl>
            </section>
          </aside>

          <main className="min-w-0 space-y-4">
            <section className="card overflow-hidden">
              <div
                className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-4"
                style={{ borderColor: "var(--line)" }}
              >
                <div>
                  <p className="t-label">Job log</p>
                  <h2 className="t-h2 mt-1">Steps and output</h2>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <div className="input h-9 min-w-[240px]">
                    <span aria-hidden="true">⌕</span>
                    <input
                      aria-label="Search log"
                      defaultValue={detail.search.query ?? ""}
                      placeholder="Search log"
                      readOnly
                    />
                  </div>
                  <button className="btn sm" disabled type="button">
                    Previous result
                  </button>
                  <button className="btn sm" disabled type="button">
                    Next result
                  </button>
                  <span className="chip soft t-num">
                    {detail.search.totalMatches} matches
                  </span>
                </div>
              </div>

              {!detail.logState.available ? (
                <div className="p-5" role="status">
                  <p className="t-label" style={{ color: "var(--err)" }}>
                    {detail.logState.status} unavailable
                  </p>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    {detail.logState.reason ??
                      "Workflow logs are unavailable for this job."}
                  </p>
                </div>
              ) : (
                <div
                  className="divide-y"
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  {detail.steps.map((step) => {
                    const stepKey = step.id ?? "job-log";
                    const expanded = expandedSteps.has(stepKey);
                    return (
                      <JobLogStep
                        expanded={expanded}
                        key={stepKey}
                        onToggle={() => toggleStep(stepKey)}
                        showTimestamps={detail.options.showTimestamps}
                        step={step}
                      />
                    );
                  })}
                </div>
              )}
            </section>

            {annotationsVisible ? (
              <section className="card overflow-hidden">
                <div
                  className="border-b px-5 py-4"
                  style={{ borderColor: "var(--line)" }}
                >
                  <p className="t-label">Annotations</p>
                  <h2 className="t-h2 mt-1">Problems in this job</h2>
                </div>
                {detail.annotations.length ? (
                  <div
                    className="divide-y"
                    style={{ borderColor: "var(--line-soft)" }}
                  >
                    {detail.annotations.map((annotation) => (
                      <div className="list-row px-5 py-4" key={annotation.id}>
                        <span
                          className={`chip ${statusTone(annotation.level)}`}
                        >
                          {titleCase(annotation.level)}
                        </span>
                        <div className="min-w-0 flex-1">
                          <p className="t-sm font-medium">
                            {annotation.title ?? annotation.message}
                          </p>
                          <p className="t-xs mt-1">
                            {annotation.path ?? "Workflow"}{" "}
                            {annotation.startLine
                              ? `line ${annotation.startLine}`
                              : ""}
                          </p>
                          {annotation.title ? (
                            <p
                              className="t-sm mt-2"
                              style={{ color: "var(--ink-3)" }}
                            >
                              {annotation.message}
                            </p>
                          ) : null}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="t-sm p-5" style={{ color: "var(--ink-3)" }}>
                    No annotations were emitted for this job.
                  </p>
                )}
              </section>
            ) : null}
          </main>
        </div>
      </div>
    </RepositoryShell>
  );
}

function MetadataRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <dt className="t-xs">{label}</dt>
      <dd className="t-sm text-right font-medium">{value}</dd>
    </div>
  );
}

function JobLogStep({
  expanded,
  onToggle,
  showTimestamps,
  step,
}: {
  expanded: boolean;
  onToggle: () => void;
  showTimestamps: boolean;
  step: ActionsJobLogStep;
}) {
  return (
    <div>
      <button
        aria-expanded={expanded}
        className="flex w-full items-center gap-3 px-5 py-4 text-left hover:bg-[var(--hover)]"
        onClick={onToggle}
        type="button"
      >
        <span
          className={`chip ${statusTone(step.conclusion ?? step.status)} h-6 w-6 justify-center px-0`}
        >
          {statusGlyph(step.status, step.conclusion)}
        </span>
        <span className="min-w-0 flex-1">
          <span className="t-sm block font-medium">{step.name}</span>
          <span className="t-xs">
            Step {step.number || "log"} · {step.lines.total} lines
            {step.matchCount ? ` · ${step.matchCount} matches` : ""}
          </span>
        </span>
        <span className="t-xs t-num">
          {durationLabel(step.durationSeconds)}
        </span>
        <span className="t-xs" aria-hidden="true">
          {expanded ? "▾" : "▸"}
        </span>
      </button>
      {expanded ? (
        <ol
          className="max-h-[420px] overflow-auto py-3"
          style={{
            background: "var(--ink-1)",
            color: "var(--surface)",
            fontFamily: "var(--mono)",
          }}
        >
          {step.lines.items.length ? (
            step.lines.items.map((line) => (
              <li
                className="grid grid-cols-[72px_minmax(0,1fr)] gap-3 px-5 py-1"
                id={`log-${line.anchor}`}
                key={line.anchor}
              >
                <a
                  className="t-mono-sm text-right hover:underline"
                  href={`#log-${line.anchor}`}
                  style={{ color: "var(--ink-4)" }}
                >
                  {line.lineNumber}
                </a>
                <code className="t-mono-sm whitespace-pre-wrap break-words">
                  {showTimestamps && line.timestamp ? (
                    <span style={{ color: "var(--ink-4)" }}>
                      {dateTimeLabel(line.timestamp)}{" "}
                    </span>
                  ) : null}
                  {line.content}
                </code>
              </li>
            ))
          ) : (
            <li
              className="t-mono-sm px-5 py-1"
              style={{ color: "var(--ink-4)" }}
            >
              No log lines in this step.
            </li>
          )}
        </ol>
      ) : null}
    </div>
  );
}
