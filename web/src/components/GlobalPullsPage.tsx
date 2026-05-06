import Link from "next/link";
import type {
  ApiErrorEnvelope,
  GlobalPullRequestListQuery,
  GlobalPullRequestListView,
  GlobalPullRequestScope,
  PullRequestListItem,
  PullRequestSort,
  PullRequestState,
} from "@/lib/api";

type GlobalPullsPageProps = {
  pulls: GlobalPullRequestListView | ApiErrorEnvelope;
};

const SCOPES: {
  value: GlobalPullRequestScope;
  label: string;
  description: string;
}[] = [
  {
    value: "created",
    label: "Created",
    description: "Pull requests you opened.",
  },
  {
    value: "assigned",
    label: "Assigned",
    description: "Pull requests assigned to you.",
  },
  {
    value: "mentioned",
    label: "Mentioned",
    description: "Pull requests where you were mentioned.",
  },
  {
    value: "review_requests",
    label: "Review requests",
    description: "Pull requests awaiting your review.",
  },
];

const SORT_LABELS: Record<PullRequestSort, string> = {
  "best-match": "Best match",
  "updated-desc": "Recently updated",
  "updated-asc": "Least recently updated",
  "created-desc": "Newest",
  "created-asc": "Oldest",
  "comments-desc": "Most commented",
  "comments-asc": "Least commented",
  "reactions-desc": "Most reactions",
  "reactions-thumbs_up-desc": "Most +1",
  "reactions-thumbs_down-desc": "Most -1",
  "reactions-laugh-desc": "Most laugh",
  "reactions-hooray-desc": "Most hooray",
  "reactions-confused-desc": "Most confused",
  "reactions-heart-desc": "Most heart",
  "reactions-rocket-desc": "Most rocket",
  "reactions-eyes-desc": "Most eyes",
};

