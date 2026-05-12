import Link from "next/link";
import type {
  ProfileRepositoryFilters,
  ProfileRepositoryList,
  ProfileRepositoryListItem,
} from "@/lib/api";
import {
  type ProfileRepositoryTabFilters,
  profileRepositoryTabHref,
} from "@/lib/navigation";

type ProfileRepositoryTabsProps = {
  list: ProfileRepositoryList;
  owner: string;
};

const REPOSITORY_SORT_LABELS: Record<string, string> = {
  "updated-desc": "Last updated",
  "name-asc": "Name",
  "stars-desc": "Stars",
};

const STAR_SORT_LABELS: Record<string, string> = {
  "recently-starred": "Recently starred",
  "recently-active": "Recently active",
  "most-stars": "Most stars",
};

const REPOSITORY_TYPE_LABELS: Record<string, string> = {
  all: "All",
  sources: "Sources",
  forks: "Forks",
  archived: "Archived",
  sponsorable: "Can be sponsored",
  mirrors: "Mirrors",
  templates: "Templates",
};

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Updated recently";
  }

  return `Updated ${new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(date)}`;
}

function formatStarredDate(value: string | null | undefined) {
  if (!value) {
    return null;
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Starred recently";
  }

  return `Starred ${new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(date)}`;
}

function repositoryBadges(repository: ProfileRepositoryListItem) {
  const badges: string[] = [];
  if (repository.visibility !== "public") {
    badges.push(repository.visibility);
  }
  if (repository.isArchived) {
    badges.push("archived");
  }
  if (repository.isFork) {
    badges.push("fork");
  }
  if (repository.isTemplate) {
    badges.push("template");
  }
  if (repository.isMirror) {
    badges.push("mirror");
  }
  if (repository.canBeSponsored) {
    badges.push("sponsorable");
  }
  return badges;
}

function LanguageDot({ color }: { color: string }) {
  return (
    <span
      aria-hidden="true"
      className="inline-block h-2 w-2 shrink-0 rounded-full"
      style={{ backgroundColor: color || "var(--ink-4)" }}
    />
  );
}

function RepositoryRow({
  repository,
  mode,
}: {
  repository: ProfileRepositoryListItem;
  mode: string;
}) {
  const badges = repositoryBadges(repository);
  const starredDate =
    mode === "stars" ? formatStarredDate(repository.starredAt) : null;

  return (
    <article className="list-row py-5">
      <div className="flex flex-wrap items-start gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <Link className="t-h3 no-underline" href={repository.href}>
              {repository.name}
            </Link>
            {badges.map((badge) => (
              <span className="chip soft" key={badge}>
                {badge}
              </span>
            ))}
          </div>
          <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
            {repository.fullName} · {repository.defaultBranch}
          </p>
        </div>
        <span className="t-xs shrink-0">
          {starredDate ?? formatDate(repository.updatedAt)}
        </span>
      </div>

      {repository.forkSource ? (
        <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
          Forked from{" "}
          <Link className="underline" href={repository.forkSource.href}>
            {repository.forkSource.owner}/{repository.forkSource.name}
          </Link>
        </p>
      ) : null}

      {repository.description ? (
        <p className="t-body mt-3 max-w-3xl" style={{ color: "var(--ink-2)" }}>
          {repository.description}
        </p>
      ) : null}

      <div className="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2">
        {repository.primaryLanguage ? (
          <span
            className="t-mono-sm inline-flex items-center gap-2"
            style={{ color: "var(--ink-3)" }}
          >
            <LanguageDot color={repository.primaryLanguage.color} />
            {repository.primaryLanguage.language}
          </span>
        ) : null}
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {repository.starsCount.toLocaleString()} stars
        </span>
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {repository.forksCount.toLocaleString()} forks
        </span>
        {repository.license ? (
          <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {repository.license.name}
          </span>
        ) : null}
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {repository.openIssuesCount.toLocaleString()} issues
        </span>
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {repository.openPullRequestsCount.toLocaleString()} PRs
        </span>
        {starredDate ? (
          <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {formatDate(repository.updatedAt)}
          </span>
        ) : null}
      </div>
    </article>
  );
}

