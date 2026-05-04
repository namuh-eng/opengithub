import Link from "next/link";
import { BranchCopyButton } from "@/components/BranchCopyButton";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryBranchDirectoryRow,
  RepositoryBranchesFetchResult,
  RepositoryBranchesView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryBranchActivityHref,
  repositoryBranchesHref,
  repositoryBranchRulesHref,
  repositoryCommitHistoryHref,
  repositoryTreeAtBranchHref,
} from "@/lib/navigation";

type RepositoryBranchesPageProps = {
  repository: RepositoryOverview;
  branchesResult: RepositoryBranchesFetchResult;
};

const BRANCH_TABS = [
  { value: "overview", label: "Overview" },
  { value: "active", label: "Active" },
  { value: "stale", label: "Stale" },
  { value: "all", label: "All" },
] as const;

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

function statusLabel(branch: RepositoryBranchDirectoryRow) {
  const checks = branch.checks;
  if (checks.totalCount === 0) {
    return "No checks";
  }
  if (checks.status === "running") {
    return `${checks.completedCount}/${checks.totalCount} running`;
  }
  if (checks.conclusion === "success") {
    return `${checks.totalCount} passed`;
  }
  if (checks.failedCount > 0 || checks.conclusion === "failure") {
    return `${checks.failedCount || 1} failed`;
  }
  return `${checks.completedCount}/${checks.totalCount} complete`;
}

function statusChipClass(branch: RepositoryBranchDirectoryRow) {
  const checks = branch.checks;
  if (checks.totalCount === 0) {
    return "chip soft";
  }
  if (checks.conclusion === "success") {
    return "chip ok";
  }
  if (checks.failedCount > 0 || checks.conclusion === "failure") {
    return "chip err";
  }
  if (checks.status === "running") {
    return "chip accent";
  }
  return "chip warn";
}

function BranchRow({
  branch,
  owner,
  repo,
}: {
  branch: RepositoryBranchDirectoryRow;
  owner: string;
  repo: string;
}) {
  const treeHref =
    branch.href ||
    repositoryTreeAtBranchHref({ owner, repo, branch: branch.name });
  const commitsHref =
    branch.commitsHref ||
    repositoryCommitHistoryHref({ owner, repo, refName: branch.name });
  const activityHref =
    branch.activityHref ||
    repositoryBranchActivityHref({ owner, repo, branch: branch.name });
  const rulesHref =
    branch.protection.href ||
    repositoryBranchRulesHref({ owner, repo, branch: branch.name });
  const latest = branch.latestCommit;

  return (
    <article className="list-row grid gap-4 px-4 py-4 lg:grid-cols-[minmax(220px,1.5fr)_minmax(220px,1.2fr)_120px_120px_130px_auto] lg:items-center">
      <div className="min-w-0">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link
            className="t-mono-sm truncate font-semibold hover:underline"
            href={treeHref}
            style={{ color: "var(--ink-1)" }}
          >
            {branch.name}
          </Link>
          {branch.isDefault ? (
            <span className="chip active">Default</span>
          ) : null}
          {branch.protection.protected ? (
            <Link className="chip warn" href={rulesHref}>
              Protected
            </Link>
          ) : (
            <span className="chip soft">Open</span>
          )}
        </div>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <BranchCopyButton branch={branch.name} />
          <Link className="btn sm ghost" href={commitsHref}>
            Commits
          </Link>
          {branch.capabilities.canViewActivity ? (
            <Link className="btn sm ghost" href={activityHref}>
              Activity
            </Link>
          ) : null}
        </div>
      </div>

      <div className="min-w-0">
        {latest ? (
          <div className="flex min-w-0 gap-2">
            <span
              aria-hidden="true"
              className="av sm shrink-0"
              title={latest.authorLogin ?? "Unknown author"}
            >
              {initials(latest.authorLogin)}
            </span>
            <div className="min-w-0">
              <Link
                className="block truncate t-sm font-medium hover:underline"
                href={latest.href}
                style={{ color: "var(--ink-1)" }}
              >
                {latest.subject}
              </Link>
              <p className="t-xs">
                {latest.authorLogin ?? "Unknown"} updated{" "}
                {formatRelativeTime(latest.committedAt)}
              </p>
            </div>
          </div>
        ) : (
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No commit indexed yet
          </p>
        )}
      </div>

      <Link className={statusChipClass(branch)} href={branch.checks.href}>
        {statusLabel(branch)}
      </Link>
      <div className="flex gap-2 t-mono-sm" style={{ color: "var(--ink-2)" }}>
        <span>{branch.behind} behind</span>
        <span>{branch.ahead} ahead</span>
      </div>
      <div>
        {branch.pullRequest ? (
          <Link
            className={branch.pullRequest.draft ? "chip warn" : "chip soft"}
            href={branch.pullRequest.href}
            title={branch.pullRequest.title}
          >
            {branch.pullRequest.draft ? "Draft " : ""}#
            {branch.pullRequest.number}
          </Link>
        ) : (
          <span className="t-xs">No pull request</span>
        )}
      </div>
      <details className="relative justify-self-start lg:justify-self-end">
        <summary className="btn sm ghost cursor-pointer list-none">
          Actions
        </summary>
        <div
          className="card absolute right-0 z-10 mt-2 grid min-w-48 gap-1 p-2"
          style={{ background: "var(--surface)" }}
        >
          <Link className="btn sm ghost justify-start" href={activityHref}>
            Activity
          </Link>
          {branch.capabilities.canViewRules ? (
            <Link className="btn sm ghost justify-start" href={rulesHref}>
              View rules
            </Link>
          ) : (
            <span className="chip soft justify-start">No visible rules</span>
          )}
          <Link className="btn sm ghost justify-start" href={treeHref}>
            Open tree
          </Link>
          <Link className="btn sm ghost justify-start" href={commitsHref}>
            Open commits
          </Link>
          {branch.capabilities.deleteDisabledReason ? (
            <span className="t-xs px-2 py-1">
              {branch.capabilities.deleteDisabledReason}
            </span>
          ) : null}
        </div>
      </details>
    </article>
  );
}

