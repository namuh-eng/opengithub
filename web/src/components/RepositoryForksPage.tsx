import Link from "next/link";
import { RepositoryForkFilters } from "@/components/RepositoryForkFilters";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryForkRow,
  RepositoryForksFetchResult,
  RepositoryForksView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryForksHref, repositoryNetworkHref } from "@/lib/navigation";

type RepositoryForksPageProps = {
  repository: RepositoryOverview;
  forksResult: RepositoryForksFetchResult;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
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

function ownerInitial(login: string) {
  return login.trim().slice(0, 2).toUpperCase() || "OG";
}

function badgeClass(badge: string) {
  if (badge === "active") return "chip ok";
  if (badge === "inactive") return "chip warn";
  if (badge === "archived") return "chip warn";
  if (badge === "starred") return "chip accent";
  return "chip soft";
}

function ForkMetric({
  href,
  label,
  value,
}: {
  href: string;
  label: string;
  value: number;
}) {
  return (
    <Link
      aria-label={`${formatNumber(value)} ${label}`}
      className="chip soft"
      href={href}
    >
      <span className="t-num">{formatNumber(value)}</span> {label}
    </Link>
  );
}

function ForkRow({ fork }: { fork: RepositoryForkRow }) {
  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex min-w-0 items-start gap-3">
          <Link
            aria-label={`${fork.ownerLogin} profile`}
            className="av shrink-0"
            href={fork.ownerHref}
          >
            {ownerInitial(fork.ownerLogin)}
          </Link>
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <Link
                className="break-words t-sm font-semibold hover:underline"
                href={fork.href}
              >
                {fork.ownerLogin}/{fork.name}
              </Link>
              <span className="chip soft">{fork.visibility}</span>
              {fork.badges.map((badge) => (
                <span className={badgeClass(badge)} key={badge}>
                  {badge}
                </span>
              ))}
            </div>
            {fork.description ? (
              <p
                className="t-sm mt-2 break-words"
                style={{ color: "var(--ink-3)" }}
              >
                {fork.description}
              </p>
            ) : (
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                No fork description provided.
              </p>
            )}
            <div className="mt-3 flex flex-wrap gap-2">
              <ForkMetric
                href={fork.href}
                label="stars"
                value={fork.starsCount}
              />
              <ForkMetric
                href={fork.networkHref}
                label="forks"
                value={fork.forksCount}
              />
              <ForkMetric
                href={`${fork.href}/issues`}
                label="issues"
                value={fork.openIssuesCount}
              />
              <ForkMetric
                href={`${fork.href}/pulls`}
                label="pull requests"
                value={fork.openPullRequestsCount}
              />
            </div>
          </div>
        </div>
      </div>
      <div className="flex flex-wrap gap-2 md:justify-end">
        <Link
          aria-label={`${fork.ownerLogin}/${fork.name} tree`}
          className="btn sm"
          href={fork.treeHref}
        >
          Tree
        </Link>
        <Link
          aria-label={`${fork.ownerLogin}/${fork.name} network`}
          className="btn sm"
          href={fork.networkHref}
        >
          Network
        </Link>
      </div>
      <p className="t-xs md:col-span-2">
        Pushed {formatRelativeTime(fork.pushedAt)} · created{" "}
        {formatRelativeTime(fork.createdAt)} · updated{" "}
        {formatRelativeTime(fork.updatedAt)}
      </p>
    </article>
  );
}

function ForksReadyPage({
  forks,
  repository,
}: {
  forks: RepositoryForksView;
  repository: RepositoryOverview;
}) {
  const owner = forks.repository.ownerLogin;
  const repo = forks.repository.name;
  const isEmpty = forks.forks.length === 0;

  return (
    <RepositoryInsightsShell activeSection="forks" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Forks
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Forked repositories
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Browse readable forks from the repository network. Filters update
              the URL and Save defaults stores your preferred view for this
              repository.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className={forks.freshness.stale ? "chip warn" : "chip ok"}>
              {forks.freshness.stale ? "Stale projection" : "Fresh projection"}
            </span>
            <span className="chip soft">{forks.freshness.cadence}</span>
            <Link className="btn" href={repositoryNetworkHref(owner, repo)}>
              Switch to tree view
            </Link>
          </div>
        </section>

        <RepositoryForkFilters
          defaultsMatch={forks.defaults.matchesCurrent}
          defaultsSaved={forks.defaults.saved}
          owner={owner}
          period={forks.filters.period.key}
          repo={repo}
          repositoryType={forks.filters.repositoryType}
          sort={forks.filters.sort}
        />

        <section
          aria-label="Fork summary metrics"
          className="grid gap-4 md:grid-cols-3"
        >
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Matching forks
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(forks.total)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Hidden private forks
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(forks.hiddenPrivateForks)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Saved view
            </p>
            <p className="t-h3 mt-3" style={{ color: "var(--ink-1)" }}>
              {forks.defaults.matchesCurrent
                ? "Matches defaults"
                : "Custom view"}
            </p>
            <p className="t-xs mt-1">
              {forks.defaults.savedAt
                ? `Saved ${formatRelativeTime(forks.defaults.savedAt)}`
                : "No saved default yet"}
            </p>
          </article>
        </section>

        <section className="card overflow-hidden">
          <div
            className="between flex-wrap gap-3 px-4 py-3"
            style={{ borderBottom: "1px solid var(--line)" }}
          >
            <div>
              <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
                Forks
              </h2>
              <p className="t-xs">
                {forks.filters.period.label} · {forks.total} matching
              </p>
            </div>
            <Link
              className="btn sm"
              href={repositoryForksHref(owner, repo, {
                period: "all",
                repositoryType: "all",
                sort: "most_starred",
              })}
            >
              Show all forks
            </Link>
          </div>
          {isEmpty ? (
            <div className="grid gap-3 p-6">
              <h3 className="t-h2" style={{ color: "var(--ink-1)" }}>
                No forks match these filters.
              </h3>
              <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
                Try all periods or open the Network page to inspect projected
                fork activity.
              </p>
              <div className="flex flex-wrap gap-2">
                <Link
                  className="btn primary"
                  href={repositoryForksHref(owner, repo, {
                    period: "all",
                    repositoryType: "all",
                    sort: "most_starred",
                  })}
                >
                  Reset fork filters
                </Link>
                <Link className="btn" href={repositoryNetworkHref(owner, repo)}>
                  Open Network
                </Link>
              </div>
            </div>
          ) : (
            <ul
              aria-label="Repository forks list"
              className="m-0 list-none p-0"
            >
              {forks.forks.map((fork) => (
                <li key={fork.repositoryId}>
                  <ForkRow fork={fork} />
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryForksPage({
  forksResult,
  repository,
}: RepositoryForksPageProps) {
  if (!forksResult.ok) {
    return (
      <RepositoryInsightsShell activeSection="forks" repository={repository}>
        <section className="card grid gap-3 p-6">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Forks
          </p>
          <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
            Forks unavailable
          </h1>
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            {forksResult.message}
          </p>
          <Link
            className="btn primary w-fit"
            href={repositoryForksHref(repository.owner_login, repository.name)}
          >
            Retry Forks
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return <ForksReadyPage forks={forksResult.forks} repository={repository} />;
}
