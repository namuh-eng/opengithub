import Link from "next/link";
import { RepositoryContributorsPeriodSelector } from "@/components/RepositoryContributorsPeriodSelector";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryContributorRow,
  RepositoryContributorsFetchResult,
  RepositoryContributorsView,
  RepositoryContributorsWeek,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryCommitHistoryHref } from "@/lib/navigation";

type RepositoryContributorsPageProps = {
  repository: RepositoryOverview;
  contributorsResult: RepositoryContributorsFetchResult;
};

function formatNumber(value: number | null | undefined) {
  if (value == null) return "omitted";
  return new Intl.NumberFormat("en").format(value);
}

function formatDate(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) return "recently";
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

function initials(login: string) {
  return login
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function lineDelta(additions: number | null, deletions: number | null) {
  if (additions == null || deletions == null) return "Line counts omitted";
  return `+${formatNumber(additions)} -${formatNumber(deletions)}`;
}

function StatPill({ label, value }: { label: string; value: number | null }) {
  return (
    <span className="chip soft">
      <span className="t-num">{formatNumber(value)}</span> {label}
    </span>
  );
}

function AuthorStatusChip({
  contributor,
}: {
  contributor: RepositoryContributorRow;
}) {
  if (contributor.isBot || contributor.authorStatus === "bot") {
    return <span className="chip info">Bot</span>;
  }
  if (contributor.authorStatus === "unmatched") {
    return <span className="chip warn">Unmatched author</span>;
  }
  return null;
}

function RepositoryCommitChart({
  weeks,
}: {
  weeks: RepositoryContributorsWeek[];
}) {
  const maxCommits = Math.max(1, ...weeks.map((week) => week.commits));

  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Commits over time
          </p>
          <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Repository-wide activity
          </h2>
        </div>
        <div className="flex flex-wrap gap-2">
          <a className="btn sm" href="#contributors-data-table">
            View as data table
          </a>
          <span className="chip soft">Accessible chart</span>
        </div>
      </div>

      <div
        aria-label="Repository commits over time chart"
        className={
          weeks.length > 0
            ? "mt-5 grid min-h-52 grid-cols-[repeat(auto-fit,minmax(34px,1fr))] items-end gap-2"
            : "mt-5"
        }
        role="img"
      >
        {weeks.length > 0 ? (
          weeks.map((week) => {
            const height = Math.max(12, (week.commits / maxCommits) * 100);
            return (
              <div
                className="flex min-w-0 flex-col items-center gap-2"
                key={week.weekStart}
              >
                <div
                  aria-hidden="true"
                  className="flex h-36 w-full items-end rounded-md"
                  style={{ background: "var(--surface-2)" }}
                >
                  <div
                    className="w-full rounded-md"
                    style={{
                      background: "var(--accent)",
                      height: `${height}%`,
                    }}
                  />
                </div>
                <span className="t-mono-sm text-center">
                  {formatNumber(week.commits)}
                </span>
                <span className="t-xs text-center">
                  {formatDate(week.weekStart)}
                </span>
              </div>
            );
          })
        ) : (
          <div
            className="rounded-md p-4"
            style={{ background: "var(--surface-2)" }}
          >
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No commits were indexed for this contributors window.
            </p>
          </div>
        )}
      </div>
    </section>
  );
}