function BranchSection({
  title,
  description,
  branches,
  owner,
  repo,
}: {
  title: string;
  description: string;
  branches: RepositoryBranchDirectoryRow[];
  owner: string;
  repo: string;
}) {
  return (
    <section className="card overflow-visible">
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
      {branches.length > 0 ? (
        <div>
          {branches.map((branch) => (
            <BranchRow
              branch={branch}
              key={branch.qualifiedName}
              owner={owner}
              repo={repo}
            />
          ))}
        </div>
      ) : (
        <div className="px-4 py-6">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            No branches to show in this section.
          </p>
        </div>
      )}
    </section>
  );
}

function BranchesReadyPage({
  repository,
  view,
}: {
  repository: RepositoryOverview;
  view: RepositoryBranchesView;
}) {
  const owner = view.repository.ownerLogin;
  const repo = view.repository.name;
  const filters = {
    tab: view.filters.tab,
    query: view.filters.query,
    page: view.page,
    pageSize: view.pageSize,
  };

  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/branches`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Branch directory
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Branches
            </h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Review default, active, and stale branches for {owner}/{repo}.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip soft">{view.tabs.all} total</span>
            <span className="chip soft">{view.tabs.active} active</span>
            <span className="chip soft">{view.tabs.stale} stale</span>
          </div>
        </section>

        <section className="card p-4">
          <div className="tabs flex gap-2 overflow-x-auto" role="tablist">
            {BRANCH_TABS.map((tab) => {
              const active = view.filters.tab === tab.value;
              const count = view.tabs[tab.value];
              return (
                <Link
                  aria-label={`${tab.label} ${count}`}
                  aria-current={active ? "page" : undefined}
                  className={`tab ${active ? "active" : ""}`}
                  href={repositoryBranchesHref(owner, repo, filters, {
                    tab: tab.value,
                    page: null,
                  })}
                  key={tab.value}
                  role="tab"
                >
                  {tab.label}
                  <span className="t-num ml-2">{count}</span>
                </Link>
              );
            })}
          </div>
          <form
            action={`/${owner}/${repo}/branches`}
            className="mt-4 flex flex-wrap gap-3"
          >
            {view.filters.tab !== "overview" ? (
              <input name="tab" type="hidden" value={view.filters.tab} />
            ) : null}
            <label className="sr-only" htmlFor="branch-search">
              Search branches
            </label>
            <input
              className="input min-w-0 flex-1"
              defaultValue={view.filters.query ?? ""}
              id="branch-search"
              name="q"
              placeholder="Search branches"
              type="search"
            />
            <button className="btn primary" type="submit">
              Search
            </button>
            {view.filters.query ? (
              <Link
                className="btn ghost"
                href={repositoryBranchesHref(owner, repo, filters, {
                  q: null,
                  page: null,
                })}
              >
                Clear
              </Link>
            ) : null}
          </form>
        </section>

        {view.filters.tab === "overview" ? (
          <>
            {view.defaultBranch ? (
              <BranchSection
                branches={[view.defaultBranch]}
                description="The branch new clones, pull requests, and release defaults resolve against."
                owner={owner}
                repo={repo}
                title="Default branch"
              />
            ) : null}
            <BranchSection
              branches={view.branches}
              description={`Branches updated in the last ${view.filters.staleCutoffDays} days.`}
              owner={owner}
              repo={repo}
              title="Active branches"
            />
          </>
        ) : view.branches.length > 0 ? (
          <BranchSection
            branches={view.branches}
            description="Filtered branch rows with their latest commit, checks, protection, and linked pull request."
            owner={owner}
            repo={repo}
            title={`${BRANCH_TABS.find((tab) => tab.value === view.filters.tab)?.label ?? "Branches"} branches`}
          />
        ) : (
          <section className="card p-6">
            <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
              {view.emptyState.title}
            </h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {view.emptyState.message}
            </p>
            <Link className="btn mt-4" href={view.emptyState.resetHref}>
              Reset branch filters
            </Link>
          </section>
        )}

        {view.hasPreviousPage || view.hasNextPage ? (
          <nav aria-label="Branch pagination" className="flex flex-wrap gap-3">
            {view.hasPreviousPage ? (
              <Link
                className="btn"
                href={repositoryBranchesHref(owner, repo, filters, {
                  page: String(Math.max(1, view.page - 1)),
                })}
              >
                Previous
              </Link>
            ) : null}
            {view.hasNextPage ? (
              <Link
                className="btn"
                href={repositoryBranchesHref(owner, repo, filters, {
                  page: String(view.page + 1),
                })}
              >
                Next
              </Link>
            ) : null}
          </nav>
        ) : null}
      </div>
    </RepositoryShell>
  );
}

export function RepositoryBranchesPage({
  repository,
  branchesResult,
}: RepositoryBranchesPageProps) {
  if (!branchesResult.ok) {
    return (
      <RepositoryShell
        activePath={`/${repository.owner_login}/${repository.name}/branches`}
        frameClassName="max-w-5xl"
        repository={repository}
      >
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Branch directory
          </p>
          <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Branches unavailable
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {branchesResult.message}
          </p>
          <Link
            className="btn mt-4"
            href={`/${repository.owner_login}/${repository.name}`}
          >
            Back to Code
          </Link>
        </section>
      </RepositoryShell>
    );
  }

  return (
    <BranchesReadyPage repository={repository} view={branchesResult.branches} />
  );
}
