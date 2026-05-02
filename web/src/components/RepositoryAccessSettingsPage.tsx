import Image from "next/image";
import Link from "next/link";
import type {
  RepositoryAccessPerson,
  RepositoryAccessRole,
  RepositoryAccessSettings,
  RepositoryAccessSettingsFetchResult,
  RepositoryAccessTeam,
  RepositoryInvitation,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryAccessSettingsPageProps = {
  query?: string;
  repository: RepositoryOverview;
  settingsResult: RepositoryAccessSettingsFetchResult;
};

type AccessCardProps = {
  children: React.ReactNode;
  kicker: string;
  title: string;
};

type AccessAvatarProps = {
  label: string;
  src: string | null;
};

function AccessCard({ children, kicker, title }: AccessCardProps) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {kicker}
      </p>
      <h2 className="t-h3 mt-2">{title}</h2>
      <div className="mt-4">{children}</div>
    </section>
  );
}

function AccessAvatar({ label, src }: AccessAvatarProps) {
  if (src) {
    return (
      <Image
        alt=""
        className="av sm"
        height={28}
        src={src}
        unoptimized
        width={28}
      />
    );
  }
  return <span className="av sm">{initials(label)}</span>;
}

function initials(value: string) {
  return value
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function roleLabel(role: RepositoryAccessRole) {
  const labels: Record<RepositoryAccessRole, string> = {
    admin: "Admin",
    maintain: "Maintain",
    owner: "Owner",
    read: "Read",
    triage: "Triage",
    write: "Write",
  };
  return labels[role] ?? role;
}

function roleChipClass(role: RepositoryAccessRole) {
  if (role === "owner" || role === "admin") {
    return "chip accent";
  }
  if (role === "maintain" || role === "write") {
    return "chip ok";
  }
  if (role === "triage") {
    return "chip info";
  }
  return "chip soft";
}

function matchesQuery(values: Array<string | null | undefined>, query: string) {
  if (!query) {
    return true;
  }
  const normalized = query.toLowerCase();
  return values.some((value) => value?.toLowerCase().includes(normalized));
}

function sourceChipClass(source: string) {
  if (source === "owner") {
    return "chip accent";
  }
  if (source === "team" || source === "inherited") {
    return "chip warn";
  }
  return "chip soft";
}

function formattedDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function RepositoryAccessUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryAccessSettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <div className="grid gap-4">
      <section className="card p-6" role="status">
        <span className={`chip ${isForbidden ? "warn" : "err"}`}>
          {isForbidden ? "Admin access required" : "Unavailable"}
        </span>
        <h2 className="t-h2 mt-4">
          {isForbidden
            ? "Repository access is restricted"
            : "Repository access could not load"}
        </h2>
        <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
          {isForbidden
            ? "Only repository owners and admins can view collaborators, teams, and pending invitations."
            : result.message}
        </p>
        <div className="mt-5 flex flex-wrap gap-2">
          <Link
            className="btn"
            href={`/${repository.owner_login}/${repository.name}`}
          >
            Repository Code
          </Link>
          <Link className="btn" href="/dashboard">
            Dashboard
          </Link>
        </div>
      </section>
    </div>
  );
}

function AccessSummary({ settings }: { settings: RepositoryAccessSettings }) {
  return (
    <div className="grid gap-3 md:grid-cols-4">
      {[
        ["People", settings.people.length],
        ["Teams", settings.teams.length],
        ["Pending", settings.invitations.length],
        [
          "Invite targets",
          settings.inviteTargets.users.length +
            settings.inviteTargets.teams.length,
        ],
      ].map(([label, value]) => (
        <div className="card p-4" key={label}>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            {label}
          </p>
          <p className="t-num mt-2 text-2xl" style={{ color: "var(--ink-1)" }}>
            {value}
          </p>
        </div>
      ))}
    </div>
  );
}

