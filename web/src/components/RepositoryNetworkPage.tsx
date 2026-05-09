import Link from "next/link";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryNetworkFetchResult,
  RepositoryNetworkForkNode,
  RepositoryNetworkView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryForksHref, repositoryNetworkHref } from "@/lib/navigation";

type RepositoryNetworkPageProps = {
  repository: RepositoryOverview;
  networkResult: RepositoryNetworkFetchResult;
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

function networkForkBadges(fork: RepositoryNetworkForkNode) {
  const badges = [fork.isArchived ? "inactive" : "active"];
  if (fork.isArchived) badges.push("archived");
  if (fork.isStarredByActor) badges.push("starred");
  return badges;
}

function badgeClass(badge: string) {
  if (badge === "active") return "chip ok";
  if (badge === "starred") return "chip accent";
  if (badge === "inactive" || badge === "archived") return "chip warn";
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

function ForkNode({
  fork,
  index,
}: {
  fork: RepositoryNetworkForkNode;
  index: number;
}) {
  const offset = Math.min(index, 5) * 16;
  const badges = networkForkBadges(fork);
  return (
    <article
      className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]"
      style={{ alignItems: "start" }}
    >
      <div className="min-w-0">
        <div className="flex min-w-0 items-start gap-3">
          <div className="relative hidden h-12 w-8 shrink-0 md:block">
            <span
              aria-hidden="true"
              className="absolute top-1/2 h-px w-full"
              style={{
                background: "var(--line-strong)",
                left: `-${offset}px`,
              }}
            />
            <span
              aria-hidden="true"
              className="absolute left-0 top-1/2 h-3 w-3 rounded-full"
              style={{
                background: "var(--accent)",
                transform: "translateY(-50%)",
              }}
            />
          </div>
          <Link
            aria-label={`${fork.ownerLogin} profile`}
            className="av sm shrink-0"
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
              {badges.map((badge) => (
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

function NetworkReadyPage({
  network,
  repository,
}: {
  network: RepositoryNetworkView;
  repository: RepositoryOverview;
}) {
  const owner = network.repository.ownerLogin;
  const repo = network.repository.name;
  const forksHref = network.links.forksHref || repositoryForksHref(owner, repo);
  const treeHref = network.links.treeHref || network.repository.treeHref;
  const isEmpty = network.forks.length === 0;

  return (
    <RepositoryInsightsShell activeSection="network" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Network
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Repository network
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {network.summary.copy}
            </p>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {network.summary.updateNote}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className={network.freshness.stale ? "chip warn" : "chip ok"}>
              {network.freshness.stale
                ? "Stale projection"
                : "Fresh projection"}
            </span>
            <span className="chip soft">{network.freshness.cadence}</span>
            <Link className="btn" href={treeHref}>
              Tree view
            </Link>
            <Link className="btn primary" href={forksHref}>
              View forks
            </Link>
          </div>
        </section>

        <section
          aria-label="Network summary metrics"
          className="grid gap-4 md:grid-cols-3"
        >
          <article className="card min-h-32 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Readable forks
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(network.summary.totalReadableForks)}
            </p>
            <p className="t-xs mt-1">visible to you</p>
          </article>
          <article className="card min-h-32 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Projected forks
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(network.summary.projectedForks)}
            </p>
            <p className="t-xs mt-1">most recently pushed</p>
          </article>
          <article className="card min-h-32 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Private forks
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(network.summary.hiddenPrivateForks)}
            </p>
            <p className="t-xs mt-1">hidden by repository permissions</p>
          </article>
        </section>

        {isEmpty ? (
          <section className="card p-5">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              No forks yet
            </p>
            <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              This repository network has no readable forks.
            </h2>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Forks appear here after readable fork repositories push branch
              activity into the daily network projection.
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <Link className="btn" href={treeHref}>
                Browse source tree
              </Link>
              <Link className="btn primary" href={forksHref}>
                Open forks list
              </Link>
            </div>
          </section>
        ) : (
          <section className="card overflow-hidden">
            <div
              className="border-b px-4 py-3"
              style={{ borderColor: "var(--line)" }}
            >
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Recent fork graph
              </p>
              <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                Showing the 50 most recently pushed readable forks connected to{" "}
                <Link
                  className="t-mono-sm hover:underline"
                  href={network.repository.href}
                >
                  {owner}/{repo}
                </Link>
                .
              </p>
            </div>
            <ul
              aria-label="Repository network fork graph"
              className="relative m-0 list-none p-0"
            >
              <li aria-hidden="true">
                <div
                  className="absolute bottom-6 left-8 top-6 hidden w-px md:block"
                  style={{ background: "var(--line)" }}
                />
              </li>
              {network.forks.map((fork, index) => (
                <li key={fork.repositoryId}>
                  <ForkNode fork={fork} index={index} />
                </li>
              ))}
            </ul>
          </section>
        )}

        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Freshness
          </p>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Projection computed{" "}
            {formatRelativeTime(network.freshness.computedAt)}. It expires{" "}
            {formatRelativeTime(network.freshness.expiresAt)}.
          </p>
        </section>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryNetworkPage({
  repository,
  networkResult,
}: RepositoryNetworkPageProps) {
  if (!networkResult.ok) {
    return (
      <RepositoryInsightsShell activeSection="network" repository={repository}>
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Network
          </p>
          <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Network unavailable
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {networkResult.message}
          </p>
          <Link
            className="btn mt-4"
            href={repositoryNetworkHref(
              repository.owner_login,
              repository.name,
            )}
          >
            Retry Network
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return (
    <NetworkReadyPage network={networkResult.network} repository={repository} />
  );
}
