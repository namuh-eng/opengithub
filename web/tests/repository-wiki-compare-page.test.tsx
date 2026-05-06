import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryWikiComparePage } from "@/components/RepositoryWikiComparePage";
import type { RepositoryOverview, RepositoryWikiCompareView } from "@/lib/api";

const routerPush = vi.fn();
const routerRefresh = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: routerPush,
    refresh: routerRefresh,
  }),
}));

afterEach(() => {
  vi.unstubAllGlobals();
  routerPush.mockReset();
  routerRefresh.mockReset();
});

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
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
      contributorsCount: 1,
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
  };
}

function compareView(
  overrides: Partial<RepositoryWikiCompareView> = {},
): RepositoryWikiCompareView {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      visibility: "private",
      defaultBranch: "main",
      wikiEnabled: true,
    },
    viewer: {
      permission: "admin",
      canRead: true,
      canEditWiki: true,
    },
    page: {
      id: "page-1",
      title: "Home",
      slug: "Home",
      href: "/namuh-eng/opengithub/wiki",
    },
    base: {
      id: "rev-1",
      author: null,
      message: "Publish wiki home",
      commitOid: "abcdef1234567890",
      shortOid: "abcdef1",
      createdAt: "2026-05-05T09:00:00Z",
      href: "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
    },
    head: {
      id: "rev-2",
      author: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
        href: "/mona",
      },
      message: "Refresh wiki home",
      commitOid: "bcdef1234567890",
      shortOid: "bcdef12",
      createdAt: "2026-05-06T10:00:00Z",
      href: "/namuh-eng/opengithub/wiki/Home/_history/bcdef1234567890",
    },
    files: [
      {
        path: "Home.md",
        oldPath: "a/Home.md",
        newPath: "b/Home.md",
        additions: 1,
        deletions: 1,
        hunks: [
          {
            header: "@@ -1,2 +1,2 @@",
            lines: [
              {
                kind: "context",
                oldNumber: 1,
                newNumber: 1,
                content: "# Home",
              },
              {
                kind: "deletion",
                oldNumber: 2,
                newNumber: null,
                content: "Original content.",
              },
              {
                kind: "addition",
                oldNumber: null,
                newNumber: 2,
                content: "Welcome to the wiki.",
              },
            ],
          },
        ],
      },
    ],
    stats: {
      additions: 1,
      deletions: 1,
      totalLines: 2,
      truncated: false,
    },
    links: {
      historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
      baseRevisionHref:
        "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
      headRevisionHref:
        "/namuh-eng/opengithub/wiki/Home/_history/bcdef1234567890",
      pageHref: "/namuh-eng/opengithub/wiki",
    },
    ...overrides,
  };
}

describe("RepositoryWikiComparePage", () => {
  it("renders base/head metadata, line diff, concrete links, and Editorial guardrails", () => {
    const { container } = render(
      <RepositoryWikiComparePage
        compareResult={{ ok: true, compare: compareView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", { name: "Compare revisions" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history",
    );
    expect(screen.getByRole("link", { name: "Page" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki",
    );
    expect(screen.getByText("Publish wiki home")).toBeVisible();
    expect(screen.getByText("Refresh wiki home")).toBeVisible();
    expect(screen.getByText("Home.md")).toBeVisible();
    expect(screen.getByText("Original content.")).toBeVisible();
    expect(screen.getByText("Welcome to the wiki.")).toBeVisible();
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("renders unavailable and truncated states without dead controls", () => {
    const { rerender, container } = render(
      <RepositoryWikiComparePage
        compareResult={{
          ok: true,
          compare: compareView({
            stats: {
              additions: 1,
              deletions: 1,
              totalLines: 2,
              truncated: true,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByText("Large wiki diff truncated for inline viewing."),
    ).toBeVisible();

    rerender(
      <RepositoryWikiComparePage
        compareResult={{
          ok: false,
          status: 422,
          code: "validation_failed",
          message: "Wiki compare revisions must be different.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Compare unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText("Wiki compare revisions must be different."),
    ).toBeVisible();
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
  });

  it("shows a permissioned revert action and redirects after the mutation succeeds", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          redirectHref: "/namuh-eng/opengithub/wiki/Home/_history",
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryWikiComparePage
        compareResult={{ ok: true, compare: compareView() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Revert Changes" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/wiki/reverts",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            pageSlug: "Home",
            baseRevisionId: "rev-1",
            expectedHeadRevisionId: "rev-2",
          }),
        }),
      );
      expect(routerPush).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/wiki/Home/_history",
      );
      expect(routerRefresh).toHaveBeenCalled();
    });
  });

  it("hides revert controls for readers and surfaces mutation errors", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          error: { message: "Wiki head revision changed." },
        }),
        { status: 409, headers: { "content-type": "application/json" } },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);
    const { rerender } = render(
      <RepositoryWikiComparePage
        compareResult={{
          ok: true,
          compare: compareView({
            viewer: {
              permission: "read",
              canRead: true,
              canEditWiki: false,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.queryByRole("button", { name: "Revert Changes" }),
    ).not.toBeInTheDocument();

    rerender(
      <RepositoryWikiComparePage
        compareResult={{ ok: true, compare: compareView() }}
        repository={repositoryOverview()}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Revert Changes" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Wiki head revision changed.",
    );
  });
});
