"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type {
  OrganizationInvitationRow,
  OrganizationPeopleAdmin,
  OrganizationPeopleAdminRow,
  OrganizationPeopleAdminTab,
  OrganizationPeopleAdminTabParam,
} from "@/lib/api";
import {
  organizationPeopleListHref,
  organizationSettingsHref,
} from "@/lib/navigation";

type OrganizationPeopleAdminPageProps = {
  admin: OrganizationPeopleAdmin;
  org: string;
};

type TabConfig = {
  label: string;
  value: OrganizationPeopleAdminTab;
  param: OrganizationPeopleAdminTabParam;
  count: (admin: OrganizationPeopleAdmin) => number;
};

const TABS: TabConfig[] = [
  {
    label: "Members",
    value: "members",
    param: "members",
    count: (admin) => admin.counts.members,
  },
  {
    label: "Outside collaborators",
    value: "outsideCollaborators",
    param: "outside_collaborators",
    count: (admin) => admin.counts.outsideCollaborators,
  },
  {
    label: "Pending collaborators",
    value: "pendingCollaborators",
    param: "pending_collaborators",
    count: (admin) => admin.counts.pendingCollaborators,
  },
  {
    label: "Invitations",
    value: "invitations",
    param: "invitations",
    count: (admin) => admin.counts.invitations,
  },
  {
    label: "Failed invitations",
    value: "failedInvitations",
    param: "failed_invitations",
    count: (admin) => admin.counts.failedInvitations,
  },
  {
    label: "Security Managers",
    value: "securityManagers",
    param: "security_managers",
    count: (admin) => admin.counts.securityManagers,
  },
];

function tabParam(
  tab: OrganizationPeopleAdminTab,
): OrganizationPeopleAdminTabParam {
  return TABS.find((item) => item.value === tab)?.param ?? "members";
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Recently";
  }
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
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

function sourceLabel(source: string) {
  if (source === "outside_collaborator") {
    return "Outside collaborator";
  }
  if (source === "security_manager") {
    return "Security manager";
  }
  return "Organization";
}

function personInitial(person: OrganizationPeopleAdminRow) {
  return (person.displayName?.trim() || person.login).slice(0, 1).toUpperCase();
}

function SearchSummary({
  admin,
  org,
}: {
  admin: OrganizationPeopleAdmin;
  org: string;
}) {
  const query = admin.filters.query;
  if (!query) {
    return null;
  }

  return (
    <div className="mt-3 flex flex-wrap items-center gap-2">
      <span className="t-label" style={{ color: "var(--ink-3)" }}>
        Active filters
      </span>
      <Link
        className="chip active no-underline"
        href={organizationPeopleListHref(
          org,
          {
            pageSize: admin.filters.pageSize,
            query,
            tab: tabParam(admin.tab),
          },
          { page: null, q: null },
        )}
      >
        Search: {query} x
      </Link>
    </div>
  );
}