function hrefFor(query: GlobalPullRequestListQuery) {
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
  if (query.sort?.trim()) {
    params.set("sort", query.sort.trim());
  }
  if (query.page && query.page > 1) {
    params.set("page", String(query.page));
  }
  const suffix = params.size ? `?${params.toString()}` : "";
  return `/pulls${suffix}`;
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

function stateLabel(state: PullRequestState, draft: boolean) {
  if (draft) {
    return "Draft";
  }
  return state === "open" ? "Open" : state === "merged" ? "Merged" : "Closed";
}

function StateMark({ pull }: { pull: PullRequestListItem }) {
  const color =
    pull.state === "open"
      ? "var(--ok)"
      : pull.state === "merged"
        ? "var(--accent)"
        : "var(--ink-3)";
  const mark = pull.isDraft
    ? "◌"
    : pull.state === "open"
      ? "↟"
      : pull.state === "merged"
        ? "◆"
        : "×";
  return (
    <span
      aria-label={`${stateLabel(pull.state, pull.isDraft)} pull request`}
      className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border text-[11px]"
      role="img"
      style={{ borderColor: color, color }}
    >
      {mark}
    </span>
  );
}

function PullRow({ pull }: { pull: PullRequestListItem }) {
  return (
    <article className="list-row items-start gap-3 px-5 py-4">
      <StateMark pull={pull} />
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link
            className="font-medium hover:underline"
            href={pull.href}
            style={{ color: "var(--ink-1)" }}
          >
            {pull.title}
          </Link>
          <Link
            className="chip soft"
            href={`/${pull.repositoryOwner}/${pull.repositoryName}`}
          >
            {pull.repositoryOwner}/{pull.repositoryName}
          </Link>
          {pull.isDraft ? <span className="chip soft">Draft</span> : null}
          {pull.labels.slice(0, 4).map((label) => (
            <span className="chip soft" key={label.id}>
              <span
                aria-hidden="true"
                className="inline-block h-2 w-2 rounded-full"
                style={{ background: label.color }}
              />
              {label.name}
            </span>
          ))}
        </div>
        <p className="t-xs mt-1" style={{ color: "var(--ink-3)" }}>
          <span className="t-mono-sm">#{pull.number}</span> opened by{" "}
          {pull.author.login} · updated {relativeTime(pull.updatedAt)} ·{" "}
          <span className="t-mono-sm">{pull.headRef}</span> into{" "}
          <span className="t-mono-sm">{pull.baseRef}</span>
        </p>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <span className="chip soft">{pull.authorRole}</span>
          <Link className="chip soft" href={pull.reviewsHref}>
            {pull.review.requestedReviewers.length
              ? `${pull.review.requestedReviewers.length} reviewers`
              : pull.review.state.replaceAll("_", " ")}
          </Link>
          <Link
            className={`chip ${
              pull.checks.failedCount > 0
                ? "err"
                : pull.checks.totalCount &&
                    pull.checks.completedCount === pull.checks.totalCount
                  ? "ok"
                  : "soft"
            }`}
            href={pull.checksHref}
          >
            {pull.checks.totalCount
              ? `${pull.checks.completedCount}/${pull.checks.totalCount} checks`
              : "No checks"}
          </Link>
          {pull.milestone ? (
            <span className="chip soft">{pull.milestone.title}</span>
          ) : null}
        </div>
      </div>
      <Link
        className="t-xs flex min-w-10 shrink-0 items-center justify-end gap-1 pt-1 hover:underline"
        href={pull.commentsHref}
        style={{ color: "var(--ink-3)" }}
      >
        <span aria-hidden="true">□</span>
        <span className="t-num">{pull.commentCount}</span>
      </Link>
    </article>
  );
}

export function GlobalPullsPage({ pulls }: GlobalPullsPageProps) {
  if ("error" in pulls) {
    return (
      <section className="mx-auto max-w-[1240px] px-6 py-8">
        <div className="card p-6">
          <p className="t-label" style={{ color: "var(--err)" }}>
            Pull requests unavailable
          </p>
          <h1 className="t-h2 mt-1">Review queue needs a signed-in session</h1>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            {pulls.error.message}
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            <Link className="btn accent" href="/login?next=/pulls">
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

  const filters = pulls.filters;
  const baseQuery: GlobalPullRequestListQuery = {
    scope: filters.scope,
    q: filters.query,
    state: filters.state,
    repo: filters.repository,
    labels: filters.labels,
    milestone: filters.milestone,
    sort: filters.sort,
  };
  const firstItem = pulls.total ? (pulls.page - 1) * pulls.pageSize + 1 : 0;
  const lastItem = Math.min(pulls.total, pulls.page * pulls.pageSize);

  return (
    <section className="mx-auto max-w-[1240px] space-y-4 px-6 py-8">
      <div className="card p-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Review queue
            </p>
            <h1 className="t-h1 mt-1">Pull requests</h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Track proposed changes you opened, own, were mentioned in, or need
              to review across every repository you can read.
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
          htmlFor="global-pull-query"
        >
          <span aria-hidden="true">⌕</span>
          <input
            aria-label="pull-query"
            defaultValue={filters.query}
            id="global-pull-query"
            name="q"
            placeholder="is:pr is:open repo:owner/name"
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
          <option value="merged">Merged</option>
        </select>
        <select
          aria-label="Repository"
          className="input h-10 max-w-[220px]"
          defaultValue={filters.repository ?? ""}
          name="repo"
        >
          <option value="">All repositories</option>
          {pulls.filterOptions.repositories.map((repo) => (
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
          {pulls.filterOptions.labels.map((label) => (
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
          {pulls.filterOptions.milestones.map((milestone) => (
            <option key={milestone.id} value={milestone.title}>
              {milestone.title}
            </option>
          ))}
        </select>
        <select
          aria-label="Sort"
          className="input h-10 max-w-[180px]"
          defaultValue={filters.sort}
          name="sort"
        >
          {pulls.filterOptions.sortOptions.map((sort) => (
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
          <nav aria-label="Pull request queue" className="tabs">
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
                <span className="badge t-num">{pulls.counts[scope.value]}</span>
              </Link>
            ))}
          </nav>
          <p className="t-xs py-3" style={{ color: "var(--ink-3)" }}>
            {firstItem}-{lastItem} of{" "}
            <span className="t-num">{pulls.total}</span>
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
          </div>
        </div>

        {pulls.items.length ? (
          pulls.items.map((pull) => <PullRow key={pull.id} pull={pull} />)
        ) : (
          <div className="px-6 py-14 text-center">
            <p className="t-h3">No pull requests matched this queue</p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Try another tab, clear filters, or open a pull request from a
              repository branch.
            </p>
            <div className="mt-4 flex justify-center">
              <Link className="btn" href="/pulls">
                Clear filters
              </Link>
            </div>
          </div>
        )}
      </div>

      {pulls.total > pulls.pageSize ? (
        <nav
          aria-label="Pull request pagination"
          className="flex flex-wrap justify-end gap-2"
        >
          <Link
            aria-disabled={pulls.page <= 1}
            className="btn sm"
            href={hrefFor({ ...baseQuery, page: Math.max(1, pulls.page - 1) })}
          >
            Previous
          </Link>
          <Link
            aria-disabled={pulls.page * pulls.pageSize >= pulls.total}
            className="btn sm"
            href={hrefFor({ ...baseQuery, page: pulls.page + 1 })}
          >
            Next
          </Link>
        </nav>
      ) : null}
    </section>
  );
}
