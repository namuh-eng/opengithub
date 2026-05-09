import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryForksPage } from "@/components/RepositoryForksPage";
import type { RepositoryForksView, RepositoryOverview } from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
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
  };
}

function forksView(
  overrides: Partial<RepositoryForksView> = {},
): RepositoryForksView {
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
    filters: {
      period: {
        key: "1m",
        label: "Last month",
        startedAt: "2026-04-05T00:00:00Z",
        endedAt: "2026-05-05T00:00:00Z",
      },
      repositoryType: "all",
      sort: "most_starred",
    },
    defaults: {
      saved: false,
      matchesCurrent: true,
      periodKey: "1m",
      repositoryType: "all",
      sortKey: "most_starred",
      savedAt: null,
    },
    total: 2,
    hiddenPrivateForks: 1,
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
        active: true,
        badges: ["active"],
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
        active: false,
        badges: ["inactive", "archived", "starred"],
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

describe("RepositoryForksPage", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders filters, saved defaults state, fork rows, and concrete links", () => {
    const { container } = render(
      <RepositoryForksPage
        forksResult={{ ok: true, forks: forksView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Forked repositories" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Forks Forked repositories" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("button", { name: "Period: Last month" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Repository type: All repositories" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Sort: Most starred" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Defaults Saved" }),
    ).toBeDisabled();
    expect(screen.getByText("Default filters")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Switch to tree view" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/tree/release%2Fmain");

    fireEvent.click(screen.getByRole("button", { name: "Period: Last month" }));
    expect(screen.getByRole("menu", { name: "Period options" })).toBeVisible();
    expect(
      screen.getByRole("menuitem", { name: /Last 24 hours/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/forks?period=24h&type=all&sort=most_starred",
    );

    const list = screen.getByRole("list", { name: "Repository forks list" });
    expect(
      within(list).getByRole("link", { name: "ashley/opengithub-labs" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs");
    expect(
      within(list).getByRole("link", { name: "ashley/opengithub-labs tree" }),
    ).toHaveAttribute("href", "/ashley/opengithub-labs/tree/release%2Fmain");
    expect(within(list).getByRole("link", { name: "3 stars" })).toHaveAttribute(
      "href",
      "/ashley/opengithub-labs",
    );
    expect(screen.getByText("inactive")).toBeVisible();
    expect(screen.getByText("archived")).toBeVisible();
    expect(screen.getByText("starred")).toBeVisible();
    expect(screen.getByText("No fork description provided.")).toBeVisible();
    expect(
      within(list).getByRole("link", {
        name: "long-owner-name-that-wraps/opengithub-experiment-with-a-very-long-name",
      }),
    ).toHaveClass("break-words");

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.querySelector(".chip.ok")).not.toBeNull();
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

  it("saves changed fork defaults through the local route", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ ok: true }), { status: 200 }),
      );
    render(
      <RepositoryForksPage
        forksResult={{
          ok: true,
          forks: forksView({
            filters: {
              ...forksView().filters,
              period: {
                key: "all",
                label: "All time",
                startedAt: null,
                endedAt: "2026-05-05T00:00:00Z",
              },
            },
            defaults: {
              saved: true,
              matchesCurrent: false,
              periodKey: "1m",
              repositoryType: "all",
              sortKey: "most_starred",
              savedAt: "2026-05-01T00:00:00Z",
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Save defaults" }));

    await waitFor(() =>
      expect(screen.getByText("Saved for this repository")).toBeVisible(),
    );
    expect(
      screen.getByRole("button", { name: "Defaults Saved" }),
    ).toBeDisabled();
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/forks/defaults",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          period: "all",
          repositoryType: "all",
          sort: "most_starred",
        }),
      }),
    );
  });

  it("renders empty and unavailable states without dead controls", () => {
    const empty = render(
      <RepositoryForksPage
        forksResult={{
          ok: true,
          forks: forksView({ forks: [], total: 0 }),
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "No forks match these filters." }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Reset fork filters" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/forks?period=all&type=all&sort=most_starred",
    );
    empty.unmount();

    const { container } = render(
      <RepositoryForksPage
        forksResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "repository was not found",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Forks unavailable" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Retry Forks" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/forks",
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });
});
