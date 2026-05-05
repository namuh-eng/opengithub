"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryCodeScanningAlertDetail,
  RepositoryCodeScanningAlertDetailFetchResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryCodeScanningAlertDetailPageProps = {
  repository: RepositoryOverview;
  detailResult: RepositoryCodeScanningAlertDetailFetchResult;
};

const DISMISSAL_REASONS = [
  { value: "false_positive", label: "False positive" },
  { value: "won_t_fix", label: "Won't fix" },
  { value: "used_in_tests", label: "Used in tests" },
  { value: "not_used", label: "Code is not used" },
];

function basePath(detail: RepositoryCodeScanningAlertDetail) {
  return `/${encodeURIComponent(detail.repository.ownerLogin)}/${encodeURIComponent(detail.repository.name)}/security/code-scanning/${encodeURIComponent(String(detail.alert.number))}`;
}

function formatDate(value: string | null) {
  if (!value) return "Unknown";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Unknown";
  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function severityClass(detail: RepositoryCodeScanningAlertDetail) {
  const severity = detail.alert.securitySeverity ?? detail.alert.severity;
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "medium" || severity === "warning") return "chip warn";
  if (detail.alert.state === "fixed") return "chip ok";
  return "chip soft";
}

function stateClass(state: string) {
  if (state === "open") return "chip warn";
  if (state === "fixed") return "chip ok";
  return "chip soft";
}

function stateLabel(state: string) {
  if (state === "fixed") return "Fixed";
  if (state === "dismissed") return "Dismissed";
  return "Open";
}