function MemberRow({
  onToggle,
  row,
  selected,
}: {
  onToggle: (checked: boolean) => void;
  row: OrganizationPeopleAdminRow;
  selected: boolean;
}) {
  const [visibilityOpen, setVisibilityOpen] = useState(false);
  const [actionsOpen, setActionsOpen] = useState(false);
  const displayName = row.displayName?.trim() || row.login;

  return (
    <article className="list-row py-4">
      <div className="grid gap-3 lg:grid-cols-[auto_auto_minmax(0,1fr)_auto] lg:items-center">
        <input
          aria-label={`Select ${displayName}`}
          checked={selected}
          className="size-4"
          onChange={(event) => onToggle(event.currentTarget.checked)}
          type="checkbox"
        />
        {row.avatarUrl ? (
          <span
            aria-hidden="true"
            className="av lg shrink-0"
            style={{
              backgroundImage: `url(${row.avatarUrl})`,
              backgroundPosition: "center",
              backgroundSize: "cover",
            }}
          />
        ) : (
          <span aria-hidden="true" className="av lg shrink-0">
            {personInitial(row)}
          </span>
        )}
        <div className="min-w-0">
          <Link
            aria-label={`Open ${displayName}`}
            className="t-h3 no-underline"
            href={row.href}
          >
            {displayName}
          </Link>
          <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
            @{row.login}
          </p>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className={row.twoFactorEnabled ? "chip ok" : "chip warn"}>
              2FA {row.twoFactorEnabled ? "on" : "off"}
            </span>
            <span className={row.hasActiveSession ? "chip ok" : "chip soft"}>
              {row.hasActiveSession ? "Active session" : "No active session"}
            </span>
            <span className="chip soft">
              {sourceLabel(row.membershipSource)}
            </span>
            {row.actionState.finalOwner ? (
              <span className="chip warn">Final owner</span>
            ) : null}
          </div>
        </div>
        <div className="grid gap-2 lg:min-w-[260px]">
          <div className="flex flex-wrap justify-start gap-2 lg:justify-end">
            <button
              aria-expanded={visibilityOpen}
              className="btn sm"
              onClick={() => setVisibilityOpen((open) => !open)}
              type="button"
            >
              Visibility: {row.membershipVisibility}
            </button>
            <span className="chip soft">{roleLabel(row.role)}</span>
            <button
              aria-expanded={actionsOpen}
              className="btn sm ghost"
              onClick={() => setActionsOpen((open) => !open)}
              type="button"
            >
              Row actions
            </button>
          </div>
          <p className="t-xs text-left lg:text-right">
            {row.teamCount.toLocaleString()} teams ·{" "}
            {row.rolesCount.toLocaleString()} roles · joined{" "}
            {formatDate(row.joinedAt)}
          </p>
          {visibilityOpen ? (
            <div
              className="card grid gap-2 p-3"
              role="menu"
              style={{ background: "var(--surface-2)" }}
            >
              <button className="btn sm" disabled type="button">
                Public membership
              </button>
              <button className="btn sm" disabled type="button">
                Private membership
              </button>
              <p className="t-xs">
                Visibility changes are wired in the next membership mutation
                phase.
              </p>
            </div>
          ) : null}
          {actionsOpen ? (
            <div
              className="card grid gap-2 p-3"
              role="menu"
              style={{ background: "var(--surface-2)" }}
            >
              <button
                className="btn sm"
                disabled={!row.actionState.canChangeRole}
                type="button"
              >
                Change role
              </button>
              <button
                className="btn sm"
                disabled={!row.actionState.canRemove}
                type="button"
              >
                Remove from organization
              </button>
              <p className="t-xs">
                {row.actionState.finalOwner
                  ? "Final owners cannot be demoted or removed."
                  : "Role and removal confirmations are enabled in the membership mutation phase."}
              </p>
            </div>
          ) : null}
        </div>
      </div>
    </article>
  );
}

function InvitationRow({
  invitation,
}: {
  invitation: OrganizationInvitationRow;
}) {
  const label = invitation.invitedLogin
    ? `@${invitation.invitedLogin}`
    : invitation.invitedEmail;

  return (
    <article className="list-row py-4">
      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
        <div className="min-w-0">
          <p className="t-h3 truncate">{label}</p>
          <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
            {invitation.invitedEmail}
          </p>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className="chip soft">{roleLabel(invitation.role)}</span>
            <span className="chip soft">
              {invitation.teamCount.toLocaleString()} teams
            </span>
            <span
              className={
                invitation.emailDeliveryStatus === "failed"
                  ? "chip err"
                  : invitation.emailDeliveryStatus === "degraded"
                    ? "chip warn"
                    : "chip ok"
              }
            >
              Email {invitation.emailDeliveryStatus}
            </span>
          </div>
          {invitation.emailDeliveryError ? (
            <p className="t-xs mt-2">{invitation.emailDeliveryError}</p>
          ) : null}
        </div>
        <div className="flex flex-wrap gap-2 md:justify-end">
          <button
            className="btn sm"
            disabled={!invitation.canRetry}
            type="button"
          >
            Retry
          </button>
          <button
            className="btn sm"
            disabled={!invitation.canCancel}
            type="button"
          >
            Cancel
          </button>
          <span className="t-xs self-center">
            Expires {formatDate(invitation.expiresAt)}
          </span>
        </div>
      </div>
    </article>
  );
}

