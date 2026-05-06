import Link from "next/link";
import type {
  ApiErrorEnvelope,
  GlobalIssueListQuery,
  GlobalIssueListView,
  GlobalIssueScope,
  IssueListItem,
  IssueSort,
  IssueState,
} from "@/lib/api";

type GlobalIssuesPageProps = {
  issues: GlobalIssueListView | ApiErrorEnvelope;
};

const SCOPES: {
  value: GlobalIssueScope;
  label: string;
  description: string;
}[] = [
  {
    value: "created",
    label: "Created",
    description: "Issues you opened.",
  },
  {
    value: "assigned",
    label: "Assigned",
    description: "Issues assigned to you.",
  },
  {
    value: "mentioned",
    label: "Mentioned",
    description: "Issues where you were mentioned.",
  },
];

const SORT_LABELS: Record<IssueSort, string> = {
  "best-match": "Best match",
  "updated-desc": "Recently updated",
  "updated-asc": "Least recently updated",
  "created-desc": "Newest",
  "created-asc": "Oldest",
  "comments-desc": "Most commented",
  "comments-asc": "Least commented",
};

function hrefFor(query: GlobalIssueListQuery) {
  const params = new URLSearchParams();
  if (query.scope) {
    params.set("scope", query.scope);
  }
  if (query.q?.trim()) {
    params.set("q", query.q.trim());
  }
  if (query.state) {
    params.set("state", query.state);
  }
  const repo = query.repository ?? query.repo;
  if (repo?.trim()) {
    params.set("repo", repo.trim());
  }
  if (query.labels?.length) {
    params.set("labels", query.labels.join(","));
  }
  if (query.milestone?.trim()) {
    params.set("milestone", query.milestone.trim());
  }
  if (query.project?.trim()) {
    params.set("project", query.project.trim());
  }
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/issues${suffix}`;
}

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) {
    return "just now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  const days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days}d ago`;
  }
  return `${Math.floor(days / 30)}mo ago`;
}

function StateMark({ state }: { state: IssueState }) {
  const open = state === "open";
  return (
    <span
      aria-label={open ? "Open issue" : "Closed issue"}
      className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-[11px]"
      role="img"
      style={{
        borderColor: open ? "var(--ok)" : "var(--ink-4)",
        color: open ? "var(--ok)" : "var(--ink-3)",
      }}
    >
      {open ? "!" : "✓"}
    </span>
  );
}

function IssueRow({ issue }: { issue: IssueListItem }) {
  return (
    <article className="list-row items-start gap-3 px-5 py-4">
      <StateMark state={issue.state} />
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link
            className="font-medium hover:underline"
            href={issue.href}
            style={{ color: "var(--ink-1)" }}
          >
            {issue.title}
          </Link>
          <Link
            className="chip soft"
            href={`/${issue.repositoryOwner}/${issue.repositoryName}`}
          >
            {issue.repositoryOwner}/{issue.repositoryName}
          </Link>
          {issue.locked ? <span className="chip warn">Locked</span> : null}
          {issue.labels.slice(0, 4).map((label) => (
            <span
              className="chip soft"
              key={label.id}
              title={label.description ?? label.name}
            >
              <span
                aria-hidden="true"
                className="inline-block h-2 w-2 rounded-full"
                style={{ background: label.color }}
              />
              {label.name}
            </span>
          ))}
          {issue.milestone ? (
            <span className="chip soft">{issue.milestone.title}</span>
          ) : null}
          {issue.linkedPullRequest ? (
            <Link className="chip soft" href={issue.linkedPullRequest.href}>
              PR #{issue.linkedPullRequest.number}
            </Link>
          ) : null}
        </div>
        <p className="t-xs mt-1" style={{ color: "var(--ink-3)" }}>
          <span className="t-mono-sm">#{issue.number}</span> opened by{" "}
          {issue.author.login} · updated {relativeTime(issue.updatedAt)}
        </p>
        {issue.assignees.length ? (
          <div className="mt-2 flex flex-wrap gap-1">
            {issue.assignees.map((assignee) => (
              <span className="chip soft" key={assignee.id}>
                @{assignee.login}
              </span>
            ))}
          </div>
        ) : null}
      </div>
      <Link
        className="t-xs flex min-w-10 shrink-0 items-center justify-end gap-1 pt-1 hover:underline"
        href={`${issue.href}#comments`}
        style={{ color: "var(--ink-3)" }}
      >
        <span aria-hidden="true">□</span>
        <span className="t-num">{issue.commentCount}</span>
      </Link>
    </article>
  );
}

