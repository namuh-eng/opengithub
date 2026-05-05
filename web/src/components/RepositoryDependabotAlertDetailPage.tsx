"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryDependabotAlertDetail,
  RepositoryDependabotAlertDetailFetchResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryDependabotAlertDetailPageProps = {
  repository: RepositoryOverview;
  detailResult: RepositoryDependabotAlertDetailFetchResult;
};

const DISMISSAL_REASONS = [
  { value: "fix_started", label: "A fix has already started" },
  { value: "inaccurate", label: "This alert is inaccurate" },
  { value: "no_bandwidth", label: "No bandwidth to fix this" },
  { value: "not_used", label: "Vulnerable code is not used" },
  { value: "tolerable_risk", label: "Risk is tolerable" },
];

function severityClass(severity: string) {
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "moderate") return "chip warn";
  return "chip soft";
}

function stateClass(state: string) {
  if (state === "open") return "chip warn";
  if (state === "fixed") return "chip ok";
  return "chip soft";
}

function stateLabel(state: string) {
  if (state === "dismissed") return "Dismissed";
  if (state === "fixed") return "Fixed";
  return "Open";
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

function basePath(detail: RepositoryDependabotAlertDetail) {
  return `/${encodeURIComponent(detail.repository.ownerLogin)}/${encodeURIComponent(detail.repository.name)}/security/dependabot/${encodeURIComponent(String(detail.alert.number))}`;
}

function ErrorState({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositoryDependabotAlertDetailFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="dependabot" repository={repository}>
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Dependabot alert
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Alert unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security/dependabot`}
        >
          Back to Dependabot alerts
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

function DetailActions({
  detail,
  onUpdate,
}: {
  detail: RepositoryDependabotAlertDetail;
  onUpdate: (next: RepositoryDependabotAlertDetail) => void;
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
          body?.error?.message ?? "Dependabot alert update failed.",
        );
      }
      onUpdate(body as RepositoryDependabotAlertDetail);
      setMessage(`${label} saved.`);
    } catch (caught) {
      setError(
        caught instanceof Error
          ? caught.message
          : "Dependabot alert update failed.",
      );
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
          Users with write access can dismiss, reopen, and assign this alert.
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
          <label className="grid gap-2 t-sm" htmlFor="dismissal-reason">
            Dismiss reason
            <select
              className="input"
              id="dismissal-reason"
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
          <label className="grid gap-2 t-sm" htmlFor="dismissal-comment">
            Optional comment
            <textarea
              className="input min-h-24"
              id="dismissal-comment"
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
        {detail.assigneeOptions.length === 0 ? (
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No assignment targets are available.
          </p>
        ) : (
          <div className="grid gap-2">
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
  initialDetail: RepositoryDependabotAlertDetail;
}) {
  const [detail, setDetail] = useState(initialDetail);
  const selectedAssignees = useMemo(
    () => detail.alert.assignees.map((assignee) => assignee.login).join(", "),
    [detail.alert.assignees],
  );

  return (
    <RepositorySecurityShell activeSection="dependabot" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Dependabot alert #{detail.alert.number}
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              {detail.advisory.title}
            </h1>
            <div className="mt-4 flex flex-wrap gap-2">
              <span className={stateClass(detail.alert.state)}>
                {stateLabel(detail.alert.state)}
              </span>
              <span className={severityClass(detail.advisory.severity)}>
                {detail.advisory.severity}
              </span>
              <span className="chip soft">{detail.alert.scope}</span>
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
            <section className="card grid gap-4 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Vulnerable dependency
              </p>
              <div className="grid gap-3 md:grid-cols-2">
                <div>
                  <p className="t-xs">Package</p>
                  <Link
                    className="t-h2 hover:underline"
                    href={detail.dependency.package.href}
                  >
                    {detail.dependency.package.name}
                  </Link>
                  <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                    {detail.dependency.package.ecosystem} ·{" "}
                    {detail.dependency.relationship}
                  </p>
                </div>
                <div>
                  <p className="t-xs">Version range</p>
                  <p className="t-mono-sm mt-1">
                    Vulnerable {detail.advisory.vulnerableRange}
                  </p>
                  <p className="t-mono-sm mt-1">
                    Current {detail.dependency.currentVersion ?? "unknown"}
                  </p>
                  <p className="t-mono-sm mt-1">
                    Fixed {detail.alert.fixedVersion ?? "not published"}
                  </p>
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                <Link
                  className="chip soft"
                  href={detail.dependency.manifestHref}
                >
                  <span className="t-mono-sm">
                    {detail.dependency.manifestPath}
                  </span>
                </Link>
                {detail.dependency.lockfileHref &&
                detail.dependency.lockfilePath ? (
                  <Link
                    className="chip soft"
                    href={detail.dependency.lockfileHref}
                  >
                    <span className="t-mono-sm">
                      {detail.dependency.lockfilePath}
                    </span>
                  </Link>
                ) : null}
              </div>
            </section>

            <section className="card grid gap-3 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Advisory
              </p>
              <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
                {detail.advisory.identifier}
              </h2>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Published {formatDate(detail.advisory.publishedAt)}
              </p>
              <Link className="btn w-fit" href={detail.advisory.href}>
                Open advisory
              </Link>
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
                aria-label="Dependabot alert timeline"
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
                Security update
              </p>
              <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
                {detail.securityUpdate.status}
              </h2>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                {detail.securityUpdate.message}
              </p>
              {detail.securityUpdate.pullRequestHref ? (
                <Link
                  className="btn primary"
                  href={detail.securityUpdate.pullRequestHref}
                >
                  Open security update PR
                </Link>
              ) : (
                <span
                  className={
                    detail.securityUpdate.supported ? "chip ok" : "chip soft"
                  }
                >
                  {detail.securityUpdate.supported
                    ? "PR can be prepared in Phase 4"
                    : "Unsupported"}
                </span>
              )}
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

export function RepositoryDependabotAlertDetailPage({
  repository,
  detailResult,
}: RepositoryDependabotAlertDetailPageProps) {
  if (!detailResult.ok) {
    return <ErrorState repository={repository} result={detailResult} />;
  }
  return (
    <DetailReadyPage
      initialDetail={detailResult.dependabotAlert}
      repository={repository}
    />
  );
}
