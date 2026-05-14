"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import type { ReactNode } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
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

function runArchiveHref(basePath: string, runId: string) {
  return `${basePath}/actions/runs/${runId}/logs/archive`;
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
  const router = useRouter();
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const jobPath = jobLogHref(basePath, detail.run.id, detail.job.id);
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(
    () => new Set(detail.steps.map((step) => step.id ?? "job-log")),
  );
  const [annotationsVisible, setAnnotationsVisible] = useState(true);
  const [optionsOpen, setOptionsOpen] = useState(false);
  const [optionMessage, setOptionMessage] = useState("");
  const [preferencesPending, setPreferencesPending] = useState(false);
  const [searchText, setSearchText] = useState(detail.search.query ?? "");
  const [copyMessage, setCopyMessage] = useState("");
  const [streamMessage, setStreamMessage] = useState(
    detail.logState.isLive && detail.logState.available
      ? "Live stream connected"
      : "",
  );
  const selectedMatch = normalizeSelectedMatch(
    detail.search.selectedMatch,
    detail.search.totalMatches,
  );
  const currentMatch = selectedMatch
    ? detail.search.matches[selectedMatch - 1]
    : null;
  const currentAnchorRef = useRef<string | null>(null);
  const groups = useMemo(() => groupedJobs(detail.jobs), [detail.jobs]);
  const matchAnchors = useMemo(
    () => new Set(detail.search.matches.map((match) => match.anchor)),
    [detail.search.matches],
  );

  useEffect(() => {
    setSearchText(detail.search.query ?? "");
  }, [detail.search.query]);

  useEffect(() => {
    if (!detail.logState.isLive || !detail.logState.available) {
      return;
    }
    if (typeof window.EventSource === "function") {
      const source = new window.EventSource(
        `${basePath}/actions/jobs/${detail.job.id}/logs/stream?after=${
          detail.logState.nextCursor ?? 0
        }`,
      );
      source.addEventListener("line", () => {
        setStreamMessage("New log lines received");
        router.refresh();
      });
      source.addEventListener("cursor", () => {
        setStreamMessage("Live stream connected");
      });
      source.onerror = () => {
        setStreamMessage("Live stream reconnecting");
        source.close();
      };
      return () => source.close();
    }
    const timer = window.setInterval(() => {
      router.refresh();
    }, 5000);
    return () => window.clearInterval(timer);
  }, [
    basePath,
    detail.job.id,
    detail.logState.available,
    detail.logState.isLive,
    detail.logState.nextCursor,
    router,
  ]);

  useEffect(() => {
    if (!currentMatch || currentAnchorRef.current === currentMatch.anchor) {
      return;
    }
    currentAnchorRef.current = currentMatch.anchor;
    window.requestAnimationFrame(() => {
      const matchElement = document.getElementById(
        `log-${currentMatch.anchor}`,
      );
      if (typeof matchElement?.scrollIntoView === "function") {
        matchElement.scrollIntoView({ block: "center" });
      }
    });
  }, [currentMatch]);

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

  function searchHref(
    nextQuery: string,
    nextMatch = 1,
    overrides: { timestamps?: boolean; raw?: boolean } = {},
  ) {
    const params = new URLSearchParams();
    const trimmed = nextQuery.trim();
    if (trimmed) {
      params.set("q", trimmed);
      params.set("match", String(nextMatch));
    }
    const timestamps = overrides.timestamps ?? detail.options.showTimestamps;
    const raw = overrides.raw ?? detail.options.rawLogs;
    if (timestamps !== true) {
      params.set("timestamps", String(timestamps));
    }
    if (raw) {
      params.set("raw", String(raw));
    }
    const queryString = params.toString();
    return queryString ? `${jobPath}?${queryString}` : jobPath;
  }

  function goToMatch(nextMatch: number) {
    if (!detail.search.query || detail.search.totalMatches < 1) {
      return;
    }
    router.push(searchHref(detail.search.query, nextMatch));
  }

  async function copyPermalink(anchor?: string) {
    const suffix = anchor ? `#log-${anchor}` : "";
    const href = `${window.location.origin}${jobPath}${suffix}`;
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(href);
      } else {
        fallbackCopyText(href);
      }
      setCopyMessage(anchor ? `Copied ${anchor}` : "Copied job permalink");
    } catch {
      try {
        fallbackCopyText(href);
        setCopyMessage(anchor ? `Copied ${anchor}` : "Copied job permalink");
      } catch {
        setCopyMessage("Copy failed");
      }
    }
    window.setTimeout(() => setCopyMessage(""), 1800);
  }

  async function updatePreferences(
    nextOptions: Partial<typeof detail.options>,
  ) {
    const merged = { ...detail.options, ...nextOptions };
    setPreferencesPending(true);
    setOptionMessage("");
    try {
      const response = await fetch(`${basePath}/actions/log-preferences`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          showTimestamps: merged.showTimestamps,
          rawLogs: merged.rawLogs,
          wrapLines: merged.wrapLines,
        }),
      });
      if (!response.ok) {
        throw new Error("preference_write_failed");
      }
      const href = searchHref(detail.search.query ?? "", selectedMatch || 1, {
        timestamps: merged.showTimestamps,
        raw: merged.rawLogs,
      });
      setOptionMessage("Saved log options");
      setOptionsOpen(false);
      router.push(href);
      router.refresh();
    } catch {
      setOptionMessage("Could not save log options");
    } finally {
      setPreferencesPending(false);
      window.setTimeout(() => setOptionMessage(""), 2200);
    }
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
                  {streamMessage ? (
                    <p
                      className="t-xs mt-2"
                      role="status"
                      style={{ color: "var(--ink-3)" }}
                    >
                      {streamMessage}
                    </p>
                  ) : null}
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
                    <button
                      className="list-row w-full justify-between gap-3 py-2 text-left"
                      disabled={preferencesPending}
                      onClick={() =>
                        updatePreferences({
                          showTimestamps: !detail.options.showTimestamps,
                        })
                      }
                      role="menuitemcheckbox"
                      aria-checked={detail.options.showTimestamps}
                      type="button"
                    >
                      <span>Show timestamps</span>
                      <span className="chip soft">
                        {detail.options.showTimestamps ? "On" : "Off"}
                      </span>
                    </button>
                    <button
                      className="list-row w-full justify-between gap-3 py-2 text-left"
                      disabled={preferencesPending}
                      onClick={() =>
                        updatePreferences({ rawLogs: !detail.options.rawLogs })
                      }
                      role="menuitemcheckbox"
                      aria-checked={detail.options.rawLogs}
                      type="button"
                    >
                      <span>Raw logs</span>
                      <span className="chip soft">
                        {detail.options.rawLogs ? "On" : "Off"}
                      </span>
                    </button>
                    <button
                      className="list-row w-full justify-between gap-3 py-2 text-left"
                      disabled={preferencesPending}
                      onClick={() =>
                        updatePreferences({
                          wrapLines: !detail.options.wrapLines,
                        })
                      }
                      role="menuitemcheckbox"
                      aria-checked={detail.options.wrapLines}
                      type="button"
                    >
                      <span>Wrap lines</span>
                      <span className="chip soft">
                        {detail.options.wrapLines ? "On" : "Off"}
                      </span>
                    </button>
                    <button
                      className="list-row w-full py-2 text-left"
                      onClick={() => copyPermalink()}
                      role="menuitem"
                      type="button"
                    >
                      Copy job permalink
                    </button>
                    {optionMessage ? (
                      <p className="t-xs mt-2" role="status">
                        {optionMessage}
                      </p>
                    ) : null}
                  </div>
                ) : null}
              </div>
              {detail.logState.available ? (
                <>
                  <Link
                    className="btn"
                    href={searchHref(
                      detail.search.query ?? "",
                      selectedMatch || 1,
                      { raw: true },
                    )}
                  >
                    Raw view
                  </Link>
                  <a
                    className="btn"
                    href={runArchiveHref(basePath, detail.run.id)}
                  >
                    Download run archive
                  </a>
                  <a
                    className="btn primary"
                    href={jobDownloadHref(basePath, detail.job.id)}
                  >
                    Download gzip
                  </a>
                </>
              ) : (
                <>
                  <button className="btn" disabled type="button">
                    Raw view
                  </button>
                  <button className="btn" disabled type="button">
                    Download run archive
                  </button>
                  <button className="btn primary" disabled type="button">
                    Download gzip
                  </button>
                </>
              )}
            </div>
          </div>
          {optionMessage ? (
            <p className="t-xs" role="status" style={{ color: "var(--ink-3)" }}>
              {optionMessage}
            </p>
          ) : null}
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
                <form
                  className="flex flex-wrap items-center gap-2"
                  onSubmit={(event) => {
                    event.preventDefault();
                    router.push(searchHref(searchText));
                  }}
                >
                  <div className="input h-9 min-w-[240px]">
                    <span aria-hidden="true">⌕</span>
                    <input
                      aria-label="Search log"
                      onChange={(event) => setSearchText(event.target.value)}
                      placeholder="Search log"
                      value={searchText}
                    />
                  </div>
                  <button className="btn sm" type="submit">
                    Search
                  </button>
                  <button
                    className="btn sm"
                    disabled={selectedMatch <= 1}
                    onClick={() => goToMatch(selectedMatch - 1)}
                    type="button"
                  >
                    Previous result
                  </button>
                  <button
                    className="btn sm"
                    disabled={
                      selectedMatch < 1 ||
                      selectedMatch >= detail.search.totalMatches
                    }
                    onClick={() => goToMatch(selectedMatch + 1)}
                    type="button"
                  >
                    Next result
                  </button>
                  <span className="chip soft t-num">
                    {detail.search.totalMatches
                      ? `${selectedMatch} of ${detail.search.totalMatches}`
                      : "0"}{" "}
                    matches
                  </span>
                </form>
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
                        onCopyPermalink={copyPermalink}
                        query={detail.search.query}
                        showTimestamps={detail.options.showTimestamps}
                        rawLogs={detail.options.rawLogs}
                        wrapLines={detail.options.wrapLines}
                        step={step}
                        currentAnchor={currentMatch?.anchor ?? null}
                        matchingAnchors={matchAnchors}
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
            <div aria-live="polite" className="t-xs min-h-4">
              {copyMessage}
            </div>
          </main>
        </div>
      </div>
    </RepositoryShell>
  );
}