export function GlobalIssuesPage({ issues }: GlobalIssuesPageProps) {
  if ("error" in issues) {
    return (
      <section className="mx-auto max-w-[1240px] px-6 py-8">
        <div className="card p-6">
          <p className="t-label" style={{ color: "var(--err)" }}>
            Issues unavailable
          </p>
          <h1 className="t-h2 mt-1">Issue queue needs a signed-in session</h1>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            {issues.error.message}
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            <Link className="btn accent" href="/login?next=/issues">
              Sign in
            </Link>
            <Link className="btn" href="/dashboard">
              Dashboard
            </Link>
          </div>
        </div>
      </section>
    );
  }

  const filters = issues.filters;
  const baseQuery: GlobalIssueListQuery = {
    scope: filters.scope,
    q: filters.query,
    state: filters.state ?? undefined,
    repo: filters.repository,
    labels: filters.labels,
    milestone: filters.milestone,
    project: filters.project,
    sort: filters.sort,
  };
  const firstItem = issues.total ? (issues.page - 1) * issues.pageSize + 1 : 0;
  const lastItem = Math.min(issues.total, issues.page * issues.pageSize);

  return (
    <section className="mx-auto max-w-[1240px] space-y-4 px-6 py-8">
      <div className="card p-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Work queue
            </p>
            <h1 className="t-h1 mt-1">Issues</h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Track issues you opened, own, or were mentioned in across every
              repository you can read.
            </p>
          </div>
          <Link className="btn" href="/dashboard">
            Dashboard
          </Link>
        </div>
      </div>

      <form className="flex flex-wrap items-center gap-3" method="get">
        <input name="scope" type="hidden" value={filters.scope} />
        <label
          className="input min-w-[260px] flex-1"
          htmlFor="global-issue-query"
        >
          <span aria-hidden="true">⌕</span>
          <input
            aria-label="issue-query"
            defaultValue={filters.query}
            id="global-issue-query"
            name="q"
            placeholder="is:issue state:open repo:owner/name"
          />
        </label>
        <select
          aria-label="State"
          className="input h-10 max-w-[150px]"
          defaultValue={filters.state ?? ""}
          name="state"
        >
          <option value="">Any state</option>
          <option value="open">Open</option>
          <option value="closed">Closed</option>
        </select>
        <select
          aria-label="Repository"
          className="input h-10 max-w-[220px]"
          defaultValue={filters.repository ?? ""}
          name="repo"
        >
          <option value="">All repositories</option>
          {issues.filterOptions.repositories.map((repo) => (
            <option key={repo.id} value={repo.fullName}>
              {repo.fullName}
            </option>
          ))}
        </select>
        <select
          aria-label="Label"
          className="input h-10 max-w-[180px]"
          defaultValue={filters.labels[0] ?? ""}
          name="labels"
        >
          <option value="">Any label</option>
          {issues.filterOptions.labels.map((label) => (
            <option key={label.id} value={label.name}>
              {label.name}
            </option>
          ))}
        </select>
        <select
          aria-label="Milestone"
          className="input h-10 max-w-[190px]"
          defaultValue={filters.milestone ?? ""}
          name="milestone"
        >
          <option value="">Any milestone</option>
          {issues.filterOptions.milestones.map((milestone) => (
            <option key={milestone.id} value={milestone.title}>
              {milestone.title}
            </option>
          ))}
        </select>
        <select
          aria-label="Project"
          className="input h-10 max-w-[180px]"
          defaultValue={filters.project ?? ""}
          name="project"
        >
          <option value="">Any project</option>
          {issues.filterOptions.projects.map((project) => (
            <option key={project.id} value={project.name}>
              {project.name}
            </option>
          ))}
        </select>
        <select
          aria-label="Sort"
          className="input h-10 max-w-[180px]"
          defaultValue={filters.sort}
          name="sort"
        >
          {issues.filterOptions.sortOptions.map((sort) => (
            <option key={sort} value={sort}>
              {SORT_LABELS[sort] ?? sort}
            </option>
          ))}
        </select>
        <button className="btn" type="submit">
          Filter
        </button>
      </form>

      <div className="card overflow-hidden">
        <div
          className="flex flex-wrap items-center justify-between gap-3 border-b px-5"
          style={{ borderColor: "var(--line)" }}
        >
          <nav aria-label="Issue queue" className="tabs">
            {SCOPES.map((scope) => (
              <Link
                aria-current={
                  filters.scope === scope.value ? "page" : undefined
                }
                className={`tab ${filters.scope === scope.value ? "active" : ""}`}
                href={hrefFor({ ...baseQuery, scope: scope.value, page: 1 })}
                key={scope.value}
                title={scope.description}
              >
                {scope.label}
                <span className="badge t-num">
                  {issues.counts[scope.value]}
                </span>
              </Link>
            ))}
          </nav>
          <p className="t-xs py-3" style={{ color: "var(--ink-3)" }}>
            {firstItem}-{lastItem} of{" "}
            <span className="t-num">{issues.total}</span>
          </p>
          <div className="flex flex-wrap gap-2 py-3">
            {filters.repository ? (
              <Link
                className="chip soft"
                href={hrefFor({ ...baseQuery, repo: null })}
              >
                repo:{filters.repository}
              </Link>
            ) : null}
            {filters.labels.map((label) => (
              <Link
                className="chip soft"
                href={hrefFor({
                  ...baseQuery,
                  labels: filters.labels.filter((value) => value !== label),
                })}
                key={label}
              >
                label:{label}
              </Link>
            ))}
            {filters.milestone ? (
              <Link
                className="chip soft"
                href={hrefFor({ ...baseQuery, milestone: null })}
              >
                milestone:{filters.milestone}
              </Link>
            ) : null}
            {filters.project ? (
              <Link
                className="chip soft"
                href={hrefFor({ ...baseQuery, project: null })}
              >
                project:{filters.project}
              </Link>
            ) : null}
          </div>
        </div>

        {issues.items.length ? (
          issues.items.map((issue) => <IssueRow issue={issue} key={issue.id} />)
        ) : (
          <div className="px-6 py-14 text-center">
            <p className="t-h3">No issues matched this queue</p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Try another tab, clear filters, or open an issue in a readable
              repository.
            </p>
            <div className="mt-4 flex justify-center">
              <Link className="btn" href="/issues">
                Clear filters
              </Link>
            </div>
          </div>
        )}
      </div>

      {issues.total > issues.pageSize ? (
        <nav aria-label="Issue pagination" className="flex justify-end gap-2">
          <Link
            aria-disabled={issues.page <= 1}
            className="btn sm"
            href={hrefFor({ ...baseQuery, page: Math.max(1, issues.page - 1) })}
          >
            Previous
          </Link>
          <Link
            aria-disabled={issues.page * issues.pageSize >= issues.total}
            className="btn sm"
            href={hrefFor({ ...baseQuery, page: issues.page + 1 })}
          >
            Next
          </Link>
        </nav>
      ) : null}
    </section>
  );
}