function PeopleRows({
  people,
  repository,
  roles,
}: {
  people: RepositoryAccessPerson[];
  repository: RepositoryOverview;
  roles: RepositoryAccessSettings["roles"];
}) {
  if (people.length === 0) {
    return (
      <div
        className="rounded-md p-5"
        style={{ background: "var(--surface-2)" }}
      >
        <p className="t-sm font-semibold">No outside collaborators</p>
        <p className="t-xs mt-1">
          Add a person when you need direct repository access outside inherited
          owner or team paths.
        </p>
        <Link className="btn sm mt-4" href="#invite-people">
          Add people
        </Link>
      </div>
    );
  }

  return (
    <div className="grid gap-0">
      {people.map((person) => (
        <div className="list-row py-4" key={person.userId}>
          <div className="flex min-w-0 flex-1 items-start gap-3">
            <AccessAvatar label={person.login} src={person.avatarUrl} />
            <div className="min-w-0">
              <Link
                className="t-sm font-semibold hover:underline"
                href={`/${person.login}`}
              >
                {person.login}
              </Link>
              <p className="t-xs mt-1 break-words">
                {person.displayName ?? person.email}
              </p>
              <p className="t-xs mt-1 break-words">{person.sourceText}</p>
              {person.teamSlug ? (
                <Link
                  className="t-xs mt-1 inline-block hover:underline"
                  href={`/orgs/${repository.owner_login}/teams/${person.teamSlug}`}
                >
                  From team {person.teamName ?? person.teamSlug}
                </Link>
              ) : null}
            </div>
          </div>
          <div className="mt-3 flex w-full flex-wrap items-center gap-2 sm:mt-0 sm:w-auto sm:justify-end">
            <span className={roleChipClass(person.role)}>
              {roleLabel(person.role)}
            </span>
            <span className={sourceChipClass(person.source)}>
              {person.source}
            </span>
            <label className="sr-only" htmlFor={`person-role-${person.userId}`}>
              Role for {person.login}
            </label>
            <select
              className="input min-w-32"
              defaultValue={person.role}
              disabled={!person.canEdit}
              id={`person-role-${person.userId}`}
            >
              {roles.map((role) => (
                <option key={role.role} value={role.role}>
                  {role.label}
                </option>
              ))}
            </select>
            <Link
              className="btn sm"
              href={
                person.canRemove ? "#direct-access-actions" : "#access-sources"
              }
            >
              {person.canRemove ? "Review direct access" : "Why disabled"}
            </Link>
          </div>
        </div>
      ))}
    </div>
  );
}

function TeamRows({
  roles,
  teams,
}: {
  roles: RepositoryAccessSettings["roles"];
  teams: RepositoryAccessTeam[];
}) {
  if (teams.length === 0) {
    return (
      <div
        className="rounded-md p-5"
        style={{ background: "var(--surface-2)" }}
      >
        <p className="t-sm font-semibold">No team grants</p>
        <p className="t-xs mt-1">
          Organization teams that can access this repository will appear here
          with their member counts and source.
        </p>
        <Link className="btn sm mt-4" href="#invite-teams">
          Add teams
        </Link>
      </div>
    );
  }

  return (
    <div className="grid gap-0">
      {teams.map((team) => (
        <div className="list-row py-4" key={team.teamId}>
          <div className="min-w-0 flex-1">
            <Link
              className="t-sm font-semibold hover:underline"
              href={team.href}
            >
              {team.name}
            </Link>
            <p className="t-xs mt-1 break-words">
              @{team.slug} · <span className="t-num">{team.memberCount}</span>{" "}
              members
            </p>
            <p className="t-xs mt-1 break-words">{team.sourceText}</p>
          </div>
          <div className="mt-3 flex w-full flex-wrap items-center gap-2 sm:mt-0 sm:w-auto sm:justify-end">
            <span className={roleChipClass(team.role)}>
              {roleLabel(team.role)}
            </span>
            <span className={sourceChipClass(team.source)}>{team.source}</span>
            <label className="sr-only" htmlFor={`team-role-${team.teamId}`}>
              Role for {team.name}
            </label>
            <select
              className="input min-w-32"
              defaultValue={team.role}
              disabled={!team.canEdit}
              id={`team-role-${team.teamId}`}
            >
              {roles.map((role) => (
                <option key={role.role} value={role.role}>
                  {role.label}
                </option>
              ))}
            </select>
            <Link
              className="btn sm"
              href={team.canRemove ? "#team-access-actions" : "#access-sources"}
            >
              {team.canRemove ? "Review team access" : "Why disabled"}
            </Link>
          </div>
        </div>
      ))}
    </div>
  );
}

