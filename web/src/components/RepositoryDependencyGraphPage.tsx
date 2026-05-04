import Link from "next/link";
import { RepositoryDependencyFilters } from "@/components/RepositoryDependencyFilters";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryDependenciesFetchResult,
  RepositoryDependenciesView,
  RepositoryDependencyAdvisorySummary,
  RepositoryDependencyManifest,
  RepositoryDependencyRow,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryDependenciesHref,
  repositoryDependencyPackageHref,
  repositoryDependentsHref,
} from "@/lib/navigation";

type RepositoryDependencyGraphPageProps = {
  repository: RepositoryOverview;
  dependenciesResult: RepositoryDependenciesFetchResult;
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

function relationshipClass(relationship: string) {
  if (relationship === "direct") return "chip ok";
  if (relationship === "transitive") return "chip soft";
  return "chip warn";
}

function severityClass(advisory: RepositoryDependencyAdvisorySummary) {
  if (advisory.severity === "critical" || advisory.severity === "high") {
    return "chip err";
  }
  if (advisory.severity === "moderate") return "chip warn";
  return "chip soft";
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
        aria-current="page"
        className="tab active"
        href={dependenciesHref || repositoryDependenciesHref(owner, repo)}
      >
        Dependencies
      </Link>
      <Link
        className="tab"
        href={dependentsHref || repositoryDependentsHref(owner, repo)}
      >
        Dependents
      </Link>
    </nav>
  );
}

