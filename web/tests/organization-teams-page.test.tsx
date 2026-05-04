import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OrganizationTeamCreatePage } from "@/components/OrganizationTeamCreatePage";
import { OrganizationTeamDetailPage } from "@/components/OrganizationTeamDetailPage";
import { OrganizationTeamsPage } from "@/components/OrganizationTeamsPage";
import type {
  OrganizationTeamDetail,
  OrganizationTeamSummary,
  OrganizationTeamsDirectory,
} from "@/lib/api";

const pushMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock }),
}));

afterEach(() => {
  vi.unstubAllGlobals();
  pushMock.mockReset();
});

function team(
  overrides: Partial<OrganizationTeamSummary> = {},
): OrganizationTeamSummary {
  return {
    id: "team-1",
    slug: "platform",
    name: "Platform",
    description: "Core runtime and infrastructure owners.",
    href: "/orgs/namuh/teams/platform",
    visibility: "visible",
    mentionable: true,
    notificationsEnabled: true,
    memberCount: 4,
    repositoryCount: 3,
    childTeamCount: 1,
    parent: null,
    viewerCapabilities: {
      canView: true,
      canManage: false,
      canJoin: false,
      canMention: true,
      isMember: true,
    },
    updatedAt: "2026-05-04T00:00:00Z",
    ...overrides,
  };
}

function teamsDirectory(
  overrides: Partial<OrganizationTeamsDirectory> = {},
): OrganizationTeamsDirectory {
  const items = overrides.items ?? [team()];
  return {
    organization: {
      id: "org-1",
      slug: "namuh",
      name: "Namuh Engineering",
      href: "/orgs/namuh",
      settingsHref: "/organizations/namuh/settings/profile",
    },
    items,
    total: overrides.total ?? items.length,
    page: overrides.page ?? 1,
    pageSize: overrides.pageSize ?? 30,
    filters: {
      query: null,
      visibility: "all",
      page: 1,
      pageSize: 30,
      ...overrides.filters,
    },
    counts: {
      total: items.length,
      visible: items.filter((item) => item.visibility !== "secret").length,
      secret: items.filter((item) => item.visibility === "secret").length,
      memberTeams: items.filter((item) => item.viewerCapabilities.isMember)
        .length,
      ...overrides.counts,
    },
    parentOptions: [],
    emptyState: {
      title: "Organize people by team",
      newTeamHref: "/orgs/namuh/teams/new",
      learnMoreHref: "/docs/api#organization-teams",
      columns: [
        {
          title: "Flexible repository access",
          body: "Grant repository permissions to a team once.",
        },
        {
          title: "Request-to-join teams",
          body: "Give members a discoverable home for shared work.",
        },
        {
          title: "Team mentions",
          body: "Mention visible teams to notify the right group.",
        },
      ],
    },
    viewerState: {
      role: "owner",
      canAdminTeams: true,
      canCreateTeam: true,
      canViewSecretTeams: true,
    },
    ...overrides,
  };
}

function teamDetail(
  overrides: Partial<OrganizationTeamDetail> = {},
): OrganizationTeamDetail {
  return {
    organization: {
      id: "org-1",
      slug: "namuh",
      name: "Namuh Engineering",
      href: "/orgs/namuh",
      settingsHref: "/organizations/namuh/settings/profile",
    },
    team: team(),
    hierarchy: {
      parentChain: [
        {
          id: "team-parent",
          slug: "engineering",
          name: "Engineering",
          href: "/orgs/namuh/teams/engineering",
          visibility: "visible",
        },
      ],
      inheritedRepositoryCount: 1,
      directRepositoryCount: 1,
      childTeamCount: 1,
    },
    members: [
      {
        userId: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
        role: "maintainer",
        href: "/mona",
      },
    ],
    repositories: [
      {
        repositoryId: "repo-1",
        name: "runtime",
        fullName: "namuh/runtime",
        href: "/namuh/runtime/settings/access",
        visibility: "private",
        role: "write",
        source: "team",
        sourceTeamSlug: "platform",
        inherited: false,
      },
      {
        repositoryId: "repo-2",
        name: "frontend",
        fullName: "namuh/frontend",
        href: "/namuh/frontend/settings/access",
        visibility: "private",
        role: "read",
        source: "team",
        sourceTeamSlug: "engineering",
        inherited: true,
      },
    ],
    childTeams: [
      team({
        id: "team-child",
        slug: "frontend",
        name: "Frontend",
        href: "/orgs/namuh/teams/frontend",
        parent: {
          id: "team-1",
          slug: "platform",
          name: "Platform",
          href: "/orgs/namuh/teams/platform",
          visibility: "visible",
        },
      }),
    ],
    mentionState: {
      mentionable: true,
      notificationsEnabled: false,
      fanoutState:
        "team mentions stay indexed, but member fanout is suppressed unless direct mention, participation, or review request rules subscribe the user.",
      recentMentions: [],
    },
    viewerState: {
      role: "owner",
      canAdminTeams: true,
      canCreateTeam: true,
      canViewSecretTeams: true,
    },
    ...overrides,
  };
}

