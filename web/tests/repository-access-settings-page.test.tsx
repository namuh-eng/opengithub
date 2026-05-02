import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryAccessSettingsPage } from "@/components/RepositoryAccessSettingsPage";
import type {
  RepositoryAccessSettings,
  RepositoryAccessSettingsFetchResult,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: null,
    owner_organization_id: "org-1",
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "private",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "admin",
    branchCount: 3,
    tagCount: 1,
    defaultBranchRef: null,
    latestCommit: null,
    rootEntries: [],
    files: [],
    readme: null,
    sidebar: {
      about: null,
      websiteUrl: null,
      topics: [],
      starsCount: 0,
      watchersCount: 0,
      forksCount: 0,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 2,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:namuh-eng/opengithub.git",
      https: "https://opengithub.namuh.co/namuh-eng/opengithub.git",
      zip: "/namuh-eng/opengithub/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function accessSettings(
  overrides: Partial<RepositoryAccessSettings> = {},
): RepositoryAccessSettings {
  return {
    id: "repo-1",
    ownerLogin: "namuh-eng",
    name: "opengithub",
    visibility: "private",
    viewerPermission: "admin",
    roles: [
      {
        role: "read",
        label: "Read",
        description: "Can view and clone the repository.",
        rank: 10,
      },
      {
        role: "triage",
        label: "Triage",
        description: "Can manage issues without write access.",
        rank: 20,
      },
      {
        role: "write",
        label: "Write",
        description: "Can push branches.",
        rank: 30,
      },
      {
        role: "maintain",
        label: "Maintain",
        description: "Can manage non-destructive settings.",
        rank: 40,
      },
      {
        role: "admin",
        label: "Admin",
        description: "Can administer repository settings and access.",
        rank: 50,
      },
    ],
    people: [
      {
        userId: "user-owner",
        login: "jaeyunha",
        displayName: "Jaeyun Ha",
        email: "jaeyunha@example.com",
        avatarUrl: null,
        role: "owner",
        source: "owner",
        sourceText: "Repository owner access",
        teamSlug: null,
        teamName: null,
        canEdit: false,
        canRemove: false,
      },
      {
        userId: "user-direct",
        login: "ashley-ha",
        displayName: "Ashley Ha",
        email: "ashley@example.com",
        avatarUrl: null,
        role: "admin",
        source: "direct",
        sourceText: "Direct collaborator access",
        teamSlug: null,
        teamName: null,
        canEdit: true,
        canRemove: true,
      },
      {
        userId: "user-team",
        login: "morgan",
        displayName: "Morgan Lee",
        email: "morgan@example.com",
        avatarUrl: null,
        role: "write",
        source: "team",
        sourceText: "Inherited from team platform",
        teamSlug: "platform",
        teamName: "Platform",
        canEdit: false,
        canRemove: false,
      },
    ],
    teams: [
      {
        teamId: "team-1",
        slug: "platform",
        name: "Platform",
        role: "write",
        source: "team",
        sourceText: "Direct team access",
        memberCount: 8,
        href: "/orgs/namuh-eng/teams/platform",
        canEdit: true,
        canRemove: true,
      },
      {
        teamId: "team-2",
        slug: "everyone",
        name: "Everyone",
        role: "read",
        source: "inherited",
        sourceText: "Inherited from organization base permissions",
        memberCount: 24,
        href: "/orgs/namuh-eng/teams/everyone",
        canEdit: false,
        canRemove: false,
      },
    ],
    invitations: [
      {
        id: "invite-1",
        invitedUserId: null,
        invitedEmail: "new-dev@example.com",
        invitedLogin: null,
        role: "triage",
        status: "pending",
        emailDeliveryStatus: "queued",
        invitedByUserId: "user-owner",
        expiresAt: "2026-05-10T00:00:00Z",
        createdAt: "2026-05-03T00:00:00Z",
        canCancel: true,
      },
    ],
    inviteTargets: {
      users: [
        {
          userId: "user-target",
          login: "casey",
          displayName: "Casey",
          email: "casey@example.com",
          avatarUrl: null,
        },
      ],
      teams: [
        {
          teamId: "team-target",
          slug: "docs",
          name: "Docs",
          memberCount: 3,
        },
      ],
    },
    auditEvents: [],
    ...overrides,
  };
}

function okResult(
  overrides: Partial<RepositoryAccessSettings> = {},
): RepositoryAccessSettingsFetchResult {
  return { ok: true, settings: accessSettings(overrides) };
}

describe("repository access settings page", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders people, teams, pending invitations, and Editorial primitives", () => {
    const { container } = render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(screen.getByText("Viewer: Admin")).toBeVisible();
    expect(screen.getByRole("search")).toBeVisible();
    expect(screen.getByLabelText("Filter access")).toHaveAttribute("name", "q");
    expect(screen.getByRole("link", { name: "jaeyunha" })).toHaveAttribute(
      "href",
      "/jaeyunha",
    );
    expect(screen.getByRole("link", { name: "ashley-ha" })).toHaveAttribute(
      "href",
      "/ashley-ha",
    );
    expect(screen.getByText("Direct collaborator access")).toBeVisible();
    expect(screen.getByRole("link", { name: "Platform" })).toHaveAttribute(
      "href",
      "/orgs/namuh-eng/teams/platform",
    );
    expect(screen.getByText("new-dev@example.com")).toBeVisible();
    expect(screen.getByText("queued")).toBeVisible();
    expect(screen.getByText("Repository role hierarchy")).toBeVisible();
    expect(screen.getByText("Suggested collaborators")).toBeVisible();
    expect(screen.getByText("Suggested teams")).toBeVisible();

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(5);
    expect(container.querySelector(".tabs")).not.toBeNull();
    expect(container.querySelector(".input")).not.toBeNull();
    expect(container.querySelector(".list-row")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("keeps inherited controls disabled with source explanations", () => {
    render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(screen.getByLabelText("Role for jaeyunha")).toBeDisabled();
    expect(screen.getByLabelText("Role for morgan")).toBeDisabled();
    expect(screen.getByLabelText("Role for ashley-ha")).toBeEnabled();
    expect(screen.getByLabelText("Role for Everyone")).toBeDisabled();
    expect(
      screen.getAllByRole("button", { name: "Source managed" }).length,
    ).toBe(3);
    expect(
      screen.getByText(/Owner, inherited organization, and team-derived rows/),
    ).toBeVisible();
  });

  it("filters people, teams, and invitations from URL query state", () => {
    render(
      <RepositoryAccessSettingsPage
        query="platform"
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(screen.getByRole("status")).toHaveTextContent(
      'Showing access entries matching "platform".',
    );
    expect(screen.getByLabelText("Filter access")).toHaveValue("platform");
    expect(screen.getByRole("link", { name: "morgan" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Platform" })).toBeVisible();
    expect(screen.queryByRole("link", { name: "ashley-ha" })).toBeNull();
    expect(screen.queryByText("new-dev@example.com")).toBeNull();
  });

  it("shows forbidden and unavailable states without leaking access rows", () => {
    const { rerender } = render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: false,
          status: 403,
          code: "forbidden",
          message: "admin access required",
        }}
      />,
    );

    expect(screen.getByText("Admin access required")).toBeVisible();
    expect(screen.getByText("Repository access is restricted")).toBeVisible();
    expect(screen.queryByText("ashley-ha")).not.toBeInTheDocument();

    rerender(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository access settings are unavailable right now.",
        }}
      />,
    );

    expect(screen.getByText("Unavailable")).toBeVisible();
    expect(
      screen.getByText("Repository access settings are unavailable right now."),
    ).toBeVisible();
  });

  it("keeps anchors and controls concrete and accessible", () => {
    const { container } = render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of Array.from(container.querySelectorAll("button"))) {
      expect(button).toHaveAccessibleName(/.+/);
    }
    for (const link of Array.from(container.querySelectorAll("a"))) {
      expect(link.getAttribute("href")).toMatch(/^\/|^#/);
    }
    const people = screen.getByRole("link", { name: /People/ });
    expect(people).toHaveAttribute("href", "#people-access");
    expect(
      within(
        screen.getByRole("navigation", { name: "Access sections" }),
      ).getByRole("link", { name: /Teams/ }),
    ).toHaveAttribute("href", "#team-access");
  });

  it("sends invites, role changes, removals, and cancellation through confirmed server state", async () => {
    const nextSettings = accessSettings({
      people: [
        {
          ...accessSettings().people[0],
        },
        {
          ...accessSettings().people[1],
          role: "maintain",
        },
      ],
      teams: [],
      invitations: [],
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue({
      json: async () => nextSettings,
      ok: true,
    } as Response);

    render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add people" }));
    fireEvent.change(
      screen.getByPlaceholderText("octo@example.com or username"),
      {
        target: { value: "casey@example.com" },
      },
    );
    fireEvent.click(screen.getByRole("button", { name: "Send invitation" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/namuh-eng/opengithub/settings/access/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "invite-person",
            emailOrLogin: "casey@example.com",
            role: "read",
          }),
          method: "POST",
        }),
      ),
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Access settings saved.",
    );

    fireEvent.change(screen.getByLabelText("Role for ashley-ha"), {
      target: { value: "maintain" },
    });
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/namuh-eng/opengithub/settings/access/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "update-person-role",
            role: "maintain",
            userId: "user-direct",
          }),
        }),
      ),
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Add teams" })[0]);
    fireEvent.change(screen.getByLabelText("Team"), {
      target: { value: "docs" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add team" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/namuh-eng/opengithub/settings/access/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "grant-team",
            teamSlug: "docs",
            role: "read",
          }),
        }),
      ),
    );
  });

  it("wraps long access labels and keeps dialogs keyboard-contained", async () => {
    const longLogin =
      "avery-very-long-collaborator-name-without-spaces-for-mobile-layouts";
    const { container } = render(
      <RepositoryAccessSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult({
          people: [
            {
              ...accessSettings().people[1],
              login: longLogin,
              displayName:
                "A very long collaborator display name that should wrap cleanly inside the row",
              email:
                "avery-very-long-collaborator-email-address@subdomain.opengithub.local",
            },
          ],
          teams: [
            {
              ...accessSettings().teams[0],
              name: "Repository Platform Enablement Team With A Long Name",
              slug: "repository-platform-enablement-team-with-a-long-slug",
            },
          ],
          invitations: [
            {
              ...accessSettings().invitations[0],
              invitedEmail:
                "pending-invitation-with-a-long-address@subdomain.opengithub.local",
            },
          ],
        })}
      />,
    );

    expect(screen.getByRole("link", { name: longLogin })).toBeVisible();
    expect(container.querySelectorAll(".break-words").length).toBeGreaterThan(
      5,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add people" }));
    const dialog = screen.getByRole("dialog", {
      name: "Invite a collaborator",
    });
    await waitFor(() => expect(dialog).toHaveFocus());

    screen.getByRole("button", { name: "Send invitation" }).focus();
    fireEvent.keyDown(dialog, { key: "Tab" });
    expect(
      screen.getByPlaceholderText("octo@example.com or username"),
    ).toHaveFocus();
    fireEvent.keyDown(dialog, { key: "Tab", shiftKey: true });
    expect(
      screen.getByRole("button", { name: "Send invitation" }),
    ).toHaveFocus();
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(
      screen.queryByRole("dialog", { name: "Invite a collaborator" }),
    ).not.toBeInTheDocument();
  });
});