function InvitationRows({
  invitations,
}: {
  invitations: RepositoryInvitation[];
}) {
  if (invitations.length === 0) {
    return (
      <p className="t-sm" style={{ color: "var(--ink-3)" }}>
        There are no pending repository invitations.
      </p>
    );
  }

  return (
    <div className="grid gap-0">
      {invitations.map((invitation) => (
        <div className="list-row py-4" key={invitation.id}>
          <div className="min-w-0 flex-1">
            <p className="t-sm font-semibold break-words">
              {invitation.invitedLogin ?? invitation.invitedEmail}
            </p>
            <p className="t-xs mt-1 break-words">
              Invited {formattedDate(invitation.createdAt)} · expires{" "}
              {formattedDate(invitation.expiresAt)}
            </p>
          </div>
          <div className="mt-3 flex w-full flex-wrap items-center gap-2 sm:mt-0 sm:w-auto sm:justify-end">
            <span className={roleChipClass(invitation.role)}>
              {roleLabel(invitation.role)}
            </span>
            <span className="chip warn">{invitation.status}</span>
            <span className="chip soft">{invitation.emailDeliveryStatus}</span>
            <Link
              className="btn sm"
              href={
                invitation.canCancel
                  ? "#pending-invitation-actions"
                  : "#access-sources"
              }
            >
              {invitation.canCancel ? "Review invitation" : "Why disabled"}
            </Link>
          </div>
        </div>
      ))}
    </div>
  );
}

function InviteTargets({ settings }: { settings: RepositoryAccessSettings }) {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <AccessCard kicker="Add people" title="Suggested collaborators">
        <div id="invite-people" className="grid gap-3">
          {settings.inviteTargets.users.length > 0 ? (
            settings.inviteTargets.users.map((target) => (
              <div className="list-row py-3" key={target.userId}>
                <div className="flex min-w-0 items-center gap-3">
                  <AccessAvatar label={target.login} src={target.avatarUrl} />
                  <div className="min-w-0">
                    <p className="t-sm font-semibold">{target.login}</p>
                    <p className="t-xs break-words">
                      {target.displayName ?? target.email}
                    </p>
                  </div>
                </div>
                <span className="chip soft">Available</span>
              </div>
            ))
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No user invite targets are available.
            </p>
          )}
        </div>
      </AccessCard>
      <AccessCard kicker="Add teams" title="Suggested teams">
        <div id="invite-teams" className="grid gap-3">
          {settings.inviteTargets.teams.length > 0 ? (
            settings.inviteTargets.teams.map((target) => (
              <div className="list-row py-3" key={target.teamId}>
                <div className="min-w-0">
                  <p className="t-sm font-semibold">{target.name}</p>
                  <p className="t-xs">
                    @{target.slug} ·{" "}
                    <span className="t-num">{target.memberCount}</span> members
                  </p>
                </div>
                <span className="chip soft">Available</span>
              </div>
            ))
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No team invite targets are available.
            </p>
          )}
        </div>
      </AccessCard>
    </div>
  );
}

function RoleDefinitions({ settings }: { settings: RepositoryAccessSettings }) {
  return (
    <AccessCard kicker="Role definitions" title="Repository role hierarchy">
      <div id="role-definitions" className="grid gap-3">
        {settings.roles.map((role) => (
          <div
            className="grid gap-2 py-3 sm:grid-cols-[120px_minmax(0,1fr)]"
            key={role.role}
            style={{ borderTop: "1px solid var(--line-soft)" }}
          >
            <dt>
              <span className={roleChipClass(role.role)}>{role.label}</span>
            </dt>
            <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
              {role.description}
            </dd>
          </div>
        ))}
      </div>
    </AccessCard>
  );
}

function AccessSources() {
  return (
    <AccessCard kicker="Guardrails" title="Why some controls are disabled">
      <div id="access-sources" className="grid gap-3 t-sm">
        <p style={{ color: "var(--ink-2)" }}>
          Owner, inherited organization, and team-derived rows are read-only on
          this page because their source of truth is outside the direct
          collaborator grant.
        </p>
        <p style={{ color: "var(--ink-2)" }}>
          Direct role changes, removals, and pending invitation cancellation are
          wired in the next vertical slice after this read shell.
        </p>
      </div>
    </AccessCard>
  );
}

