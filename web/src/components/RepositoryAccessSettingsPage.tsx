"use client";

import Image from "next/image";
import Link from "next/link";
import type { KeyboardEvent } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
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

type WritableRole = Exclude<RepositoryAccessRole, "owner">;
type AccessAction =
  | { action: "invite-person"; emailOrLogin: string; role: WritableRole }
  | { action: "grant-team"; teamSlug: string; role: WritableRole }
  | { action: "update-person-role"; userId: string; role: WritableRole }
  | { action: "update-team-role"; teamId: string; role: WritableRole }
  | { action: "remove-person"; userId: string }
  | { action: "remove-team"; teamId: string }
  | { action: "cancel-invitation"; invitationId: string };
type DialogState =
  | { kind: "person" }
  | { kind: "team" }
  | { kind: "remove-person"; person: RepositoryAccessPerson }
  | { kind: "remove-team"; team: RepositoryAccessTeam }
  | { kind: "cancel-invitation"; invitation: RepositoryInvitation }
  | null;

const writableRoleValues: WritableRole[] = [
  "read",
  "triage",
  "write",
  "maintain",
  "admin",
];

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
  if (role === "owner" || role === "admin") return "chip accent";
  if (role === "maintain" || role === "write") return "chip ok";
  if (role === "triage") return "chip info";
  return "chip soft";
}

function sourceChipClass(source: string) {
  if (source === "owner") return "chip accent";
  if (source === "team" || source === "inherited") return "chip warn";
  return "chip soft";
}

function matchesQuery(values: Array<string | null | undefined>, query: string) {
  if (!query) return true;
  const normalized = query.toLowerCase();
  return values.some((value) => value?.toLowerCase().includes(normalized));
}

function formattedDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function AccessAvatar({ label, src }: { label: string; src: string | null }) {
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

function AccessCard({
  children,
  kicker,
  title,
}: {
  children: React.ReactNode;
  kicker: string;
  title: string;
}) {
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

function useDialogFocus(open: boolean, onClose: () => void) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const dialog = ref.current;
    const previous = document.activeElement;
    dialog?.focus();

    return () => {
      if (previous instanceof HTMLElement) {
        previous.focus();
      }
    };
  }, [open]);

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== "Tab") return;

    const dialog = ref.current;
    if (!dialog) return;
    const focusable = Array.from(
      dialog.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((element) => !element.hasAttribute("disabled"));
    if (focusable.length === 0) {
      event.preventDefault();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  return { onKeyDown, ref };
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

function roleOptions(settings: RepositoryAccessSettings) {
  return settings.roles.filter((role) =>
    writableRoleValues.includes(role.role as WritableRole),
  );
}

function RoleSelect({
  disabled,
  label,
  onChange,
  roles,
  value,
}: {
  disabled: boolean;
  label: string;
  onChange: (role: WritableRole) => void;
  roles: RepositoryAccessSettings["roles"];
  value: RepositoryAccessRole;
}) {
  return (
    <>
      <label className="sr-only" htmlFor={label}>
        {label}
      </label>
      <select
        className="input min-w-32"
        disabled={disabled}
        id={label}
        onChange={(event) =>
          onChange(event.currentTarget.value as WritableRole)
        }
        value={value === "owner" ? "admin" : value}
      >
        {roles.map((role) => (
          <option key={role.role} value={role.role}>
            {role.label}
          </option>
        ))}
      </select>
    </>
  );
}

function ConfirmDialog({
  busy,
  dialog,
  error,
  onClose,
  onConfirm,
}: {
  busy: boolean;
  dialog: Exclude<DialogState, null | { kind: "person" } | { kind: "team" }>;
  error: string | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const target =
    dialog.kind === "remove-person"
      ? dialog.person.login
      : dialog.kind === "remove-team"
        ? dialog.team.name
        : (dialog.invitation.invitedLogin ?? dialog.invitation.invitedEmail);
  const label =
    dialog.kind === "cancel-invitation"
      ? "Cancel pending invitation"
      : "Remove direct access";
  const { onKeyDown, ref } = useDialogFocus(true, onClose);

  return (
    <div
      aria-labelledby="access-confirm-title"
      className="card fixed left-1/2 top-1/2 z-50 w-[min(92vw,440px)] -translate-x-1/2 -translate-y-1/2 p-5"
      onKeyDown={onKeyDown}
      ref={ref}
      role="dialog"
      style={{ background: "var(--surface)" }}
      tabIndex={-1}
    >
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Confirmation
      </p>
      <h2 className="t-h3 mt-2" id="access-confirm-title">
        {label}
      </h2>
      <p className="t-sm mt-3" style={{ color: "var(--ink-2)" }}>
        Type confirmed server state will remove or cancel access for{" "}
        <strong>{target}</strong>. The list updates only after the API accepts
        the change.
      </p>
      {error ? (
        <p className="t-sm mt-3" role="alert" style={{ color: "var(--err)" }}>
          {error}
        </p>
      ) : null}
      <div className="mt-5 flex flex-wrap justify-end gap-2">
        <button className="btn sm" onClick={onClose} type="button">
          Keep access
        </button>
        <button
          className="btn sm primary"
          disabled={busy}
          onClick={onConfirm}
          type="button"
        >
          {busy ? "Saving..." : label}
        </button>
      </div>
    </div>
  );
}

function InviteDialog({
  busy,
  dialog,
  error,
  onClose,
  onSubmit,
  settings,
}: {
  busy: boolean;
  dialog: Extract<DialogState, { kind: "person" } | { kind: "team" }>;
  error: string | null;
  onClose: () => void;
  onSubmit: (action: AccessAction) => void;
  settings: RepositoryAccessSettings;
}) {
  const roles = roleOptions(settings);
  const defaultRole = "read" as WritableRole;
  const isPerson = dialog.kind === "person";
  const { onKeyDown, ref } = useDialogFocus(true, onClose);
  return (
    <div
      aria-labelledby="access-invite-title"
      className="card fixed left-1/2 top-1/2 z-50 w-[min(92vw,520px)] -translate-x-1/2 -translate-y-1/2 p-5"
      onKeyDown={onKeyDown}
      ref={ref}
      role="dialog"
      style={{ background: "var(--surface)" }}
      tabIndex={-1}
    >
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {isPerson ? "Add people" : "Add teams"}
      </p>
      <h2 className="t-h3 mt-2" id="access-invite-title">
        {isPerson ? "Invite a collaborator" : "Grant a team access"}
      </h2>
      <form
        className="mt-4 grid gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          const data = new FormData(event.currentTarget);
          const role = String(data.get("role") ?? defaultRole) as WritableRole;
          if (isPerson) {
            onSubmit({
              action: "invite-person",
              emailOrLogin: String(data.get("emailOrLogin") ?? ""),
              role,
            });
          } else {
            onSubmit({
              action: "grant-team",
              teamSlug: String(data.get("teamSlug") ?? ""),
              role,
            });
          }
        }}
      >
        {isPerson ? (
          <label className="grid gap-2" htmlFor="access-email-or-login">
            <span className="t-label">User or email</span>
            <input
              className="input"
              id="access-email-or-login"
              list="access-user-targets"
              name="emailOrLogin"
              placeholder="octo@example.com or username"
              required
            />
            <datalist id="access-user-targets">
              {settings.inviteTargets.users.map((target) => (
                <option key={target.userId} value={target.email}>
                  {target.login}
                </option>
              ))}
            </datalist>
          </label>
        ) : (
          <label className="grid gap-2" htmlFor="access-team-slug">
            <span className="t-label">Team</span>
            <select
              className="input"
              id="access-team-slug"
              name="teamSlug"
              required
            >
              <option value="">Select a team</option>
              {settings.inviteTargets.teams.map((target) => (
                <option key={target.teamId} value={target.slug}>
                  {target.name} (@{target.slug})
                </option>
              ))}
            </select>
          </label>
        )}
        <label className="grid gap-2" htmlFor="access-invite-role">
          <span className="t-label">Role</span>
          <select
            className="input"
            defaultValue={defaultRole}
            id="access-invite-role"
            name="role"
          >
            {roles.map((role) => (
              <option key={role.role} value={role.role}>
                {role.label}
              </option>
            ))}
          </select>
        </label>
        {error ? (
          <p className="t-sm" role="alert" style={{ color: "var(--err)" }}>
            {error}
          </p>
        ) : null}
        <div className="flex flex-wrap justify-end gap-2">
          <button className="btn sm" onClick={onClose} type="button">
            Cancel
          </button>
          <button className="btn sm primary" disabled={busy} type="submit">
            {busy ? "Saving..." : isPerson ? "Send invitation" : "Add team"}
          </button>
        </div>
      </form>
    </div>
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

  return (
    <RepositoryAccessSettingsContent
      query={query}
      repository={repository}
      initialSettings={settingsResult.settings}
    />
  );
}

function RepositoryAccessSettingsContent({
  initialSettings,
  query = "",
  repository,
}: {
  initialSettings: RepositoryAccessSettings;
  query?: string;
  repository: RepositoryOverview;
}) {
  const [settings, setSettings] = useState(initialSettings);
  const [dialog, setDialog] = useState<DialogState>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const actionUrl = `/${repository.owner_login}/${repository.name}/settings/access/actions`;
  const base = `/${repository.owner_login}/${repository.name}/settings/access`;
  const normalizedQuery = query.trim();
  const roles = roleOptions(settings);
  const filteredPeople = useMemo(
    () =>
      settings.people.filter((person) =>
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
      ),
    [settings.people, normalizedQuery],
  );
  const filteredTeams = useMemo(
    () =>
      settings.teams.filter((team) =>
        matchesQuery(
          [team.name, team.slug, team.source, team.sourceText],
          normalizedQuery,
        ),
      ),
    [settings.teams, normalizedQuery],
  );
  const filteredInvitations = useMemo(
    () =>
      settings.invitations.filter((invitation) =>
        matchesQuery(
          [
            invitation.invitedEmail,
            invitation.invitedLogin,
            invitation.status,
            invitation.emailDeliveryStatus,
          ],
          normalizedQuery,
        ),
      ),
    [settings.invitations, normalizedQuery],
  );

  async function mutate(action: AccessAction, success: string) {
    setBusy(true);
    setError(null);
    setNotice(null);
    const response = await fetch(actionUrl, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(action),
    });
    const body = await response.json().catch(() => null);
    setBusy(false);
    if (!response.ok) {
      setError(
        body?.error?.message ?? "Repository access update failed. Try again.",
      );
      return;
    }
    setSettings(body as RepositoryAccessSettings);
    setDialog(null);
    setNotice(success);
  }

  const viewerRole = settings.viewerPermission as RepositoryAccessRole;

  return (
    <div className="grid gap-6">
      {dialog?.kind === "person" || dialog?.kind === "team" ? (
        <InviteDialog
          busy={busy}
          dialog={dialog}
          error={error}
          onClose={() => {
            setDialog(null);
            setError(null);
          }}
          onSubmit={(action) => mutate(action, "Access settings saved.")}
          settings={settings}
        />
      ) : null}
      {dialog?.kind === "remove-person" ||
      dialog?.kind === "remove-team" ||
      dialog?.kind === "cancel-invitation" ? (
        <ConfirmDialog
          busy={busy}
          dialog={dialog}
          error={error}
          onClose={() => {
            setDialog(null);
            setError(null);
          }}
          onConfirm={() => {
            if (dialog.kind === "remove-person") {
              void mutate(
                { action: "remove-person", userId: dialog.person.userId },
                `${dialog.person.login} was removed from direct access.`,
              );
            } else if (dialog.kind === "remove-team") {
              void mutate(
                { action: "remove-team", teamId: dialog.team.teamId },
                `${dialog.team.name} team access was removed.`,
              );
            } else {
              void mutate(
                {
                  action: "cancel-invitation",
                  invitationId: dialog.invitation.id,
                },
                "Pending invitation was canceled.",
              );
            }
          }}
        />
      ) : null}

      <div className="flex flex-wrap items-center gap-2">
        <span className="chip active">Access</span>
        <span className={roleChipClass(viewerRole)}>
          Viewer: {roleLabel(viewerRole)}
        </span>
        <span className="chip soft">{settings.visibility}</span>
      </div>

      {notice ? (
        <p className="chip ok w-fit" role="status">
          {notice}
        </p>
      ) : null}

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
            <p
              className="t-num mt-2 text-2xl"
              style={{ color: "var(--ink-1)" }}
            >
              {value}
            </p>
          </div>
        ))}
      </div>

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
          <button
            className="btn sm primary"
            onClick={() => setDialog({ kind: "person" })}
            type="button"
          >
            Add people
          </button>
          <Link className="btn sm" href="#role-definitions">
            Role guide
          </Link>
        </div>
        <div className="grid gap-0" id="people-access">
          {filteredPeople.length > 0 ? (
            filteredPeople.map((person) => (
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
                  </div>
                </div>
                <div className="mt-3 flex w-full flex-wrap items-center gap-2 sm:mt-0 sm:w-auto sm:justify-end">
                  <span className={roleChipClass(person.role)}>
                    {roleLabel(person.role)}
                  </span>
                  <span className={sourceChipClass(person.source)}>
                    {person.source}
                  </span>
                  <RoleSelect
                    disabled={!person.canEdit || busy}
                    label={`Role for ${person.login}`}
                    onChange={(role) =>
                      void mutate(
                        {
                          action: "update-person-role",
                          role,
                          userId: person.userId,
                        },
                        `${person.login} role was updated.`,
                      )
                    }
                    roles={roles}
                    value={person.role}
                  />
                  <button
                    className="btn sm"
                    disabled={!person.canRemove || busy}
                    onClick={() => setDialog({ kind: "remove-person", person })}
                    type="button"
                  >
                    {person.canRemove ? "Remove" : "Source managed"}
                  </button>
                </div>
              </div>
            ))
          ) : (
            <div
              className="rounded-md p-5"
              style={{ background: "var(--surface-2)" }}
            >
              <p className="t-sm font-semibold">No outside collaborators</p>
              <p className="t-xs mt-1">
                Add a person when you need direct repository access outside
                inherited owner or team paths.
              </p>
              <button
                className="btn sm mt-4"
                onClick={() => setDialog({ kind: "person" })}
                type="button"
              >
                Add people
              </button>
            </div>
          )}
        </div>
      </AccessCard>

      <AccessCard kicker="Teams" title="Organization teams with access">
        <div className="mb-4 flex flex-wrap gap-2">
          <button
            className="btn sm primary"
            onClick={() => setDialog({ kind: "team" })}
            type="button"
          >
            Add teams
          </button>
          <Link className="btn sm" href="#access-sources">
            Source rules
          </Link>
        </div>
        <div className="grid gap-0" id="team-access">
          {filteredTeams.length > 0 ? (
            filteredTeams.map((team) => (
              <div className="list-row py-4" key={team.teamId}>
                <div className="min-w-0 flex-1">
                  <Link
                    className="t-sm font-semibold hover:underline"
                    href={team.href}
                  >
                    {team.name}
                  </Link>
                  <p className="t-xs mt-1 break-words">
                    @{team.slug} ·{" "}
                    <span className="t-num">{team.memberCount}</span> members
                  </p>
                  <p className="t-xs mt-1 break-words">{team.sourceText}</p>
                </div>
                <div className="mt-3 flex w-full flex-wrap items-center gap-2 sm:mt-0 sm:w-auto sm:justify-end">
                  <span className={roleChipClass(team.role)}>
                    {roleLabel(team.role)}
                  </span>
                  <span className={sourceChipClass(team.source)}>
                    {team.source}
                  </span>
                  <RoleSelect
                    disabled={!team.canEdit || busy}
                    label={`Role for ${team.name}`}
                    onChange={(role) =>
                      void mutate(
                        {
                          action: "update-team-role",
                          role,
                          teamId: team.teamId,
                        },
                        `${team.name} team role was updated.`,
                      )
                    }
                    roles={roles}
                    value={team.role}
                  />
                  <button
                    className="btn sm"
                    disabled={!team.canRemove || busy}
                    onClick={() => setDialog({ kind: "remove-team", team })}
                    type="button"
                  >
                    {team.canRemove ? "Remove" : "Source managed"}
                  </button>
                </div>
              </div>
            ))
          ) : (
            <div
              className="rounded-md p-5"
              style={{ background: "var(--surface-2)" }}
            >
              <p className="t-sm font-semibold">No team grants</p>
              <p className="t-xs mt-1">
                Organization teams that can access this repository will appear
                here with their member counts and source.
              </p>
              <button
                className="btn sm mt-4"
                onClick={() => setDialog({ kind: "team" })}
                type="button"
              >
                Add teams
              </button>
            </div>
          )}
        </div>
      </AccessCard>

      <AccessCard kicker="Pending" title="Pending invitations">
        <div className="grid gap-0" id="pending-invitations">
          {filteredInvitations.length > 0 ? (
            filteredInvitations.map((invitation) => (
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
                  <span className="chip soft">
                    {invitation.emailDeliveryStatus}
                  </span>
                  <button
                    className="btn sm"
                    disabled={!invitation.canCancel || busy}
                    onClick={() =>
                      setDialog({ kind: "cancel-invitation", invitation })
                    }
                    type="button"
                  >
                    {invitation.canCancel
                      ? "Cancel invitation"
                      : "Source managed"}
                  </button>
                </div>
              </div>
            ))
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              There are no pending repository invitations.
            </p>
          )}
        </div>
      </AccessCard>

      <div className="grid gap-4 md:grid-cols-2">
        <AccessCard kicker="Add people" title="Suggested collaborators">
          <div id="invite-people" className="grid gap-3">
            {settings.inviteTargets.users.length > 0 ? (
              settings.inviteTargets.users.map((target) => (
                <button
                  className="list-row py-3 text-left"
                  key={target.userId}
                  onClick={() => setDialog({ kind: "person" })}
                  type="button"
                >
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
                </button>
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
                <button
                  className="list-row py-3 text-left"
                  key={target.teamId}
                  onClick={() => setDialog({ kind: "team" })}
                  type="button"
                >
                  <div className="min-w-0">
                    <p className="t-sm font-semibold">{target.name}</p>
                    <p className="t-xs">
                      @{target.slug} ·{" "}
                      <span className="t-num">{target.memberCount}</span>{" "}
                      members
                    </p>
                  </div>
                  <span className="chip soft">Available</span>
                </button>
              ))
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                No team invite targets are available.
              </p>
            )}
          </div>
        </AccessCard>
      </div>

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

      <AccessCard kicker="Guardrails" title="Why some controls are disabled">
        <div id="access-sources" className="grid gap-3 t-sm">
          <p style={{ color: "var(--ink-2)" }}>
            Owner, inherited organization, and team-derived rows are read-only
            on this page because their source of truth is outside the direct
            collaborator grant.
          </p>
          <p style={{ color: "var(--ink-2)" }}>
            Direct collaborator role changes, direct team grants, removals, and
            pending invitation cancellation save through the Rust access API and
            refresh this page from confirmed server state.
          </p>
        </div>
      </AccessCard>
    </div>
  );
}
