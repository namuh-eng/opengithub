import Link from "next/link";
import { RepositoryDependentsFilters } from "@/components/RepositoryDependentsFilters";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryDependentRow,
  RepositoryDependentsFetchResult,
  RepositoryDependentsView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryDependenciesHref,
  repositoryDependentsHref,
} from "@/lib/navigation";

type RepositoryDependentsPageProps = {
  repository: RepositoryOverview;
  dependentsResult: RepositoryDependentsFetchResult;
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

function DependencyTabs({
  owner,
  repo,
  dependenciesHref,
  dependentsHref,
}: {
  owner: string;
  repo: string;
  dependenciesHref?: string;
  dependentsHref?: string;
}) {
  return (
    <nav aria-label="Dependency graph tabs" className="tabs">
      <Link
        className="tab"
        href={dependenciesHref || repositoryDependenciesHref(owner, repo)}
      >
        Dependencies
      </Link>
      <Link
        aria-current="page"
        className="tab active"
        href={dependentsHref || repositoryDependentsHref(owner, repo)}
      >
        Dependents
      </Link>
    </nav>
  );
}

function DependentRow({ dependent }: { dependent: RepositoryDependentRow }) {
  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <Link
            className="break-words t-sm font-semibold hover:underline"
            href={dependent.href}
          >
            {dependent.ownerLogin}/{dependent.name}
          </Link>
          <span className="chip soft">{dependent.visibility}</span>
          <Link className="chip soft" href={dependent.packageHref}>
            {dependent.package.ecosystem}:{dependent.package.name}
          </Link>
        </div>
        {dependent.description ? (
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            {dependent.description}
          </p>
        ) : null}
        <p className="t-xs mt-2">
          Detected {formatRelativeTime(dependent.detectedAt)}
          {dependent.manifestPath ? " in " : null}
          {dependent.manifestPath ? (
            <span className="t-mono-sm">{dependent.manifestPath}</span>
          ) : null}
        </p>
      </div>
      <div className="flex flex-wrap gap-2 md:justify-end">
        <Link className="btn sm" href={dependent.ownerHref}>
          Owner
        </Link>
        <Link className="btn sm" href={dependent.href}>
          Repository
        </Link>
        <span className="chip soft">
          Stars{" "}
          <span className="t-num">{formatNumber(dependent.starsCount)}</span>
        </span>
        <span className="chip soft">
          Forks{" "}
          <span className="t-num">{formatNumber(dependent.forksCount)}</span>
        </span>
        <span className="chip soft">
          Issues{" "}
          <span className="t-num">
            {formatNumber(dependent.openIssuesCount)}
          </span>
        </span>
        <span className="chip soft">
          PRs{" "}
          <span className="t-num">
            {formatNumber(dependent.openPullRequestsCount)}
          </span>
        </span>
      </div>
    </article>
  );
}

function DependentsReadyPage({
  dependents,
  repository,
}: {
  dependents: RepositoryDependentsView;
  repository: RepositoryOverview;
}) {
  const owner = dependents.repository.ownerLogin;
  const repo = dependents.repository.name;
  const isEmpty = dependents.dependents.length === 0;

  return (
    <RepositoryInsightsShell
      activeSection="dependency-graph"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Dependency graph
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Dependents
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {dependents.availability.message}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip ok">Public index</span>
            <span className="chip soft">{dependents.freshness.cadence}</span>
            {dependents.summary.hiddenPrivateCount > 0 ? (
              <span className="chip warn">
                {formatNumber(dependents.summary.hiddenPrivateCount)} private
                hidden
              </span>
            ) : null}
          </div>
        </section>

        <DependencyTabs
          dependenciesHref={dependents.links.dependenciesHref}
          dependentsHref={dependents.links.dependentsHref}
          owner={owner}
          repo={repo}
        />

        <RepositoryDependentsFilters
          owner={owner}
          ownerFilter={dependents.filters.owner}
          packageFilter={dependents.filters.package}
          packages={dependents.packages}
          repo={repo}
        />

        <section
          aria-label="Dependents summary metrics"
          className="grid gap-4 md:grid-cols-3"
        >
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repositories
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependents.summary.repositoryCount)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Packages
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependents.summary.packageCount)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Private hidden
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependents.summary.hiddenPrivateCount)}
            </p>
          </article>
        </section>

        <details className="card p-4">
          <summary
            className="t-h3 cursor-pointer"
            style={{ color: "var(--ink-1)" }}
          >
            Counts are approximate
          </summary>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Dependents are computed from repositories with public indexed
            dependency graphs. Private consumers are counted only as hidden
            totals and are never named.
          </p>
        </details>

        {isEmpty ? (
          <section className="card p-5">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Empty dependents
            </p>
            <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              No public dependents matched these filters.
            </h2>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Clear the package or owner filter to see all public repositories
              currently indexed against this dependency graph.
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <Link
                className="btn primary"
                href={repositoryDependentsHref(owner, repo)}
              >
                Clear filters
              </Link>
              <Link className="btn" href={dependents.links.dependenciesHref}>
                Back to dependencies
              </Link>
            </div>
          </section>
        ) : (
          <section className="card overflow-hidden">
            <div
              className="between flex-wrap gap-3 px-4 py-3"
              style={{ borderBottom: "1px solid var(--line)" }}
            >
              <div>
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Public dependents
                </p>
                <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
                  {formatNumber(dependents.summary.repositoryCount)}{" "}
                  repositories
                </h2>
              </div>
            </div>
            <ul
              aria-label="Repository dependents list"
              className="m-0 list-none p-0"
            >
              {dependents.dependents.map((dependent) => (
                <li key={`${dependent.repositoryId}-${dependent.package.id}`}>
                  <DependentRow dependent={dependent} />
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
            Dependents computed{" "}
            {formatRelativeTime(dependents.freshness.computedAt)}. It expires{" "}
            {formatRelativeTime(dependents.freshness.expiresAt)}.
          </p>
        </section>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryDependentsPage({
  dependentsResult,
  repository,
}: RepositoryDependentsPageProps) {
  if (!dependentsResult.ok) {
    return (
      <RepositoryInsightsShell
        activeSection="dependency-graph"
        repository={repository}
      >
        <section className="card grid gap-3 p-6">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Dependency graph
          </p>
          <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
            Dependents unavailable
          </h1>
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            {dependentsResult.message}
          </p>
          <Link
            className="btn primary w-fit"
            href={repositoryDependentsHref(
              repository.owner_login,
              repository.name,
            )}
          >
            Retry dependents
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return (
    <DependentsReadyPage
      dependents={dependentsResult.dependents}
      repository={repository}
    />
  );
}
