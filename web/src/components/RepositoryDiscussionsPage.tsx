import Link from "next/link";
import { RepositoryDiscussionFilters } from "@/components/RepositoryDiscussionFilters";
import { RepositoryDiscussionVoteButton } from "@/components/RepositoryDiscussionVoteButton";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  DiscussionAuthorSummary,
  DiscussionCategorySummary,
  DiscussionRow,
  RepositoryDiscussionsView,
  RepositoryOverview,
} from "@/lib/api";
import {
  type RepositoryDiscussionHrefQuery,
  repositoryDiscussionCategoryHref,
  repositoryDiscussionDetailHref,
  repositoryDiscussionsHref,
  repositoryNewDiscussionHref,
} from "@/lib/navigation";

type RepositoryDiscussionsPageProps = {
  repository: RepositoryOverview;
  discussions: RepositoryDiscussionsView;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) return "recently";
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  return `${Math.floor(months / 12)}y ago`;
}

function Avatar({ user }: { user: DiscussionAuthorSummary }) {
  const label = user.displayName || user.login;
  return (
    <span className="av sm" title={label}>
      {user.login.slice(0, 1).toUpperCase()}
    </span>
  );
}

function CategoryChip({ category }: { category: DiscussionCategorySummary }) {
  return (
    <Link className="chip soft" href={category.href}>
      <span aria-hidden="true">{category.emoji}</span>
      {category.name}
    </Link>
  );
}

function PollSummaryChip({ discussion }: { discussion: DiscussionRow }) {
  if (!discussion.pollSummary) return null;
  return (
    <span
      className="chip accent"
      title={
        discussion.pollSummary.allowsMultiple
          ? "Poll allows multiple choices"
          : "Poll allows one choice"
      }
    >
      <span aria-hidden="true">📊</span>
      {discussion.categoryQualifier ?? "Poll"}
    </span>
  );
}

function discussionListHref(
  owner: string,
  repo: string,
  query: RepositoryDiscussionHrefQuery = {},
  categorySlug?: string | null,
) {
  return categorySlug
    ? repositoryDiscussionCategoryHref(owner, repo, categorySlug, query)
    : repositoryDiscussionsHref(owner, repo, query);
}

function DiscussionRowItem({
  discussion,
  discussions,
  owner,
  repo,
}: {
  discussion: DiscussionRow;
  discussions: RepositoryDiscussionsView;
  owner: string;
  repo: string;
}) {
  const href =
    discussion.href ||
    repositoryDiscussionDetailHref(owner, repo, discussion.number);

  return (
    <article className="list-row grid gap-3 px-5 py-4 md:grid-cols-[auto_minmax(0,1fr)_auto] md:items-start">
      <div className="flex items-center gap-2 md:flex-col md:items-stretch">
        <span
          aria-label={
            discussion.state === "open"
              ? "Open discussion"
              : "Closed discussion"
          }
          className={discussion.state === "open" ? "chip ok" : "chip soft"}
          role="img"
        >
          {discussion.state === "open" ? "Open" : "Closed"}
        </span>
        <RepositoryDiscussionVoteButton
          authenticated={discussions.viewer.authenticated}
          canVote={discussions.viewer.canVote}
          discussionNumber={discussion.number}
          initialViewerVoted={discussion.viewerVoted}
          initialVotesCount={discussion.votesCount}
          owner={owner}
          repo={repo}
        />
      </div>
      <div className="min-w-0">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          {discussion.pinned ? (
            <span className="chip accent">Pinned</span>
          ) : null}
          {discussion.answered ? (
            <span className="chip ok">Answered</span>
          ) : null}
          {discussion.locked ? <span className="chip warn">Locked</span> : null}
          <PollSummaryChip discussion={discussion} />
          <CategoryChip category={discussion.category} />
          {discussion.labels.map((label) => (
            <span
              className="chip soft"
              key={label.id}
              title={label.description ?? label.name}
            >
              {label.name}
            </span>
          ))}
        </div>
        <Link
          className="mt-2 block break-words t-h3 hover:underline"
          href={href}
          style={{ color: "var(--ink-1)" }}
        >
          {discussion.title}
        </Link>
        <p className="t-xs mt-2 break-words" style={{ color: "var(--ink-3)" }}>
          <span className="t-mono-sm">#{discussion.number}</span> opened by{" "}
          {discussion.author.login} · updated{" "}
          {relativeTime(discussion.lastActivityAt)}
        </p>
        {discussion.pollSummary ? (
          <div className="mt-3 rounded-[var(--radius)] border border-[var(--line-soft)] bg-[var(--surface-2)] p-3">
            <p className="t-sm break-words" style={{ color: "var(--ink-1)" }}>
              {discussion.pollSummary.question}
            </p>
            <p className="t-xs mt-1" style={{ color: "var(--ink-3)" }}>
              <span className="t-num">
                {formatNumber(discussion.pollSummary.optionCount)}
              </span>{" "}
              options ·{" "}
              {discussion.pollSummary.allowsMultiple
                ? "multiple choice"
                : "single choice"}{" "}
              ·{" "}
              {discussion.viewerCanVote
                ? "voting available"
                : (discussion.pollUnavailableReasons?.[0] ??
                  "voting unavailable")}
            </p>
          </div>
        ) : null}
      </div>
      <div className="flex items-center gap-3 md:justify-end">
        <Avatar user={discussion.author} />
        <span
          className="t-xs flex min-w-12 items-center justify-end gap-1"
          style={{ color: "var(--ink-3)" }}
        >
          <span aria-hidden="true">□</span>
          <span className="t-num">
            {formatNumber(discussion.commentsCount)}
          </span>
        </span>
        {discussion.pollSummary ? (
          <span
            className="t-xs flex min-w-12 items-center justify-end gap-1"
            style={{ color: "var(--ink-3)" }}
          >
            <span aria-hidden="true">📊</span>
            <span className="t-num">
              {formatNumber(discussion.pollSummary.totalVotes)}
            </span>
          </span>
        ) : null}
      </div>
    </article>
  );
}

