import Link from "next/link";
import type {
  ProfileRepositoryFilters,
  ProfileRepositoryList,
  ProfileRepositoryListItem,
} from "@/lib/api";

type ProfileRepositoryTabsProps = {
  list: ProfileRepositoryList;
  owner: string;
};

const REPOSITORY_SORT_LABELS: Record<string, string> = {
  "updated-desc": "Last updated",
  "name-asc": "Name",
  "stars-desc": "Stars",
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

function repositoryTabHref(
  owner: string,
  filters: ProfileRepositoryFilters,
  overrides: Partial<Record<"q" | "type" | "language" | "sort", string | null>>,
) {
  const params = new URLSearchParams();
  params.set("tab", "repositories");

  const nextQuery =
    overrides.q === undefined ? filters.query : overrides.q?.trim() || null;
  const nextType =
    overrides.type === undefined
      ? filters.repositoryType
      : overrides.type?.trim() || "all";
  const nextLanguage =
    overrides.language === undefined
      ? filters.language
      : overrides.language?.trim() || null;
  const nextSort =
    overrides.sort === undefined ? filters.sort : overrides.sort?.trim() || "";

  if (nextQuery) {
    params.set("q", nextQuery);
  }
  if (nextType && nextType !== "all") {
    params.set("type", nextType);
  }
  if (nextLanguage) {
    params.set("language", nextLanguage);
  }
  if (nextSort && nextSort !== "updated-desc") {
    params.set("sort", nextSort);
  }

  return `/${encodeURIComponent(owner)}?${params.toString()}`;
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Updated recently";
  }

  return `Updated ${new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
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
}: {
  repository: ProfileRepositoryListItem;
}) {
  const badges = repositoryBadges(repository);

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
          {formatDate(repository.updatedAt)}
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
      </div>
    </article>
  );
}

function ActiveFilters({
  filters,
  owner,
}: {
  filters: ProfileRepositoryFilters;
  owner: string;
}) {
  const chips = [
    filters.query
      ? {
          href: repositoryTabHref(owner, filters, { q: null }),
          label: `Search: ${filters.query}`,
        }
      : null,
    filters.repositoryType !== "all"
      ? {
          href: repositoryTabHref(owner, filters, { type: "all" }),
          label:
            REPOSITORY_TYPE_LABELS[filters.repositoryType] ??
            filters.repositoryType,
        }
      : null,
    filters.language
      ? {
          href: repositoryTabHref(owner, filters, { language: null }),
          label: filters.language,
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
        href={`/${encodeURIComponent(owner)}?tab=repositories`}
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
              Repository tab
            </p>
            <h2 className="t-h2 mt-1" id="profile-repositories">
              Repositories
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
          <input name="tab" type="hidden" value="repositories" />
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
              {Object.entries(REPOSITORY_SORT_LABELS).map(([value, label]) => (
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
          <ActiveFilters filters={filters} owner={owner} />
        </div>
      </div>

      {list.items.length > 0 ? (
        <div className="px-5">
          {list.items.map((repository) => (
            <RepositoryRow key={repository.id} repository={repository} />
          ))}
        </div>
      ) : (
        <div className="p-8">
          <p className="t-h3">No repositories matched these filters.</p>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Clear the active filters to return to every visible repository for
            this profile.
          </p>
          <Link
            className="btn mt-4 inline-flex no-underline"
            href={`/${encodeURIComponent(owner)}?tab=repositories`}
          >
            Clear filters
          </Link>
        </div>
      )}
    </section>
  );
}
