"use client";

import Link from "next/link";
import { RepositorySecurityAdvisoryFilters } from "@/components/RepositorySecurityAdvisoryFilters";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecurityAdvisoriesFetchResult,
  RepositorySecurityAdvisoriesView,
  RepositorySecurityAdvisoryRow,
} from "@/lib/api";
import { repositorySecurityAdvisoriesHref } from "@/lib/navigation";

type RepositorySecurityAdvisoriesPageProps = {
  repository: RepositoryOverview;
  advisoriesResult: RepositorySecurityAdvisoriesFetchResult;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function formatDate(value: string | null) {
  if (!value) return "Not published";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function severityClass(severity: string) {
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "moderate") return "chip warn";
  if (severity === "low") return "chip info";
  return "chip soft";
}

function stateClass(state: string) {
  if (state === "published") return "chip ok";
  if (state === "draft") return "chip warn";
  if (state === "withdrawn") return "chip err";
  return "chip soft";
}

function AdvisoryTabs({
  owner,
  repo,
  view,
}: {
  owner: string;
  repo: string;
  view: RepositorySecurityAdvisoriesView;
}) {
  const current = view.filters.state;
  const tabs = [
    { state: "published", label: "Published", count: view.counts.published },
    { state: "draft", label: "Draft", count: view.counts.draft },
    { state: "withdrawn", label: "Withdrawn", count: view.counts.withdrawn },
  ].filter((tab) => tab.count !== null);

  return (
    <nav aria-label="Security advisory states" className="tabs">
      {tabs.map((tab) => (
        <Link
          aria-current={current === tab.state ? "page" : undefined}
          className={current === tab.state ? "tab active" : "tab"}
          href={repositorySecurityAdvisoriesHref(owner, repo, {
            ...view.filters,
            state: tab.state,
            page: 1,
          })}
          key={tab.state}
        >
          {tab.label}{" "}
          <span className="t-num">{formatNumber(tab.count ?? 0)}</span>
        </Link>
      ))}
    </nav>
  );
}

function PackageSummary({
  advisory,
}: {
  advisory: RepositorySecurityAdvisoryRow;
}) {
  if (!advisory.package?.name) {
    return <span className="chip soft">No package metadata</span>;
  }
  return (
    <span className="chip soft">
      <span className="t-mono-sm">
        {advisory.package.ecosystem ?? "package"}:{advisory.package.name}
      </span>
    </span>
  );
}

function AdvisoryRow({
  advisory,
}: {
  advisory: RepositorySecurityAdvisoryRow;
}) {
  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <span className={stateClass(advisory.state)}>{advisory.state}</span>
          <span className={severityClass(advisory.severity)}>
            {advisory.severity}
          </span>
          <span className="chip soft t-mono-sm">{advisory.ghsaId}</span>
          {advisory.cveId ? (
            <span className="chip soft t-mono-sm">{advisory.cveId}</span>
          ) : null}
          <PackageSummary advisory={advisory} />
        </div>
        <Link
          className="mt-2 block break-words t-h3 hover:underline"
          href={advisory.href}
          style={{ color: "var(--ink-1)" }}
        >
          {advisory.title}
        </Link>
        <p className="t-sm mt-1 break-words" style={{ color: "var(--ink-3)" }}>
          {advisory.summary}
        </p>
        <p className="t-xs mt-2">
          {advisory.state === "published"
            ? `Published ${formatDate(advisory.publishedAt)}`
            : `Updated ${formatDate(advisory.updatedAt)}`}
          {advisory.author ? ` by ${advisory.author.login}` : ""}
        </p>
      </div>
      <div className="flex flex-wrap items-start gap-2 md:justify-end">
        {advisory.author ? (
          <Link className="chip soft" href={advisory.author.profileHref}>
            {advisory.author.login}
          </Link>
        ) : null}
        <Link className="btn sm" href={advisory.href}>
          View advisory
        </Link>
      </div>
    </article>
  );
}