function PinnedDiscussions({
  owner,
  repo,
  discussions,
}: {
  owner: string;
  repo: string;
  discussions: RepositoryDiscussionsView;
}) {
  if (!discussions.pinned.length) return null;
  return (
    <section
      aria-label="Pinned discussions"
      className="grid gap-3 md:grid-cols-2"
    >
      {discussions.pinned.map(({ discussion, position }) => (
        <article className="card min-w-0 p-4" key={discussion.id}>
          <div className="flex flex-wrap items-center gap-2">
            <span className="chip accent">Pinned {position}</span>
            <CategoryChip category={discussion.category} />
            {discussion.answered ? (
              <span className="chip ok">Answered</span>
            ) : null}
          </div>
          <Link
            className="mt-3 block break-words t-h3 hover:underline"
            href={
              discussion.href ||
              repositoryDiscussionDetailHref(owner, repo, discussion.number)
            }
            style={{ color: "var(--ink-1)" }}
          >
            {discussion.title}
          </Link>
          <p className="t-xs mt-2" style={{ color: "var(--ink-3)" }}>
            {formatNumber(discussion.votesCount)} votes ·{" "}
            {formatNumber(discussion.commentsCount)} comments · updated{" "}
            {relativeTime(discussion.lastActivityAt)}
          </p>
        </article>
      ))}
    </section>
  );
}

function CategoryRail({
  owner,
  repo,
  discussions,
}: {
  owner: string;
  repo: string;
  discussions: RepositoryDiscussionsView;
}) {
  const selectedCategory = discussions.filters.category;

  return (
    <aside className="space-y-4">
      <section className="card p-4">
        <div className="between gap-3">
          <h2 className="t-h3">Categories</h2>
          <Link
            className="t-xs hover:underline"
            href={repositoryDiscussionsHref(owner, repo, {
              q: discussions.filters.query,
              label: discussions.filters.label,
              state: discussions.filters.state,
              answered: discussions.filters.answered,
              locked: discussions.filters.locked,
              pinned: discussions.filters.pinned,
              sort: discussions.filters.sort,
            })}
          >
            All
          </Link>
        </div>
        <nav aria-label="Discussion categories" className="mt-3 grid gap-1">
          {discussions.categories.map((category) => (
            <Link
              aria-current={category.active ? "page" : undefined}
              className="flex min-w-0 items-center justify-between gap-3 rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryDiscussionCategoryHref(
                owner,
                repo,
                category.slug,
                {
                  q: discussions.filters.query,
                  label: discussions.filters.label,
                  state: discussions.filters.state,
                  answered: discussions.filters.answered,
                  locked: discussions.filters.locked,
                  pinned: discussions.filters.pinned,
                  sort: discussions.filters.sort,
                },
              )}
              key={category.id}
              style={
                category.active
                  ? { background: "var(--surface-2)", color: "var(--ink-1)" }
                  : { color: "var(--ink-2)" }
              }
            >
              <span className="min-w-0 truncate">
                <span aria-hidden="true">{category.emoji}</span> {category.name}
              </span>
              <span className="t-xs shrink-0">
                <span className="t-num">
                  {formatNumber(category.openCount)}
                </span>
                {selectedCategory === category.slug ? (
                  <span className="sr-only"> active category</span>
                ) : null}
              </span>
            </Link>
          ))}
        </nav>
      </section>

      <section className="card p-4">
        <h2 className="t-h3">Most helpful</h2>
        <p className="t-xs mt-1" style={{ color: "var(--ink-3)" }}>
          Last 30 days
        </p>
        <div className="mt-3 grid gap-3">
          {discussions.helpfulContributors.length ? (
            discussions.helpfulContributors.map((contributor) => (
              <div
                className="flex items-center gap-3"
                key={contributor.user.login}
              >
                <Avatar user={contributor.user} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">
                    {contributor.user.login}
                  </p>
                  <p className="t-xs">
                    {formatNumber(contributor.helpfulCount)} helpful ·{" "}
                    {formatNumber(contributor.commentsCount)} comments
                  </p>
                </div>
              </div>
            ))
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No helpful contributors yet.
            </p>
          )}
        </div>
      </section>

      <section className="card p-4">
        <h2 className="t-h3">Community</h2>
        <div className="mt-3 grid gap-2">
          {discussions.communityLinks.length ? (
            discussions.communityLinks.map((link) => (
              <Link
                className="t-sm hover:underline"
                href={link.href}
                key={link.id}
              >
                {link.label}
              </Link>
            ))
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              Community links have not been published for this repository.
            </p>
          )}
        </div>
      </section>
    </aside>
  );
}

