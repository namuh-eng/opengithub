import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizationPeopleAdminPage } from "@/components/OrganizationPeopleAdminPage";
import type {
  OrganizationInvitationRow,
  OrganizationPeopleAdmin,
  OrganizationPeopleAdminRow,
} from "@/lib/api";

function member(
  overrides: Partial<OrganizationPeopleAdminRow> = {},
): OrganizationPeopleAdminRow {
  return {
    userId: "user-1",
    login: "ashley",
    displayName: "Ashley Ha",
    avatarUrl: null,
    href: "/ashley",
    role: "owner",
    membershipVisibility: "public",
    outsideCollaborator: false,
    securityManager: false,
    twoFactorEnabled: true,
    hasActiveSession: true,
    teamCount: 2,
    rolesCount: 3,
    membershipSource: "organization",
    joinedAt: "2026-04-01T00:00:00Z",
    actionState: {
      canChangeVisibility: true,
      canChangeRole: false,
      canRemove: false,
      finalOwner: true,
      reason: "final_owner",
    },
    ...overrides,
  };
}

function invitation(
  overrides: Partial<OrganizationInvitationRow> = {},
): OrganizationInvitationRow {
  return {
    id: "invite-1",
    invitedUserId: null,
    invitedLogin: null,
    invitedEmail: "pending@example.com",
    role: "member",
    teamCount: 1,
    status: "pending",
    emailDeliveryStatus: "degraded",
    emailDeliveryError: null,
    invitedByUserId: "user-1",
    expiresAt: "2026-05-11T00:00:00Z",
    createdAt: "2026-05-04T00:00:00Z",
    canRetry: false,
    canCancel: true,
    ...overrides,
  };
}

function adminPeople(
  overrides: Partial<OrganizationPeopleAdmin> = {},
): OrganizationPeopleAdmin {
  const rows = overrides.rows?.items ?? [
    member(),
    member({
      userId: "user-2",
      login: "jaeyun",
      displayName: "Jaeyun Ha",
      role: "admin",
      membershipVisibility: "private",
      twoFactorEnabled: false,
      teamCount: 1,
      rolesCount: 2,
      actionState: {
        canChangeVisibility: true,
        canChangeRole: true,
        canRemove: true,
        finalOwner: false,
        reason: null,
      },
    }),
  ];

  return {
    organization: {
      id: "org-1",
      slug: "namuh",
      name: "Namuh Engineering",
      href: "/orgs/namuh",
      settingsHref: "/organizations/namuh/settings/profile",
    },
    tab: "members",
    filters: {
      page: 1,
      pageSize: 30,
      query: null,
      tab: "members",
    },
    counts: {
      members: 2,
      outsideCollaborators: 1,
      pendingCollaborators: 1,
      invitations: 1,
      failedInvitations: 1,
      securityManagers: 0,
    },
    rows: {
      items: rows,
      page: 1,
      pageSize: 30,
      total: rows.length,
    },
    invitations: {
      items: [],
      page: 1,
      pageSize: 30,
      total: 0,
    },
    exports: [
      {
        available: true,
        format: "json",
        href: "/api/orgs/namuh/people/export?format=json&tab=members",
      },
      {
        available: true,
        format: "csv",
        href: "/api/orgs/namuh/people/export?format=csv&tab=members",
      },
    ],
    viewerState: {
      role: "owner",
      canAdminPeople: true,
      canInvite: true,
      canExport: true,
    },
    ...overrides,
  };
}