function ActiveFilters({
  filters,
  mode,
  owner,
  sortLabels,
  defaultSort,
}: {
  filters: ProfileRepositoryTabFilters &
    Pick<
      ProfileRepositoryFilters,
      "query" | "repositoryType" | "language" | "sort"
    >;
  owner: string;
  mode: string;
  sortLabels: Record<string, string>;
  defaultSort: string;
}) {
  const chips = [
    filters.query
      ? {
          href: profileRepositoryTabHref(owner, filters, { q: null }),
          label: `Search: ${filters.query}`,
        }
      : null,
    filters.repositoryType !== "all"
      ? {
          href: profileRepositoryTabHref(owner, filters, { type: "all" }),
          label:
            REPOSITORY_TYPE_LABELS[filters.repositoryType] ??
            filters.repositoryType,
        }
      : null,
    filters.language
      ? {
          href: profileRepositoryTabHref(owner, filters, { language: null }),
          label: filters.language,
        }
      : null,
    filters.sort !== defaultSort
      ? {
          href: profileRepositoryTabHref(owner, filters, {
            sort: defaultSort,
          }),
          label: `Sort: ${sortLabels[filters.sort] ?? filters.sort}`,
        }
      : null,
  ].filter((chip): chip is { href: string; label: string } => Boolean(chip));

  if (chips.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="t-label" style={{ color: "var(--ink-3)" }}>
        Active filters
      </span>
      {chips.map((chip) => (
        <Link
          className="chip active no-underline"
          href={chip.href}
          key={chip.label}
        >
          {chip.label} x
        </Link>
      ))}
      <Link
        className="chip soft no-underline"
        href={profileRepositoryTabHref(owner, { mode })}
      >
        Clear filters
      </Link>
    </div>
  );
}

export function ProfileRepositoryTabs({
  list,
  owner,
}: ProfileRepositoryTabsProps) {
  const filters = list.filters;
  const mode = list.mode === "stars" ? "stars" : "repositories";
  const sortLabels =
    mode === "stars" ? STAR_SORT_LABELS : REPOSITORY_SORT_LABELS;
  const defaultSort = mode === "stars" ? "recently-starred" : "updated-desc";
  const title = mode === "stars" ? "Starred repositories" : "Repositories";
  const eyebrow = mode === "stars" ? "Stars tab" : "Repository tab";
  const emptyTitle =
    mode === "stars"
      ? "No starred repositories matched these filters."
      : "No repositories matched these filters.";
  const showingFrom =
    list.total === 0 ? 0 : (list.page - 1) * list.pageSize + 1;
  const showingTo = Math.min(list.page * list.pageSize, list.total);

  return (
    <section
      className="card overflow-hidden"
      aria-labelledby="profile-repositories"
    >
      <div className="border-b border-[var(--line)] p-5">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              {eyebrow}
            </p>
            <h2 className="t-h2 mt-1" id="profile-repositories">
              {title}
            </h2>
          </div>
          <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {showingFrom}-{showingTo} of {list.total.toLocaleString()}
          </p>
        </div>

        <form
          action={`/${encodeURIComponent(owner)}`}
          className="mt-5 grid gap-3 lg:grid-cols-[minmax(180px,1fr)_160px_160px_170px_auto]"
        >
          <input name="tab" type="hidden" value={mode} />
          <label className="grid gap-1">
            <span className="t-label">Search</span>
            <input
              aria-label="Search"
              className="input"
              defaultValue={filters.query ?? ""}
              name="q"
              placeholder="Find a repository..."
              type="search"
            />
          </label>
          {mode === "repositories" ? (
            <label className="grid gap-1">
              <span className="t-label">Type</span>
              <select
                aria-label="Type"
                className="input"
                defaultValue={filters.repositoryType}
                name="type"
              >
                <option value="all">All</option>
                {list.availableTypes.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label} ({option.count})
                  </option>
                ))}
              </select>
            </label>
          ) : (
            <input name="type" type="hidden" value="all" />
          )}
          <label className="grid gap-1">
            <span className="t-label">Language</span>
            <select
              aria-label="Language"
              className="input"
              defaultValue={filters.language ?? ""}
              name="language"
            >
              <option value="">All languages</option>
              {list.availableLanguages.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label} ({option.count})
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-1">
            <span className="t-label">Sort</span>
            <select
              aria-label="Sort"
              className="input"
              defaultValue={filters.sort}
              name="sort"
            >
              {Object.entries(sortLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <div className="flex items-end">
            <button className="btn primary w-full" type="submit">
              Filter
            </button>
          </div>
        </form>

        <div className="mt-4">
          <ActiveFilters
            defaultSort={defaultSort}
            filters={{ ...filters, mode }}
            mode={mode}
            owner={owner}
            sortLabels={sortLabels}
          />
        </div>
      </div>

      {list.items.length > 0 ? (
        <div className="px-5">
          {list.items.map((repository) => (
            <RepositoryRow
              key={repository.id}
              mode={mode}
              repository={repository}
            />
          ))}
        </div>
      ) : (
        <div className="p-8">
          <p className="t-h3">{emptyTitle}</p>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Clear the active filters to return to every visible repository for
            this profile.
          </p>
          <Link
            className="btn mt-4 inline-flex no-underline"
            href={profileRepositoryTabHref(owner, { mode })}
          >
            Clear filters
          </Link>
        </div>
      )}
    </section>
  );
}
