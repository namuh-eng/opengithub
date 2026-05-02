import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { UserProfilePage } from "@/components/UserProfilePage";
import type {
  AuthSession,
  ProfileContributionSummary,
  ProfileIdentity,
  ProfileTabCounts,
  ProfileViewerState,
  PublicUserProfile,
} from "@/lib/api";

const session: AuthSession = {
  authenticated: false,
  user: null,
};

type ProfileOverrides = Partial<
  Omit<
    PublicUserProfile,
    "contributionSummary" | "identity" | "tabCounts" | "viewerState"
  >
> & {
  contributionSummary?: Partial<ProfileContributionSummary>;
  identity?: Partial<ProfileIdentity>;
  tabCounts?: Partial<ProfileTabCounts>;
  viewerState?: Partial<ProfileViewerState>;
};

function profile(overrides: ProfileOverrides = {}): PublicUserProfile {
  const base: PublicUserProfile = {
    identity: {
      id: "user-1",
      login: "ashley",
      name: "Ashley Ha",
      avatarUrl: null,
      bio: "Building calm developer tools at Namuh.",
      company: "Namuh",
      location: "San Francisco",
      websiteUrl: "https://namuh.co",
      htmlUrl: "/ashley",
      profileVisibility: "public",
      isPrivate: false,
      joinedAt: "2025-02-01T00:00:00Z",
      followerCount: 42,
      followingCount: 18,
    },
    readme: {
      body: "# Hello\nI ship collaboration tools.",
      renderedHtml: "<h1>Hello</h1>",
      updatedAt: "2026-05-01T00:00:00Z",
    },
    pinnedRepositories: [
      {
        id: "repo-1",
        owner: "ashley",
        name: "opengithub",
        description: "A rust-first collaboration platform.",
        visibility: "public",
        href: "/ashley/opengithub",
        defaultBranch: "main",
        primaryLanguage: {
          language: "Rust",
          color: "#dea584",
          byteCount: 1024,
        },
        languages: [],
        starsCount: 128,
        forksCount: 12,
        updatedAt: "2026-05-01T00:00:00Z",
      },
      {
        id: "repo-2",
        owner: "ashley",
        name: "notes",
        description: "Profile notes and experiments.",
        visibility: "public",
        href: "/ashley/notes",
        defaultBranch: "main",
        primaryLanguage: null,
        languages: [],
        starsCount: 9,
        forksCount: 1,
        updatedAt: "2026-05-01T00:00:00Z",
      },
    ],
    achievements: [
      {
        slug: "arctic-code",
        name: "Archive contributor",
        description: "Contributed to preserved open source",
        icon: "A",
        awardedAt: "2026-01-01T00:00:00Z",
      },
    ],
    organizations: [
      {
        id: "org-1",
        slug: "namuh",
        name: "Namuh",
        avatarUrl: null,
        href: "/orgs/namuh",
      },
    ],
    contributionSummary: {
      total: 84,
      year: 2026,
      days: [
        { date: "2026-04-29", count: 0, intensity: 0 },
        { date: "2026-04-30", count: 3, intensity: 2 },
        { date: "2026-05-01", count: 8, intensity: 4 },
      ],
      recentEvents: [
        {
          id: "event-1",
          eventType: "commit",
          title: "Pushed profile page",
          targetHref: "/ashley/opengithub/commit/abc",
          occurredAt: "2026-05-01T00:00:00Z",
          repository: {
            owner: "ashley",
            name: "opengithub",
            href: "/ashley/opengithub",
          },
        },
      ],
    },
    tabCounts: {
      repositories: 24,
      projects: 2,
      packages: 3,
      stars: 138,
    },
    viewerState: {
      authenticated: false,
      isSelf: false,
      isFollowing: false,
      isBlocking: false,
      canFollow: false,
      canBlock: false,
      canReport: false,
    },
  };

  return {
    ...base,
    ...overrides,
    identity: { ...base.identity, ...overrides.identity },
    contributionSummary: {
      ...base.contributionSummary,
      ...overrides.contributionSummary,
    },
    tabCounts: { ...base.tabCounts, ...overrides.tabCounts },
    viewerState: { ...base.viewerState, ...overrides.viewerState },
  };
}

describe("UserProfilePage", () => {
  it("renders the Editorial public overview from real profile data", () => {
    const { container } = render(
      <UserProfilePage
        activeTab="overview"
        profile={profile()}
        session={session}
      />,
    );

    expect(screen.getByRole("heading", { name: "Ashley Ha" })).toBeVisible();
    expect(screen.getByText("@ashley")).toBeVisible();
    expect(screen.getByText("42 followers · 18 following")).toBeVisible();
    expect(screen.getByRole("heading", { name: "README" })).toBeVisible();
    expect(screen.getByText(/I ship collaboration tools/)).toBeVisible();
    const pins = screen
      .getByRole("heading", { name: "Pinned repositories" })
      .closest("section");
    expect(pins).not.toBeNull();
    expect(
      within(pins as HTMLElement).getByRole("link", { name: /opengithub/ }),
    ).toHaveAttribute("href", "/ashley/opengithub");
    expect(screen.getByRole("link", { name: "Namuh" })).toHaveAttribute(
      "href",
      "/orgs/namuh",
    );
    expect(screen.getByText("Archive contributor")).toBeVisible();
    expect(
      screen.getByLabelText("8 contributions on May 1, 2026"),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "2026", current: "page" }),
    ).toHaveAttribute("href", "/ashley?year=2026");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("keeps tab URLs stable and count-bearing", () => {
    render(
      <UserProfilePage
        activeTab="repositories"
        profile={profile()}
        session={session}
      />,
    );

    const tabs = screen.getByRole("navigation", { name: "Profile sections" });
    expect(
      within(tabs).getByRole("link", { name: "Repositories 24" }),
    ).toHaveAttribute("href", "/ashley?tab=repositories");
    expect(
      within(tabs).getByRole("link", { name: "Stars 138" }),
    ).toHaveAttribute("href", "/ashley?tab=stars");
    expect(
      screen.getByRole("heading", { name: "Repositories for ashley" }),
    ).toBeVisible();
  });

  it("redacts private profile secondary data", () => {
    render(
      <UserProfilePage
        activeTab="stars"
        profile={profile({
          identity: {
            profileVisibility: "private",
            isPrivate: true,
            followerCount: null,
            followingCount: null,
          },
          pinnedRepositories: [],
          achievements: [],
          organizations: [],
          contributionSummary: {
            total: 0,
            year: 2026,
            days: [],
            recentEvents: [],
          },
          tabCounts: {
            repositories: 0,
            projects: 0,
            packages: 0,
            stars: 0,
          },
        })}
        session={session}
      />,
    );

    expect(screen.getByText("Private profile")).toBeVisible();
    expect(screen.queryByText(/followers/)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Pinned repositories" }),
    ).toBeNull();
    expect(
      screen.queryByRole("heading", { name: /contributions this year/ }),
    ).toBeNull();
    const tabs = screen.getByRole("navigation", { name: "Profile sections" });
    expect(within(tabs).getAllByRole("link")).toHaveLength(1);
    expect(
      within(tabs).getByRole("link", { name: "Overview" }),
    ).toHaveAttribute("href", "/ashley?tab=overview");
  });
});