describe("OrganizationPeopleAdminPage", () => {
  it("renders tabs, URL-backed search, export links, and no dead anchors", () => {
    const { container } = render(
      <OrganizationPeopleAdminPage admin={adminPeople()} org="namuh" />,
    );

    expect(
      screen.getByRole("heading", { name: "People administration" }),
    ).toBeVisible();
    const tabs = screen.getByRole("navigation", {
      name: "People administration tabs",
    });
    expect(
      within(tabs).getByRole("link", { name: "Members 2" }),
    ).toHaveAttribute("href", "/orgs/namuh/people");
    expect(
      within(tabs).getByRole("link", { name: "Outside collaborators 1" }),
    ).toHaveAttribute("href", "/orgs/namuh/people?tab=outside_collaborators");
    expect(screen.getByLabelText("Search organization people")).toHaveAttribute(
      "name",
      "q",
    );
    expect(screen.getByRole("button", { name: "Filter" })).toHaveAttribute(
      "type",
      "submit",
    );

    fireEvent.click(screen.getByRole("button", { name: "Export" }));
    const exports = screen.getByLabelText("Export organization people");
    expect(
      within(exports).getByRole("link", { name: "Export JSON" }),
    ).toHaveAttribute(
      "href",
      "/api/orgs/namuh/people/export?format=json&tab=members",
    );
    expect(
      within(exports).getByRole("link", { name: "Export CSV" }),
    ).toHaveAttribute(
      "href",
      "/api/orgs/namuh/people/export?format=csv&tab=members",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("enables bulk controls only after row selection and exposes safe row menus", () => {
    render(<OrganizationPeopleAdminPage admin={adminPeople()} org="namuh" />);

    expect(screen.getByRole("button", { name: "Bulk action" })).toBeDisabled();
    fireEvent.click(screen.getByLabelText("Select Jaeyun Ha"));
    expect(
      screen.getByRole("button", { name: "Bulk action (1)" }),
    ).not.toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Bulk action (1)" }));
    expect(screen.getByText(/Bulk membership mutations/)).toBeVisible();

    const rowActions = screen.getAllByRole("button", { name: "Row actions" });
    fireEvent.click(rowActions[0]);
    expect(
      screen.getByText("Final owners cannot be demoted or removed."),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Change role" })).toBeDisabled();

    fireEvent.click(
      screen.getByRole("button", { name: "Visibility: private" }),
    );
    expect(screen.getByText(/Visibility changes are wired/)).toBeVisible();
  });

  it("renders invitation tabs without leaking token material", () => {
    const { container } = render(
      <OrganizationPeopleAdminPage
        admin={adminPeople({
          tab: "failedInvitations",
          filters: {
            page: 1,
            pageSize: 30,
            query: "failed",
            tab: "failedInvitations",
          },
          rows: { items: [], page: 1, pageSize: 30, total: 0 },
          invitations: {
            items: [
              invitation({
                id: "invite-2",
                invitedEmail: "failed@example.com",
                status: "failed",
                emailDeliveryStatus: "failed",
                emailDeliveryError: "SES sandbox rejected recipient",
                canRetry: true,
                canCancel: true,
              }),
            ],
            page: 1,
            pageSize: 30,
            total: 1,
          },
        })}
        org="namuh"
      />,
    );

    expect(screen.getAllByText("failed@example.com")).toHaveLength(2);
    expect(screen.getByText("Email failed")).toBeVisible();
    expect(screen.getByRole("button", { name: "Retry" })).not.toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).not.toBeDisabled();
    expect(
      screen.getByRole("link", { name: "Search: failed x" }),
    ).toHaveAttribute("href", "/orgs/namuh/people?tab=failed_invitations");
    expect(container).not.toHaveTextContent("sha256:");
    expect(container).not.toHaveTextContent("token");
  });

  it("opens a concrete invite unavailable dialog instead of an inert CTA", () => {
    render(<OrganizationPeopleAdminPage admin={adminPeople()} org="namuh" />);

    fireEvent.click(screen.getByRole("button", { name: "Invite member" }));
    const dialog = screen.getByLabelText("Invite member dialog");
    expect(within(dialog).getByText("Invite member")).toBeVisible();
    expect(
      within(dialog).getByPlaceholderText("member@example.com"),
    ).toBeDisabled();
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(screen.queryByLabelText("Invite member dialog")).toBeNull();
  });
});