export function RepositoryAccessSettingsPage({
  query = "",
  repository,
  settingsResult,
}: RepositoryAccessSettingsPageProps) {
  if (!settingsResult.ok) {
    return (
      <RepositoryAccessUnavailable
        repository={repository}
        result={settingsResult}
      />
    );
  }

  const { settings } = settingsResult;
  const base = `/${repository.owner_login}/${repository.name}/settings/access`;
  const normalizedQuery = query.trim();
  const filteredPeople = settings.people.filter((person) =>
    matchesQuery(
      [
        person.login,
        person.displayName,
        person.email,
        person.source,
        person.sourceText,
        person.teamName,
        person.teamSlug,
      ],
      normalizedQuery,
    ),
  );
  const filteredTeams = settings.teams.filter((team) =>
    matchesQuery(
      [team.name, team.slug, team.source, team.sourceText],
      normalizedQuery,
    ),
  );
  const filteredInvitations = settings.invitations.filter((invitation) =>
    matchesQuery(
      [
        invitation.invitedEmail,
        invitation.invitedLogin,
        invitation.status,
        invitation.emailDeliveryStatus,
      ],
      normalizedQuery,
    ),
  );
  const viewerRole = settings.viewerPermission as RepositoryAccessRole;

  return (
    <div className="grid gap-6">
      <div className="flex flex-wrap items-center gap-2">
        <span className="chip active">Access</span>
        <span className={roleChipClass(viewerRole)}>
          Viewer: {roleLabel(viewerRole)}
        </span>
        <span className="chip soft">{settings.visibility}</span>
      </div>

      <AccessSummary settings={settings} />

      {/* biome-ignore lint/a11y/useSemanticElements: React/jsdom do not consistently expose the HTML search landmark yet. */}
      <form action={base} className="card p-4" role="search">
        <label className="t-label" htmlFor="access-filter">
          Filter access
        </label>
        <div className="mt-3 flex flex-col gap-3 sm:flex-row">
          <input
            className="input min-w-0 flex-1"
            defaultValue={normalizedQuery}
            id="access-filter"
            name="q"
            placeholder="Search people, teams, or invitations"
            type="search"
          />
          <button className="btn" type="submit">
            Filter
          </button>
          <Link className="btn" href={base}>
            Clear
          </Link>
        </div>
      </form>

      <nav aria-label="Access sections" className="tabs">
        <a className="tab active" href="#people-access">
          People <span className="t-num">{filteredPeople.length}</span>
        </a>
        <a className="tab" href="#team-access">
          Teams <span className="t-num">{filteredTeams.length}</span>
        </a>
        <a className="tab" href="#pending-invitations">
          Pending <span className="t-num">{filteredInvitations.length}</span>
        </a>
      </nav>

      {normalizedQuery ? (
        <p className="t-sm" role="status" style={{ color: "var(--ink-3)" }}>
          Showing access entries matching "{normalizedQuery}".
        </p>
      ) : null}

      <AccessCard kicker="People" title="People with repository access">
        <div className="mb-4 flex flex-wrap gap-2">
          <Link className="btn sm primary" href="#invite-people">
            Add people
          </Link>
          <Link className="btn sm" href="#role-definitions">
            Role guide
          </Link>
        </div>
        <div id="people-access">
          <PeopleRows
            people={filteredPeople}
            repository={repository}
            roles={settings.roles}
          />
        </div>
      </AccessCard>

      <AccessCard kicker="Teams" title="Organization teams with access">
        <div className="mb-4 flex flex-wrap gap-2">
          <Link className="btn sm primary" href="#invite-teams">
            Add teams
          </Link>
          <Link className="btn sm" href="#access-sources">
            Source rules
          </Link>
        </div>
        <div id="team-access">
          <TeamRows roles={settings.roles} teams={filteredTeams} />
        </div>
      </AccessCard>

      <AccessCard kicker="Pending" title="Pending invitations">
        <div id="pending-invitations">
          <InvitationRows invitations={filteredInvitations} />
        </div>
      </AccessCard>

      <InviteTargets settings={settings} />
      <RoleDefinitions settings={settings} />
      <AccessSources />
    </div>
  );
}
