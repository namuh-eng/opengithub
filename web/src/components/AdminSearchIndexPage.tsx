import Link from "next/link";
import type { ApiErrorEnvelope, SearchIndexStatus } from "@/lib/api";

type AdminSearchIndexPageProps = {
  status: SearchIndexStatus | ApiErrorEnvelope;
};

function isError(
  value: SearchIndexStatus | ApiErrorEnvelope,
): value is ApiErrorEnvelope {
  return "error" in value;
}

function formatDate(value: string | null) {
  if (!value) return "Never";
  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function statusChip(status: string) {
  if (status === "completed") return "chip ok";
  if (status === "failed") return "chip err";
  if (status === "running") return "chip warn";
  return "chip soft";
}

export function AdminSearchIndexPage({ status }: AdminSearchIndexPageProps) {
  if (isError(status)) {
    return (
      <main className="mx-auto max-w-[1240px] px-6 py-10">
        <section className="card p-6">
          <p className="t-label">Search index</p>
          <h1 className="t-h1 mt-2">Index status unavailable</h1>
          <p className="t-body mt-3 text-[color:var(--ink-3)]">
            {status.error.message}
          </p>
          <Link className="btn mt-5" href="/dashboard">
            Return to dashboard
          </Link>
        </section>
      </main>
    );
  }

  const totalDocuments = status.documents.reduce(
    (sum, document) => sum + document.total,
    0,
  );
  const activeEvents = status.events.queued + status.events.running;

  return (
    <main className="mx-auto max-w-[1240px] px-6 py-10">
      <header className="mb-8 grid gap-5 lg:grid-cols-[1fr_320px]">
        <div>
          <p className="t-label">Admin / Search</p>
          <h1 className="t-h1 mt-2">Indexing pipeline</h1>
          <p className="t-body mt-3 max-w-3xl text-[color:var(--ink-3)]">
            Watch write-time indexing for code, commits, issues, pull requests,
            repositories, users, and organizations.
          </p>
        </div>
        <div className="card grid grid-cols-2 gap-4 p-5">
          <div>
            <p className="t-label">Documents</p>
            <p className="t-num mt-2 text-3xl">{totalDocuments}</p>
          </div>
          <div>
            <p className="t-label">Active jobs</p>
            <p className="t-num mt-2 text-3xl">{activeEvents}</p>
          </div>
        </div>
      </header>

      <section className="grid gap-5 lg:grid-cols-4">
        {[
          ["Queued", status.events.queued, "soft"],
          ["Running", status.events.running, "warn"],
          ["Completed", status.events.completed, "ok"],
          ["Failed", status.events.failed, "err"],
        ].map(([label, count, tone]) => (
          <div className="card p-5" key={String(label)}>
            <span className={`chip ${tone}`}>{label}</span>
            <p className="t-num mt-4 text-3xl">{count}</p>
          </div>
        ))}
      </section>

      <section className="mt-8 grid gap-8 lg:grid-cols-[minmax(0,1fr)_360px]">
        <div className="card overflow-hidden">
          <div className="border-b border-[color:var(--line)] p-5">
            <h2 className="t-h2">Recent events</h2>
          </div>
          {status.recentEvents.length ? (
            <div>
              {status.recentEvents.map((event) => (
                <div className="list-row px-5 py-4" key={event.id}>
                  <div className="flex flex-wrap items-center gap-2">
                    <span className={statusChip(event.status)}>
                      {event.status}
                    </span>
                    <span className="t-mono-sm">{event.eventType}</span>
                    <span className="t-xs">{formatDate(event.createdAt)}</span>
                  </div>
                  <p className="t-body mt-2">{event.resourceKind}</p>
                  <p className="t-mono-sm mt-1 break-all text-[color:var(--ink-3)]">
                    {event.resourceId}
                  </p>
                  {event.lastError ? (
                    <p className="t-sm mt-2 text-[color:var(--err)]">
                      {event.lastError}
                    </p>
                  ) : null}
                </div>
              ))}
            </div>
          ) : (
            <p className="t-body p-5 text-[color:var(--ink-3)]">
              No index events have been recorded yet.
            </p>
          )}
        </div>

        <aside className="space-y-5">
          <section className="card p-5">
            <h2 className="t-h2">Documents</h2>
            <div className="mt-4 space-y-3">
              {status.documents.map((document) => (
                <div
                  className="flex items-start justify-between gap-4"
                  key={document.kind}
                >
                  <div>
                    <p className="t-mono-sm">{document.kind}</p>
                    <p className="t-xs">
                      {formatDate(document.latestIndexedAt)}
                    </p>
                  </div>
                  <p className="t-num">{document.total}</p>
                </div>
              ))}
            </div>
          </section>

          <section className="card p-5">
            <h2 className="t-h2">Needs attention</h2>
            <div className="mt-4 space-y-3">
              {status.staleRepositories.length ? (
                status.staleRepositories.map((repository) => (
                  <div key={repository.repositoryId}>
                    <Link
                      className="t-body underline"
                      href={`/${repository.ownerLogin}/${repository.name}`}
                    >
                      {repository.ownerLogin}/{repository.name}
                    </Link>
                    <p className="t-xs">
                      {repository.pendingEvents} pending,{" "}
                      {repository.failedEvents} failed
                    </p>
                  </div>
                ))
              ) : (
                <p className="t-body text-[color:var(--ink-3)]">
                  Every repository with indexed content is current.
                </p>
              )}
            </div>
          </section>
        </aside>
      </section>
    </main>
  );
}