function EmptyState({
  owner,
  repo,
  discussions,
}: {
  owner: string;
  repo: string;
  discussions: RepositoryDiscussionsView;
}) {
  const activeCategory = discussions.categories.find(
    (category) => category.active,
  );
  const isPollCategory =
    activeCategory?.slug === "polls" ||
    discussions.filters.category === "polls" ||
    discussions.items.some((item) => item.pollSummary);
  return (
    <div className="grid justify-items-center gap-3 px-6 py-14 text-center">
      <span className="chip soft">
        {isPollCategory ? "No polls" : "No discussions"}
      </span>
      <h2 className="t-h2">
        {isPollCategory
          ? "No poll discussions match this view."
          : activeCategory
            ? `No ${activeCategory.name} discussions match this view.`
            : "No discussions match this view."}
      </h2>
      <p className="t-sm max-w-xl" style={{ color: "var(--ink-3)" }}>
        {isPollCategory
          ? "Adjust the query, pick another category, or start a poll for this repository decision."
          : "Adjust the query, pick a different category, or start the first thread for this part of the repository."}
      </p>
      {discussions.viewer.canCreate ? (
        <Link
          className="btn primary"
          href={repositoryNewDiscussionHref(
            owner,
            repo,
            isPollCategory ? "polls" : activeCategory?.slug,
          )}
        >
          {isPollCategory ? "Start poll" : "New discussion"}
        </Link>
      ) : null}
    </div>
  );
}

function Pagination({
  owner,
  repo,
  discussions,
}: {
  owner: string;
  repo: string;
  discussions: RepositoryDiscussionsView;
}) {
  const categorySlug = discussions.filters.category;

  return (
    <nav
      aria-label="Discussion pagination"
      className="between flex-wrap gap-3 px-5 py-4"
    >
      <span className="t-xs">
        Page <span className="t-num">{discussions.page}</span> ·{" "}
        <span className="t-num">{formatNumber(discussions.total)}</span> total
      </span>
      <div className="flex flex-wrap gap-2">
        <Link
          aria-disabled={discussions.page <= 1}
          className={discussions.page <= 1 ? "btn sm opacity-50" : "btn sm"}
          href={discussionListHref(
            owner,
            repo,
            {
              ...discussions.filters,
              page: Math.max(1, discussions.page - 1),
            },
            categorySlug,
          )}
        >
          Previous
        </Link>
        <Link
          aria-disabled={!discussions.hasNextPage}
          className={!discussions.hasNextPage ? "btn sm opacity-50" : "btn sm"}
          href={discussionListHref(
            owner,
            repo,
            {
              ...discussions.filters,
              page: discussions.page + 1,
            },
            categorySlug,
          )}
        >
          Next
        </Link>
      </div>
    </nav>
  );
}

