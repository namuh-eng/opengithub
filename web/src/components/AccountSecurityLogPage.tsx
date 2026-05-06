import Link from "next/link";
import type {
  AccountSecurityLog,
  AccountSecurityLogEvent,
  AccountSecurityLogFetchResult,
} from "@/lib/api";

type AccountSecurityLogPageProps = {
  action: string | null;
  logResult: AccountSecurityLogFetchResult;
  page: number;
};

export function AccountSecurityLogPage({
  action,
  logResult,
  page,
}: AccountSecurityLogPageProps) {
  if (!logResult.ok) {
    return (
      <article className="card p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Account security
        </p>
        <h1 className="t-h2 mt-2">Security log unavailable</h1>
        <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
          {logResult.message}
        </p>
      </article>
    );
  }

  const { log } = logResult;
  const selectedAction = action ?? log.filters.action;

  return (
    <article className="min-w-0">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Account security
          </p>
          <h1 className="t-h2 mt-2">Security log</h1>
          <p
            className="t-body mt-3 max-w-3xl"
            style={{ color: "var(--ink-3)" }}
          >
            Review immutable account-security events. Exports preserve the
            current action filter and are streamed by the API as attachments.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Link className="btn" href={exportHref(selectedAction, "csv")}>
            Export CSV
          </Link>
          <Link className="btn" href={exportHref(selectedAction, "json")}>
            Export JSON
          </Link>
        </div>
      </div>

      <section className="card mt-6 overflow-hidden">
        <div
          className="flex flex-wrap items-end justify-between gap-4 p-5"
          style={{ borderBottom: "1px solid var(--line)" }}
        >
          <div>
            <h2 className="t-h3">Account events</h2>
            <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
              {log.pagination.total} event
              {log.pagination.total === 1 ? "" : "s"} recorded
            </p>
          </div>
          <form
            action="/settings/security-log"
            className="flex flex-wrap gap-2"
          >
            <label className="grid gap-1">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Action
              </span>
              <select
                className="input min-w-[220px]"
                defaultValue={selectedAction ?? ""}
                name="action"
              >
                <option value="">All actions</option>
                {log.actions.map((eventAction) => (
                  <option key={eventAction} value={eventAction}>
                    {labelForAction(eventAction)}
                  </option>
                ))}
              </select>
            </label>
            <button className="btn primary self-end" type="submit">
              Filter
            </button>
          </form>
        </div>

        {log.events.length ? (
          <SecurityLogTable events={log.events} />
        ) : (
          <div className="p-5">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No security events match this filter.
            </p>
          </div>
        )}
      </section>

      <Pagination log={log} selectedAction={selectedAction} page={page} />
    </article>
  );
}

function SecurityLogTable({ events }: { events: AccountSecurityLogEvent[] }) {
  return (
    <div className="overflow-x-auto">
      <table
        aria-label="Security log events"
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
              Timestamp
            </th>
            <th className="px-5 py-3" scope="col">
              Action
            </th>
            <th className="px-5 py-3" scope="col">
              IP
            </th>
            <th className="px-5 py-3" scope="col">
              Location
            </th>
            <th className="px-5 py-3" scope="col">
              User-agent
            </th>
          </tr>
        </thead>
        <tbody>
          {events.map((event) => (
            <tr
              key={event.id}
              style={{ borderBottom: "1px solid var(--line-soft)" }}
            >
              <td className="min-w-[170px] px-5 py-4 align-top">
                <span className="t-sm">{formatDate(event.createdAt)}</span>
              </td>
              <td className="min-w-[210px] px-5 py-4 align-top">
                <span className="chip soft">
                  {labelForAction(event.action)}
                </span>
                <p className="t-mono-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  {event.action}
                </p>
              </td>
              <td className="min-w-[130px] px-5 py-4 align-top">
                <span className="t-mono-sm">
                  {event.ipAddress ?? "Not recorded"}
                </span>
              </td>
              <td className="min-w-[180px] px-5 py-4 align-top">
                <span className="t-sm">{event.location}</span>
              </td>
              <td className="max-w-[360px] px-5 py-4 align-top">
                <span className="block truncate t-xs">
                  {event.userAgent ?? "No user-agent recorded"}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Pagination({
  log,
  page,
  selectedAction,
}: {
  log: AccountSecurityLog;
  page: number;
  selectedAction: string | null;
}) {
  return (
    <nav
      aria-label="Security log pagination"
      className="mt-5 flex flex-wrap items-center justify-between gap-3"
    >
      <p className="t-sm" style={{ color: "var(--ink-3)" }}>
        Page {log.pagination.page} of {log.pagination.totalPages}
      </p>
      <div className="flex gap-2">
        <Link
          aria-disabled={!log.pagination.hasPrevious}
          className="btn"
          href={pageHref(selectedAction, Math.max(1, page - 1))}
        >
          Previous
        </Link>
        <Link
          aria-disabled={!log.pagination.hasNext}
          className="btn primary"
          href={pageHref(selectedAction, page + 1)}
        >
          Next
        </Link>
      </div>
    </nav>
  );
}

function pageHref(action: string | null, page: number) {
  const params = new URLSearchParams();
  if (action) params.set("action", action);
  if (page > 1) params.set("page", String(page));
  const query = params.toString();
  return `/settings/security-log${query ? `?${query}` : ""}`;
}

function exportHref(action: string | null, format: "csv" | "json") {
  const params = new URLSearchParams({ format });
  if (action) params.set("action", action);
  return `/settings/security-log/export?${params.toString()}`;
}

function labelForAction(action: string) {
  return action
    .split(".")
    .map((part) => part.replaceAll("_", " "))
    .join(" / ");
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}
