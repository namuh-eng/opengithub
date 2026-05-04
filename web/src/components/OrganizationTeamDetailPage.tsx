import Link from "next/link";
import type {
  OrganizationTeamDetail,
  OrganizationTeamMemberRow,
  OrganizationTeamRepositoryPermission,
  OrganizationTeamSummary,
} from "@/lib/api";
import {
  organizationPeopleListHref,
  organizationTeamsHref,
} from "@/lib/navigation";

type OrganizationTeamDetailPageProps = {
  detail: OrganizationTeamDetail;
  org: string;
};

function formatCount(value: number, singular: string) {
  if (value === 1) {
    return `1 ${singular}`;
  }
  return `${value.toLocaleString()} ${singular}s`;
}

function roleLabel(role: string) {
  return role
    .split("_")
    .join(" ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function visibilityChip(value: string) {
  return value === "secret" ? "chip warn" : "chip ok";
}

function TeamMiniRow({ team }: { team: OrganizationTeamSummary }) {
  return (
    <Link className="list-row block px-4 py-3 no-underline" href={team.href}>
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <span className="av sm" aria-hidden="true">
          {team.name.slice(0, 1).toUpperCase()}
        </span>
        <span className="t-h3 min-w-0" style={{ overflowWrap: "anywhere" }}>
          {team.name}
        </span>
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          @{team.slug}
        </span>
        <span className={visibilityChip(team.visibility)}>
          {team.visibility}
        </span>
      </div>
    </Link>
  );
}

function MemberRow({ member }: { member: OrganizationTeamMemberRow }) {
  return (
    <Link className="list-row block px-4 py-3 no-underline" href={member.href}>
      <div className="flex min-w-0 flex-wrap items-center justify-between gap-3">
        <span className="flex min-w-0 items-center gap-2">
          <span className="av sm" aria-hidden="true">
            {member.login.slice(0, 1).toUpperCase()}
          </span>
          <span className="min-w-0">
            <span className="t-h3 block" style={{ overflowWrap: "anywhere" }}>
              {member.displayName ?? member.login}
            </span>
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              @{member.login}
            </span>
          </span>
        </span>
        <span className="chip soft">{roleLabel(member.role)}</span>
      </div>
    </Link>
  );
}

function RepositoryRow({
  repository,
}: {
  repository: OrganizationTeamRepositoryPermission;
}) {
  return (
    <Link
      className="list-row block px-4 py-3 no-underline"
      href={repository.href}
    >
      <div className="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <span className="t-h3" style={{ overflowWrap: "anywhere" }}>
              {repository.fullName}
            </span>
            <span className="chip soft">{repository.visibility}</span>
          </div>
          <p className="t-xs mt-1">
            {repository.inherited
              ? `Inherited from @${repository.sourceTeamSlug}`
              : "Direct team repository permission"}
          </p>
        </div>
        <span className={repository.inherited ? "chip info" : "chip active"}>
          {roleLabel(repository.role)}
        </span>
      </div>
    </Link>
  );
}

export function OrganizationTeamDetailPage({
  detail,
  org,
}: OrganizationTeamDetailPageProps) {
  const { team } = detail;
  const memberPreview = detail.members.slice(0, 8);
  const repositoryPreview = detail.repositories.slice(0, 8);

  return (
    <div className="grid gap-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h2 className="t-h2" style={{ overflowWrap: "anywhere" }}>
              {team.name}
            </h2>
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              @{team.slug}
            </span>
            <span className={visibilityChip(team.visibility)}>
              {team.visibility}
            </span>
          </div>
          <p
            className="t-body mt-2 max-w-3xl"
            style={{ color: "var(--ink-2)" }}
          >
            {team.description ?? "No team description yet."}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Link className="btn sm" href={organizationTeamsHref(org)}>
            All teams
          </Link>
          <Link className="btn sm" href={organizationPeopleListHref(org)}>
            Members
          </Link>
        </div>
      </div>

      <div className="grid gap-3 md:grid-cols-4">
        <section className="card p-4">
          <p className="t-label">Members</p>
          <p className="t-h2 mt-1">{formatCount(team.memberCount, "member")}</p>
        </section>
        <section className="card p-4">
          <p className="t-label">Repositories</p>
          <p className="t-h2 mt-1">
            {formatCount(detail.repositories.length, "repository")}
          </p>
        </section>
        <section className="card p-4">
          <p className="t-label">Inherited</p>
          <p className="t-h2 mt-1">
            {formatCount(detail.hierarchy.inheritedRepositoryCount, "grant")}
          </p>
        </section>
        <section className="card p-4">
          <p className="t-label">Mentions</p>
          <p className="t-h2 mt-1">
            {detail.mentionState.notificationsEnabled ? "Enabled" : "Indexed"}
          </p>
        </section>
      </div>

      <section className="card overflow-hidden">
        <div className="border-b border-[var(--line)] p-4">
          <p className="t-label">Overview</p>
          <h3 className="t-h3 mt-1">Hierarchy and mention delivery</h3>
        </div>
        <div className="grid gap-4 p-4 md:grid-cols-2">
          <div>
            <p className="t-sm" style={{ color: "var(--ink-2)" }}>
              {detail.hierarchy.parentChain.length
                ? "Parent permissions cascade into this team for repository access and review ownership."
                : "This is a root team. Repository permissions granted here can cascade into child teams."}
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              {detail.hierarchy.parentChain.map((parent) => (
                <Link
                  className="chip soft no-underline"
                  href={parent.href}
                  key={parent.id}
                >
                  @{parent.slug}
                </Link>
              ))}
              {!detail.hierarchy.parentChain.length ? (
                <span className="chip soft">No parent team</span>
              ) : null}
            </div>
          </div>
          <div>
            <p className="t-sm" style={{ color: "var(--ink-2)" }}>
              {detail.mentionState.fanoutState}
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              <span
                className={
                  detail.mentionState.mentionable ? "chip ok" : "chip soft"
                }
              >
                {detail.mentionState.mentionable
                  ? "Mentionable"
                  : "Restricted mentions"}
              </span>
              <span
                className={
                  detail.mentionState.notificationsEnabled
                    ? "chip info"
                    : "chip soft"
                }
              >
                {detail.mentionState.notificationsEnabled
                  ? "Fanout enabled"
                  : "Fanout suppressed"}
              </span>
            </div>
          </div>
        </div>
      </section>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_340px]">
        <section className="card overflow-hidden">
          <div className="border-b border-[var(--line)] p-4">
            <p className="t-label">Repositories</p>
            <h3 className="t-h3 mt-1">Direct and inherited access</h3>
          </div>
          {repositoryPreview.length ? (
            repositoryPreview.map((repository) => (
              <RepositoryRow
                key={`${repository.repositoryId}-${repository.sourceTeamSlug}`}
                repository={repository}
              />
            ))
          ) : (
            <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
              No repositories are connected to this team yet.
            </p>
          )}
        </section>

        <aside className="grid gap-6 self-start">
          <section className="card overflow-hidden">
            <div className="border-b border-[var(--line)] p-4">
              <p className="t-label">Members</p>
              <h3 className="t-h3 mt-1">Team roster</h3>
            </div>
            {memberPreview.length ? (
              memberPreview.map((member) => (
                <MemberRow key={member.userId} member={member} />
              ))
            ) : (
              <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
                This team has no members yet.
              </p>
            )}
          </section>

          <section className="card overflow-hidden">
            <div className="border-b border-[var(--line)] p-4">
              <p className="t-label">Child Teams</p>
              <h3 className="t-h3 mt-1">Nested teams</h3>
            </div>
            {detail.childTeams.length ? (
              detail.childTeams.map((child) => (
                <TeamMiniRow key={child.id} team={child} />
              ))
            ) : (
              <p className="t-sm p-4" style={{ color: "var(--ink-3)" }}>
                No child teams yet.
              </p>
            )}
          </section>
        </aside>
      </div>
    </div>
  );
}
