import Link from "next/link";
import type {
  OrganizationTeamSummary,
  OrganizationTeamsDirectory,
  OrganizationTeamsFilters,
} from "@/lib/api";
import {
  organizationPeopleListHref,
  organizationRepositoryListHref,
  organizationSettingsHref,
  organizationTeamsHref,
} from "@/lib/navigation";

type OrganizationTeamsPageProps = {
  directory: OrganizationTeamsDirectory;
  org: string;
};

const VISIBILITY_OPTIONS = [
  { label: "All", value: "all" },
  { label: "Visible", value: "visible" },
  { label: "Secret", value: "secret" },
  { label: "My teams", value: "member" },
] as const;

function teamInitial(team: OrganizationTeamSummary) {
  return team.name.trim().slice(0, 1).toUpperCase() || "T";
}

function visibilityLabel(value: string) {
  return value === "secret" ? "Secret" : "Visible";
}

function visibilityChipClass(value: string) {
  return value === "secret" ? "chip warn" : "chip ok";
}

function teamUpdatedAt(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Updated recently";
  }
  return `Updated ${new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(date)}`;
}

function formatCount(value: number, singular: string) {
  if (value === 1) {
    return `1 ${singular}`;
  }
  if (singular === "repository") {
    return `${value.toLocaleString()} repositories`;
  }
  return `${value.toLocaleString()} ${singular}s`;
}

function ActiveFilters({
  filters,
  org,
}: {
  filters: OrganizationTeamsFilters;
  org: string;
}) {
  const hasQuery = Boolean(filters.query);
  const hasVisibility = filters.visibility !== "all";
  if (!hasQuery && !hasVisibility) {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="t-label" style={{ color: "var(--ink-3)" }}>
        Active filters
      </span>
      {hasQuery ? (
        <Link
          className="chip active max-w-full no-underline"
          href={organizationTeamsHref(org, filters, { page: null, q: null })}
        >
          <span className="min-w-0 overflow-hidden text-ellipsis">
            Search: {filters.query}
          </span>
          <span aria-hidden="true">x</span>
        </Link>
      ) : null}
      {hasVisibility ? (
        <Link
          className="chip active no-underline"
          href={organizationTeamsHref(org, filters, {
            page: null,
            visibility: null,
          })}
        >
          {visibilityLabel(filters.visibility)}
          <span aria-hidden="true">x</span>
        </Link>
      ) : null}
      <Link
        className="chip soft no-underline"
        href={organizationTeamsHref(org)}
      >
        Clear filters
      </Link>
    </div>
  );
}

function TeamsSideNav({
  directory,
  org,
}: {
  directory: OrganizationTeamsDirectory;
  org: string;
}) {
  return (
    <aside
      aria-label="Organization teams navigation"
      className="card self-start p-4"
    >
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Team access
      </p>
      <nav className="mt-3 grid gap-1" aria-label="Teams sections">
        <Link
          aria-current="page"
          className="chip active justify-start no-underline"
          href={organizationTeamsHref(org)}
        >
          Teams
        </Link>
        <Link
          className="chip soft justify-start no-underline"
          href={organizationPeopleListHref(org)}
        >
          Members
        </Link>
        <Link
          className="chip soft justify-start no-underline"
          href={organizationRepositoryListHref(org)}
        >
          Repositories
        </Link>
        {directory.viewerState.canAdminTeams ? (
          <Link
            className="chip soft justify-start no-underline"
            href={organizationSettingsHref(org)}
          >
            Settings
          </Link>
        ) : null}
      </nav>
      <div className="mt-4 grid gap-2">
        <p className="t-sm" style={{ color: "var(--ink-2)" }}>
          {directory.viewerState.canViewSecretTeams
            ? "Owners and admins can see visible and secret teams."
            : "Members see visible teams and secret teams they belong to."}
        </p>
        <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {formatCount(directory.counts.total, "team")} ·{" "}
          {formatCount(directory.counts.memberTeams, "membership")}
        </p>
      </div>
    </aside>
  );
}