function fallbackCopyText(value: string) {
  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  const copied = document.execCommand("copy");
  document.body.removeChild(textarea);
  if (!copied) {
    throw new Error("copy_failed");
  }
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
  currentAnchor,
  expanded,
  matchingAnchors,
  onCopyPermalink,
  onToggle,
  query,
  rawLogs,
  showTimestamps,
  step,
  wrapLines,
}: {
  currentAnchor: string | null;
  expanded: boolean;
  matchingAnchors: Set<string>;
  onCopyPermalink: (anchor: string) => void;
  onToggle: () => void;
  query: string | null;
  rawLogs: boolean;
  showTimestamps: boolean;
  step: ActionsJobLogStep;
  wrapLines: boolean;
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
                className="grid grid-cols-[72px_minmax(0,1fr)_auto] gap-3 px-5 py-1"
                id={`log-${line.anchor}`}
                key={line.anchor}
                style={{
                  background:
                    line.anchor === currentAnchor
                      ? "var(--accent-soft)"
                      : "transparent",
                }}
              >
                <a
                  className="t-mono-sm text-right hover:underline"
                  href={`#log-${line.anchor}`}
                  style={{ color: "var(--ink-4)" }}
                >
                  {line.lineNumber}
                </a>
                <code
                  className={`t-mono-sm ${wrapLines ? "whitespace-pre-wrap break-words" : "whitespace-pre"}`}
                >
                  {showTimestamps && line.timestamp ? (
                    <span style={{ color: "var(--ink-4)" }}>
                      {dateTimeLabel(line.timestamp)}{" "}
                    </span>
                  ) : null}
                  {rawLogs ? (
                    <span style={{ color: "var(--ink-4)" }}>
                      {line.anchor}{" "}
                    </span>
                  ) : null}
                  {renderHighlightedLogLine(
                    line.content,
                    query,
                    matchingAnchors.has(line.anchor),
                  )}
                </code>
                <button
                  aria-label={`Copy permalink for line ${line.lineNumber}`}
                  className="btn sm"
                  onClick={() => onCopyPermalink(line.anchor)}
                  type="button"
                >
                  Copy
                </button>
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

function normalizeSelectedMatch(
  selectedMatch: number | null,
  totalMatches: number,
) {
  if (totalMatches < 1) {
    return 0;
  }
  if (!selectedMatch || selectedMatch < 1) {
    return 1;
  }
  return Math.min(selectedMatch, totalMatches);
}

function renderHighlightedLogLine(
  content: string,
  query: string | null,
  isMatchedLine: boolean,
) {
  const trimmed = query?.trim();
  if (!trimmed || !isMatchedLine) {
    return content;
  }
  const lowerContent = content.toLowerCase();
  const lowerQuery = trimmed.toLowerCase();
  const parts: ReactNode[] = [];
  let cursor = 0;
  let index = lowerContent.indexOf(lowerQuery);
  while (index >= 0) {
    if (index > cursor) {
      parts.push(content.slice(cursor, index));
    }
    const end = index + trimmed.length;
    parts.push(
      <mark
        key={`${index}-${end}`}
        style={{
          background: "var(--accent-soft)",
          color: "var(--surface)",
          outline: "1px solid var(--accent)",
        }}
      >
        {content.slice(index, end)}
      </mark>,
    );
    cursor = end;
    index = lowerContent.indexOf(lowerQuery, cursor);
  }
  if (cursor < content.length) {
    parts.push(content.slice(cursor));
  }
  return parts;
}