function ManifestSummary({
  manifests,
}: {
  manifests: RepositoryDependencyManifest[];
}) {
  if (manifests.length === 0) return null;

  return (
    <section className="card overflow-hidden">
      <div
        className="between flex-wrap gap-3 px-4 py-3"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Manifests
          </p>
          <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
            Indexed dependency manifests
          </h2>
        </div>
      </div>
      <ul
        aria-label="Indexed dependency manifests"
        className="m-0 list-none p-0"
      >
        {manifests.map((manifest) => (
          <li className="list-row px-4 py-3" key={manifest.id}>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <Link
                  className="break-words t-mono-sm hover:underline"
                  href={manifest.href}
                >
                  {manifest.path}
                </Link>
                <span className="chip soft">{manifest.ecosystem}</span>
                <span className="chip soft">
                  <span className="t-num">
                    {formatNumber(manifest.dependencyCount)}
                  </span>{" "}
                  dependencies
                </span>
              </div>
              <p className="t-xs mt-1">
                Detected {formatRelativeTime(manifest.detectedAt)}
                {manifest.lockfilePath ? " with lockfile " : null}
                {manifest.lockfilePath && manifest.lockfileHref ? (
                  <Link
                    className="t-mono-sm hover:underline"
                    href={manifest.lockfileHref}
                  >
                    {manifest.lockfilePath}
                  </Link>
                ) : null}
              </p>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}

function DependencyRow({
  dependency,
}: {
  dependency: RepositoryDependencyRow;
}) {
  const packageHref = repositoryDependencyPackageHref({
    ecosystem: dependency.package.ecosystem,
    fallbackHref: dependency.package.href,
    name: dependency.package.name,
  });

  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <Link
            className="break-words t-sm font-semibold hover:underline"
            href={packageHref}
          >
            {dependency.package.name}
          </Link>
          {dependency.version ? (
            <span className="chip soft t-mono-sm">{dependency.version}</span>
          ) : null}
          <span className={relationshipClass(dependency.relationship)}>
            {dependency.relationship}
          </span>
          <span className="chip soft">{dependency.package.ecosystem}</span>
          {dependency.license ? (
            <span className="chip soft">{dependency.license}</span>
          ) : null}
        </div>
        <div className="mt-3 flex flex-wrap gap-2">
          <Link className="chip soft" href={dependency.manifestHref}>
            <span className="t-mono-sm">{dependency.manifestPath}</span>
          </Link>
          {dependency.lockfilePath && dependency.lockfileHref ? (
            <Link className="chip soft" href={dependency.lockfileHref}>
              <span className="t-mono-sm">{dependency.lockfilePath}</span>
            </Link>
          ) : null}
        </div>
        <p className="t-xs mt-2">
          Detected {formatRelativeTime(dependency.detectedAt)}
        </p>
        {dependency.advisories.length > 0 ? (
          <div className="mt-3 flex flex-wrap gap-2">
            {dependency.advisories.map((advisory) => (
              <Link
                className={severityClass(advisory)}
                href={advisory.href}
                key={advisory.identifier}
              >
                {advisory.identifier} · {advisory.severity}
              </Link>
            ))}
          </div>
        ) : null}
      </div>
      <div className="flex flex-wrap gap-2 md:justify-end">
        <Link
          aria-label={`${dependency.package.name} package details`}
          className="btn sm"
          href={dependency.detailsHref}
        >
          Details
        </Link>
        {dependency.advisoryHref ? (
          <Link
            aria-label={`${dependency.package.name} advisories`}
            className="btn sm"
            href={dependency.advisoryHref}
          >
            Advisories
          </Link>
        ) : null}
      </div>
    </article>
  );
}

function DependenciesReadyPage({
  dependencies,
  repository,
}: {
  dependencies: RepositoryDependenciesView;
  repository: RepositoryOverview;
}) {
  const owner = dependencies.repository.ownerLogin;
  const repo = dependencies.repository.name;
  const isEmpty = dependencies.dependencies.length === 0;

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
              Dependencies
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {dependencies.availability.message}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span
              className={
                dependencies.availability.indexed ? "chip ok" : "chip warn"
              }
            >
              {dependencies.availability.indexed ? "Indexed" : "Unindexed"}
            </span>
            <span
              className={dependencies.freshness.stale ? "chip warn" : "chip ok"}
            >
              {dependencies.freshness.stale ? "Stale graph" : "Fresh graph"}
            </span>
            <span className="chip soft">{dependencies.freshness.cadence}</span>
            <Link
              aria-disabled={!dependencies.export.supported}
              className={`btn ${dependencies.export.supported ? "primary" : ""}`}
              href={
                dependencies.export.supported
                  ? dependencies.links.exportSbomHref
                  : dependencies.links.dependenciesHref
              }
            >
              Export SBOM
            </Link>
          </div>
        </section>

        <DependencyTabs
          dependenciesHref={dependencies.links.dependenciesHref}
          dependentsHref={dependencies.links.dependentsHref}
          owner={owner}
          repo={repo}
        />

        <RepositoryDependencyFilters
          ecosystem={dependencies.filters.ecosystem}
          owner={owner}
          query={dependencies.filters.query}
          relationship={dependencies.filters.relationship}
          repo={repo}
          supportedEcosystems={dependencies.availability.supportedEcosystems}
        />

        <section
          aria-label="Dependency summary metrics"
          className="grid gap-4 md:grid-cols-4"
        >
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Dependencies
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependencies.summary.total)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Direct
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependencies.summary.directCount)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Transitive
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependencies.summary.transitiveCount)}
            </p>
          </article>
          <article className="card min-h-28 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Advisories
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(dependencies.summary.advisoryCount)}
            </p>
          </article>
        </section>

        <section
          aria-label="Dependency ecosystem totals"
          className="flex flex-wrap gap-2"
        >
          {dependencies.summary.ecosystemCounts.map((count) => (
            <Link
              className="chip soft"
              href={repositoryDependenciesHref(owner, repo, {
                ecosystem: count.ecosystem,
                query: dependencies.filters.query,
                relationship: dependencies.filters.relationship,
              })}
              key={count.ecosystem}
            >
              {count.ecosystem}{" "}
              <span className="t-num">{formatNumber(count.count)}</span>
            </Link>
          ))}
        </section>

        {isEmpty ? (
          <section className="card p-5">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Empty graph
            </p>
            <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              No matching dependencies were found.
            </h2>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Adjust the search or ecosystem filter, or browse the default
              branch to add a supported manifest file.
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <Link className="btn" href={dependencies.repository.treeHref}>
                Browse source tree
              </Link>
              <Link
                className="btn primary"
                href={repositoryDependenciesHref(owner, repo)}
              >
                Clear filters
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
                  Indexed packages
                </p>
                <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
                  {formatNumber(dependencies.summary.total)} dependencies
                </h2>
              </div>
              <Link className="btn" href={dependencies.links.dependentsHref}>
                View dependents
              </Link>
            </div>
            <ul
              aria-label="Repository dependencies list"
              className="m-0 list-none p-0"
            >
              {dependencies.dependencies.map((dependency) => (
                <li key={dependency.id}>
                  <DependencyRow dependency={dependency} />
                </li>
              ))}
            </ul>
          </section>
        )}

        <ManifestSummary manifests={dependencies.manifests} />

        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Freshness
          </p>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Graph computed{" "}
            {formatRelativeTime(dependencies.freshness.computedAt)}. It expires{" "}
            {formatRelativeTime(dependencies.freshness.expiresAt)}.
          </p>
        </section>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryDependencyGraphPage({
  dependenciesResult,
  repository,
}: RepositoryDependencyGraphPageProps) {
  if (!dependenciesResult.ok) {
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
            Dependencies unavailable
          </h1>
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            {dependenciesResult.message}
          </p>
          <Link
            className="btn primary w-fit"
            href={repositoryDependenciesHref(
              repository.owner_login,
              repository.name,
            )}
          >
            Retry dependencies
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return (
    <DependenciesReadyPage
      dependencies={dependenciesResult.dependencies}
      repository={repository}
    />
  );
}