function TeamRow({ team }: { team: OrganizationTeamSummary }) {
  return (
    <Link
      aria-label={`Open ${team.name}`}
      className="list-row block px-5 py-4 no-underline"
      href={team.href}
    >
      <article className="grid gap-3 sm:grid-cols-[auto_minmax(0,1fr)_auto] sm:items-start">
        <span className="av lg shrink-0" aria-hidden="true">
          {teamInitial(team)}
        </span>
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h3 className="t-h3 min-w-0" style={{ overflowWrap: "anywhere" }}>
              {team.name}
            </h3>
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              @{team.slug}
            </span>
            <span className={visibilityChipClass(team.visibility)}>
              {visibilityLabel(team.visibility)}
            </span>
            {team.mentionable ? (
              <span className="chip soft">Mentionable</span>
            ) : null}
            {team.viewerCapabilities.isMember ? (
              <span className="chip active">Your team</span>
            ) : null}
          </div>
          {team.description ? (
            <p
              className="t-body mt-2"
              style={{ color: "var(--ink-2)", overflowWrap: "anywhere" }}
            >
              {team.description}
            </p>
          ) : (
            <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
              No team description yet.
            </p>
          )}
          <div className="mt-3 flex flex-wrap items-center gap-2">
            {team.parent ? (
              <span className="chip soft">
                Parent{" "}
                <span className="t-mono-sm" style={{ color: "var(--ink-2)" }}>
                  @{team.parent.slug}
                </span>
              </span>
            ) : null}
            <span className="t-xs">
              {formatCount(team.memberCount, "member")}
            </span>
            <span className="t-xs">
              {formatCount(team.repositoryCount, "repository")}
            </span>
            <span className="t-xs">
              {formatCount(team.childTeamCount, "child team")}
            </span>
          </div>
        </div>
        <div className="grid justify-start gap-2 sm:justify-items-end">
          <span
            className={team.notificationsEnabled ? "chip info" : "chip soft"}
          >
            {team.notificationsEnabled
              ? "Mention notifications"
              : "Notifications off"}
          </span>
          <span className="t-xs">{teamUpdatedAt(team.updatedAt)}</span>
        </div>
      </article>
    </Link>
  );
}

function EmptyTeamsState({
  directory,
}: {
  directory: OrganizationTeamsDirectory;
}) {
  return (
    <div className="grid gap-5 p-6">
      <div>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Empty team directory
        </p>
        <h3 className="t-h2 mt-1">{directory.emptyState.title}</h3>
      </div>
      <div className="grid gap-3 md:grid-cols-3">
        {directory.emptyState.columns.map((column) => (
          <section className="card p-4" key={column.title}>
            <h4 className="t-h3">{column.title}</h4>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              {column.body}
            </p>
          </section>
        ))}
      </div>
      <div className="flex flex-wrap gap-2">
        {directory.viewerState.canCreateTeam ? (
          <Link
            className="btn primary no-underline"
            href={directory.emptyState.newTeamHref}
          >
            New team
          </Link>
        ) : null}
        <Link
          className="btn ghost no-underline"
          href={directory.emptyState.learnMoreHref}
        >
          Learn more
        </Link>
      </div>
    </div>
  );
}

function TeamsPagination({
  directory,
  org,
}: {
  directory: OrganizationTeamsDirectory;
  org: string;
}) {
  const showingTo = Math.min(
    directory.page * directory.pageSize,
    directory.total,
  );
  const previousHref =
    directory.page > 1
      ? organizationTeamsHref(org, directory.filters, {
          page: String(directory.page - 1),
        })
      : null;
  const nextHref =
    showingTo < directory.total
      ? organizationTeamsHref(org, directory.filters, {
          page: String(directory.page + 1),
        })
      : null;

  return (
    <nav
      aria-label="Teams pagination"
      className="flex flex-wrap items-center justify-between gap-3 border-t border-[var(--line)] p-5"
    >
      <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
        Page {directory.page.toLocaleString()}
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
  );
}