function ContributorSections({
  contributors,
}: {
  contributors: RepositoryContributorRow[];
}) {
  const maxCommits = Math.max(
    1,
    ...contributors.map((contributor) => contributor.totalCommits),
  );

  return (
    <section className="grid gap-4">
      <div>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Contributors
        </p>
        <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
          Top contributors
        </h2>
      </div>
      {contributors.length > 0 ? (
        contributors.map((contributor) => {
          const totalWidth = Math.max(
            8,
            (contributor.totalCommits / maxCommits) * 100,
          );
          return (
            <article className="card p-5" key={contributor.login}>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="flex min-w-0 items-center gap-3">
                  <span aria-hidden="true" className="av">
                    {initials(contributor.login)}
                  </span>
                  <div className="min-w-0">
                    <Link
                      className="break-words t-h3 hover:underline"
                      href={contributor.profileHref}
                    >
                      {contributor.login}
                    </Link>
                    <p className="t-xs mt-1">
                      {lineDelta(
                        contributor.totalAdditions,
                        contributor.totalDeletions,
                      )}
                    </p>
                  </div>
                  <AuthorStatusChip contributor={contributor} />
                </div>
                <Link className="btn sm" href={contributor.commitsHref}>
                  {formatNumber(contributor.totalCommits)} commits
                </Link>
              </div>
              <div className="mt-5 grid gap-3">
                <div
                  aria-hidden="true"
                  className="h-3 overflow-hidden rounded-md"
                  style={{ background: "var(--surface-2)" }}
                >
                  <div
                    className="h-full rounded-md"
                    style={{
                      background: "var(--accent)",
                      width: `${totalWidth}%`,
                    }}
                  />
                </div>
                <div
                  aria-label={`${contributor.login} weekly commits chart`}
                  className="grid grid-cols-[repeat(auto-fit,minmax(30px,1fr))] items-end gap-2"
                  role="img"
                >
                  {contributor.weeks.map((week) => {
                    const height = Math.max(
                      10,
                      (week.commits / contributor.totalCommits) * 100,
                    );
                    return (
                      <div
                        className="grid min-w-0 gap-1"
                        key={`${contributor.login}-${week.weekStart}`}
                      >
                        <div
                          aria-hidden="true"
                          className="flex h-16 items-end rounded-md"
                          style={{ background: "var(--surface-2)" }}
                        >
                          <div
                            className="w-full rounded-md"
                            style={{
                              background: "var(--accent)",
                              height: `${height}%`,
                            }}
                          />
                        </div>
                        <span className="t-mono-sm text-center">
                          {formatNumber(week.commits)}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </article>
          );
        })
      ) : (
        <section className="card p-5">
          <h3 className="t-h3" style={{ color: "var(--ink-1)" }}>
            No contributor activity
          </h3>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            This repository has no default-branch commits with file changes in
            the selected period.
          </p>
        </section>
      )}
    </section>
  );
}

function ContributorsDataTable({
  contributors,
  weeks,
}: {
  contributors: RepositoryContributorRow[];
  weeks: RepositoryContributorsWeek[];
}) {
  return (
    <section className="card overflow-hidden" id="contributors-data-table">
      <div
        className="border-b px-4 py-3"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Data table
        </p>
        <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
          Weekly contributor values
        </h2>
      </div>
      <div className="overflow-x-auto p-4">
        <table className="w-full text-left t-sm">
          <caption className="sr-only">
            Repository contributors data table
          </caption>
          <thead className="t-label" style={{ color: "var(--ink-3)" }}>
            <tr>
              <th className="py-2 pr-3">Scope</th>
              <th className="py-2 pr-3">Week</th>
              <th className="py-2 pr-3 text-right">Commits</th>
              <th className="py-2 pr-3 text-right">Additions</th>
              <th className="py-2 text-right">Deletions</th>
            </tr>
          </thead>
          <tbody>
            {weeks.map((week) => (
              <tr
                className="border-t"
                key={`repo-${week.weekStart}`}
                style={{ borderColor: "var(--line-soft)" }}
              >
                <td className="py-2 pr-3">Repository</td>
                <td className="py-2 pr-3">{formatDate(week.weekStart)}</td>
                <td className="py-2 pr-3 text-right t-num">
                  {formatNumber(week.commits)}
                </td>
                <td className="py-2 pr-3 text-right t-num">
                  {formatNumber(week.additions)}
                </td>
                <td className="py-2 text-right t-num">
                  {formatNumber(week.deletions)}
                </td>
              </tr>
            ))}
            {contributors.flatMap((contributor) =>
              contributor.weeks.map((week) => (
                <tr
                  className="border-t"
                  key={`${contributor.login}-${week.weekStart}`}
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  <td className="py-2 pr-3">
                    <Link
                      className="break-words hover:underline"
                      href={contributor.profileHref}
                    >
                      {contributor.login}
                    </Link>
                  </td>
                  <td className="py-2 pr-3">{formatDate(week.weekStart)}</td>
                  <td className="py-2 pr-3 text-right t-num">
                    <Link
                      className="hover:underline"
                      href={contributor.commitsHref}
                    >
                      {formatNumber(week.commits)}
                    </Link>
                  </td>
                  <td className="py-2 pr-3 text-right t-num">
                    {formatNumber(week.additions)}
                  </td>
                  <td className="py-2 text-right t-num">
                    {formatNumber(week.deletions)}
                  </td>
                </tr>
              )),
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ContributorsReadyPage({
  contributors,
  repository,
}: {
  contributors: RepositoryContributorsView;
  repository: RepositoryOverview;
}) {
  const owner = contributors.repository.ownerLogin;
  const repo = contributors.repository.name;
  const dateRange = `${formatDate(contributors.period.startedAt)} - ${formatDate(
    contributors.period.endedAt,
  )}`;
  const commitHistoryHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName: contributors.repository.defaultBranch,
  });

  return (
    <RepositoryInsightsShell
      activeSection="contributors"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Contributors
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Contributor analytics
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Default branch scope:{" "}
              <span className="t-mono-sm">
                {contributors.repository.defaultBranch}
              </span>{" "}
              across {dateRange}.
            </p>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Merge commits and empty commits are excluded from this graph.
              {contributors.threshold.lineCountsOmitted
                ? ` ${contributors.threshold.message}`
                : " Line additions and deletions are included for this range."}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <RepositoryContributorsPeriodSelector
              activePeriod={contributors.period.key}
              owner={owner}
              repo={repo}
            />
            <a className="btn" href="#contributors-data-table">
              View as data table
            </a>
            <Link className="btn primary" href={commitHistoryHref}>
              Commit history
            </Link>
          </div>
        </section>

        <section className="card p-5">
          <div className="flex flex-wrap gap-2">
            <StatPill label="authors" value={contributors.totals.authors} />
            <StatPill label="commits" value={contributors.totals.commits} />
            <StatPill label="additions" value={contributors.totals.additions} />
            <StatPill label="deletions" value={contributors.totals.deletions} />
            <span
              className={
                contributors.threshold.lineCountsOmitted
                  ? "chip warn"
                  : "chip ok"
              }
            >
              {contributors.threshold.lineCountsOmitted
                ? "Line counts omitted"
                : "Line counts included"}
            </span>
          </div>
          <p className="t-xs mt-3">
            Snapshot computed{" "}
            {formatRelativeTime(contributors.snapshot.computedAt)}.
          </p>
        </section>

        <RepositoryCommitChart weeks={contributors.weeks} />
        <ContributorSections contributors={contributors.contributors} />
        <ContributorsDataTable
          contributors={contributors.contributors}
          weeks={contributors.weeks}
        />
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryContributorsPage({
  contributorsResult,
  repository,
}: RepositoryContributorsPageProps) {
  if (!contributorsResult.ok) {
    return (
      <RepositoryInsightsShell
        activeSection="contributors"
        repository={repository}
      >
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Contributors
          </p>
          <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Contributors unavailable
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {contributorsResult.message}
          </p>
          <Link
            className="btn mt-4"
            href={`/${repository.owner_login}/${repository.name}`}
          >
            Back to Code
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return (
    <ContributorsReadyPage
      contributors={contributorsResult.contributors}
      repository={repository}
    />
  );
}
