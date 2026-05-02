import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { UserProfilePage } from "@/components/UserProfilePage";
import type {
  AuthSession,
  ProfileRepositoryList,
  ProfileRepositoryListItem,
  PublicUserProfile,
} from "@/lib/api";

const session: AuthSession = {
  authenticated: false,
  user: null,
};

function profile(): PublicUserProfile {
  return {
    identity: {
      id: "user-1",
      login: "ashley",
      name: "Ashley Ha",
      avatarUrl: null,
      bio: "Building calm developer tools.",
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
    readme: null,
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
      repositories: 2,
      projects: 0,
      packages: 0,
      stars: 4,
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
}

function repository(
  overrides: Partial<ProfileRepositoryListItem> = {},
): ProfileRepositoryListItem {
  return {
    id: "repo-1",
    owner: "ashley",
    name: "opengithub",
    fullName: "ashley/opengithub",
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
    openIssuesCount: 7,
    openPullRequestsCount: 3,
    license: {
      slug: "mit",
      name: "MIT License",
    },
    isArchived: false,
    isFork: false,
    isTemplate: false,
    isMirror: false,
    canBeSponsored: false,
    forkSource: null,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

function repositoryList(
  overrides: Partial<ProfileRepositoryList> = {},
): ProfileRepositoryList {
  return {
    items: [
      repository(),
      repository({
        id: "repo-2",
        name: "quiet-fork",
        fullName: "ashley/quiet-fork",
        description: "A fork with template metadata.",
        visibility: "private",
        href: "/ashley/quiet-fork",
        primaryLanguage: {
          language: "TypeScript",
          color: "#3178c6",
          byteCount: 512,
        },
        starsCount: 9,
        forksCount: 2,
        openIssuesCount: 0,
        openPullRequestsCount: 1,
        license: null,
        isFork: true,
        isTemplate: true,
        isMirror: true,
        canBeSponsored: true,
        forkSource: {
          owner: "namuh",
          name: "quiet",
          href: "/namuh/quiet",
        },
      }),
    ],
    total: 2,
    page: 1,
    pageSize: 30,
    filters: {
      query: "quiet",
      repositoryType: "forks",
      language: "TypeScript",
      sort: "stars-desc",
      page: 1,
      pageSize: 30,
    },
    availableLanguages: [
      { value: "Rust", label: "Rust", count: 1 },
      { value: "TypeScript", label: "TypeScript", count: 1 },
    ],
    availableTypes: [
      { value: "sources", label: "Sources", count: 1 },
      { value: "forks", label: "Forks", count: 1 },
      { value: "templates", label: "Templates", count: 1 },
    ],
    tabCounts: {
      repositories: 2,
      projects: 0,
      packages: 0,
      stars: 4,
    },
    ...overrides,
  };
}

describe("profile repository tab", () => {
  it("renders repository rows with badges, metadata, filters, and concrete links", () => {
    const { container } = render(
      <UserProfilePage
        activeTab="repositories"
        profile={profile()}
        repositoryList={repositoryList()}
        session={session}
      />,
    );

    expect(screen.getByRole("heading", { name: "Repositories" })).toBeVisible();
    expect(screen.getByText("1-2 of 2")).toBeVisible();

    const firstRow = screen
      .getByRole("link", { name: "opengithub" })
      .closest("article");
    expect(firstRow).not.toBeNull();
    expect(
      within(firstRow as HTMLElement).getByRole("link", { name: "opengithub" }),
    ).toHaveAttribute("href", "/ashley/opengithub");
    expect(within(firstRow as HTMLElement).getByText("Rust")).toBeVisible();
    expect(
      within(firstRow as HTMLElement).getByText("128 stars"),
    ).toBeVisible();
    expect(within(firstRow as HTMLElement).getByText("12 forks")).toBeVisible();
    expect(
      within(firstRow as HTMLElement).getByText("MIT License"),
    ).toBeVisible();
    expect(within(firstRow as HTMLElement).getByText("7 issues")).toBeVisible();
    expect(within(firstRow as HTMLElement).getByText("3 PRs")).toBeVisible();

    const secondRow = screen
      .getByRole("link", { name: "quiet-fork" })
      .closest("article");
    expect(secondRow).not.toBeNull();
    expect(within(secondRow as HTMLElement).getByText("private")).toBeVisible();
    expect(within(secondRow as HTMLElement).getByText("fork")).toBeVisible();
    expect(
      within(secondRow as HTMLElement).getByText("template"),
    ).toBeVisible();
    expect(within(secondRow as HTMLElement).getByText("mirror")).toBeVisible();
    expect(
      within(secondRow as HTMLElement).getByText("sponsorable"),
    ).toBeVisible();
    expect(
      within(secondRow as HTMLElement).getByRole("link", {
        name: "namuh/quiet",
      }),
    ).toHaveAttribute("href", "/namuh/quiet");

    expect(screen.getByLabelText("Search")).toHaveValue("quiet");
    expect(screen.getByLabelText("Type")).toHaveValue("forks");
    expect(screen.getByLabelText("Language")).toHaveValue("TypeScript");
    expect(screen.getByLabelText("Sort")).toHaveValue("stars-desc");
    expect(screen.getByRole("button", { name: "Filter" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Search: quiet x" }),
    ).toHaveAttribute(
      "href",
      "/ashley?tab=repositories&type=forks&language=TypeScript&sort=stars-desc",
    );
    expect(screen.getByRole("link", { name: "Sort: Stars x" })).toHaveAttribute(
      "href",
      "/ashley?tab=repositories&q=quiet&type=forks&language=TypeScript",
    );
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/ashley?tab=repositories",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("renders an empty state with a working clear action", () => {
    render(
      <UserProfilePage
        activeTab="repositories"
        profile={profile()}
        repositoryList={repositoryList({ items: [], total: 0 })}
        session={session}
      />,
    );

    expect(
      screen.getByText("No repositories matched these filters."),
    ).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Clear filters" })[0],
    ).toHaveAttribute("href", "/ashley?tab=repositories");
  });
});
