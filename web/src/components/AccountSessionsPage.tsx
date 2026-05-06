"use client";

import { useMemo, useState } from "react";
import type {
  AccountSessionSummary,
  AccountSessions,
  AccountSessionsFetchResult,
} from "@/lib/api";

type AccountSessionsPageProps = {
  sessionsResult: AccountSessionsFetchResult;
};

export function AccountSessionsPage({
  sessionsResult,
}: AccountSessionsPageProps) {
  const [sessions, setSessions] = useState<AccountSessions | null>(
    sessionsResult.ok ? sessionsResult.sessions : null,
  );
  const [message, setMessage] = useState<string | null>(
    sessionsResult.ok ? null : sessionsResult.message,
  );
  const [pendingId, setPendingId] = useState<string | null>(null);

  const otherSessionCount = useMemo(
    () =>
      sessions?.sessions.filter((session) => !session.isCurrent).length ?? 0,
    [sessions],
  );

  async function runAction(input: { action: string; sessionId?: string }) {
    setPendingId(input.sessionId ?? input.action);
    setMessage(null);
    try {
      const response = await fetch("/settings/sessions/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(input),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Session action failed.");
      }
      setSessions(body as AccountSessions);
      setMessage(
        input.action === "sign_out_everywhere"
          ? "Other sessions have been signed out."
          : "Session revoked.",
      );
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Session action failed.",
      );
    } finally {
      setPendingId(null);
    }
  }

  return (
    <article className="min-w-0">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Account security
          </p>
          <h1 className="t-h2 mt-2">Active sessions</h1>
          <p
            className="mt-3 max-w-3xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            Review browsers signed in to this account. Revoke an individual
            device, or sign out everywhere while keeping this session active.
          </p>
        </div>
        <button
          className="btn primary"
          disabled={!sessions || otherSessionCount === 0 || pendingId !== null}
          onClick={() => runAction({ action: "sign_out_everywhere" })}
          type="button"
        >
          {pendingId === "sign_out_everywhere"
            ? "Signing out..."
            : "Sign out everywhere"}
        </button>
      </div>

      {message ? (
        <p
          className="mt-4 t-sm"
          role="status"
          style={{ color: "var(--ink-2)" }}
        >
          {message}
        </p>
      ) : null}

      <section className="card mt-6 overflow-hidden">
        <div
          className="flex flex-wrap items-center justify-between gap-3 p-5"
          style={{ borderBottom: "1px solid var(--line)" }}
        >
          <div>
            <h2 className="t-h3">Signed-in devices</h2>
            <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
              {sessions
                ? `${sessions.activeCount} active session${sessions.activeCount === 1 ? "" : "s"}`
                : "Session data could not be loaded."}
            </p>
          </div>
          <span className="chip soft">Current session protected</span>
        </div>

        {sessions?.sessions.length ? (
          <div className="overflow-x-auto">
            <table
              aria-label="Active web sessions"
              className="w-full border-collapse text-left"
            >
              <thead>
                <tr
                  className="t-label"
                  style={{
                    background: "var(--surface-2)",
                    borderBottom: "1px solid var(--line-soft)",
                    color: "var(--ink-3)",
                  }}
                >
                  <th className="px-5 py-3" scope="col">
                    Device
                  </th>
                  <th className="px-5 py-3" scope="col">
                    Browser
                  </th>
                  <th className="px-5 py-3" scope="col">
                    Location
                  </th>
                  <th className="px-5 py-3" scope="col">
                    Last active
                  </th>
                  <th className="px-5 py-3" scope="col">
                    Action
                  </th>
                </tr>
              </thead>
              <tbody>
                {sessions.sessions.map((session) => (
                  <SessionRow
                    key={session.id}
                    onRevoke={() =>
                      runAction({ action: "revoke", sessionId: session.id })
                    }
                    pending={pendingId === session.id}
                    session={session}
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="p-5">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No active web sessions were returned by the API.
            </p>
          </div>
        )}
      </section>
    </article>
  );
}

function SessionRow({
  onRevoke,
  pending,
  session,
}: {
  onRevoke: () => void;
  pending: boolean;
  session: AccountSessionSummary;
}) {
  return (
    <tr style={{ borderBottom: "1px solid var(--line-soft)" }}>
      <td className="min-w-[220px] px-5 py-4 align-middle">
        <div className="flex flex-wrap items-center gap-2">
          <span className="t-sm font-semibold">{session.device}</span>
          {session.isCurrent ? <span className="chip ok">Current</span> : null}
        </div>
        <p className="mt-1 truncate t-xs">
          Signed in {formatDate(session.signedInAt)}
        </p>
      </td>
      <td className="min-w-[120px] px-5 py-4 align-middle">
        <span className="t-sm">{session.browser}</span>
      </td>
      <td className="min-w-[150px] px-5 py-4 align-middle">
        <p className="t-sm">{session.location}</p>
        <p
          className="mt-1 truncate t-mono-sm"
          style={{ color: "var(--ink-3)" }}
        >
          {session.ipAddress ?? "No IP recorded"}
        </p>
      </td>
      <td className="min-w-[160px] px-5 py-4 align-middle">
        <span className="t-sm">{formatDate(session.lastActiveAt)}</span>
      </td>
      <td className="min-w-[120px] px-5 py-4 align-middle">
        <button
          className="btn sm"
          disabled={session.isCurrent || pending}
          onClick={onRevoke}
          type="button"
        >
          {pending ? "Revoking..." : "Revoke"}
        </button>
      </td>
    </tr>
  );
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}