describe("OrganizationTeamsPage", () => {
  it("renders populated team rows with filters, counts, and concrete navigation", () => {
    const { container } = render(
      <OrganizationTeamsPage
        directory={teamsDirectory({
          items: [
            team(),
            team({
              id: "team-2",
              slug: "security",
              name: "Security Response",
              description: null,
              href: "/orgs/namuh/teams/security",
              visibility: "secret",
              mentionable: false,
              notificationsEnabled: false,
              memberCount: 2,
              repositoryCount: 1,
              childTeamCount: 0,
              parent: {
                id: "team-1",
                slug: "platform",
                name: "Platform",
                href: "/orgs/namuh/teams/platform",
                visibility: "visible",
              },
              viewerCapabilities: {
                canView: true,
                canManage: true,
                canJoin: false,
                canMention: true,
                isMember: false,
              },
            }),
          ],
          total: 2,
          counts: {
            total: 2,
            visible: 1,
            secret: 1,
            memberTeams: 1,
          },
          filters: {
            query: "sec",
            visibility: "secret",
            page: 1,
            pageSize: 30,
          },
        })}
        org="namuh"
      />,
    );

    expect(screen.getByRole("heading", { name: "Teams" })).toBeVisible();
    expect(screen.getByLabelText("Search organization teams")).toHaveValue(
      "sec",
    );
    expect(screen.getByLabelText("Filter team visibility")).toHaveValue(
      "secret",
    );
    expect(screen.getByRole("button", { name: "Filter" })).toHaveAttribute(
      "type",
      "submit",
    );
    expect(screen.getByRole("link", { name: "New team" })).toHaveAttribute(
      "href",
      "/orgs/namuh/teams/new",
    );
    expect(screen.getByRole("link", { name: "Open Platform" })).toHaveAttribute(
      "href",
      "/orgs/namuh/teams/platform",
    );
    expect(
      screen.getByRole("link", { name: "Open Security Response" }),
    ).toHaveAttribute("href", "/orgs/namuh/teams/security");
    expect(screen.getAllByText("Secret").length).toBeGreaterThan(0);
    expect(screen.getByText("Parent")).toBeVisible();
    expect(screen.getAllByText("@platform").length).toBeGreaterThan(0);
    expect(screen.getByText("Notifications off")).toBeVisible();
    expect(screen.getByText("Search: sec")).toBeVisible();
    expect(screen.getByText("Mentionable")).toBeVisible();

    const sideNav = screen.getByRole("complementary", {
      name: "Organization teams navigation",
    });
    expect(
      within(sideNav).getByRole("link", { name: "Members" }),
    ).toHaveAttribute("href", "/orgs/namuh/people");
    expect(
      within(sideNav).getByRole("link", { name: "Repositories" }),
    ).toHaveAttribute("href", "/orgs/namuh/repositories");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(
      Array.from(container.querySelectorAll("button")).every(
        (button) => button.type === "submit" || button.disabled,
      ),
    ).toBe(true);
  });

  it("renders the required empty-state columns and CTAs", () => {
    render(
      <OrganizationTeamsPage
        directory={teamsDirectory({
          items: [],
          total: 0,
          counts: { total: 0, visible: 0, secret: 0, memberTeams: 0 },
        })}
        org="namuh"
      />,
    );

    expect(screen.getByText("Organize people by team")).toBeVisible();
    expect(screen.getByText("Flexible repository access")).toBeVisible();
    expect(screen.getByText("Request-to-join teams")).toBeVisible();
    expect(screen.getByText("Team mentions")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "New team" })[0],
    ).toHaveAttribute("href", "/orgs/namuh/teams/new");
    expect(screen.getByRole("link", { name: "Learn more" })).toHaveAttribute(
      "href",
      "/docs/api#organization-teams",
    );
  });

  it("hides secret tabs and creation actions when the viewer lacks capabilities", () => {
    render(
      <OrganizationTeamsPage
        directory={teamsDirectory({
          viewerState: {
            role: "member",
            canAdminTeams: false,
            canCreateTeam: false,
            canViewSecretTeams: false,
          },
          items: [
            team({
              viewerCapabilities: {
                ...team().viewerCapabilities,
                isMember: false,
              },
            }),
          ],
        })}
        org="namuh"
      />,
    );

    expect(screen.queryByRole("link", { name: "New team" })).toBeNull();
    expect(screen.queryByRole("link", { name: /Secret/ })).toBeNull();
    expect(
      screen.queryByText("Owners and admins can see visible and secret teams."),
    ).toBeNull();
    expect(
      screen.getByText(
        "Members see visible teams and secret teams they belong to.",
      ),
    ).toBeVisible();
  });

  it("renders filtered empty recovery and pagination links that preserve query state", () => {
    render(
      <OrganizationTeamsPage
        directory={teamsDirectory({
          items: [],
          total: 35,
          page: 2,
          pageSize: 10,
          counts: { total: 35, visible: 30, secret: 5, memberTeams: 2 },
          filters: {
            query: "frontend",
            visibility: "visible",
            page: 2,
            pageSize: 10,
          },
        })}
        org="namuh"
      />,
    );

    expect(screen.getByText("No teams matched these filters.")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Clear filters" })[0],
    ).toHaveAttribute("href", "/orgs/namuh/teams");
    const pagination = screen.getByRole("navigation", {
      name: "Teams pagination",
    });
    expect(
      within(pagination).getByRole("link", { name: "Previous" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/teams?q=frontend&visibility=visible&pageSize=10",
    );
    expect(
      within(pagination).getByRole("link", { name: "Next" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/teams?q=frontend&visibility=visible&page=3&pageSize=10",
    );
  });

  it("renders the team create form with parent options and submits concrete payloads", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        destinationHref: "/orgs/namuh/teams/release-infrastructure",
      }),
    });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <OrganizationTeamCreatePage
        directory={teamsDirectory({
          parentOptions: [
            {
              id: "team-1",
              slug: "platform",
              name: "Platform",
              href: "/orgs/namuh/teams/platform",
              visibility: "visible",
            },
          ],
        })}
        org="namuh"
      />,
    );

    fireEvent.change(screen.getByLabelText("Team name"), {
      target: { value: "Release Infrastructure!" },
    });
    fireEvent.change(screen.getByLabelText("Description"), {
      target: { value: "Owns release trains." },
    });
    fireEvent.change(screen.getByLabelText("Parent team"), {
      target: { value: "team-1" },
    });
    fireEvent.click(screen.getByLabelText("Disabled"));
    fireEvent.click(screen.getByRole("button", { name: "Create team" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith("/orgs/namuh/teams/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          name: "Release Infrastructure!",
          description: "Owns release trains.",
          parentTeamId: "team-1",
          visibility: "visible",
          notificationsEnabled: false,
        }),
      }),
    );
    await waitFor(() =>
      expect(pushMock).toHaveBeenCalledWith(
        "/orgs/namuh/teams/release-infrastructure",
      ),
    );
    expect(screen.getByText("@release-infrastructure")).toBeVisible();
    expect(screen.getByRole("link", { name: "Cancel" })).toHaveAttribute(
      "href",
      "/orgs/namuh/teams",
    );
  });

  it("blocks secret nested teams before submitting", () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    render(
      <OrganizationTeamCreatePage
        directory={teamsDirectory({
          parentOptions: [
            {
              id: "team-1",
              slug: "platform",
              name: "Platform",
              href: "/orgs/namuh/teams/platform",
              visibility: "visible",
            },
          ],
        })}
        org="namuh"
      />,
    );

    fireEvent.change(screen.getByLabelText("Team name"), {
      target: { value: "Private Child" },
    });
    fireEvent.click(screen.getByLabelText("Secret"));
    fireEvent.change(screen.getByLabelText("Parent team"), {
      target: { value: "team-1" },
    });

    expect(screen.getByRole("button", { name: "Create team" })).toBeDisabled();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("renders team detail with members, inherited repository access, child teams, and mention state", () => {
    const { container } = render(
      <OrganizationTeamDetailPage detail={teamDetail()} org="namuh" />,
    );

    expect(screen.getByRole("heading", { name: "Platform" })).toBeVisible();
    expect(screen.getByText("@platform")).toBeVisible();
    expect(screen.getByText("namuh/runtime")).toBeVisible();
    expect(screen.getByText("namuh/frontend")).toBeVisible();
    expect(screen.getByText("Inherited from @engineering")).toBeVisible();
    expect(screen.getByRole("link", { name: /Mona/ })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getByRole("link", { name: /Frontend/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/teams/frontend",
    );
    expect(screen.getByText("Fanout suppressed")).toBeVisible();
    expect(screen.getByRole("link", { name: "All teams" })).toHaveAttribute(
      "href",
      "/orgs/namuh/teams",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.querySelector("button")).toBeNull();
  });
});