export function OrganizationPeopleAdminPage({
  admin,
  org,
}: OrganizationPeopleAdminPageProps) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [exportOpen, setExportOpen] = useState(false);
  const [bulkOpen, setBulkOpen] = useState(false);
  const [inviteOpen, setInviteOpen] = useState(false);
  const activeTabParam = tabParam(admin.tab);
  const visibleRows = admin.rows.items;
  const visibleInvitations = admin.invitations.items;
  const total =
    admin.tab === "members" ||
    admin.tab === "outsideCollaborators" ||
    admin.tab === "securityManagers"
      ? admin.rows.total
      : admin.invitations.total;
  const page = admin.filters.page;
  const pageSize = admin.filters.pageSize;
  const showingFrom = total === 0 ? 0 : (page - 1) * pageSize + 1;
  const showingTo = Math.min(page * pageSize, total);
  const selectedCount = selected.size;
  const tabFilters = useMemo(
    () => ({
      pageSize,
      query: admin.filters.query,
      tab: activeTabParam,
    }),
    [activeTabParam, admin.filters.query, pageSize],
  );

  function hrefForTab(param: OrganizationPeopleAdminTabParam) {
    return organizationPeopleListHref(
      org,
      {
        pageSize,
        query: admin.filters.query,
        tab: param,
      },
      { page: null },
    );
  }

  function setRowSelected(id: string, checked: boolean) {
    setSelected((current) => {
      const next = new Set(current);
      if (checked) {
        next.add(id);
      } else {
        next.delete(id);
      }
      return next;
    });
  }

  return (
    <section
      aria-labelledby="organization-people-admin-title"
      className="grid gap-5"
    >
      <div className="card overflow-hidden">
        <div className="border-b border-[var(--line)] p-5">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Organization people
              </p>
              <h2 className="t-h2 mt-1" id="organization-people-admin-title">
                People administration
              </h2>
              <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
                {admin.organization.name} · signed in as{" "}
                {roleLabel(admin.viewerState.role).toLowerCase()}
              </p>
            </div>
            <Link
              className="btn sm ghost no-underline"
              href={organizationSettingsHref(org)}
            >
              Organization settings
            </Link>
          </div>

          <nav aria-label="People administration tabs" className="tabs mt-5">
            {TABS.map((tab) => (
              <Link
                aria-current={admin.tab === tab.value ? "page" : undefined}
                className={admin.tab === tab.value ? "tab active" : "tab"}
                href={hrefForTab(tab.param)}
                key={tab.value}
              >
                {tab.label}{" "}
                <span className="t-num">
                  {tab.count(admin).toLocaleString()}
                </span>
              </Link>
            ))}
          </nav>

          <form
            action={`/orgs/${encodeURIComponent(org)}/people`}
            className="mt-5 grid gap-3 lg:grid-cols-[minmax(180px,1fr)_auto_auto_auto]"
          >
            <input name="tab" type="hidden" value={activeTabParam} />
            {pageSize !== 30 ? (
              <input name="pageSize" type="hidden" value={pageSize} />
            ) : null}
            <label className="grid gap-1">
              <span className="t-label">Member search</span>
              <input
                aria-label="Search organization people"
                className="input"
                defaultValue={admin.filters.query ?? ""}
                name="q"
                placeholder="Search members, invitations, or emails..."
                type="search"
              />
            </label>
            <div className="flex items-end">
              <button className="btn primary w-full" type="submit">
                Filter
              </button>
            </div>
            <div className="relative flex items-end">
              <button
                aria-expanded={bulkOpen}
                className="btn w-full"
                disabled={selectedCount === 0}
                onClick={() => setBulkOpen((open) => !open)}
                type="button"
              >
                Bulk action
                {selectedCount > 0 ? ` (${selectedCount})` : ""}
              </button>
            </div>
            <div className="flex flex-wrap items-end gap-2">
              <button
                aria-expanded={exportOpen}
                className="btn"
                onClick={() => setExportOpen((open) => !open)}
                type="button"
              >
                Export
              </button>
              <button
                aria-expanded={inviteOpen}
                className="btn accent"
                onClick={() => setInviteOpen((open) => !open)}
                type="button"
              >
                Invite member
              </button>
            </div>
          </form>

          <SearchSummary admin={admin} org={org} />

          {bulkOpen && selectedCount > 0 ? (
            <div
              className="card mt-3 grid gap-2 p-3"
              style={{ background: "var(--surface-2)" }}
            >
              <p className="t-sm">
                {selectedCount.toLocaleString()} selected. Bulk membership
                mutations are enabled in the membership mutation phase.
              </p>
              <button className="btn sm" disabled type="button">
                Change selected roles
              </button>
            </div>
          ) : null}

          {exportOpen ? (
            <section
              aria-label="Export organization people"
              className="card mt-3 flex flex-wrap gap-2 p-3"
              style={{ background: "var(--surface-2)" }}
            >
              {admin.exports.map((item) =>
                item.available ? (
                  <a
                    className="btn sm no-underline"
                    href={item.href}
                    key={item.format}
                  >
                    Export {item.format.toUpperCase()}
                  </a>
                ) : (
                  <button
                    className="btn sm"
                    disabled
                    key={item.format}
                    type="button"
                  >
                    Export {item.format.toUpperCase()}
                  </button>
                ),
              )}
            </section>
          ) : null}

          {inviteOpen ? (
            <div
              aria-label="Invite member dialog"
              className="card mt-3 grid gap-3 p-4"
              role="dialog"
              style={{ background: "var(--surface-2)" }}
            >
              <div>
                <p className="t-h3">Invite member</p>
                <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
                  Invitation sending, role choice, and team assignment are wired
                  to SES in the next phase.
                </p>
              </div>
              <label className="grid gap-1">
                <span className="t-label">Username or email</span>
                <input
                  className="input"
                  disabled
                  placeholder="member@example.com"
                  type="text"
                />
              </label>
              <button
                className="btn sm"
                onClick={() => setInviteOpen(false)}
                type="button"
              >
                Close
              </button>
            </div>
          ) : null}
        </div>

        <div className="border-b border-[var(--line)] px-5 py-3">
          <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {showingFrom}-{showingTo} of {total.toLocaleString()}
          </p>
        </div>

        {visibleRows.length > 0 ? (
          <div className="px-5">
            {visibleRows.map((row) => (
              <MemberRow
                key={row.userId}
                onToggle={(checked) => setRowSelected(row.userId, checked)}
                row={row}
                selected={selected.has(row.userId)}
              />
            ))}
          </div>
        ) : visibleInvitations.length > 0 ? (
          <div className="px-5">
            {visibleInvitations.map((invitation) => (
              <InvitationRow invitation={invitation} key={invitation.id} />
            ))}
          </div>
        ) : (
          <div className="p-8">
            <p className="t-h3">No people matched this view.</p>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Change tabs or clear the search to return to the full organization
              people roster.
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
          aria-label="People administration pagination"
          className="flex flex-wrap items-center justify-between gap-3 border-t border-[var(--line)] p-5"
        >
          <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            Page {page.toLocaleString()}
          </p>
          <div className="flex gap-2">
            {page > 1 ? (
              <Link
                className="btn sm ghost"
                href={organizationPeopleListHref(org, tabFilters, {
                  page: String(page - 1),
                })}
              >
                Previous
              </Link>
            ) : (
              <button className="btn sm" disabled type="button">
                Previous
              </button>
            )}
            {showingTo < total ? (
              <Link
                className="btn sm ghost"
                href={organizationPeopleListHref(org, tabFilters, {
                  page: String(page + 1),
                })}
              >
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