function ErrorState({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositoryCodeScanningAlertDetailFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell
      activeSection="code-scanning"
      repository={repository}
    >
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Code scanning alert
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Alert unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security/code-scanning`}
        >
          Back to Code scanning alerts
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

function CodeSnippet({
  detail,
}: {
  detail: RepositoryCodeScanningAlertDetail;
}) {
  const lines = (detail.location.codeSnippet ?? "").split("\n").filter(Boolean);
  const startLine = detail.location.startLine;
  const numberedLines = lines.map((line, index) => ({
    line,
    lineNumber: startLine + index,
  }));
  return (
    <section className="card overflow-hidden">
      <div
        className="flex flex-wrap items-center gap-2 px-5 py-4"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Location
        </p>
        <Link className="chip soft" href={detail.location.pathHref}>
          <span className="t-mono-sm">
            {detail.location.path}:{detail.location.startLine}
          </span>
        </Link>
        <Link className="chip soft" href={detail.location.rawHref}>
          Raw
        </Link>
      </div>
      {lines.length > 0 ? (
        <pre
          className="m-0 overflow-x-auto p-5 t-mono-sm"
          style={{ background: "var(--surface-2)", color: "var(--ink-1)" }}
        >
          {numberedLines.map(({ line, lineNumber }) => (
            <code className="block" key={`${lineNumber}-${line}`}>
              <span style={{ color: "var(--ink-4)" }}>
                {String(lineNumber).padStart(4, " ")}
              </span>{" "}
              {line}
            </code>
          ))}
        </pre>
      ) : (
        <p className="p-5 t-sm" style={{ color: "var(--ink-3)" }}>
          Source snippet is not available for this alert.
        </p>
      )}
    </section>
  );
}

function DetailActions({
  detail,
  onUpdate,
}: {
  detail: RepositoryCodeScanningAlertDetail;
  onUpdate: (next: RepositoryCodeScanningAlertDetail) => void;
}) {
  const [reason, setReason] = useState(DISMISSAL_REASONS[0]?.value ?? "");
  const [comment, setComment] = useState("");
  const [selectedAssignees, setSelectedAssignees] = useState<string[]>(
    detail.assigneeOptions
      .filter((option) => option.selected)
      .map((option) => option.id),
  );
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<string | null>(null);

  async function submit(payload: Record<string, unknown>, label: string) {
    setPendingAction(label);
    setMessage(null);
    setError(null);
    try {
      const response = await fetch(`${basePath(detail)}/actions`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Code scanning alert update failed.",
        );
      }
      onUpdate(body as RepositoryCodeScanningAlertDetail);
      setMessage(`${label} saved.`);
    } catch (caught) {
      setError(
        caught instanceof Error
          ? caught.message
          : "Code scanning alert update failed.",
      );
    } finally {
      setPendingAction(null);
    }
  }

  async function createIssue() {
    setPendingAction("Issue link");
    setMessage(null);
    setError(null);
    try {
      const response = await fetch(`${basePath(detail)}/issue`, {
        method: "POST",
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Issue link failed.");
      }
      onUpdate(body as RepositoryCodeScanningAlertDetail);
      setMessage("Issue linked.");
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Issue link failed.");
    } finally {
      setPendingAction(null);
    }
  }

  function toggleAssignee(id: string) {
    setSelectedAssignees((current) =>
      current.includes(id)
        ? current.filter((value) => value !== id)
        : [...current, id],
    );
  }

  if (!detail.viewer.canWrite) {
    return (
      <section className="card grid gap-3 p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Triage
        </p>
        <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
          Read-only access
        </h2>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          Users with write access can dismiss, reopen, assign, and link issues.
        </p>
      </section>
    );
  }

  return (
    <section className="card grid gap-4 p-5" aria-label="Alert triage actions">
      <div>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Triage
        </p>
        <h2 className="t-h2 mt-1" style={{ color: "var(--ink-1)" }}>
          Manage alert
        </h2>
      </div>

      {detail.alert.state === "open" ? (
        <div className="grid gap-3">
          <label className="grid gap-2 t-sm" htmlFor="code-dismissal-reason">
            Dismiss reason
            <select
              className="input"
              id="code-dismissal-reason"
              onChange={(event) => setReason(event.target.value)}
              value={reason}
            >
              {DISMISSAL_REASONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-2 t-sm" htmlFor="code-dismissal-comment">
            Optional comment
            <textarea
              className="input min-h-24"
              id="code-dismissal-comment"
              maxLength={500}
              onChange={(event) => setComment(event.target.value)}
              value={comment}
            />
          </label>
          <button
            className="btn primary w-fit"
            disabled={pendingAction !== null}
            onClick={() =>
              submit(
                {
                  action: "dismiss",
                  dismissalComment: comment,
                  dismissalReason: reason,
                },
                "Dismiss",
              )
            }
            type="button"
          >
            Dismiss alert
          </button>
        </div>
      ) : null}

      {detail.alert.state === "dismissed" ? (
        <button
          className="btn primary w-fit"
          disabled={pendingAction !== null}
          onClick={() => submit({ action: "reopen" }, "Reopen")}
          type="button"
        >
          Reopen alert
        </button>
      ) : null}

      <div className="grid gap-2">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Assignees
        </p>
        {detail.assigneeOptions.map((option) => (
          <label
            className="list-row flex items-center gap-3 py-2"
            key={option.id}
          >
            <input
              checked={selectedAssignees.includes(option.id)}
              onChange={() => toggleAssignee(option.id)}
              type="checkbox"
            />
            <span className="t-sm">{option.login}</span>
          </label>
        ))}
        <button
          className="btn w-fit"
          disabled={pendingAction !== null}
          onClick={() =>
            submit(
              { action: "assign", assigneeIds: selectedAssignees },
              "Assignments",
            )
          }
          type="button"
        >
          Save assignments
        </button>
      </div>

      <div className="grid gap-2">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Linked issue
        </p>
        {detail.linkedIssue.issue ? (
          <Link
            className="btn primary w-fit"
            href={detail.linkedIssue.issue.href}
          >
            Open linked issue #{detail.linkedIssue.issue.number}
          </Link>
        ) : (
          <button
            className="btn w-fit"
            disabled={pendingAction !== null || !detail.linkedIssue.canLink}
            onClick={createIssue}
            type="button"
          >
            Create linked issue
          </button>
        )}
      </div>

      {pendingAction ? (
        <span className="chip soft">Saving {pendingAction}</span>
      ) : null}
      {message ? <span className="chip ok">{message}</span> : null}
      {error ? <span className="chip err">{error}</span> : null}
    </section>
  );
}

function DetailReadyPage({
  repository,
  initialDetail,
}: {
  repository: RepositoryOverview;
  initialDetail: RepositoryCodeScanningAlertDetail;
}) {
  const [detail, setDetail] = useState(initialDetail);
  const selectedAssignees = useMemo(
    () => detail.alert.assignees.map((assignee) => assignee.login).join(", "),
    [detail.alert.assignees],
  );

  return (
    <RepositorySecurityShell
      activeSection="code-scanning"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Code scanning alert #{detail.alert.number}
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              style={{ color: "var(--ink-1)" }}
            >
              {detail.rule.name}
            </h1>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {detail.alert.message}
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <span className={stateClass(detail.alert.state)}>
                {stateLabel(detail.alert.state)}
              </span>
              <span className={severityClass(detail)}>
                {detail.alert.securitySeverity ?? detail.alert.severity}
              </span>
              <span className="chip soft">{detail.alert.toolName}</span>
              <span className="chip soft">{detail.viewer.permission}</span>
            </div>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            <Link className="btn" href={detail.links.listHref}>
              Back to alerts
            </Link>
            <Link className="btn" href={detail.links.settingsHref}>
              Alert settings
            </Link>
          </div>
        </section>

        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_340px]">
          <main className="grid gap-5">
            <CodeSnippet detail={detail} />

            <section className="card grid gap-3 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Rule
              </p>
              <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
                {detail.rule.id}
              </h2>
              {detail.rule.description ? (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  {detail.rule.description}
                </p>
              ) : null}
              <details className="grid gap-2">
                <summary className="btn w-fit cursor-pointer">
                  Show more remediation guidance
                </summary>
                <p
                  className="t-sm whitespace-pre-wrap"
                  style={{ color: "var(--ink-3)" }}
                >
                  {detail.rule.helpMarkdown ??
                    "Review the affected path, apply validation or escaping, and rerun code scanning."}
                </p>
                {detail.rule.helpUri ? (
                  <Link className="btn w-fit" href={detail.rule.helpUri}>
                    Open rule reference
                  </Link>
                ) : null}
              </details>
            </section>

            <section className="card overflow-hidden">
              <div
                className="px-5 py-4"
                style={{ borderBottom: "1px solid var(--line)" }}
              >
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Timeline
                </p>
              </div>
              <ul
                className="m-0 list-none p-0"
                aria-label="Code scanning alert timeline"
              >
                {detail.timeline.map((event) => (
                  <li className="list-row grid gap-1 px-5 py-4" key={event.id}>
                    <p className="t-sm" style={{ color: "var(--ink-1)" }}>
                      {event.message}
                    </p>
                    <p className="t-xs">
                      {event.actor ? `${event.actor.login} · ` : ""}
                      {formatDate(event.createdAt)}
                    </p>
                  </li>
                ))}
              </ul>
            </section>
          </main>

          <aside className="grid content-start gap-5">
            <section className="card grid gap-3 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Paths
              </p>
              <details>
                <summary className="btn w-fit cursor-pointer">
                  Show paths
                </summary>
                <div className="mt-3 grid gap-2">
                  <Link
                    className="chip soft w-fit"
                    href={detail.location.pathHref}
                  >
                    <span className="t-mono-sm">
                      {detail.location.path}:{detail.location.startLine}
                    </span>
                  </Link>
                  <span className="chip soft w-fit t-mono-sm">
                    {detail.location.refName}
                  </span>
                </div>
              </details>
            </section>

            <section className="card grid gap-3 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Assigned to
              </p>
              <p className="t-sm" style={{ color: "var(--ink-1)" }}>
                {selectedAssignees || "No assignees"}
              </p>
            </section>

            <DetailActions detail={detail} onUpdate={setDetail} />
          </aside>
        </div>
      </div>
    </RepositorySecurityShell>
  );
}

export function RepositoryCodeScanningAlertDetailPage({
  repository,
  detailResult,
}: RepositoryCodeScanningAlertDetailPageProps) {
  if (!detailResult.ok) {
    return <ErrorState repository={repository} result={detailResult} />;
  }
  return (
    <DetailReadyPage
      initialDetail={detailResult.codeScanningAlert}
      repository={repository}
    />
  );
}
