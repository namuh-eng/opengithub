import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import type {
  AuthSession,
  OrganizationIdentity,
  OrganizationRepositoryList,
  OrganizationRepositoryPreview,
  OrganizationSponsorshipState,
  OrganizationTabCounts,
  OrganizationViewerState,
  PublicOrganizationProfile,
} from "@/lib/api";

const session: AuthSession = {
  authenticated: false,
  user: null,
};

type OrganizationOverrides = Partial<
  Omit<
    PublicOrganizationProfile,
    "identity" | "sponsorship" | "tabCounts" | "viewerState"
  >
> & {
  identity?: Partial<OrganizationIdentity>;
  sponsorship?: Partial<OrganizationSponsorshipState>;
  tabCounts?: Partial<OrganizationTabCounts>;
  viewerState?: Partial<OrganizationViewerState>;
};

function repository(
  overrides: Partial<OrganizationRepositoryPreview> = {},
): OrganizationRepositoryPreview {
  return {
    id: "repo-1",
    owner: "namuh",
    name: "opengithub",
    fullName: "namuh/opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "public",
    href: "/namuh/opengithub",
    defaultBranch: "main",
    primaryLanguage: {
      language: "Rust",
      color: "#dea584",
      byteCount: 9000,
    },
    languages: [],
    topics: ["forge"],
    starsCount: 142,
    forksCount: 18,
    openIssuesCount: 5,
    openPullRequestsCount: 2,
    isArchived: false,
    isTemplate: false,
    isMirror: false,
    license: { slug: "mit", name: "MIT" },
    updatedAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

function organization(
  overrides: OrganizationOverrides = {},
): PublicOrganizationProfile {
  const base: PublicOrganizationProfile = {
    identity: {
      id: "org-1",
      slug: "namuh",
      name: "Namuh Engineering",
      description: "Shipping side projects in the open.",
      avatarUrl: null,
      websiteUrl: "https://namuh.co",
      location: "Seoul",
      htmlUrl: "/orgs/namuh",
      profileVisibility: "public",
      isPrivate: false,
      followerCount: 128,
      publicMemberCount: 4,
      repositoryCount: 12,
      createdAt: "2025-02-01T00:00:00Z",
    },
    verifiedDomains: [
      {
        domain: "namuh.co",
        verifiedAt: "2026-01-01T00:00:00Z",
        href: "/docs/verification",
      },
    ],
    pinnedRepositories: [repository()],
    repositoryPreview: [
      repository({
        id: "repo-2",
        name: "ralph",
        fullName: "namuh/ralph",
        description: "Autonomous build loop tooling.",
        href: "/namuh/ralph",
        primaryLanguage: null,
        topics: [],
        starsCount: 64,
        forksCount: 7,
        openIssuesCount: 3,
        openPullRequestsCount: 1,
        license: null,
      }),
    ],
    peoplePreview: [
      {
        id: "user-1",
        login: "ashley",
        name: "Ashley Ha",
        avatarUrl: null,
        href: "/ashley",
        role: "owner",
      },
    ],
    topLanguages: [
      { language: "Rust", color: "#dea584", byteCount: 9000 },
      { language: "TypeScript", color: "#3178c6", byteCount: 3000 },
    ],
    topTopics: [
      {
        topic: "developer-tools",
        count: 4,
        href: "/search?q=topic%3Adeveloper-tools",
      },
    ],
    sponsorship: {
      enabled: false,
      sponsorCount: 0,
      href: null,
      unavailableReason: "Sponsorships are not implemented in opengithub yet.",
    },
    tabCounts: {
      repositories: 12,
      projects: 2,
      packages: 3,
      people: 4,
      sponsoring: 0,
    },
    viewerState: {
      authenticated: false,
      isMember: false,
      role: null,
      canViewInternal: false,
      canAdmin: false,
      isFollowing: false,
    },
  };

  return {
    ...base,
    ...overrides,
    identity: { ...base.identity, ...overrides.identity },
    sponsorship: { ...base.sponsorship, ...overrides.sponsorship },
    tabCounts: { ...base.tabCounts, ...overrides.tabCounts },
    viewerState: { ...base.viewerState, ...overrides.viewerState },
  };
}

function organizationRepositoryList(): OrganizationRepositoryList {
  return {
    items: [
      {
        id: "repo-list-1",
        owner: "namuh",
        name: "opengithub",
        fullName: "namuh/opengithub",
        description: "A rust-first collaboration platform.",
        visibility: "public",
        href: "/namuh/opengithub",
        defaultBranch: "main",
        primaryLanguage: {
          language: "Rust",
          color: "#b7410e",
          byteCount: 9000,
        },
        languages: [],
        topics: ["forge"],
        starsCount: 142,
        forksCount: 18,
        openIssuesCount: 5,
        openPullRequestsCount: 2,
        license: { slug: "mit", name: "MIT" },
        isArchived: false,
        isFork: false,
        isTemplate: true,
        isMirror: false,
        canAdmin: false,
        contributedByViewer: false,
        forkSource: null,
        createdAt: "2026-04-01T00:00:00Z",
        updatedAt: "2026-05-01T00:00:00Z",
      },
    ],
    total: 1,
    page: 1,
    pageSize: 30,
    mode: "repositories",
    filters: {
      query: null,
      repositoryType: "all",
      language: null,
      sort: "updated-desc",
      density: "comfortable",
      page: 1,
      pageSize: 30,
    },
    availableLanguages: [{ value: "Rust", label: "Rust", count: 1 }],
    availableTypes: [{ value: "templates", label: "Templates", count: 1 }],
    tabCounts: {
      repositories: 1,
      projects: 0,
      packages: 0,
      people: 1,
      sponsoring: 0,
    },
    viewerState: {
      authenticated: false,
      isMember: false,
      role: null,
      canViewInternal: false,
      canAdmin: false,
      isFollowing: false,
    },
  };
}

describe("OrganizationProfilePage", () => {
  it("renders the Editorial organization header from the profile contract", () => {
    const { container } = render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization()}
        session={session}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Namuh Engineering" }),
    ).toBeVisible();
    expect(screen.getByText("@namuh")).toBeVisible();
    expect(
      screen.getByText("Shipping side projects in the open."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Verified" })).toHaveAttribute(
      "href",
      "/docs/verification",
    );
    expect(screen.getByRole("link", { name: "Verified" })).toHaveAttribute(
      "title",
      "Verified domain namuh.co",
    );
    expect(
      screen.getByRole("link", { name: "Verified domain namuh.co" }),
    ).toHaveAttribute("href", "/docs/verification");
    expect(
      screen.getByRole("link", { name: "Website namuh.co" }),
    ).toHaveAttribute("href", "https://namuh.co");
    expect(screen.getByText("128 followers")).toBeVisible();
    expect(screen.getByText("4 public members")).toBeVisible();
    expect(screen.getByText("12 repositories")).toBeVisible();
    expect(screen.getByText(/Sponsorships are unavailable/i)).toBeVisible();
    expect(screen.getByRole("button", { name: "Sponsor" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Sponsor" })).toHaveAttribute(
      "type",
      "button",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("keeps organization tabs count-bearing and concrete", () => {
    render(
      <OrganizationProfilePage
        activeTab="repositories"
        profile={organization()}
        repositoryList={organizationRepositoryList()}
        session={session}
      />,
    );

    const tabs = screen.getByRole("navigation", {
      name: "Organization sections",
    });
    expect(
      within(tabs).getByRole("link", { name: "Repositories 12" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=repositories");
    expect(
      within(tabs).getByRole("link", { name: "Projects 2" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=projects");
    expect(
      within(tabs).getByRole("link", { name: "People 4" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=people");
    expect(screen.getByRole("heading", { name: "Repositories" })).toBeVisible();
    expect(screen.getByRole("link", { name: "opengithub" })).toHaveAttribute(
      "href",
      "/namuh/opengithub",
    );
  });

  it("renders overview links and hides admin settings for public viewers", () => {
    render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization()}
        session={session}
      />,
    );

    expect(
      screen.getByRole("link", { name: "Open namuh/opengithub" }),
    ).toHaveAttribute("href", "/namuh/opengithub");
    expect(
      screen.getByRole("link", { name: "Open namuh/ralph" }),
    ).toHaveAttribute("href", "/namuh/ralph");
    expect(
      screen.getByRole("link", { name: "Open Ashley Ha" }),
    ).toHaveAttribute("href", "/ashley");
    expect(
      screen.getByRole("link", { name: "developer-tools, 4 repositories" }),
    ).toHaveAttribute("href", "/search?q=topic%3Adeveloper-tools");
    expect(screen.queryByRole("link", { name: "Settings" })).toBeNull();
  });

  it("shows settings only for admins and redacts private organization tabs", () => {
    render(
      <OrganizationProfilePage
        activeTab="people"
        profile={organization({
          identity: {
            profileVisibility: "private",
            isPrivate: true,
          },
          viewerState: {
            authenticated: true,
            isMember: true,
            role: "owner",
            canViewInternal: true,
            canAdmin: true,
            isFollowing: false,
          },
        })}
        session={session}
      />,
    );

    expect(screen.getByText("Private")).toBeVisible();
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/orgs/namuh/settings",
    );
    const tabs = screen.getByRole("navigation", {
      name: "Organization sections",
    });
    expect(within(tabs).getAllByRole("link")).toHaveLength(1);
    expect(
      within(tabs).getByRole("link", { name: "Overview" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=overview");
  });

  it("renders pinned repository cards with metrics, badges, topics, and stable order", () => {
    render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          pinnedRepositories: [
            repository({
              id: "repo-first",
              fullName: "namuh/first",
              name: "first",
              href: "/namuh/first",
              topics: ["platform", "search", "automation"],
              isTemplate: true,
            }),
            repository({
              id: "repo-second",
              fullName: "namuh/second",
              name: "second",
              href: "/namuh/second",
              visibility: "internal",
              starsCount: 0,
              forksCount: 0,
              openIssuesCount: 0,
              openPullRequestsCount: 0,
              isArchived: true,
              license: null,
            }),
          ],
        })}
        session={session}
      />,
    );

    const cards = screen.getAllByRole("link", { name: /^Open namuh\// });
    expect(cards.map((card) => card.getAttribute("href"))).toEqual([
      "/namuh/first",
      "/namuh/second",
      "/namuh/ralph",
    ]);
    expect(screen.getByText("142 stars")).toBeVisible();
    expect(screen.getByText("18 forks")).toBeVisible();
    expect(screen.getByText("5 open issues")).toBeVisible();
    expect(screen.getByText("2 open pull requests")).toBeVisible();
    expect(screen.getByText("MIT")).toBeVisible();
    expect(screen.getByText("platform")).toBeVisible();
    expect(screen.getByText("Template")).toBeVisible();
    expect(screen.getByText("Archived")).toBeVisible();
    expect(screen.getByText("internal")).toBeVisible();
  });

  it("uses concrete empty-state actions for repository preview states", () => {
    render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          pinnedRepositories: [],
          repositoryPreview: [],
          viewerState: {
            authenticated: true,
            isMember: true,
            role: "owner",
            canViewInternal: true,
            canAdmin: true,
            isFollowing: false,
          },
        })}
        session={session}
      />,
    );

    expect(
      screen.getByText("No pinned repositories are visible yet."),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Browse repositories" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=repositories");
    expect(
      screen.getByText("No repositories are visible to this viewer."),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Create repository" }),
    ).toHaveAttribute("href", "/new");
  });

  it("redacts public people roles but shows member-visible roles and counts", () => {
    const { rerender } = render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          peoplePreview: [
            {
              id: "user-1",
              login: "ashley",
              name: "Ashley Ha",
              avatarUrl: null,
              href: "/ashley",
              role: null,
            },
          ],
          tabCounts: { people: 1 },
        })}
        session={session}
      />,
    );

    expect(screen.getByText("1 visible person.")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Open Ashley Ha" }),
    ).toHaveAttribute("href", "/ashley");
    expect(screen.queryByText("owner")).toBeNull();

    rerender(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          peoplePreview: [
            {
              id: "user-1",
              login: "ashley",
              name: "Ashley Ha",
              avatarUrl: null,
              href: "/ashley",
              role: "owner",
            },
          ],
          tabCounts: { people: 2 },
          viewerState: {
            authenticated: true,
            isMember: true,
            role: "owner",
            canViewInternal: true,
            canAdmin: true,
            isFollowing: false,
          },
        })}
        session={session}
      />,
    );

    expect(
      screen.getByText("2 visible people including private members."),
    ).toBeVisible();
    expect(screen.getAllByText("owner").length).toBeGreaterThan(0);
    expect(screen.getByText("Owner view")).toBeVisible();
  });

  it("renders sorted language bars, counted topic links, and disabled sponsoring state", () => {
    render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          topLanguages: [
            { language: "Rust", color: "#dea584", byteCount: 9000 },
            { language: "TypeScript", color: "#3178c6", byteCount: 3000 },
          ],
          topTopics: [
            {
              topic: "automation",
              count: 8,
              href: "/orgs/namuh/repositories?q=topic%3Aautomation",
            },
            {
              topic: "developer-tools",
              count: 4,
              href: "/orgs/namuh/repositories?q=topic%3Adeveloper-tools",
            },
          ],
        })}
        session={session}
      />,
    );

    expect(
      screen.getByLabelText("Rust 75% of visible organization code"),
    ).toBeVisible();
    expect(
      screen.getByLabelText("TypeScript 25% of visible organization code"),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "automation, 8 repositories" }),
    ).toHaveAttribute("href", "/orgs/namuh/repositories?q=topic%3Aautomation");
    expect(
      screen.getByRole("button", { name: "Sponsor preview unavailable" }),
    ).toBeDisabled();
    expect(
      screen.getByText("Sponsorships are not available for organizations yet."),
    ).toBeVisible();
  });

  it("keeps final empty secondary panels accessible without dead controls", () => {
    const { container } = render(
      <OrganizationProfilePage
        activeTab="overview"
        profile={organization({
          pinnedRepositories: [],
          repositoryPreview: [],
          peoplePreview: [],
          topLanguages: [],
          topTopics: [],
          tabCounts: {
            repositories: 0,
            projects: 0,
            packages: 0,
            people: 0,
            sponsoring: 0,
          },
          identity: {
            repositoryCount: 0,
            publicMemberCount: 0,
          },
        })}
        session={session}
      />,
    );

    expect(
      screen.getByText("No pinned repositories are visible yet."),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Browse repositories" }),
    ).toHaveAttribute("href", "/orgs/namuh?tab=repositories");
    expect(
      screen.getByText("No repositories are visible to this viewer."),
    ).toBeVisible();
    expect(
      screen.queryByRole("link", { name: "Create repository" }),
    ).toBeNull();
    expect(screen.getByText("No public members are visible.")).toBeVisible();
    expect(screen.getByText("No language data yet.")).toBeVisible();
    expect(screen.getByText("No topics have been published.")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Sponsor preview unavailable" }),
    ).toBeDisabled();
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });
});
