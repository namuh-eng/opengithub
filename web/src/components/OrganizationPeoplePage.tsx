import Link from "next/link";
import type {
  OrganizationPeopleList,
  OrganizationPeopleListItem,
} from "@/lib/api";
import {
  type OrganizationPeopleListFilters,
  organizationPeopleListHref,
  organizationRepositoryListHref,
  organizationSettingsHref,
  organizationTeamHref,
} from "@/lib/navigation";

type OrganizationPeoplePageProps = {
  list: OrganizationPeopleList;
  org: string;
};

function formatJoinedDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Joined recently";
  }

  return `Joined ${new Intl.DateTimeFormat("en", {
    month: "short",
    year: "numeric",
  }).format(date)}`;
}

function personInitial(person: OrganizationPeopleListItem) {
  return (person.name?.trim() || person.login).slice(0, 1).toUpperCase();
}

function roleLabel(role: string) {
  if (role === "owner") {
    return "Owner";
  }
  if (role === "admin") {
    return "Admin";
  }
  return "Member";
}

function Avatar({ person }: { person: OrganizationPeopleListItem }) {
  if (person.avatarUrl) {
    return (
      <span
        aria-hidden="true"
        className="av lg shrink-0"
        style={{
          backgroundImage: `url(${person.avatarUrl})`,
          backgroundPosition: "center",
          backgroundSize: "cover",
        }}
      />
    );
  }

  return (
    <span aria-hidden="true" className="av lg shrink-0">
      {personInitial(person)}
    </span>
  );
}

function PersonRow({ person }: { person: OrganizationPeopleListItem }) {
  const displayName = person.name?.trim() || person.login;

  return (
    <article className="list-row py-4">
      <div className="grid gap-3 sm:grid-cols-[auto_minmax(0,1fr)_auto] sm:items-center">
        <Avatar person={person} />
        <div className="min-w-0">
          <Link
            aria-label={`Open ${displayName}`}
            className="t-h3 no-underline"
            href={person.href}
          >
            {displayName}
          </Link>
          <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
            @{person.login}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2 sm:justify-end">
          {person.role ? (
            <span className="chip soft">{roleLabel(person.role)}</span>
          ) : null}
          <span className="t-xs" style={{ color: "var(--ink-3)" }}>
            {formatJoinedDate(person.joinedAt)}
          </span>
        </div>
      </div>
    </article>
  );
}

function ActiveFilters({
  filters,
  org,
}: {
  filters: OrganizationPeopleListFilters;
  org: string;
}) {
  if (!filters.query) {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="t-label" style={{ color: "var(--ink-3)" }}>
        Active filters
      </span>
      <Link
        className="chip active no-underline"
        href={organizationPeopleListHref(org, filters, {
          page: null,
          q: null,
        })}
      >
        Search: {filters.query} x
      </Link>
      <Link
        className="chip soft no-underline"
        href={organizationPeopleListHref(org)}
      >
        Clear filters
      </Link>
    </div>
  );
}

export function OrganizationPeoplePage({
  list,
  org,
}: OrganizationPeoplePageProps) {
  const filters = list.filters;
  const showingFrom =
    list.total === 0 ? 0 : (list.page - 1) * list.pageSize + 1;
  const showingTo = Math.min(list.page * list.pageSize, list.total);
  const previousHref =
    list.page > 1
      ? organizationPeopleListHref(org, filters, {
          page: String(list.page - 1),
        })
      : null;
  const nextHref =
    showingTo < list.total
      ? organizationPeopleListHref(org, filters, {
          page: String(list.page + 1),
        })
      : null;

  return (
    <section
      aria-labelledby="organization-people-title"
      className="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]"
    >
      <aside
        aria-label="Organization permissions"
        className="card self-start p-4"
      >
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Organization permissions
        </p>
        <nav className="mt-3 grid gap-1" aria-label="People sections">
          <Link
            aria-current="page"
            className="chip active justify-start no-underline"
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
          <Link
            className="chip soft justify-start no-underline"
            href={organizationTeamHref(org, "core")}
          >
            Teams
          </Link>
          {list.viewerState.canAdmin ? (
            <Link
              className="chip soft justify-start no-underline"
              href={organizationSettingsHref(org)}
            >
              Settings
            </Link>
          ) : null}
        </nav>
        <p className="t-sm mt-4" style={{ color: "var(--ink-2)" }}>
          {list.viewerState.isMember
            ? "Membership roles are visible to organization members."
            : "Signed-out and outside viewers see public members only."}
        </p>
      </aside>

      <div className="card overflow-hidden">
        <div className="border-b border-[var(--line)] p-5">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Members
              </p>
              <h2 className="t-h2 mt-1" id="organization-people-title">
                People
              </h2>
            </div>
            <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {showingFrom}-{showingTo} of {list.total.toLocaleString()}
            </p>
          </div>

          <form
            action={`/orgs/${encodeURIComponent(org)}/people`}
            className="mt-5 grid gap-3 sm:grid-cols-[minmax(180px,1fr)_auto]"
          >
            <label className="grid gap-1">
              <span className="t-label">Search</span>
              <input
                aria-label="Search organization people"
                className="input"
                defaultValue={filters.query ?? ""}
                name="q"
                placeholder="Find a member..."
                type="search"
              />
            </label>
            {filters.pageSize !== 30 ? (
              <input name="pageSize" type="hidden" value={filters.pageSize} />
            ) : null}
            <div className="flex items-end">
              <button className="btn primary w-full" type="submit">
                Filter
              </button>
            </div>
          </form>

          <div className="mt-4">
            <ActiveFilters filters={filters} org={org} />
          </div>
        </div>

        {list.items.length > 0 ? (
          <div className="px-5">
            {list.items.map((person) => (
              <PersonRow key={person.id} person={person} />
            ))}
          </div>
        ) : (
          <div className="p-8">
            <p className="t-h3">No visible members matched these filters.</p>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Clear the search to return to every visible public member in this
              organization.
            </p>
            <Link
              className="btn mt-4 inline-flex no-underline"
              href={organizationPeopleListHref(org)}
            >
              Clear filters
            </Link>
          </div>
        )}

        <nav
          aria-label="People pagination"
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
      </div>
    </section>
  );
}
