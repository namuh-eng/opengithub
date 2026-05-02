import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import type {
  AuthSession,
  OrganizationIdentity,
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
    pinnedRepositories: [
      {
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
      },
    ],
    repositoryPreview: [
      {
        id: "repo-2",
        owner: "namuh",
        name: "ralph",
        fullName: "namuh/ralph",
        description: "Autonomous build loop tooling.",
        visibility: "public",
        href: "/namuh/ralph",
        defaultBranch: "main",
        primaryLanguage: null,
        languages: [],
        topics: [],
        starsCount: 64,
        forksCount: 7,
        openIssuesCount: 3,
        openPullRequestsCount: 1,
        isArchived: false,
        isTemplate: false,
        isMirror: false,
        license: null,
        updatedAt: "2026-05-01T00:00:00Z",
      },
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
    expect(
      screen.getByRole("link", { name: "Website namuh.co" }),
    ).toHaveAttribute("href", "https://namuh.co");
    expect(screen.getByText("128 followers")).toBeVisible();
    expect(screen.getByText("4 public members")).toBeVisible();
    expect(screen.getByText("12 repositories")).toBeVisible();
    expect(screen.getByText(/Sponsorships are unavailable/i)).toBeVisible();
    expect(screen.getByRole("button", { name: "Sponsor" })).toBeDisabled();
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("keeps organization tabs count-bearing and concrete", () => {
    render(
      <OrganizationProfilePage
        activeTab="repositories"
        profile={organization()}
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
    expect(
      screen.getByRole("heading", { name: "Repositories for namuh" }),
    ).toBeVisible();
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
      screen.getByRole("link", { name: "namuh/opengithub" }),
    ).toHaveAttribute("href", "/namuh/opengithub");
    expect(screen.getByRole("link", { name: /namuh\/ralph/ })).toHaveAttribute(
      "href",
      "/namuh/ralph",
    );
    expect(screen.getByRole("link", { name: "Ashley Ha" })).toHaveAttribute(
      "href",
      "/ashley",
    );
    expect(
      screen.getByRole("link", { name: "developer-tools" }),
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
});
