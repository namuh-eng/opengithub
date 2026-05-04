import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryBranchActivityFetchResult,
  RepositoryBranchActivityView,
  RepositoryCommitListItem,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryBranchActivityPageProps = {
  repository: RepositoryOverview;
  activityResult: RepositoryBranchActivityFetchResult;
  branchName: string;
};

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const diffMs = Date.now() - timestamp;
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 1000 * 60 * 60 * 24 * 365],
    ["month", 1000 * 60 * 60 * 24 * 30],
    ["day", 1000 * 60 * 60 * 24],
    ["hour", 1000 * 60 * 60],
    ["minute", 1000 * 60],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, unitMs] of units) {
    if (Math.abs(diffMs) >= unitMs) {
      return formatter.format(Math.round(-diffMs / unitMs), unit);
    }
  }
  return "just now";
}

function initials(login: string | null | undefined) {
  const fallback = login?.trim() || "unknown";
  return fallback
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function checkLabel(commit: RepositoryCommitListItem) {
  if (commit.status.totalCount === 0) {
    return "No checks";
  }
  if (commit.status.conclusion === "success") {
    return `${commit.status.totalCount} passed`;
  }
  if (commit.status.failedCount > 0 || commit.status.conclusion === "failure") {
    return `${commit.status.failedCount || 1} failed`;
  }
  return `${commit.status.completedCount}/${commit.status.totalCount} complete`;
}

function checkChip(commit: RepositoryCommitListItem) {
  if (commit.status.totalCount === 0) {
    return "chip soft";
  }
  if (commit.status.conclusion === "success") {
    return "chip ok";
  }
  if (commit.status.failedCount > 0 || commit.status.conclusion === "failure") {
    return "chip err";
  }
  return "chip warn";
}

function BranchActivityReadyPage({
  repository,
  activity,
}: {
  repository: RepositoryOverview;
  activity: RepositoryBranchActivityView;
}) {
  const branch = activity.branch;
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/branches`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div className="min-w-0">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Branch activity
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              style={{ color: "var(--ink-1)" }}
            >
              {branch.name}
            </h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Last updated {formatRelativeTime(branch.updatedAt)} with{" "}
              {branch.ahead} ahead and {branch.behind} behind the default
              branch.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn" href={activity.links.branchesHref}>
              Branches
            </Link>
            <Link className="btn" href={activity.links.treeHref}>
              Open tree
            </Link>
            <Link className="btn primary" href={activity.links.commitsHref}>
              Commit history
            </Link>
          </div>
        </section>

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
          <div className="grid gap-4">
            <section className="card overflow-hidden">
              <div
                className="border-b px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Recent commits
                </p>
              </div>
              {activity.recentCommits.length > 0 ? (
                <div>
                  {activity.recentCommits.map((commit) => (
                    <article className="list-row px-4 py-4" key={commit.oid}>
                      <span className="av sm shrink-0" aria-hidden="true">
                        {initials(commit.authorLogin)}
                      </span>
                      <div className="min-w-0 flex-1">
                        <Link
                          className="block truncate t-sm font-medium hover:underline"
                          href={commit.href}
                          style={{ color: "var(--ink-1)" }}
                        >
                          {commit.subject}
                        </Link>
                        <p className="t-xs mt-1">
                          <span className="t-mono-sm">{commit.shortOid}</span>{" "}
                          by {commit.authorLogin ?? "Unknown"}{" "}
                          {formatRelativeTime(commit.committedAt)}
                        </p>
                      </div>
                      <Link
                        className={checkChip(commit)}
                        href={commit.status.href}
                      >
                        {checkLabel(commit)}
                      </Link>
                    </article>
                  ))}
                </div>
              ) : (
                <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
                  No commits are indexed for this branch yet.
                </p>
              )}
            </section>

            <section className="card overflow-hidden">
              <div
                className="border-b px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Pull requests
                </p>
              </div>
              {activity.recentPullRequests.length > 0 ? (
                <div>
                  {activity.recentPullRequests.map((pullRequest) => (
                    <article
                      className="list-row px-4 py-4"
                      key={pullRequest.number}
                    >
                      <Link
                        className="t-sm font-medium hover:underline"
                        href={pullRequest.href}
                        style={{ color: "var(--ink-1)" }}
                      >
                        #{pullRequest.number} {pullRequest.title}
                      </Link>
                      <span
                        className={
                          pullRequest.draft ? "chip warn" : "chip soft"
                        }
                      >
                        {pullRequest.draft ? "Draft" : pullRequest.state}
                      </span>
                    </article>
                  ))}
                </div>
              ) : (
                <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
                  No pull requests currently point at this branch.
                </p>
              )}
            </section>
          </div>

          <aside className="grid content-start gap-4">
            <section className="card p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Summary
              </p>
              <div className="mt-3 flex flex-wrap gap-2">
                {branch.isDefault ? (
                  <span className="chip active">Default</span>
                ) : null}
                <span
                  className={
                    branch.protection.protected ? "chip warn" : "chip soft"
                  }
                >
                  {branch.protection.protected ? "Protected" : "Open"}
                </span>
                <Link className="chip soft" href={activity.links.compareHref}>
                  Compare
                </Link>
              </div>
              <dl className="mt-4 grid gap-3 t-sm">
                <div className="flex justify-between gap-3">
                  <dt style={{ color: "var(--ink-3)" }}>Ahead</dt>
                  <dd className="t-mono-sm">{branch.ahead}</dd>
                </div>
                <div className="flex justify-between gap-3">
                  <dt style={{ color: "var(--ink-3)" }}>Behind</dt>
                  <dd className="t-mono-sm">{branch.behind}</dd>
                </div>
                <div className="flex justify-between gap-3">
                  <dt style={{ color: "var(--ink-3)" }}>Checks</dt>
                  <dd className="t-mono-sm">{branch.checks.totalCount}</dd>
                </div>
              </dl>
            </section>

            <section className="card p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Rules
              </p>
              {activity.protectionEvents.length > 0 ? (
                <div className="mt-3 grid gap-3">
                  {activity.protectionEvents.map((event) => (
                    <Link
                      className="rounded-md border p-3 hover:bg-[var(--hover)]"
                      href={event.href}
                      key={`${event.sourceType}-${event.name}`}
                      style={{ borderColor: "var(--line)" }}
                    >
                      <span
                        className="t-sm font-medium"
                        style={{ color: "var(--ink-1)" }}
                      >
                        {event.name}
                      </span>
                      <span className="t-xs mt-1 block">
                        {event.sourceType} - {event.enforcement}
                      </span>
                      {event.requiredStatusChecks.length > 0 ? (
                        <span className="t-xs mt-2 block">
                          {event.requiredStatusChecks.join(", ")}
                        </span>
                      ) : null}
                    </Link>
                  ))}
                </div>
              ) : (
                <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                  No branch rules currently match this branch.
                </p>
              )}
              <Link
                className="btn mt-4 w-full justify-center"
                href={activity.links.rulesHref}
              >
                View rules
              </Link>
            </section>
          </aside>
        </section>
      </div>
    </RepositoryShell>
  );
}

export function RepositoryBranchActivityPage({
  repository,
  activityResult,
  branchName,
}: RepositoryBranchActivityPageProps) {
  if (!activityResult.ok) {
    return (
      <RepositoryShell
        activePath={`/${repository.owner_login}/${repository.name}/branches`}
        frameClassName="max-w-5xl"
        repository={repository}
      >
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Branch activity
          </p>
          <h1
            className="t-h2 mt-2 break-words"
            style={{ color: "var(--ink-1)" }}
          >
            {branchName}
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {activityResult.message}
          </p>
          <Link
            className="btn mt-4"
            href={`/${repository.owner_login}/${repository.name}/branches`}
          >
            Back to Branches
          </Link>
        </section>
      </RepositoryShell>
    );
  }

  return (
    <BranchActivityReadyPage
      activity={activityResult.activity}
      repository={repository}
    />
  );
}
