import Link from "next/link";
import type {
  OrganizationRepositoryFilters,
  OrganizationRepositoryList,
  OrganizationRepositoryListItem,
} from "@/lib/api";
import {
  type OrganizationRepositoryListFilters,
  organizationRepositoryListHref,
} from "@/lib/navigation";

type OrganizationRepositoriesPageProps = {
  list: OrganizationRepositoryList;
  org: string;
};

const SORT_LABELS: Record<string, string> = {
  "updated-desc": "Last updated",
  "name-asc": "Name",
  "stars-desc": "Stars",
};

const TYPE_LABELS: Record<string, string> = {
  all: "All",
  contributed: "Contributed by me",
  admin: "Admin access",
  public: "Public",
  sources: "Sources",
  forks: "Forks",
  archived: "Archived",
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
  }).format(date)}`;
}

function repositoryBadges(repository: OrganizationRepositoryListItem) {
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
  if (repository.canAdmin) {
    badges.push("admin");
  }
  if (repository.contributedByViewer) {
    badges.push("contributed");
  }
  return badges;
}

function LanguageDot({ color }: { color: string }) {
  return (
    <span
      aria-hidden="true"
      className="inline-block size-2 shrink-0 rounded-full"
      style={{ backgroundColor: color || "var(--ink-4)" }}
    />
  );
}

function ActiveFilters({
  filters,
  org,
}: {
  filters: OrganizationRepositoryListFilters &
    Pick<
      OrganizationRepositoryFilters,
      "query" | "repositoryType" | "language" | "sort" | "density"
    >;
  org: string;
}) {
  const chips = [
    filters.query
      ? {
          href: organizationRepositoryListHref(org, filters, { q: null }),
          label: `Search: ${filters.query}`,
        }
      : null,
    filters.repositoryType !== "all"
      ? {
          href: organizationRepositoryListHref(org, filters, { type: "all" }),
          label: TYPE_LABELS[filters.repositoryType] ?? filters.repositoryType,
        }
      : null,
    filters.language
      ? {
          href: organizationRepositoryListHref(org, filters, {
            language: null,
          }),
          label: filters.language,
        }
      : null,
    filters.sort !== "updated-desc"
      ? {
          href: organizationRepositoryListHref(org, filters, {
            sort: "updated-desc",
          }),
          label: `Sort: ${SORT_LABELS[filters.sort] ?? filters.sort}`,
        }
      : null,
    filters.density !== "comfortable"
      ? {
          href: organizationRepositoryListHref(org, filters, {
            density: "comfortable",
          }),
          label: "Compact density",
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
        href={organizationRepositoryListHref(org)}
      >
        Clear filters
      </Link>
    </div>
  );
}

function RepositoryRow({
  density,
  repository,
}: {
  density: string;
  repository: OrganizationRepositoryListItem;
}) {
  const badges = repositoryBadges(repository);
  const compact = density === "compact";

  return (
    <article className={`list-row ${compact ? "py-3" : "py-5"}`}>
      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <Link className="t-h3 min-w-0 no-underline" href={repository.href}>
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
          {repository.forkSource ? (
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Forked from{" "}
              <Link className="underline" href={repository.forkSource.href}>
                {repository.forkSource.owner}/{repository.forkSource.name}
              </Link>
            </p>
          ) : null}
          {repository.description ? (
            <p
              className={`${compact ? "t-sm" : "t-body"} mt-3 max-w-3xl`}
              style={{ color: "var(--ink-2)" }}
            >
              {repository.description}
            </p>
          ) : null}
          {repository.topics.length > 0 ? (
            <div className="mt-3 flex flex-wrap gap-1.5">
              {repository.topics.slice(0, compact ? 3 : 6).map((topic) => (
                <span className="chip soft" key={topic}>
                  {topic}
                </span>
              ))}
            </div>
          ) : null}
        </div>
        <span className="t-xs shrink-0" style={{ color: "var(--ink-3)" }}>
          {formatDate(repository.updatedAt)}
        </span>
      </div>

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

export function OrganizationRepositoriesPage({
  list,
  org,
}: OrganizationRepositoriesPageProps) {
  const filters = list.filters;
  const showingFrom =
    list.total === 0 ? 0 : (list.page - 1) * list.pageSize + 1;
  const showingTo = Math.min(list.page * list.pageSize, list.total);
  const previousHref =
    list.page > 1
      ? organizationRepositoryListHref(org, filters, {
          page: String(list.page - 1),
        })
      : null;
  const nextHref =
    showingTo < list.total
      ? organizationRepositoryListHref(org, filters, {
          page: String(list.page + 1),
        })
      : null;

  return (
    <section
      className="card overflow-hidden"
      aria-labelledby="organization-repositories-title"
    >
      <div className="border-b border-[var(--line)] p-5">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository directory
            </p>
            <h2 className="t-h2 mt-1" id="organization-repositories-title">
              Repositories
            </h2>
          </div>
          <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {showingFrom}-{showingTo} of {list.total.toLocaleString()}
          </p>
        </div>

        <form
          action={`/orgs/${encodeURIComponent(org)}/repositories`}
          className="mt-5 grid gap-3 xl:grid-cols-[minmax(180px,1fr)_160px_160px_170px_auto]"
        >
          <label className="grid gap-1">
            <span className="t-label">Search</span>
            <input
              aria-label="Search organization repositories"
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
              aria-label="Repository type"
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
              {Object.entries(SORT_LABELS).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <input name="density" type="hidden" value={filters.density} />
          <div className="flex items-end">
            <button className="btn primary w-full" type="submit">
              Filter
            </button>
          </div>
        </form>

        <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
          <ActiveFilters filters={filters} org={org} />
          <fieldset className="inline-flex rounded-[var(--radius)] border border-[var(--line)] p-1">
            <legend className="sr-only">Display density</legend>
            <Link
              aria-label="Comfortable density"
              className={`btn sm ghost ${filters.density === "comfortable" ? "active" : ""}`}
              href={organizationRepositoryListHref(org, filters, {
                density: "comfortable",
              })}
            >
              Wide
            </Link>
            <Link
              aria-label="Compact density"
              className={`btn sm ghost ${filters.density === "compact" ? "active" : ""}`}
              href={organizationRepositoryListHref(org, filters, {
                density: "compact",
              })}
            >
              Tight
            </Link>
          </fieldset>
        </div>
      </div>

      {list.items.length > 0 ? (
        <div className="px-5">
          {list.items.map((repository) => (
            <RepositoryRow
              density={filters.density}
              key={repository.id}
              repository={repository}
            />
          ))}
        </div>
      ) : (
        <div className="p-8">
          <p className="t-h3">No repositories matched these filters.</p>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Clear the active filters to return to every visible repository in
            this organization.
          </p>
          <Link
            className="btn mt-4 inline-flex no-underline"
            href={organizationRepositoryListHref(org)}
          >
            Clear filters
          </Link>
        </div>
      )}

      <nav
        aria-label="Repository pagination"
        className="flex flex-wrap items-center justify-between gap-3 border-t border-[var(--line)] p-5"
      >
        <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          Page {list.page.toLocaleString()}
        </p>
        <div className="flex gap-2">
          {previousHref ? (
            <Link className="btn sm ghost" href={previousHref}>
              Previous
            </Link>
          ) : (
            <button className="btn sm" disabled type="button">
              Previous
            </button>
          )}
          {nextHref ? (
            <Link className="btn sm ghost" href={nextHref}>
              Next
            </Link>
          ) : (
            <button className="btn sm" disabled type="button">
              Next
            </button>
          )}
        </div>
      </nav>
    </section>
  );
}