export function OrganizationTeamsPage({
  directory,
  org,
}: OrganizationTeamsPageProps) {
  const showingFrom =
    directory.total === 0 ? 0 : (directory.page - 1) * directory.pageSize + 1;
  const showingTo = Math.min(
    directory.page * directory.pageSize,
    directory.total,
  );

  return (
    <section
      aria-labelledby="organization-teams-title"
      className="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]"
    >
      <TeamsSideNav directory={directory} org={org} />

      <div className="card min-w-0 overflow-hidden">
        <div className="border-b border-[var(--line)] p-5">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Organization teams
              </p>
              <h2 className="t-h2 mt-1" id="organization-teams-title">
                Teams
              </h2>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                {showingFrom}-{showingTo} of {directory.total.toLocaleString()}
              </p>
              {directory.viewerState.canCreateTeam ? (
                <Link
                  className="btn primary no-underline"
                  href={directory.emptyState.newTeamHref}
                >
                  New team
                </Link>
              ) : null}
            </div>
          </div>

          <form
            action={organizationTeamsHref(org)}
            className="mt-5 grid gap-3 md:grid-cols-[minmax(180px,1fr)_180px_auto]"
          >
            <label className="grid gap-1">
              <span className="t-label">Search</span>
              <input
                aria-label="Search organization teams"
                className="input"
                defaultValue={directory.filters.query ?? ""}
                name="q"
                placeholder="Find a team..."
                type="search"
              />
            </label>
            <label className="grid gap-1">
              <span className="t-label">Visibility</span>
              <select
                aria-label="Filter team visibility"
                className="input"
                defaultValue={directory.filters.visibility}
                name="visibility"
              >
                {VISIBILITY_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            {directory.filters.pageSize !== 30 ? (
              <input
                name="pageSize"
                type="hidden"
                value={directory.filters.pageSize}
              />
            ) : null}
            <div className="flex items-end">
              <button className="btn primary w-full" type="submit">
                Filter
              </button>
            </div>
          </form>

          <div className="mt-4">
            <ActiveFilters filters={directory.filters} org={org} />
          </div>
        </div>

        {directory.items.length > 0 ? (
          <div>
            <nav className="tabs px-5" aria-label="Team visibility summary">
              <Link
                className={`tab ${directory.filters.visibility === "all" ? "active" : ""}`}
                href={organizationTeamsHref(org, directory.filters, {
                  visibility: null,
                  page: null,
                })}
              >
                All{" "}
                <span className="badge t-num">{directory.counts.total}</span>
              </Link>
              <Link
                className={`tab ${directory.filters.visibility === "visible" ? "active" : ""}`}
                href={organizationTeamsHref(org, directory.filters, {
                  visibility: "visible",
                  page: null,
                })}
              >
                Visible{" "}
                <span className="badge t-num">{directory.counts.visible}</span>
              </Link>
              {directory.viewerState.canViewSecretTeams ? (
                <Link
                  className={`tab ${directory.filters.visibility === "secret" ? "active" : ""}`}
                  href={organizationTeamsHref(org, directory.filters, {
                    visibility: "secret",
                    page: null,
                  })}
                >
                  Secret{" "}
                  <span className="badge t-num">{directory.counts.secret}</span>
                </Link>
              ) : null}
              <Link
                className={`tab ${directory.filters.visibility === "member" ? "active" : ""}`}
                href={organizationTeamsHref(org, directory.filters, {
                  visibility: "member",
                  page: null,
                })}
              >
                My teams{" "}
                <span className="badge t-num">
                  {directory.counts.memberTeams}
                </span>
              </Link>
            </nav>
            {directory.items.map((team) => (
              <TeamRow key={team.id} team={team} />
            ))}
          </div>
        ) : directory.counts.total === 0 ? (
          <EmptyTeamsState directory={directory} />
        ) : (
          <div className="p-8">
            <p className="t-h3">No teams matched these filters.</p>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Clear the search or visibility filter to return to every team you
              can view.
            </p>
            <Link
              className="btn mt-4 inline-flex no-underline"
              href={organizationTeamsHref(org)}
            >
              Clear filters
            </Link>
          </div>
        )}

        <TeamsPagination directory={directory} org={org} />
      </div>
    </section>
  );
}
