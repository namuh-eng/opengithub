import Link from "next/link";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import { RepositoryPulsePeriodSelector } from "@/components/RepositoryPulsePeriodSelector";
import type {
  RepositoryOverview,
  RepositoryPulseActivityItem,
  RepositoryPulseCommitter,
  RepositoryPulseFetchResult,
  RepositoryPulseMetric,
  RepositoryPulseView,
} from "@/lib/api";
import {
  repositoryCommitHistoryHref,
  repositoryIssuesHref,
  repositoryProfileHref,
  repositoryPullRequestsHref,
} from "@/lib/navigation";

type RepositoryPulsePageProps = {
  repository: RepositoryOverview;
  pulseResult: RepositoryPulseFetchResult;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function formatDate(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) {
    return "recently";
  }
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
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

function initials(login: string | null | undefined) {
  const fallback = login?.trim() || "unknown";
  return fallback
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function metricTone(metric: RepositoryPulseMetric) {
  if (metric.key.includes("merged") || metric.key.includes("closed")) {
    return "chip ok";
  }
  if (metric.key.includes("new")) {
    return "chip accent";
  }
  return "chip soft";
}

function MetricCard({ metric }: { metric: RepositoryPulseMetric }) {
  return (
    <Link
      aria-label={`${metric.label} ${formatNumber(metric.count)}`}
      className="card block min-h-36 p-4 hover:no-underline"
      href={metric.href}
      style={{ color: "var(--ink-1)" }}
    >
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {metric.label}
      </p>
      <div className="mt-3 flex items-end justify-between gap-3">
        <span className="t-h1 t-num">{formatNumber(metric.count)}</span>
        <span className={metricTone(metric)}>Open filtered list</span>
      </div>
    </Link>
  );
}

function StatPill({ label, value }: { label: string; value: number }) {
  return (
    <span className="chip soft">
      <span className="t-num">{formatNumber(value)}</span> {label}
    </span>
  );
}

function CommitterAvatar({
  committer,
}: {
  committer: RepositoryPulseCommitter;
}) {
  return (
    <span aria-hidden="true" className="av sm">
      {initials(committer.login)}
    </span>
  );
}

function AuthorStatusChip({ status }: { status?: string | null }) {
  if (status === "bot") {
    return <span className="chip info">Bot</span>;
  }
  if (status === "unmatched" || status === "unavailable") {
    return <span className="chip warn">Unavailable author</span>;
  }
  return null;
}

function ActivityAuthor({ item }: { item: RepositoryPulseActivityItem }) {
  const label = item.authorLogin?.trim() || "Unavailable author";
  if (item.authorProfileHref) {
    return (
      <Link className="hover:underline" href={item.authorProfileHref}>
        {label}
      </Link>
    );
  }
  return <span>{label}</span>;
}

function TopCommitters({
  committers,
}: {
  committers: RepositoryPulseCommitter[];
}) {
  const maxCommits = Math.max(1, ...committers.map((item) => item.commits));

  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Top committers
          </p>
          <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Contribution share
          </h2>
        </div>
        <span className="chip soft">Table included</span>
      </div>

      {committers.length > 0 ? (
        <div className="mt-5 grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
          <div
            aria-label="Top committers bar chart"
            className="grid gap-3"
            role="img"
          >
            {committers.map((committer) => {
              const width = Math.max(8, (committer.commits / maxCommits) * 100);
              return (
                <div className="grid gap-2" key={committer.login}>
                  <div className="flex items-center justify-between gap-3">
                    <Link
                      className="flex min-w-0 items-center gap-2 hover:underline"
                      href={
                        committer.profileHref ||
                        repositoryProfileHref(committer.login)
                      }
                    >
                      <CommitterAvatar committer={committer} />
                      <span className="truncate t-sm font-semibold">
                        {committer.login}
                      </span>
                    </Link>
                    <AuthorStatusChip status={committer.authorStatus} />
                    <Link
                      className="shrink-0 t-mono-sm hover:underline"
                      href={committer.commitsHref}
                    >
                      {formatNumber(committer.commits)} commits
                    </Link>
                  </div>
                  <div
                    aria-hidden="true"
                    className="h-3 overflow-hidden rounded-md"
                    style={{ background: "var(--surface-2)" }}
                  >
                    <div
                      className="h-full rounded-md"
                      style={{
                        background: "var(--accent)",
                        width: `${width}%`,
                      }}
                    />
                  </div>
                </div>
              );
            })}
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left t-sm">
              <caption className="sr-only">Top committers data table</caption>
              <thead className="t-label" style={{ color: "var(--ink-3)" }}>
                <tr>
                  <th className="py-2 pr-3">Committer</th>
                  <th className="py-2 pr-3 text-right">Commits</th>
                  <th className="py-2 pr-3 text-right">Files</th>
                  <th className="py-2 text-right">Delta</th>
                </tr>
              </thead>
              <tbody>
                {committers.map((committer) => (
                  <tr
                    className="border-t"
                    key={committer.login}
                    style={{ borderColor: "var(--line-soft)" }}
                  >
                    <td className="py-2 pr-3">
                      <Link
                        className="break-words hover:underline"
                        href={committer.profileHref}
                      >
                        {committer.login}
                      </Link>
                      {committer.isBot ? (
                        <span className="chip info ml-2">Bot</span>
                      ) : null}
                    </td>
                    <td className="py-2 pr-3 text-right t-num">
                      <Link
                        className="hover:underline"
                        href={committer.commitsHref}
                      >
                        {formatNumber(committer.commits)}
                      </Link>
                    </td>
                    <td className="py-2 pr-3 text-right t-num">
                      {formatNumber(committer.filesChanged)}
                    </td>
                    <td className="whitespace-nowrap py-2 text-right t-num">
                      +{formatNumber(committer.additions)} -
                      {formatNumber(committer.deletions)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div
          className="mt-5 rounded-md p-4"
          style={{ background: "var(--surface-2)" }}
        >
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No commits were indexed for this Pulse window.
          </p>
        </div>
      )}
    </section>
  );
}

function ActivityList({
  title,
  description,
  items,
  emptyHref,
  emptyLabel,
}: {
  title: string;
  description: string;
  items: RepositoryPulseActivityItem[];
  emptyHref: string;
  emptyLabel: string;
}) {
  return (
    <section className="card overflow-hidden">
      <div
        className="border-b px-4 py-3"
        style={{ borderColor: "var(--line)" }}
      >
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          {title}
        </p>
        <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
          {description}
        </p>
      </div>
      {items.length > 0 ? (
        <div>
          {items.map((item) => (
            <div
              className="list-row px-4 py-3"
              key={`${item.kind}-${item.number ?? item.href}`}
            >
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 flex-wrap items-center gap-2">
                  <span
                    className={
                      item.state === "open"
                        ? "chip accent"
                        : item.state === "closed" || item.state === "merged"
                          ? "chip ok"
                          : "chip soft"
                    }
                  >
                    {item.state}
                  </span>
                  {item.number ? (
                    <span className="t-mono-sm">#{item.number}</span>
                  ) : null}
                  <Link
                    aria-label={`${item.title}${item.number ? ` #${item.number}` : ""}`}
                    className="min-w-0 truncate t-sm font-semibold hover:underline"
                    href={item.href}
                    style={{ color: "var(--ink-1)" }}
                  >
                    {item.title}
                  </Link>
                </div>
                <p className="t-xs mt-1">
                  <ActivityAuthor item={item} /> ·{" "}
                  {formatRelativeTime(item.occurredAt)}
                  {item.authorStatus === "bot" ? " · bot" : ""}
                </p>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="px-4 py-5">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No activity in this section for the selected period.
          </p>
          <Link className="btn sm mt-3" href={emptyHref}>
            {emptyLabel}
          </Link>
        </div>
      )}
    </section>
  );
}

function PulseReadyPage({
  repository,
  pulse,
}: {
  repository: RepositoryOverview;
  pulse: RepositoryPulseView;
}) {
  const owner = pulse.repository.ownerLogin;
  const repo = pulse.repository.name;
  const commitHistoryHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName: pulse.repository.defaultBranch,
  });
  const dateRange = `${formatDate(pulse.period.startedAt)} - ${formatDate(
    pulse.period.endedAt,
  )}`;

  return (
    <RepositoryInsightsShell activeSection="pulse" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Pulse
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Insights
            </h1>
            <h2 className="t-h2 mt-3" style={{ color: "var(--ink-1)" }}>
              Repository activity
            </h2>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Pulse activity summary for {owner}/{repo}.
            </p>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {dateRange}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <RepositoryPulsePeriodSelector
              activePeriod={pulse.period.key}
              owner={owner}
              repo={repo}
            />
            <span className="chip active">Active period</span>
            <Link className="btn primary" href={commitHistoryHref}>
              Commit history
            </Link>
          </div>
        </section>

        <section className="card p-5">
          <div className="flex flex-wrap gap-2">
            <StatPill label="authors" value={pulse.summary.authors} />
            <StatPill label="commits" value={pulse.summary.commits} />
            <StatPill label="files" value={pulse.summary.filesChanged} />
            <StatPill label="additions" value={pulse.summary.additions} />
            <StatPill label="deletions" value={pulse.summary.deletions} />
          </div>
          <p
            className="t-body mt-4 max-w-3xl"
            style={{ color: "var(--ink-2)" }}
          >
            {pulse.summary.sentence}
          </p>
          <p className="t-xs mt-3">
            Snapshot computed {formatRelativeTime(pulse.snapshot.computedAt)}.
          </p>
        </section>

        <section
          aria-label="Pulse overview metrics"
          className="grid gap-4 md:grid-cols-2 xl:grid-cols-4"
        >
          {pulse.metrics.map((metric) => (
            <MetricCard key={metric.key} metric={metric} />
          ))}
        </section>

        <TopCommitters committers={pulse.topCommitters} />

        <div className="grid gap-4 xl:grid-cols-3">
          <ActivityList
            description="Published release activity during this Pulse window."
            emptyHref={`/${owner}/${repo}/releases`}
            emptyLabel="View releases"
            items={pulse.releases}
            title="Releases"
          />
          <ActivityList
            description="Pull requests merged during this Pulse window."
            emptyHref={repositoryPullRequestsHref(owner, repo, {
              state: "merged",
            })}
            emptyLabel="View pull requests"
            items={pulse.mergedPullRequests}
            title="Merged pull requests"
          />
          <ActivityList
            description="Issue activity opened or closed during this Pulse window."
            emptyHref={repositoryIssuesHref(owner, repo)}
            emptyLabel="View issues"
            items={pulse.issueActivity}
            title="Issue activity"
          />
        </div>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryPulsePage({
  repository,
  pulseResult,
}: RepositoryPulsePageProps) {
  if (!pulseResult.ok) {
    return (
      <RepositoryInsightsShell activeSection="pulse" repository={repository}>
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Pulse
          </p>
          <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Pulse unavailable
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {pulseResult.message}
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

  return <PulseReadyPage pulse={pulseResult.pulse} repository={repository} />;
}
