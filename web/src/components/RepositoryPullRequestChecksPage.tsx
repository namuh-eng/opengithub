"use client";

import Link from "next/link";
import { useState } from "react";
import type { PullRequestChecksView, RepositoryOverview } from "@/lib/api";

type RepositoryPullRequestChecksPageProps = {
  repository: RepositoryOverview;
  checks: PullRequestChecksView;
  viewerAuthenticated: boolean;
};

function chipClass(conclusion: string | null, status: string) {
  if (conclusion === "success" || conclusion === "skipped") return "chip ok";
  if (conclusion === "failure" || conclusion === "cancelled") return "chip err";
  if (status === "in_progress" || status === "queued") return "chip accent";
  return "chip soft";
}

function checkLabel(conclusion: string | null, status: string) {
  return conclusion ?? status.replaceAll("_", " ");
}

function summaryText(summary: PullRequestChecksView["summary"]) {
  if (summary.totalCount === 0) return "No checks have reported.";
  if (summary.conclusion === "success") return "All checks passed.";
  if (summary.failedCount > 0 || summary.conclusion === "failure") {
    return `${summary.failedCount || 1} check failed.`;
  }
  return `${summary.completedCount} of ${summary.totalCount} checks complete.`;
}

export function RepositoryPullRequestChecksPage({
  repository,
  checks,
  viewerAuthenticated,
}: RepositoryPullRequestChecksPageProps) {
  const [openRunIds, setOpenRunIds] = useState<Set<string>>(
    () =>
      new Set(
        checks.checkRuns
          .filter((run) => run.annotationsCount > 0)
          .map((run) => run.id),
      ),
  );
  const [message, setMessage] = useState<string | null>(null);
  const [rerunningId, setRerunningId] = useState<string | null>(null);
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const activePath = `${basePath}/pull/${checks.pullRequest.number}`;

  async function rerunCheck(href: string, id: string) {
    setRerunningId(id);
    setMessage(null);
    try {
      const response = await fetch(href, { method: "POST" });
      if (!response.ok) {
        const body = await response.json().catch(() => null);
        setMessage(
          body?.error?.message ??
            "Check could not be re-run from this session.",
        );
        return;
      }
      setMessage("Check re-run queued.");
    } finally {
      setRerunningId(null);
    }
  }

  return (
    <div className="mx-auto max-w-6xl px-6 py-8">
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label">
            {repository.owner_login} / {repository.name}
          </p>
          <h1 className="t-h1 mt-2">Checks for #{checks.pullRequest.number}</h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {checks.pullRequest.title}
          </p>
        </div>
        <span
          className={chipClass(
            checks.summary.conclusion,
            checks.summary.status,
          )}
        >
          {summaryText(checks.summary)}
        </span>
      </div>

      <nav aria-label="Pull request tabs" className="tabs mb-6">
        <Link className="tab" href={activePath}>
          Conversation
        </Link>
        <Link className="tab" href={`${activePath}/commits`}>
          Commits
        </Link>
        <Link className="tab active" href={`${activePath}/checks`}>
          Checks
        </Link>
        <Link className="tab" href={`${activePath}/files`}>
          Files changed
        </Link>
      </nav>

      <section className="card overflow-hidden">
        <div
          className="border-b px-5 py-4"
          style={{ borderColor: "var(--line)" }}
        >
          <h2 className="t-h3">Required checks</h2>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            {checks.requiredStatusChecks.length
              ? checks.requiredStatusChecks.join(", ")
              : "No branch protection checks are required for this base branch."}
          </p>
        </div>

        {checks.checkRuns.length ? (
          <div className="divide-y" style={{ borderColor: "var(--line)" }}>
            {checks.checkRuns.map((run) => {
              const expanded = openRunIds.has(run.id);
              const rerunHref = run.rerunHref;
              return (
                <article className="px-5 py-4" key={run.id}>
                  <div className="flex flex-wrap items-start gap-4">
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <h3 className="t-h3">{run.name}</h3>
                        <span className={chipClass(run.conclusion, run.status)}>
                          {checkLabel(run.conclusion, run.status)}
                        </span>
                        {run.required ? (
                          <span className="chip warn">Required</span>
                        ) : null}
                        {run.annotationsCount ? (
                          <span className="chip soft">
                            <span className="t-num">
                              {run.annotationsCount}
                            </span>{" "}
                            annotations
                          </span>
                        ) : null}
                      </div>
                      {run.outputSummary ? (
                        <p
                          className="t-sm mt-2"
                          style={{ color: "var(--ink-3)" }}
                        >
                          {run.outputSummary}
                        </p>
                      ) : null}
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {run.detailsHref ? (
                        <Link className="btn sm" href={run.detailsHref}>
                          View details
                        </Link>
                      ) : null}
                      {run.annotations.length ? (
                        <button
                          className="btn sm"
                          onClick={() =>
                            setOpenRunIds((current) => {
                              const next = new Set(current);
                              if (next.has(run.id)) next.delete(run.id);
                              else next.add(run.id);
                              return next;
                            })
                          }
                          type="button"
                        >
                          {expanded ? "Hide annotations" : "Show annotations"}
                        </button>
                      ) : null}
                      {viewerAuthenticated && rerunHref ? (
                        <button
                          className="btn sm"
                          disabled={rerunningId === run.id}
                          onClick={() => void rerunCheck(rerunHref, run.id)}
                          type="button"
                        >
                          Re-run job
                        </button>
                      ) : null}
                    </div>
                  </div>
                  {expanded && run.annotations.length ? (
                    <div className="mt-4 grid gap-2">
                      {run.annotations.map((annotation) => (
                        <div
                          className="rounded-[var(--radius)] border p-3"
                          key={annotation.id}
                          style={{
                            background: "var(--surface-2)",
                            borderColor: "var(--line-soft)",
                          }}
                        >
                          <div className="mb-1 flex flex-wrap gap-2">
                            <span
                              className={`chip ${
                                annotation.level === "failure"
                                  ? "err"
                                  : annotation.level === "warning"
                                    ? "warn"
                                    : "soft"
                              }`}
                            >
                              {annotation.level}
                            </span>
                            {annotation.path ? (
                              <span className="t-mono-sm">
                                {annotation.path}
                                {annotation.startLine
                                  ? `:${annotation.startLine}`
                                  : ""}
                              </span>
                            ) : null}
                          </div>
                          <p className="t-sm">{annotation.message}</p>
                        </div>
                      ))}
                    </div>
                  ) : null}
                </article>
              );
            })}
          </div>
        ) : (
          <div className="px-5 py-8">
            <p className="t-sm" role="status" style={{ color: "var(--ink-3)" }}>
              No workflow checks have reported for this head SHA yet.
            </p>
          </div>
        )}
      </section>

      {message ? (
        <p
          className="t-sm mt-4"
          role="status"
          style={{ color: "var(--ink-2)" }}
        >
          {message}
        </p>
      ) : null}
    </div>
  );
}