function Pagination({
  owner,
  repo,
  view,
}: {
  owner: string;
  repo: string;
  view: RepositorySecurityAdvisoriesView;
}) {
  const previousPage = Math.max(1, view.filters.page - 1);
  const nextPage = view.filters.page + 1;
  return (
    <nav aria-label="Advisory pagination" className="between flex-wrap gap-2">
      <span className="t-xs">
        Page <span className="t-num">{view.filters.page}</span> of{" "}
        <span className="t-num">
          {Math.max(1, Math.ceil(view.filters.total / view.filters.pageSize))}
        </span>
      </span>
      <div className="flex flex-wrap gap-2">
        <Link
          aria-disabled={view.filters.page <= 1}
          className={view.filters.page <= 1 ? "btn sm opacity-50" : "btn sm"}
          href={repositorySecurityAdvisoriesHref(owner, repo, {
            ...view.filters,
            page: previousPage,
          })}
        >
          Previous
        </Link>
        <Link
          aria-disabled={!view.filters.hasNextPage}
          className={!view.filters.hasNextPage ? "btn sm opacity-50" : "btn sm"}
          href={repositorySecurityAdvisoriesHref(owner, repo, {
            ...view.filters,
            page: nextPage,
          })}
        >
          Next
        </Link>
      </div>
    </nav>
  );
}

function AdvisoriesReadyPage({
  repository,
  view,
}: {
  repository: RepositoryOverview;
  view: RepositorySecurityAdvisoriesView;
}) {
  const owner = view.repository.ownerLogin;
  const repo = view.repository.name;
  return (
    <RepositorySecurityShell activeSection="advisories" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Security advisories
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Create, publish, and review repository-owned vulnerability
              disclosures with package, CVE, CVSS, CWE, credit, and collaborator
              metadata.
            </p>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            {view.links.newHref ? (
              <Link className="btn primary" href={view.links.newHref}>
                New draft security advisory
              </Link>
            ) : null}
            <Link className="btn" href={view.repository.securityHref}>
              Security overview
            </Link>
          </div>
        </section>

        <AdvisoryTabs owner={owner} repo={repo} view={view} />
        <RepositorySecurityAdvisoryFilters
          filters={view.filters}
          owner={owner}
          repo={repo}
        />

        <section
          aria-label="Security advisory summary"
          className="grid gap-4 md:grid-cols-3"
        >
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Published
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.published)}
            </p>
          </article>
          {view.counts.draft !== null ? (
            <article className="card min-h-24 p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Draft
              </p>
              <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
                {formatNumber(view.counts.draft)}
              </p>
            </article>
          ) : null}
          {view.counts.withdrawn !== null ? (
            <article className="card min-h-24 p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Withdrawn
              </p>
              <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
                {formatNumber(view.counts.withdrawn)}
              </p>
            </article>
          ) : null}
        </section>

        <section className="card overflow-hidden">
          <div
            className="between flex-wrap gap-3 px-4 py-3"
            style={{ borderBottom: "1px solid var(--line)" }}
          >
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Advisory queue
              </p>
              <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
                {formatNumber(view.filters.total)} matching advisories
              </h2>
            </div>
            <span className="chip soft">
              <span className="t-num">{view.advisories.length}</span> visible
            </span>
          </div>
          {view.advisories.length === 0 ? (
            <div className="grid gap-3 p-5">
              <h3 className="t-h2" style={{ color: "var(--ink-1)" }}>
                No matching advisories.
              </h3>
              <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
                Published advisories appear here for readers. Maintainers can
                clear filters or start a draft disclosure when a vulnerability
                needs private coordination.
              </p>
              <div className="flex flex-wrap gap-2">
                <Link className="btn primary" href={view.links.listHref}>
                  Clear filters
                </Link>
                {view.links.newHref ? (
                  <Link className="btn" href={view.links.newHref}>
                    Start draft
                  </Link>
                ) : null}
              </div>
            </div>
          ) : (
            <ul aria-label="Security advisories" className="m-0 list-none p-0">
              {view.advisories.map((advisory) => (
                <li key={advisory.id}>
                  <AdvisoryRow advisory={advisory} />
                </li>
              ))}
            </ul>
          )}
        </section>

        <Pagination owner={owner} repo={repo} view={view} />
      </div>
    </RepositorySecurityShell>
  );
}

function AdvisoriesUnavailablePage({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositorySecurityAdvisoriesFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="advisories" repository={repository}>
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Security advisories
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Security advisories unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security`}
        >
          Back to security overview
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

export function RepositorySecurityAdvisoriesPage({
  repository,
  advisoriesResult,
}: RepositorySecurityAdvisoriesPageProps) {
  if (!advisoriesResult.ok) {
    return (
      <AdvisoriesUnavailablePage
        repository={repository}
        result={advisoriesResult}
      />
    );
  }

  return (
    <AdvisoriesReadyPage
      repository={repository}
      view={advisoriesResult.advisories}
    />
  );
}
