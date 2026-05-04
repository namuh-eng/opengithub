import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryNetworkPage } from "@/components/RepositoryNetworkPage";
import type { RepositoryNetworkView, RepositoryOverview } from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "private",
    default_branch: "release/main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "read",
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
      forksCount: 2,
      releasesCount: 1,
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
      zip: "/namuh-eng/opengithub/archive/refs/heads/release/main.zip",
    },
    ...overrides,
  };
}

function networkView(
  overrides: Partial<RepositoryNetworkView> = {},
): RepositoryNetworkView {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "release/main",
      visibility: "private",
      viewerPermission: "read",
      href: "/namuh-eng/opengithub",
      treeHref: "/namuh-eng/opengithub/tree/release%2Fmain",
    },
    summary: {
      totalReadableForks: 2,
      projectedForks: 2,
      hiddenPrivateForks: 1,
      copy: "Network graph shows the most recently pushed readable forks in this repository network.",
      updateNote:
        "Repository network projections refresh daily from fork and branch activity.",
    },
    forks: [
      {
        repositoryId: "fork-1",
        ownerLogin: "ashley",
        ownerAvatarUrl: null,
        name: "opengithub-labs",
        description: "Fork with active release work.",
        visibility: "public",
        defaultBranch: "release/main",
        isArchived: false,
        isStarredByActor: false,
        starsCount: 3,
        forksCount: 1,
        openIssuesCount: 2,
        openPullRequestsCount: 1,
        createdAt: "2026-04-30T00:00:00Z",
        updatedAt: "2026-05-06T00:00:00Z",
        pushedAt: "2026-05-06T12:00:00Z",
        href: "/ashley/opengithub-labs",
        ownerHref: "/ashley",
        treeHref: "/ashley/opengithub-labs/tree/release%2Fmain",
        networkHref: "/ashley/opengithub-labs/network",
      },
      {
        repositoryId: "fork-2",
        ownerLogin: "long-owner-name-that-wraps",
        ownerAvatarUrl: null,
        name: "opengithub-experiment-with-a-very-long-name",
        description: null,
        visibility: "public",
        defaultBranch: "main",
        isArchived: true,
        isStarredByActor: true,
        starsCount: 1,
        forksCount: 0,
        openIssuesCount: 0,
        openPullRequestsCount: 0,
        createdAt: "2026-03-30T00:00:00Z",
        updatedAt: "2026-04-06T00:00:00Z",
        pushedAt: "2026-04-06T12:00:00Z",
        href: "/long-owner-name-that-wraps/opengithub-experiment-with-a-very-long-name",
        ownerHref: "/long-owner-name-that-wraps",
        treeHref:
          "/long-owner-name-that-wraps/opengithub-experiment-with-a-very-long-name/tree/main",
        networkHref:
          "/long-owner-name-that-wraps/opengithub-experiment-with-a-very-long-name/network",
      },
    ],
    freshness: {
      computedAt: "2026-05-07T01:00:00Z",
      expiresAt: "2026-05-08T01:00:00Z",
      stale: false,
      cadence: "daily",
    },
    links: {
      forksHref: "/namuh-eng/opengithub/forks",
      treeHref: "/namuh-eng/opengithub/tree/release%2Fmain",
    },
    ...overrides,
  };
}

describe("RepositoryNetworkPage", () => {
  it("renders the Insights shell, network graph rows, and concrete fork links", () => {
    const { container } = render(
      <RepositoryNetworkPage
        networkResult={{ ok: true, network: networkView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Repository network" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Network Repository network activity",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.getByText("Fresh projection")).toBeVisible();
    expect(screen.getByText("daily")).toBeVisible();
    expect(screen.getByRole("link", { name: "Tree view" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/release%2Fmain",
    );
    expect(screen.getByRole("link", { name: "View forks" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/forks",
    );

    const metrics = screen.getByLabelText("Network summary metrics");
    expect(within(metrics).getByText("Readable forks")).toBeVisible();
    expect(within(metrics).getByText("Projected forks")).toBeVisible();
    expect(within(metrics).getByText("Private forks")).toBeVisible();
    expect(within(metrics).getAllByText("2").length).toBeGreaterThanOrEqual(2);
    expect(within(metrics).getByText("1")).toBeVisible();

    const graph = screen.getByRole("list", {
      name: "Repository network fork graph",
    });
    expect(
      within(graph).getByRole("link", { name: "ashley profile" }),
    ).toHaveAttribute("href", "/ashley");
    expect(
      within(graph).getByRole("link", { name: "ashley/opengithub-labs" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs");
    expect(
      within(graph).getByRole("link", {
        name: "ashley/opengithub-labs tree",
      }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/tree/release%2Fmain");
    expect(
      within(graph).getByRole("link", {
        name: "ashley/opengithub-labs network",
      }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/network");
    expect(
      within(graph).getByRole("link", { name: "3 stars" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs");
    expect(
      within(graph).getByRole("link", { name: "1 forks" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/network");
    expect(
      within(graph).getByRole("link", { name: "2 issues" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/issues");
    expect(
      within(graph).getByRole("link", { name: "1 pull requests" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/pulls");
    expect(screen.getByText("archived")).toBeVisible();
    expect(screen.getByText("starred")).toBeVisible();
    expect(screen.getByText("No fork description provided.")).toBeVisible();

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.querySelector(".chip.ok")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--accent)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|#f1aeb5|#fff1f3|@primer\/|Octicon/i,
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("renders empty network states with truthful recovery links", () => {
    render(
      <RepositoryNetworkPage
        networkResult={{
          ok: true,
          network: networkView({
            forks: [],
            summary: {
              ...networkView().summary,
              totalReadableForks: 0,
              projectedForks: 0,
              hiddenPrivateForks: 0,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Repository network" }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "This repository network has no readable forks.",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Browse source tree" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/tree/release%2Fmain");
    expect(
      screen.getByRole("link", { name: "Open forks list" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/forks");
  });

  it("renders unavailable state without dead controls", () => {
    const { container } = render(
      <RepositoryNetworkPage
        networkResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "repository was not found",
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Network unavailable" }),
    ).toBeVisible();
    expect(screen.getByText("repository was not found")).toBeVisible();
    expect(screen.getByRole("link", { name: "Retry Network" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });
});