export function RepositoryDiscussionsPage({
  repository,
  discussions,
}: RepositoryDiscussionsPageProps) {
  const owner = repository.owner_login;
  const repo = repository.name;
  const activeCategory = discussions.categories.find(
    (category) => category.active,
  );
  const categorySlug = activeCategory?.slug ?? discussions.filters.category;
  const isPollCategory =
    activeCategory?.slug === "polls" ||
    discussions.filters.category === "polls" ||
    discussions.items.some((item) => item.pollSummary);

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/discussions`}
      frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0 space-y-5">
        <section className="card p-4">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Collaboration
              </p>
              <h1 className="t-h2 mt-1 break-words">
                {isPollCategory
                  ? "📊 Polls"
                  : activeCategory
                    ? `${activeCategory.emoji} ${activeCategory.name}`
                    : "Discussions"}
              </h1>
              <p
                className="t-sm mt-2 max-w-2xl"
                style={{ color: "var(--ink-3)" }}
              >
                {isPollCategory
                  ? (activeCategory?.description ??
                    "Vote on options, compare repository decisions, and keep poll results beside the discussion timeline.")
                  : (activeCategory?.description ??
                    "Ask questions, share ideas, and keep repository conversations separate from issue tracking.")}
              </p>
              {activeCategory ? (
                <div className="mt-3 flex flex-wrap items-center gap-2">
                  <span className="chip accent">
                    {isPollCategory
                      ? "category:polls"
                      : `category:${activeCategory.slug}`}
                  </span>
                  <Link
                    className="chip soft hover:underline"
                    href={repositoryDiscussionsHref(owner, repo, {
                      q: discussions.filters.query,
                      label: discussions.filters.label,
                      state: discussions.filters.state,
                      answered: discussions.filters.answered,
                      locked: discussions.filters.locked,
                      pinned: discussions.filters.pinned,
                      sort: discussions.filters.sort,
                    })}
                  >
                    View all discussions
                  </Link>
                </div>
              ) : null}
            </div>
            <Link
              className={
                discussions.viewer.canCreate ? "btn accent" : "btn opacity-60"
              }
              href={repositoryNewDiscussionHref(
                owner,
                repo,
                isPollCategory ? "polls" : activeCategory?.slug,
              )}
            >
              {isPollCategory ? "Start poll" : "New discussion"}
            </Link>
          </div>
        </section>

        {!discussions.enabled ? (
          <section
            className="card p-4"
            style={{
              background: "var(--warn-soft)",
              borderColor: "var(--warn)",
            }}
          >
            <p className="t-label" style={{ color: "var(--warn)" }}>
              Discussions disabled
            </p>
            <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
              {discussions.disabledReason ??
                "Repository discussions are disabled by organization policy."}
            </p>
          </section>
        ) : null}

        <RepositoryDiscussionFilters
          categorySlug={categorySlug}
          filters={discussions.filters}
          labels={discussions.labels}
          owner={owner}
          repo={repo}
        />

        <PinnedDiscussions
          owner={owner}
          repo={repo}
          discussions={discussions}
        />

        <section className="card overflow-hidden">
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5"
            style={{ borderColor: "var(--line)" }}
          >
            <nav aria-label="Discussion state" className="tabs">
              {[
                ["open", "Open", discussions.openCount],
                ["closed", "Closed", discussions.closedCount],
                ["all", "All", discussions.total],
              ].map(([state, label, count]) => (
                <Link
                  aria-current={
                    discussions.filters.state === state ? "page" : undefined
                  }
                  className={
                    discussions.filters.state === state ? "tab active" : "tab"
                  }
                  href={discussionListHref(
                    owner,
                    repo,
                    {
                      ...discussions.filters,
                      state: state as string,
                      page: 1,
                    },
                    categorySlug,
                  )}
                  key={state}
                >
                  {label}{" "}
                  <span className="t-num">{formatNumber(count as number)}</span>
                </Link>
              ))}
            </nav>
            <p className="t-xs py-3" style={{ color: "var(--ink-3)" }}>
              Sort:{" "}
              <span className="t-mono-sm">{discussions.filters.sort}</span>
            </p>
          </div>
          {discussions.items.length ? (
            <ul aria-label="Repository discussions">
              {discussions.items.map((discussion) => (
                <li key={discussion.id}>
                  <DiscussionRowItem
                    discussion={discussion}
                    discussions={discussions}
                    owner={owner}
                    repo={repo}
                  />
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState owner={owner} repo={repo} discussions={discussions} />
          )}
          <Pagination owner={owner} repo={repo} discussions={discussions} />
        </section>
      </main>

      <CategoryRail owner={owner} repo={repo} discussions={discussions} />
    </RepositoryShell>
  );
}
