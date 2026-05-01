"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ActionsArtifactDownload,
  ActionsJobLog,
  ActionsRunJobDetail,
  ApiErrorEnvelope,
  RepositoryActionsRunDetail,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsRunPageProps = {
  repository: RepositoryOverview;
  detail: RepositoryActionsRunDetail;
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

function bytesLabel(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  const kib = value / 1024;
  if (kib < 1024) {
    return `${kib.toFixed(kib >= 10 ? 0 : 1)} KB`;
  }
  const mib = kib / 1024;
  return `${mib.toFixed(mib >= 10 ? 0 : 1)} MB`;
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

function jobHref(basePath: string, runId: string, jobId: string) {
  return `${basePath}/actions/runs/${runId}#job-${jobId}`;
}

function jobState(job: ActionsRunJobDetail) {
  return statusLabel(job.status, job.conclusion);
}

function jobLogsPath(basePath: string, jobId: string, query?: string) {
  const params = new URLSearchParams();
  if (query?.trim()) {
    params.set("q", query.trim());
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `${basePath}/actions/jobs/${jobId}/logs${suffix}`;
}

function jobLogDownloadPath(basePath: string, jobId: string) {
  return `${basePath}/actions/jobs/${jobId}/logs/download`;
}

function artifactDownloadPath(
  basePath: string,
  artifactId: string,
  options: { metadata?: boolean } = {},
) {
  const suffix = options.metadata ? "?metadata=1" : "";
  return `${basePath}/actions/artifacts/${artifactId}/download${suffix}`;
}

function runMutationPath(
  basePath: string,
  runId: string,
  action: "rerun" | "cancel" | "logs",
) {
  return `${basePath}/actions/runs/${runId}/${action}`;
}

export function RepositoryActionsRunPage({
  repository,
  detail,
  validationError,
}: RepositoryActionsRunPageProps) {
  const router = useRouter();
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const [selectedJobId, setSelectedJobId] = useState(
    detail.jobs[0]?.id ?? "summary",
  );
  const [logQuery, setLogQuery] = useState("");
  const [submittedLogQuery, setSubmittedLogQuery] = useState("");
  const [jobLog, setJobLog] = useState<ActionsJobLog | null>(null);
  const [jobLogState, setJobLogState] = useState<"idle" | "loading" | "error">(
    "idle",
  );
  const [jobLogMessage, setJobLogMessage] = useState("");
  const [artifactMessage, setArtifactMessage] = useState("");
  const [actionMessage, setActionMessage] = useState("");
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [confirmDeleteLogs, setConfirmDeleteLogs] = useState(false);
  const selectedJob = detail.jobs.find((job) => job.id === selectedJobId);
  const groupedJobs = useMemo(() => {
    const groups = new Map<string, ActionsRunJobDetail[]>();
    for (const job of detail.jobs) {
      const group = job.groupName ?? "All jobs";
      groups.set(group, [...(groups.get(group) ?? []), job]);
    }
    return [...groups.entries()];
  }, [detail.jobs]);
  const actionDisabledReason =
    detail.actionState.disabledReason ?? "This action lands in the next phase.";

  useEffect(() => {
    if (!selectedJob?.logAvailable) {
      setJobLog(null);
      setJobLogMessage("");
      setJobLogState("idle");
      return;
    }

    let cancelled = false;
    setJobLogState("loading");
    setJobLogMessage("");
    fetch(jobLogsPath(basePath, selectedJob.id, submittedLogQuery), {
      cache: "no-store",
    })
      .then(async (response) => {
        const body = await response.json().catch(() => null);
        if (!response.ok) {
          throw new Error(
            body?.error?.message ?? "Job logs could not be loaded.",
          );
        }
        return body as ActionsJobLog;
      })
      .then((body) => {
        if (!cancelled) {
          setJobLog(body);
          setJobLogState("idle");
        }
      })
      .catch((error: Error) => {
        if (!cancelled) {
          setJobLog(null);
          setJobLogState("error");
          setJobLogMessage(error.message);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [basePath, selectedJob, submittedLogQuery]);

  async function copyArtifactDownload(artifactId: string) {
    setArtifactMessage("");
    try {
      const response = await fetch(
        artifactDownloadPath(basePath, artifactId, { metadata: true }),
        {
          cache: "no-store",
        },
      );
      const body = (await response.json().catch(() => null)) as
        | ActionsArtifactDownload
        | ApiErrorEnvelope
        | null;
      if (!response.ok || !body || "error" in body) {
        throw new Error(
          body && "error" in body
            ? body.error.message
            : "Artifact download is unavailable.",
        );
      }
      await navigator.clipboard?.writeText(body.downloadUrl);
      setArtifactMessage(`Copied ${body.filename} download URL.`);
    } catch (error) {
      setArtifactMessage(
        error instanceof Error
          ? error.message
          : "Artifact download is unavailable.",
      );
    }
  }

  async function mutateRun(
    action: "rerun" | "cancel" | "logs",
    options: { mode?: "all" | "failed" | "job"; jobId?: string } = {},
  ) {
    const label =
      action === "logs"
        ? "delete logs"
        : action === "cancel"
          ? "cancel run"
          : options.mode === "failed"
            ? "re-run failed jobs"
            : options.mode === "job"
              ? "re-run job"
              : "re-run all jobs";
    setPendingAction(label);
    setActionMessage("");
    try {
      const response = await fetch(
        runMutationPath(basePath, detail.run.id, action),
        {
          method: action === "logs" ? "DELETE" : "POST",
          headers:
            action === "rerun" ? { "content-type": "application/json" } : {},
          body:
            action === "rerun"
              ? JSON.stringify({
                  mode: options.mode ?? "all",
                  jobId: options.jobId ?? null,
                })
              : undefined,
          cache: "no-store",
        },
      );
      const body = (await response.json().catch(() => null)) as
        | RepositoryActionsRunDetail
        | ApiErrorEnvelope
        | null;
      if (!response.ok || !body || "error" in body) {
        throw new Error(
          body && "error" in body
            ? body.error.message
            : "Workflow run action failed.",
        );
      }
      setConfirmDeleteLogs(false);
      setActionMessage(`${titleCase(label)} queued.`);
      router.refresh();
    } catch (error) {
      setActionMessage(
        error instanceof Error ? error.message : "Workflow run action failed.",
      );
    } finally {
      setPendingAction(null);
    }
  }

  return (
    <RepositoryShell
      activePath={`${basePath}/actions/runs/${detail.run.id}`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="space-y-6">
        {validationError ? (
          <div className="card p-4" role="status">
            <p className="t-label" style={{ color: "var(--err)" }}>
              Actions unavailable
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              {validationError.error.message}
            </p>
          </div>
        ) : null}

        <section className="space-y-4">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0 flex-1">
              <div className="mb-3 flex flex-wrap items-center gap-2">
                <Link
                  className="t-sm hover:underline"
                  href={`${basePath}/actions`}
                >
                  Actions
                </Link>
                <span className="t-xs">/</span>
                <Link
                  className="t-sm hover:underline"
                  href={`${basePath}/actions/workflows/${detail.workflow.path}`}
                >
                  {detail.workflow.name}
                </Link>
              </div>
              <div className="flex items-start gap-3">
                <span
                  aria-label={`${statusLabel(detail.run.status, detail.run.conclusion)} run`}
                  className={`chip ${statusTone(detail.run.conclusion ?? detail.run.status)}`}
                  role="img"
                >
                  {statusGlyph(detail.run.status, detail.run.conclusion)}
                </span>
                <div className="min-w-0">
                  <h1 className="t-h1 break-words">
                    {detail.run.displayTitle}
                    <span
                      className="ml-2 font-normal"
                      style={{ color: "var(--ink-4)" }}
                    >
                      #{detail.run.runNumber}
                    </span>
                  </h1>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    {titleCase(detail.run.event)} on{" "}
                    <span className="t-mono-sm">{detail.run.headBranch}</span>
                    {detail.run.shortSha ? (
                      <>
                        {" "}
                        at{" "}
                        <span className="t-mono-sm">{detail.run.shortSha}</span>
                      </>
                    ) : null}
                    {detail.run.actor ? ` by ${detail.run.actor.login}` : ""}
                  </p>
                </div>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Link className="btn" href={detail.workflow.sourceHref}>
                Workflow file
              </Link>
              <button
                className="btn"
                disabled={!detail.actionState.canRerun}
                onClick={() => mutateRun("rerun", { mode: "all" })}
                title={
                  !detail.actionState.canRerun
                    ? actionDisabledReason
                    : undefined
                }
                type="button"
              >
                Re-run all
              </button>
              <button
                className="btn"
                disabled={!detail.actionState.canRerunFailed}
                onClick={() => mutateRun("rerun", { mode: "failed" })}
                title={
                  !detail.actionState.canRerunFailed
                    ? actionDisabledReason
                    : undefined
                }
                type="button"
              >
                Re-run failed
              </button>
              <button
                className="btn"
                disabled={!detail.actionState.canCancel}
                onClick={() => mutateRun("cancel")}
                title={
                  !detail.actionState.canCancel
                    ? actionDisabledReason
                    : undefined
                }
                type="button"
              >
                Cancel run
              </button>
              <button
                className="btn"
                disabled={!detail.actionState.canDeleteLogs}
                onClick={() => setConfirmDeleteLogs(true)}
                title={
                  !detail.actionState.canDeleteLogs
                    ? actionDisabledReason
                    : undefined
                }
                type="button"
              >
                Delete logs
              </button>
            </div>
          </div>
          {confirmDeleteLogs ? (
            <div className="card flex flex-wrap items-center justify-between gap-3 p-4">
              <p className="t-sm" style={{ color: "var(--ink-2)" }}>
                Delete stored logs for this run? Log downloads will become
                unavailable.
              </p>
              <div className="flex gap-2">
                <button
                  className="btn sm"
                  onClick={() => setConfirmDeleteLogs(false)}
                  type="button"
                >
                  Cancel
                </button>
                <button
                  className="btn sm accent"
                  disabled={pendingAction !== null}
                  onClick={() => mutateRun("logs")}
                  type="button"
                >
                  Confirm delete
                </button>
              </div>
            </div>
          ) : null}
          {actionMessage || pendingAction ? (
            <p className="t-sm" role="status" style={{ color: "var(--ink-2)" }}>
              {pendingAction ? `${titleCase(pendingAction)}...` : actionMessage}
            </p>
          ) : null}

          <div className="grid gap-3 md:grid-cols-4">
            <SummaryCard
              label="Status"
              value={statusLabel(detail.run.status, detail.run.conclusion)}
            />
            <SummaryCard
              label="Duration"
              value={durationLabel(detail.run.durationSeconds)}
            />
            <SummaryCard
              label="Jobs"
              value={`${detail.run.jobSummary.completed}/${detail.run.jobSummary.total} complete`}
            />
            <SummaryCard
              label="Started"
              value={dateTimeLabel(detail.run.startedAt)}
            />
          </div>
        </section>

        <div className="grid grid-cols-[300px_minmax(0,1fr)] gap-6 max-lg:grid-cols-1">
          <aside className="space-y-4">
            <div className="card p-3">
              <p className="t-label mb-2">Attempts</p>
              <div className="space-y-1">
                {detail.attempts.map((attempt) => (
                  <Link
                    className="list-row rounded-[var(--radius)] px-3 py-2"
                    href={`${basePath}/actions/runs/${detail.run.id}?attempt=${attempt.attemptNumber}`}
                    key={`${attempt.id ?? "initial"}-${attempt.attemptNumber}`}
                  >
                    <span className="t-sm font-medium">
                      Attempt {attempt.attemptNumber}
                    </span>
                    <span
                      className={`chip ${statusTone(attempt.conclusion ?? attempt.status)}`}
                    >
                      {titleCase(attempt.triggerKind)}
                    </span>
                  </Link>
                ))}
              </div>
            </div>

            <nav aria-label="Workflow run jobs" className="card p-3">
              <div className="mb-2 flex items-center justify-between gap-3">
                <p className="t-label">Jobs</p>
                <span className="chip soft t-num">{detail.jobs.length}</span>
              </div>
              <button
                className="list-row w-full rounded-[var(--radius)] px-3 py-2 text-left"
                onClick={() => setSelectedJobId("summary")}
                style={{
                  background:
                    selectedJobId === "summary"
                      ? "var(--accent-soft)"
                      : "transparent",
                }}
                type="button"
              >
                <span className="t-sm font-medium">Summary</span>
              </button>
              {groupedJobs.map(([group, jobs]) => (
                <div key={group} className="mt-3">
                  <p className="t-xs mb-1 px-3">{group}</p>
                  <div className="space-y-1">
                    {jobs.map((job) => (
                      <a
                        aria-current={
                          selectedJobId === job.id ? "true" : undefined
                        }
                        className="flex w-full items-center gap-2 rounded-[var(--radius)] px-3 py-2 text-left text-sm hover:bg-[var(--hover)]"
                        href={jobHref(basePath, detail.run.id, job.id)}
                        key={job.id}
                        onClick={(event) => {
                          event.preventDefault();
                          setSelectedJobId(job.id);
                          document
                            .getElementById(`job-${job.id}`)
                            ?.scrollIntoView({ block: "start" });
                        }}
                        style={{
                          background:
                            selectedJobId === job.id
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
                      </a>
                    ))}
                  </div>
                </div>
              ))}
            </nav>
          </aside>

          <main className="min-w-0 space-y-4">
            <section className="card p-5">
              <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="t-label">Run summary</p>
                  <h2 className="t-h2 mt-1">Checks and artifacts</h2>
                </div>
                <span className="chip soft">
                  {detail.annotations.length} annotations
                </span>
              </div>
              <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                <SummaryCard
                  label="Succeeded"
                  value={String(detail.run.jobSummary.success)}
                />
                <SummaryCard
                  label="Failed"
                  value={String(detail.run.jobSummary.failure)}
                />
                <SummaryCard
                  label="Queued"
                  value={String(detail.run.jobSummary.queued)}
                />
                <SummaryCard
                  label="Artifacts"
                  value={String(detail.artifacts.length)}
                />
              </div>
            </section>

            <section
              className="card scroll-mt-6 overflow-hidden"
              id={selectedJob ? `job-${selectedJob.id}` : "run-summary"}
            >
              <div
                className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-4"
                style={{ borderColor: "var(--line)" }}
              >
                <div>
                  <p className="t-label">
                    {selectedJob ? "Selected job" : "Summary"}
                  </p>
                  <h2 className="t-h2 mt-1">
                    {selectedJob?.name ?? "All jobs"}
                  </h2>
                </div>
                {selectedJob ? (
                  <span
                    className={`chip ${statusTone(selectedJob.conclusion ?? selectedJob.status)}`}
                  >
                    {jobState(selectedJob)}
                  </span>
                ) : null}
              </div>
              {selectedJob ? (
                <JobDetail
                  basePath={basePath}
                  job={selectedJob}
                  log={jobLog}
                  logMessage={jobLogMessage}
                  logQuery={logQuery}
                  logState={jobLogState}
                  onLogQueryChange={setLogQuery}
                  onLogSearch={(event) => {
                    event.preventDefault();
                    setSubmittedLogQuery(logQuery);
                  }}
                  onRerunJob={(jobId) =>
                    mutateRun("rerun", { mode: "job", jobId })
                  }
                  rerunDisabled={!detail.actionState.canRerun}
                />
              ) : (
                <div className="p-5">
                  <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                    Select a job to inspect its steps, runner metadata, and log
                    availability.
                  </p>
                </div>
              )}
            </section>

            <AnnotationsList detail={detail} />
            <ArtifactsTable
              detail={detail}
              message={artifactMessage}
              onCopyDownload={copyArtifactDownload}
            />
            <RunMetadata detail={detail} />
          </main>
        </div>
      </div>
    </RepositoryShell>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="card p-4">
      <p className="t-label">{label}</p>
      <p className="t-sm mt-2 font-medium" style={{ color: "var(--ink-1)" }}>
        {value}
      </p>
    </div>
  );
}

function JobDetail({
  basePath,
  job,
  log,
  logMessage,
  logQuery,
  logState,
  onLogQueryChange,
  onLogSearch,
  onRerunJob,
  rerunDisabled,
}: {
  basePath: string;
  job: ActionsRunJobDetail;
  log: ActionsJobLog | null;
  logMessage: string;
  logQuery: string;
  logState: "idle" | "loading" | "error";
  onLogQueryChange: (value: string) => void;
  onLogSearch: (event: FormEvent<HTMLFormElement>) => void;
  onRerunJob: (jobId: string) => void;
  rerunDisabled: boolean;
}) {
  return (
    <div className="space-y-5 p-5">
      <div className="mb-4 flex flex-wrap gap-2">
        <span className="chip soft">Attempt {job.attemptNumber}</span>
        {job.runnerLabel ? (
          <span className="chip soft">{job.runnerLabel}</span>
        ) : null}
        <span className={job.logAvailable ? "chip ok" : "chip warn"}>
          {job.logDeletedAt
            ? "Logs deleted"
            : job.logAvailable
              ? "Logs available"
              : "Logs unavailable"}
        </span>
        {job.logAvailable ? (
          <a className="btn sm" href={jobLogDownloadPath(basePath, job.id)}>
            Download log
          </a>
        ) : null}
        <button
          className="btn sm"
          disabled={rerunDisabled}
          onClick={() => onRerunJob(job.id)}
          type="button"
        >
          Re-run job
        </button>
      </div>
      <div className="space-y-2">
        {job.steps.map((step) => (
          <div
            className="list-row rounded-[var(--radius)] px-3 py-3"
            key={step.id}
          >
            <span
              className={`chip ${statusTone(step.conclusion ?? step.status)} h-6 w-6 justify-center px-0`}
            >
              {statusGlyph(step.status, step.conclusion)}
            </span>
            <div className="min-w-0 flex-1">
              <p className="t-sm font-medium">{step.name}</p>
              <p className="t-xs">Step {step.number}</p>
            </div>
            <span className="t-xs t-num">
              {durationLabel(step.durationSeconds)}
            </span>
          </div>
        ))}
      </div>
      <div
        className="rounded-[var(--radius)] border"
        style={{ borderColor: "var(--line)" }}
      >
        <div
          className="flex flex-wrap items-center justify-between gap-3 border-b p-3"
          style={{ borderColor: "var(--line-soft)" }}
        >
          <div>
            <p className="t-label">Job log</p>
            <p className="t-xs mt-1">
              {job.logAvailable
                ? "Search and anchor lines from the stored job log."
                : "This job does not have readable logs."}
            </p>
          </div>
          {job.logAvailable ? (
            <form className="flex min-w-[260px] gap-2" onSubmit={onLogSearch}>
              <input
                aria-label="Search job log"
                className="input h-9"
                onChange={(event) => onLogQueryChange(event.target.value)}
                placeholder="Search log"
                value={logQuery}
              />
              <button className="btn sm" type="submit">
                Search
              </button>
            </form>
          ) : null}
        </div>
        {job.logDeletedAt ? (
          <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
            Logs were deleted for this job.
          </p>
        ) : !job.logAvailable ? (
          <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
            Logs are not available yet.
          </p>
        ) : logState === "loading" ? (
          <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
            Loading logs...
          </p>
        ) : logState === "error" ? (
          <p className="t-sm p-4" role="status" style={{ color: "var(--err)" }}>
            {logMessage}
          </p>
        ) : log ? (
          <div>
            <p
              className="t-xs border-b px-4 py-2"
              style={{ borderColor: "var(--line-soft)" }}
            >
              {log.total} matching lines
            </p>
            <ol className="max-h-[360px] overflow-auto py-2">
              {log.lines.map((line) => (
                <li
                  className="grid grid-cols-[64px_minmax(0,1fr)] gap-3 px-4 py-1"
                  id={`log-${line.anchor}`}
                  key={line.anchor}
                >
                  <a
                    className="t-mono-sm text-right hover:underline"
                    href={`#log-${line.anchor}`}
                  >
                    {line.lineNumber}
                  </a>
                  <code className="t-mono-sm whitespace-pre-wrap break-words">
                    {line.content}
                  </code>
                </li>
              ))}
            </ol>
          </div>
        ) : null}
      </div>
    </div>
  );
}

function AnnotationsList({ detail }: { detail: RepositoryActionsRunDetail }) {
  return (
    <section className="card overflow-hidden">
      <div
        className="border-b px-5 py-4"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label">Annotations</p>
        <h2 className="t-h2 mt-1">Problems found in this run</h2>
      </div>
      {detail.annotations.length ? (
        <div className="divide-y" style={{ borderColor: "var(--line-soft)" }}>
          {detail.annotations.map((annotation) => (
            <div className="list-row px-5 py-4" key={annotation.id}>
              <span className={`chip ${statusTone(annotation.level)}`}>
                {titleCase(annotation.level)}
              </span>
              <div className="min-w-0 flex-1">
                <p className="t-sm font-medium">
                  {annotation.title ?? annotation.message}
                </p>
                <p className="t-xs mt-1">
                  {annotation.path ?? "Workflow"}{" "}
                  {annotation.startLine ? `line ${annotation.startLine}` : ""}
                </p>
                {annotation.title ? (
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    {annotation.message}
                  </p>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="t-sm p-5" style={{ color: "var(--ink-3)" }}>
          No annotations were emitted for this run.
        </p>
      )}
    </section>
  );
}

function ArtifactsTable({
  detail,
  message,
  onCopyDownload,
}: {
  detail: RepositoryActionsRunDetail;
  message: string;
  onCopyDownload: (artifactId: string) => void;
}) {
  return (
    <section className="card overflow-hidden">
      <div
        className="border-b px-5 py-4"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label">Artifacts</p>
        <h2 className="t-h2 mt-1">Generated files</h2>
      </div>
      {detail.artifacts.length ? (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[620px] text-left text-sm">
            <thead>
              <tr style={{ color: "var(--ink-3)" }}>
                <th className="px-5 py-3 font-medium">Name</th>
                <th className="px-5 py-3 font-medium">Digest</th>
                <th className="px-5 py-3 font-medium">Size</th>
                <th className="px-5 py-3 font-medium">State</th>
                <th className="px-5 py-3 font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {detail.artifacts.map((artifact) => (
                <tr
                  className="border-t"
                  key={artifact.id}
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  <td className="px-5 py-3 font-medium">{artifact.name}</td>
                  <td className="px-5 py-3">
                    <span className="t-mono-sm">
                      {artifact.digest ?? "none"}
                    </span>
                  </td>
                  <td className="px-5 py-3 t-num">
                    {bytesLabel(artifact.sizeBytes)}
                  </td>
                  <td className="px-5 py-3">
                    <span
                      className={
                        artifact.downloadAvailable ? "chip ok" : "chip warn"
                      }
                    >
                      {artifact.downloadAvailable ? "Available" : "Expired"}
                    </span>
                  </td>
                  <td className="px-5 py-3">
                    <div className="flex flex-wrap gap-2">
                      <a
                        className="btn sm"
                        href={artifactDownloadPath(
                          `/${detail.repository.ownerLogin}/${detail.repository.name}`,
                          artifact.id,
                        )}
                      >
                        Download
                      </a>
                      <button
                        className="btn sm"
                        disabled={!artifact.downloadAvailable}
                        onClick={() => onCopyDownload(artifact.id)}
                        type="button"
                      >
                        Copy URL
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {message ? (
            <p className="t-sm px-5 pb-4" role="status">
              {message}
            </p>
          ) : null}
        </div>
      ) : (
        <p className="t-sm p-5" style={{ color: "var(--ink-3)" }}>
          This run did not upload artifacts.
        </p>
      )}
    </section>
  );
}

function RunMetadata({ detail }: { detail: RepositoryActionsRunDetail }) {
  return (
    <section className="card p-5">
      <p className="t-label">Metadata</p>
      <dl className="mt-4 grid gap-4 sm:grid-cols-2">
        <div>
          <dt className="t-xs">Workflow path</dt>
          <dd className="t-mono-sm mt-1">{detail.workflow.path}</dd>
        </div>
        <div>
          <dt className="t-xs">Source branch</dt>
          <dd className="t-mono-sm mt-1">{detail.workflow.sourceBranch}</dd>
        </div>
        <div>
          <dt className="t-xs">Created</dt>
          <dd className="t-sm mt-1">{dateTimeLabel(detail.run.createdAt)}</dd>
        </div>
        <div>
          <dt className="t-xs">Completed</dt>
          <dd className="t-sm mt-1">{dateTimeLabel(detail.run.completedAt)}</dd>
        </div>
      </dl>
    </section>
  );
}
