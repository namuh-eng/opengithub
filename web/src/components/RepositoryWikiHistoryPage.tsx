"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryOverview,
  RepositoryWikiHistoryFetchResult,
  RepositoryWikiHistoryRevision,
  RepositoryWikiHistoryView,
} from "@/lib/api";

type RepositoryWikiHistoryPageProps = {
  repository: RepositoryOverview;
  historyResult: RepositoryWikiHistoryFetchResult;
};

function formatDateTime(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

function relativeTime(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  const diffMs = Date.now() - date.getTime();
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 365 * 24 * 60 * 60 * 1000],
    ["month", 30 * 24 * 60 * 60 * 1000],
    ["day", 24 * 60 * 60 * 1000],
    ["hour", 60 * 60 * 1000],
    ["minute", 60 * 1000],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, size] of units) {
    if (Math.abs(diffMs) >= size) {
      return formatter.format(-Math.round(diffMs / size), unit);
    }
  }
  return "just now";
}

function authorName(revision: RepositoryWikiHistoryRevision) {
  return revision.author?.displayName ?? revision.author?.login ?? "Unknown";
}

function RevisionAvatar({
  revision,
}: {
  revision: RepositoryWikiHistoryRevision;
}) {
  const label = authorName(revision).slice(0, 2).toUpperCase();
  return (
    <span aria-hidden="true" className="av sm">
      {label}
    </span>
  );
}

function HistoryReader({ history }: { history: RepositoryWikiHistoryView }) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const selectedRows = useMemo(
    () => history.revisions.filter((revision) => selected.has(revision.id)),
    [history.revisions, selected],
  );
  const compareEnabled = selectedRows.length === 2;
  const scopedPage = history.scope.page;

  function toggleRevision(revisionId: string) {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(revisionId)) {
        next.delete(revisionId);
      } else {
        next.add(revisionId);
      }
      return next;
    });
  }

  return (
    <div className="grid gap-5">
      <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository wiki
          </p>
          <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
            History
          </h1>
          <p className="t-sm mt-3 max-w-3xl" style={{ color: "var(--ink-3)" }}>
            {scopedPage ? (
              <>
                Revision history for{" "}
                <Link className="hover:underline" href={scopedPage.href}>
                  {scopedPage.title}
                </Link>
                .
              </>
            ) : (
              "Revision history across all wiki pages."
            )}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Link className="btn sm" href={history.links.homeHref}>
            Wiki Home
          </Link>
          <Link className="btn sm" href={history.links.pagesHref}>
            Pages
          </Link>
          <button
            aria-describedby="wiki-history-compare-help"
            className="btn primary sm"
            disabled={!compareEnabled}
            type="button"
          >
            Compare Revisions
          </button>
        </div>
      </section>

      <p className="t-xs" id="wiki-history-compare-help">
        Select two revisions to compare. Compare opens in the next wiki history
        phase.
      </p>

      <section className="card overflow-hidden" aria-label="Wiki revisions">
        {history.revisions.length > 0 ? (
          <ul className="divide-y" style={{ borderColor: "var(--line)" }}>
            {history.revisions.map((revision) => (
              <li className="list-row p-0" key={revision.id}>
                <div className="grid gap-4 px-4 py-4 md:grid-cols-[auto_minmax(0,1fr)_auto] md:items-start">
                  <label className="flex items-start gap-3">
                    <input
                      aria-label={`Select revision ${revision.message}`}
                      checked={selected.has(revision.id)}
                      className="mt-1"
                      onChange={() => toggleRevision(revision.id)}
                      type="checkbox"
                    />
                    <RevisionAvatar revision={revision} />
                  </label>
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <Link
                        className="t-h3 break-words hover:underline"
                        href={revision.revisionHref}
                        style={{ color: "var(--ink-1)" }}
                      >
                        {revision.message}
                      </Link>
                      <span className="chip soft">{revision.pageTitle}</span>
                    </div>
                    <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                      {revision.author ? (
                        <Link
                          className="hover:underline"
                          href={revision.author.href}
                        >
                          {authorName(revision)}
                        </Link>
                      ) : (
                        "Unknown author"
                      )}{" "}
                      committed {relativeTime(revision.createdAt)}
                    </p>
                    <p className="t-xs mt-1">
                      <time dateTime={revision.createdAt}>
                        {formatDateTime(revision.createdAt)}
                      </time>
                    </p>
                  </div>
                  <div className="flex flex-wrap items-center gap-2 md:justify-end">
                    <Link className="btn sm" href={revision.pageHref}>
                      Page
                    </Link>
                    <Link
                      className="btn sm t-mono-sm"
                      href={revision.revisionHref}
                    >
                      {revision.shortOid ?? "revision"}
                    </Link>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <div className="p-5">
            <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
              No wiki history yet
            </h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Saved wiki revisions will appear here after the first page edit.
            </p>
          </div>
        )}
      </section>

      <nav
        aria-label="Wiki history pagination"
        className="flex flex-wrap justify-between gap-2"
      >
        {history.pagination.newerHref ? (
          <Link className="btn sm" href={history.pagination.newerHref}>
            Newer
          </Link>
        ) : (
          <span className="btn sm" aria-disabled="true">
            Newer
          </span>
        )}
        {history.pagination.olderHref ? (
          <Link className="btn sm" href={history.pagination.olderHref}>
            Older
          </Link>
        ) : (
          <span className="btn sm" aria-disabled="true">
            Older
          </span>
        )}
      </nav>
    </div>
  );
}

function WikiHistoryUnavailable({ message }: { message: string }) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Repository wiki
      </p>
      <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
        History unavailable
      </h1>
      <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
        {message}
      </p>
    </section>
  );
}

export function RepositoryWikiHistoryPage({
  repository,
  historyResult,
}: RepositoryWikiHistoryPageProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/wiki`}
      repository={repository}
    >
      {historyResult.ok ? (
        <HistoryReader history={historyResult.history} />
      ) : (
        <WikiHistoryUnavailable message={historyResult.message} />
      )}
    </RepositoryShell>
  );
}
