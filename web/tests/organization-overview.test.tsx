import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { OrganizationOverviewPage } from "@/components/OrganizationOverviewPage";
import type { AuthSession, OrganizationOverview } from "@/lib/api";

vi.mock("@/components/AppShell", () => ({
  AppShell: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

vi.mock("@/components/AppShellFrame", () => ({
  AppShellFrame: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

const session: AuthSession = {
  authenticated: false,
  user: null,
};

function organizationOverview(): OrganizationOverview {
  return {
    id: "org-1",
    slug: "open-labs",
    displayName: "Open Labs",
    description: "Verified maintainers building calm developer tools.",
    avatarUrl: null,
    websiteUrl: "https://openlabs.example",
    location: null,
    verifiedDomain: {
      domain: "openlabs.example",
      verifiedAt: "2026-05-02T00:00:00Z",
      href: "/orgs/open-labs/settings/verified-domains",
    },
    viewerRole: "owner",
    viewerCanAdmin: true,
    followerCount: 1200,
    memberCount: 42,
    repositoryCount: 9,
    pinnedRepositories: [
      {
        id: "repo-1",
        name: "editorial-shell",
        description: "A pinned organization repository",
        visibility: "public",
        href: "/open-labs/editorial-shell",
        primaryLanguage: {
          language: "TypeScript",
          color: "3178c6",
          byteCount: 3000,
          percentage: 75,
        },
        topics: ["editorial", "organizations"],
        starsCount: 19,
        forksCount: 3,
        openIssuesCount: 1,
        openPullRequestsCount: 2,
        updatedAt: "2026-05-02T00:00:00Z",
        isPinned: true,
      },
    ],
    repositories: [
      {
        id: "repo-1",
        name: "editorial-shell",
        description: "A pinned organization repository",
        visibility: "public",
        href: "/open-labs/editorial-shell",
        primaryLanguage: {
          language: "TypeScript",
          color: "3178c6",
          byteCount: 3000,
          percentage: 75,
        },
        topics: ["editorial", "organizations"],
        starsCount: 19,
        forksCount: 3,
        openIssuesCount: 1,
        openPullRequestsCount: 2,
        updatedAt: "2026-05-02T00:00:00Z",
        isPinned: true,
      },
    ],
    members: [
      {
        id: "user-1",
        login: "mona",
        displayName: "Mona Maintainer",
        avatarUrl: null,
        role: "owner",
        href: "/mona",
      },
    ],
    languages: [
      {
        language: "TypeScript",
        color: "3178c6",
        byteCount: 3000,
        percentage: 75,
      },
      {
        language: "Rust",
        color: "dea584",
        byteCount: 1000,
        percentage: 25,
      },
    ],
    topics: [
      {
        topic: "editorial",
        repositoryCount: 1,
        href: "/orgs/open-labs?tab=repositories&q=topic%3Aeditorial",
      },
    ],
    sponsorship: {
      enabled: false,
      sponsorHref: null,
      note: "Sponsorships are not enabled in this OpenGitHub MVP.",
    },
    projectsHref: "/orgs/open-labs/projects",
    settingsHref: "/orgs/open-labs/settings",
    peopleHref: "/orgs/open-labs?tab=people",
    repositoriesHref: "/orgs/open-labs?tab=repositories",
    packagesHref: "/orgs/open-labs?tab=packages",
    updatedAt: "2026-05-02T00:00:00Z",
  };
}

describe("OrganizationOverviewPage", () => {
  it("renders verified identity, pinned repositories, preview rows, members, and facets", () => {
    render(
      <OrganizationOverviewPage
        organization={organizationOverview()}
        session={session}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Open Labs" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Verified")).toHaveAttribute(
      "title",
      "Verified domain: openlabs.example",
    );
    expect(screen.getByRole("link", { name: "42 people" })).toHaveAttribute(
      "href",
      "/orgs/open-labs?tab=people",
    );
    expect(
      screen.getByRole("button", { name: "Sponsor unavailable" }),
    ).toBeDisabled();
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/orgs/open-labs/settings",
    );

    const pinned = screen
      .getByRole("heading", { name: "Pinned repositories" })
      .closest("section");
    expect(pinned).not.toBeNull();
    expect(
      within(pinned as HTMLElement).getByRole("link", {
        name: "editorial-shell repository",
      }),
    ).toHaveAttribute("href", "/open-labs/editorial-shell");

    const preview = screen
      .getByRole("heading", { name: "Active public work" })
      .closest("section");
    expect(preview).not.toBeNull();
    expect(
      within(preview as HTMLElement).getByText("1 open issues"),
    ).toBeInTheDocument();
    expect(
      within(preview as HTMLElement).getByText("2 pull requests"),
    ).toBeInTheDocument();

    expect(screen.getByText("Mona Maintainer")).toBeInTheDocument();
    expect(screen.getAllByText("TypeScript").length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "editorial · 1" })).toHaveAttribute(
      "href",
      "/orgs/open-labs?tab=repositories&q=topic%3Aeditorial",
    );
  });
});
