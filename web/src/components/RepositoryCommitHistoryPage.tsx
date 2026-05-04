import Link from "next/link";
import type {
  RepositoryCommitGroup,
  RepositoryCommitHistoryView,
  RepositoryCommitListItem,
  RepositoryCommitStatusSummary,
  RepositoryCommitVerificationSummary,
} from "@/lib/api";
import {
  repositoryBrowseAtCommitHref,
  repositoryCommitDetailHref,
  repositoryCommitHistoryHref,
  repositoryCommitStatusHref,
} from "@/lib/navigation";

type RepositoryCommitHistoryPageProps = {
  history: RepositoryCommitHistoryView;
};

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "long",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(`${value}T00:00:00Z`));
}

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const diffMs = Date.now() - timestamp;
  const absMs = Math.abs(diffMs);
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 1000 * 60 * 60 * 24 * 365],
    ["month", 1000 * 60 * 60 * 24 * 30],
    ["day", 1000 * 60 * 60 * 24],
    ["hour", 1000 * 60 * 60],
    ["minute", 1000 * 60],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, unitMs] of units) {
    if (absMs >= unitMs) {
      return formatter.format(Math.round(-diffMs / unitMs), unit);
    }
  }
  return "just now";
}

function initials(login: string | null) {
  const fallback = login?.trim() || "unknown";
  return fallback
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function statusLabel(status: RepositoryCommitStatusSummary) {
  if (status.totalCount === 0) {
    return "No checks";
  }
  if (status.status === "running") {
    return `${status.completedCount}/${status.totalCount} checks running`;
  }
  if (status.conclusion === "success") {
    return `${status.totalCount} checks passed`;
  }
  if (status.failedCount > 0 || status.conclusion === "failure") {
    return `${status.failedCount || 1} checks failed`;
  }
  return `${status.completedCount}/${status.totalCount} checks complete`;
}

function statusChipClass(status: RepositoryCommitStatusSummary) {
  if (status.totalCount === 0) {
    return "chip soft";
  }
  if (status.conclusion === "success") {
    return "chip ok";
  }
  if (status.failedCount > 0 || status.conclusion === "failure") {
    return "chip err";
  }
  if (status.status === "running") {
    return "chip accent";
  }
  return "chip warn";
}

function verificationLabel(verification: RepositoryCommitVerificationSummary) {
  if (verification.verified) {
    return "Verified";
  }
  if (verification.signatureState === "vigilant_unverified") {
    return "Partially verified";
  }
  return "Unverified";
}

function verificationClass(verification: RepositoryCommitVerificationSummary) {
  if (verification.verified) {
    return "chip ok";
  }
  if (verification.signatureState === "vigilant_unverified") {
    return "chip warn";
  }
  return "chip soft";
}

function CommitAvatar({ commit }: { commit: RepositoryCommitListItem }) {
  const label = commit.authorLogin ?? "Unknown author";
  return (
    <span aria-hidden="true" className="av sm shrink-0" title={label}>
      {initials(label)}
    </span>
  );
}

function CommitRow({
  commit,
  owner,
  repo,
  path,
}: {
  commit: RepositoryCommitListItem;
  owner: string;
  repo: string;
  path: string | null;
}) {
  const detailHref =
    commit.href || repositoryCommitDetailHref({ owner, repo, oid: commit.oid });
  const browseHref =
    path && commit.browseHref.endsWith(`/${commit.oid}`)
      ? repositoryBrowseAtCommitHref({ owner, repo, oid: commit.oid, path })
      : commit.browseHref;
  const statusHref =
    commit.status.href ||
    repositoryCommitStatusHref({ owner, repo, oid: commit.oid });

  return (
    <article className="list-row grid gap-3 px-4 py-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
      <div className="flex min-w-0 gap-3">
        <CommitAvatar commit={commit} />
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
            <Link
              className="min-w-0 truncate font-semibold hover:underline"
              href={detailHref}
              style={{ color: "var(--ink-1)" }}
            >
              {commit.subject || commit.message}
            </Link>
            {commit.pullRequests.map((pullRequest) => (
              <Link
                className="chip soft"
                href={pullRequest.href}
                key={pullRequest.number}
                title={pullRequest.title}
              >
                #{pullRequest.number}
              </Link>
            ))}
          </div>
          {commit.body ? (
            <details className="mt-1">
              <summary
                className="inline-flex cursor-pointer list-none items-center gap-1 t-xs"
                style={{ color: "var(--accent)" }}
              >
                Expand message
              </summary>
              <p
                className="mt-2 max-w-3xl whitespace-pre-wrap t-sm"
                style={{ color: "var(--ink-2)" }}
              >
                {commit.body}
              </p>
            </details>
          ) : null}
          <div
            className="mt-2 flex flex-wrap items-center gap-2 t-xs"
            style={{ color: "var(--ink-3)" }}
          >
            <span>{commit.authorLogin ?? "Unknown author"}</span>
            <span aria-hidden="true">·</span>
            <time dateTime={commit.committedAt}>
              {formatRelativeTime(commit.committedAt)}
            </time>
            <span
              className={verificationClass(commit.verification)}
              title={commit.verification.signatureSummary ?? undefined}
            >
              {verificationLabel(commit.verification)}
            </span>
            {commit.verification.signatureSummary ? (
              <span className="max-w-xl truncate">
                {commit.verification.signatureSummary}
              </span>
            ) : null}
          </div>
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-2 md:justify-end">
        <Link
          className={statusChipClass(commit.status)}
          href={statusHref}
          title={statusLabel(commit.status)}
        >
          {statusLabel(commit.status)}
        </Link>
        <Link className="btn sm t-mono-sm" href={detailHref}>
          {commit.shortOid}
        </Link>
        <Link
          aria-label={`Browse repository at ${commit.shortOid}`}
          className="btn sm"
          href={browseHref}
        >
          Browse
        </Link>
        <details className="relative">
          <summary
            className="btn sm inline-flex cursor-pointer list-none"
            aria-label={`More actions for ${commit.shortOid}`}
          >
            More
          </summary>
          <div
            className="absolute right-0 z-20 mt-2 w-56 overflow-hidden rounded-md py-1 text-sm"
            style={{
              background: "var(--surface)",
              border: "1px solid var(--line)",
              boxShadow: "var(--shadow-md)",
            }}
          >
            <Link
              className="block px-3 py-2 hover:bg-[var(--surface-2)]"
              href={detailHref}
            >
              Open commit detail
            </Link>
            <Link
              className="block px-3 py-2 hover:bg-[var(--surface-2)]"
              href={statusHref}
            >
              View check summary
            </Link>
            <Link
              className="block px-3 py-2 hover:bg-[var(--surface-2)]"
              href={browseHref}
            >
              Browse files at this commit
            </Link>
          </div>
        </details>
      </div>
    </article>
  );
}

function CommitGroupView({
  group,
  owner,
  repo,
  path,
}: {
  group: RepositoryCommitGroup;
  owner: string;
  repo: string;
  path: string | null;
}) {
  return (
    <section aria-labelledby={`commit-group-${group.date}`}>
      <div
        className="flex items-center justify-between border-b px-4 py-2"
        style={{
          borderColor: "var(--line-soft)",
          background: "var(--surface-2)",
        }}
      >
        <h2 className="t-label" id={`commit-group-${group.date}`}>
          {formatDate(group.date)}
        </h2>
        <span className="t-xs t-num">
          {group.commits.length}{" "}
          {group.commits.length === 1 ? "commit" : "commits"}
        </span>
      </div>
      {group.commits.map((commit) => (
        <CommitRow
          commit={commit}
          key={commit.oid}
          owner={owner}
          path={path}
          repo={repo}
        />
      ))}
    </section>
  );
}

export function RepositoryCommitHistoryPage({
  history,
}: RepositoryCommitHistoryPageProps) {
  const owner = history.repository.ownerLogin;
  const repo = history.repository.name;
  const refName = history.resolvedRef.shortName;
  const activePath = history.filters.path;
  const totalLabel = `${history.total} ${history.total === 1 ? "commit" : "commits"}`;
  const previousHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName,
    path: activePath,
    author: history.filters.author,
    until: history.filters.until,
    page: Math.max(1, history.page - 1),
    pageSize: history.pageSize,
  });
  const nextHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName,
    path: activePath,
    author: history.filters.author,
    until: history.filters.until,
    page: history.page + 1,
    pageSize: history.pageSize,
  });

  return (
    <div>
      <header
        className="border-b px-6 py-6"
        style={{ background: "var(--surface-2)", borderColor: "var(--line)" }}
      >
        <div className="mx-auto max-w-7xl">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                {owner}
              </p>
              <div className="mt-1 flex flex-wrap items-center gap-2">
                <Link
                  className="t-h2 hover:underline"
                  href={`/${owner}/${repo}`}
                  style={{ color: "var(--ink-1)" }}
                >
                  {repo}
                </Link>
                <span className="chip soft capitalize">
                  {history.repository.visibility}
                </span>
              </div>
            </div>
            <Link className="btn sm" href={history.resolvedRef.href}>
              Browse {refName}
            </Link>
          </div>
          <div className="mt-6 flex flex-wrap items-end justify-between gap-4">
            <div>
              <h1 className="t-h1">Commit history</h1>
              <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                {activePath ? (
                  <>
                    Path <span className="t-mono-sm">{activePath}</span> on{" "}
                    <span className="t-mono-sm">{refName}</span>
                  </>
                ) : (
                  <>
                    Default history for{" "}
                    <span className="t-mono-sm">{refName}</span>
                  </>
                )}
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <span className="chip soft t-num">{totalLabel}</span>
              <span className="chip soft">{history.resolvedRef.kind}</span>
              {history.resolvedRef.targetOid ? (
                <span className="chip soft t-mono-sm">
                  {history.resolvedRef.targetOid.slice(0, 7)}
                </span>
              ) : null}
            </div>
          </div>
        </div>
      </header>
      <main className="mx-auto max-w-7xl space-y-4 px-6 py-6">
        <section className="card p-3" aria-label="Commit history filters">
          <div className="flex flex-wrap items-center gap-2">
            <Link
              className="btn sm"
              href={repositoryCommitHistoryHref({
                owner,
                repo,
                refName,
                path: activePath,
              })}
            >
              {refName}
            </Link>
            <details className="relative">
              <summary className="btn sm inline-flex cursor-pointer list-none">
                {history.filters.author ?? "All users"}
              </summary>
              <div
                className="absolute left-0 z-20 mt-2 w-72 overflow-hidden rounded-md py-2"
                style={{
                  background: "var(--surface)",
                  border: "1px solid var(--line)",
                  boxShadow: "var(--shadow-md)",
                }}
              >
                <Link
                  className="block px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
                  href={repositoryCommitHistoryHref({
                    owner,
                    repo,
                    refName,
                    path: activePath,
                    until: history.filters.until,
                  })}
                >
                  All users
                </Link>
                {history.authorOptions.map((author) => (
                  <Link
                    className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
                    href={repositoryCommitHistoryHref({
                      owner,
                      repo,
                      refName,
                      path: activePath,
                      author: author.login,
                      until: history.filters.until,
                    })}
                    key={author.login}
                  >
                    <span className="av sm">{initials(author.login)}</span>
                    <span className="min-w-0 flex-1 truncate">
                      {author.login}
                    </span>
                    <span className="t-xs t-num">{author.count}</span>
                    {author.active ? (
                      <span className="chip active">Selected</span>
                    ) : null}
                  </Link>
                ))}
              </div>
            </details>
            <Link
              className="btn sm"
              href={repositoryCommitHistoryHref({
                owner,
                repo,
                refName,
                path: activePath,
                author: history.filters.author,
              })}
            >
              {history.filters.until
                ? `Until ${history.filters.until.slice(0, 10)}`
                : "All time"}
            </Link>
            {history.filters.author || history.filters.until ? (
              <Link
                className="chip active"
                href={repositoryCommitHistoryHref({
                  owner,
                  repo,
                  refName,
                  path: activePath,
                })}
              >
                Clear filters
              </Link>
            ) : null}
          </div>
        </section>
        <section className="card overflow-hidden" aria-label="Grouped commits">
          {history.groups.length > 0 ? (
            history.groups.map((group) => (
              <CommitGroupView
                group={group}
                key={group.date}
                owner={owner}
                path={activePath}
                repo={repo}
              />
            ))
          ) : (
            <div className="p-8">
              <h2 className="t-h2">No commits found</h2>
              <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                Clear filters or choose another branch to inspect repository
                history.
              </p>
              <Link
                className="btn mt-4"
                href={repositoryCommitHistoryHref({
                  owner,
                  repo,
                  refName,
                  path: activePath,
                })}
              >
                Clear commit filters
              </Link>
            </div>
          )}
        </section>
        {history.hasPreviousPage || history.hasNextPage ? (
          <nav
            aria-label="Commit pagination"
            className="flex flex-wrap justify-between gap-2"
          >
            {history.hasPreviousPage ? (
              <Link className="btn sm" href={previousHref}>
                Previous
              </Link>
            ) : (
              <span />
            )}
            {history.hasNextPage ? (
              <Link className="btn sm" href={nextHref}>
                Next
              </Link>
            ) : null}
          </nav>
        ) : null}
      </main>
    </div>
  );
}
