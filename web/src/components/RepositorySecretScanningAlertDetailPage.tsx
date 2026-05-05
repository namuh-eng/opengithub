"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecretScanningAlertDetail,
  RepositorySecretScanningAlertDetailFetchResult,
} from "@/lib/api";

type RepositorySecretScanningAlertDetailPageProps = {
  repository: RepositoryOverview;
  detailResult: RepositorySecretScanningAlertDetailFetchResult;
};

const RESOLUTION_REASONS = [
  { value: "revoked", label: "Revoked" },
  { value: "false_positive", label: "False positive" },
  { value: "used_in_tests", label: "Used in tests" },
  { value: "wont_fix", label: "Won't fix" },
];

const VALIDITY_STATES = [
  { value: "active", label: "Active" },
  { value: "inactive", label: "Inactive" },
  { value: "unknown", label: "Unknown" },
  { value: "unsupported", label: "Unsupported" },
];

function basePath(detail: RepositorySecretScanningAlertDetail) {
  return `/${encodeURIComponent(detail.repository.ownerLogin)}/${encodeURIComponent(detail.repository.name)}/security/secret-scanning/${encodeURIComponent(String(detail.alert.number))}`;
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

function stateClass(state: string) {
  return state === "open" ? "chip warn" : "chip ok";
}

function validityClass(status: string) {
  if (status === "active") return "chip err";
  if (status === "inactive") return "chip ok";
  if (status === "checking") return "chip warn";
  return "chip soft";
}

function ErrorState({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<
    RepositorySecretScanningAlertDetailFetchResult,
    { ok: false }
  >;
}) {
  return (
    <RepositorySecurityShell
      activeSection="secret-scanning"
      repository={repository}
    >
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Secret scanning alert
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Alert unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security/secret-scanning`}
        >
          Back to Secret scanning alerts
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

function RedactedEvidence({
  detail,
}: {
  detail: RepositorySecretScanningAlertDetail;
}) {
  const primaryLocation = detail.locations[0] ?? detail.alert.primaryLocation;
  return (
    <section className="card overflow-hidden">
      <div
        className="flex flex-wrap items-center gap-2 px-5 py-4"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Redacted evidence
        </p>
        <span className="chip soft">{detail.pattern.provider}</span>
        <span className="chip soft">{detail.pattern.resultKind}</span>
      </div>
      <div className="grid gap-4 p-5">
        <pre
          className="m-0 overflow-x-auto rounded-md p-4 t-mono-sm"
          style={{ background: "var(--surface-2)", color: "var(--ink-1)" }}
        >
          {detail.alert.redactedContext ??
            primaryLocation?.redactedSnippet ??
            detail.alert.redactedSecret}
        </pre>
        <div className="flex flex-wrap gap-2">
          {primaryLocation ? (
            <>
              <Link className="chip soft" href={primaryLocation.pathHref}>
                <span className="t-mono-sm">
                  {primaryLocation.path}:{primaryLocation.startLine}
                </span>
              </Link>
              <Link className="chip soft" href={primaryLocation.rawHref}>
                Raw
              </Link>
              {primaryLocation.commitHref ? (
                <Link className="chip soft" href={primaryLocation.commitHref}>
                  Commit
                </Link>
              ) : null}
            </>
          ) : (
            <span className="chip soft">No file location available</span>
          )}
        </div>
      </div>
    </section>
  );
}

function DetailActions({
  detail,
  onUpdate,
}: {
  detail: RepositorySecretScanningAlertDetail;
  onUpdate: (next: RepositorySecretScanningAlertDetail) => void;
}) {
  const [resolution, setResolution] = useState(
    RESOLUTION_REASONS[0]?.value ?? "",
  );
  const [comment, setComment] = useState("");
  const [validity, setValidity] = useState(detail.validity.status);
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
          body?.error?.message ?? "Secret scanning alert update failed.",
        );
      }
      onUpdate(body as RepositorySecretScanningAlertDetail);
      setMessage(`${label} saved.`);
    } catch (caught) {
      setError(
        caught instanceof Error
          ? caught.message
          : "Secret scanning alert update failed.",
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
          Users with write access can resolve, reopen, assign, and update
          validity.
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
          <label className="grid gap-2 t-sm" htmlFor="secret-resolution">
            Resolution
            <select
              className="input"
              id="secret-resolution"
              onChange={(event) => setResolution(event.target.value)}
              value={resolution}
            >
              {RESOLUTION_REASONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-2 t-sm" htmlFor="secret-resolution-note">
            Optional comment
            <textarea
              className="input min-h-24"
              id="secret-resolution-note"
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
                  action: "resolve",
                  resolution,
                  resolutionComment: comment,
                },
                "Resolution",
              )
            }
            type="button"
          >
            Resolve alert
          </button>
        </div>
      ) : (
        <button
          className="btn primary w-fit"
          disabled={pendingAction !== null}
          onClick={() => submit({ action: "reopen" }, "Reopen")}
          type="button"
        >
          Reopen alert
        </button>
      )}

      <label className="grid gap-2 t-sm" htmlFor="secret-validity">
        Token validity
        <select
          className="input"
          id="secret-validity"
          onChange={(event) => setValidity(event.target.value)}
          value={validity}
        >
          {VALIDITY_STATES.map((item) => (
            <option key={item.value} value={item.value}>
              {item.label}
            </option>
          ))}
        </select>
      </label>
      <button
        className="btn w-fit"
        disabled={pendingAction !== null}
        onClick={() => submit({ action: "validity", validity }, "Validity")}
        type="button"
      >
        Save validity
      </button>

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
  initialDetail: RepositorySecretScanningAlertDetail;
}) {
  const [detail, setDetail] = useState(initialDetail);
  const selectedAssignees = useMemo(
    () => detail.alert.assignees.map((assignee) => assignee.login).join(", "),
    [detail.alert.assignees],
  );

  return (
    <RepositorySecurityShell
      activeSection="secret-scanning"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Secret scanning alert #{detail.alert.number}
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              style={{ color: "var(--ink-1)" }}
            >
              {detail.pattern.displayName}
            </h1>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {detail.alert.redactedSecret} was detected and redacted before it
              reached the interface.
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <span className={stateClass(detail.alert.state)}>
                {detail.alert.state === "open" ? "Open" : "Resolved"}
              </span>
              <span className={validityClass(detail.validity.status)}>
                {detail.validity.status}
              </span>
              <span className="chip soft">{detail.pattern.provider}</span>
              <span className="chip soft">{detail.pattern.secretType}</span>
              {detail.alert.bypassed ? (
                <span className="chip warn">Bypassed push protection</span>
              ) : null}
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
            <RedactedEvidence detail={detail} />

            <section className="card grid gap-3 p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Validity
              </p>
              <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
                {detail.validity.message}
              </h2>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Provider: {detail.validity.provider}. Last checked{" "}
                {formatDate(detail.validity.checkedAt)}.
              </p>
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
                aria-label="Secret scanning alert timeline"
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
                Locations
              </p>
              <details>
                <summary className="btn w-fit cursor-pointer">
                  Show paths
                </summary>
                <div className="mt-3 grid gap-2">
                  {detail.locations.map((location) => (
                    <Link
                      className="chip soft w-fit"
                      href={location.pathHref}
                      key={`${location.path}-${location.startLine}`}
                    >
                      <span className="t-mono-sm">
                        {location.path}:{location.startLine}
                      </span>
                    </Link>
                  ))}
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

            {detail.bypasses.length > 0 ? (
              <section className="card grid gap-3 p-5">
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Push protection bypasses
                </p>
                {detail.bypasses.map((bypass) => (
                  <div className="grid gap-1" key={bypass.id}>
                    <p className="t-sm" style={{ color: "var(--ink-1)" }}>
                      {bypass.reason}
                    </p>
                    <p className="t-xs">
                      {bypass.actor ? `${bypass.actor.login} · ` : ""}
                      {bypass.status} · {formatDate(bypass.createdAt)}
                    </p>
                  </div>
                ))}
              </section>
            ) : null}

            <DetailActions detail={detail} onUpdate={setDetail} />
          </aside>
        </div>
      </div>
    </RepositorySecurityShell>
  );
}

export function RepositorySecretScanningAlertDetailPage({
  repository,
  detailResult,
}: RepositorySecretScanningAlertDetailPageProps) {
  if (!detailResult.ok) {
    return <ErrorState repository={repository} result={detailResult} />;
  }
  return (
    <DetailReadyPage
      initialDetail={detailResult.secretScanningAlert}
      repository={repository}
    />
  );
}
