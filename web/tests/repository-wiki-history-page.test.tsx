import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryWikiHistoryPage } from "@/components/RepositoryWikiHistoryPage";
import type { RepositoryOverview, RepositoryWikiHistoryView } from "@/lib/api";

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

function historyView(
  overrides: Partial<RepositoryWikiHistoryView> = {},
): RepositoryWikiHistoryView {
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
    scope: {
      kind: "page",
      page: {
        id: "page-1",
        title: "Home",
        slug: "Home",
        href: "/namuh-eng/opengithub/wiki",
        active: true,
        hasOutline: true,
        updatedAt: "2026-05-06T10:00:00Z",
      },
    },
    revisions: [
      {
        id: "rev-2",
        pageId: "page-1",
        pageTitle: "Home",
        pageSlug: "Home",
        pageHref: "/namuh-eng/opengithub/wiki",
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
        revisionHref:
          "/namuh-eng/opengithub/wiki/Home/_history/bcdef1234567890",
      },
      {
        id: "rev-1",
        pageId: "page-1",
        pageTitle: "Home",
        pageSlug: "Home",
        pageHref: "/namuh-eng/opengithub/wiki",
        author: null,
        message: "Publish wiki home",
        commitOid: "abcdef1234567890",
        shortOid: "abcdef1",
        createdAt: "2026-05-05T09:00:00Z",
        href: "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
        revisionHref:
          "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
      },
    ],
    pagination: {
      page: 2,
      pageSize: 30,
      hasNewer: true,
      hasOlder: true,
      newerHref: "/namuh-eng/opengithub/wiki/Home/_history",
      olderHref: "/namuh-eng/opengithub/wiki/Home/_history?page=3",
    },
    links: {
      homeHref: "/namuh-eng/opengithub/wiki",
      pagesHref: "/namuh-eng/opengithub/wiki/_pages",
      historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
    },
    ...overrides,
  };
}

describe("RepositoryWikiHistoryPage", () => {
  it("renders revision rows with accessible selection, concrete links, metadata, and pagination", () => {
    const { container } = render(
      <RepositoryWikiHistoryPage
        historyResult={{ ok: true, history: historyView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("heading", { name: "History" })).toBeVisible();
    expect(screen.getByText(/Revision history for/)).toBeVisible();
    expect(screen.getByRole("link", { name: "Home" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki",
    );
    expect(screen.getByRole("link", { name: "Wiki Home" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki",
    );
    expect(screen.getByRole("link", { name: "Pages" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/_pages",
    );

    expect(screen.getByText("Refresh wiki home")).toBeVisible();
    expect(
      screen.getByRole("checkbox", {
        name: "Select revision Refresh wiki home",
      }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "bcdef12" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history/bcdef1234567890",
    );
    expect(screen.getAllByRole("link", { name: "Page" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki",
    );
    expect(screen.getByRole("link", { name: "Mona" })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getAllByText(/committed/)).toHaveLength(2);

    const compare = screen.getByRole("button", { name: "Compare Revisions" });
    expect(compare).toBeDisabled();
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: "Select revision Refresh wiki home",
      }),
    );
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: "Select revision Publish wiki home",
      }),
    );
    expect(compare).toBeEnabled();

    expect(screen.getByRole("link", { name: "Newer" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history",
    );
    expect(screen.getByRole("link", { name: "Older" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history?page=3",
    );
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAttribute("type", "button");
    }
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("renders empty and unavailable states without placeholder links", () => {
    const { rerender, container } = render(
      <RepositoryWikiHistoryPage
        historyResult={{
          ok: true,
          history: historyView({
            revisions: [],
            scope: { kind: "all_pages", page: null },
            pagination: {
              page: 1,
              pageSize: 30,
              hasNewer: false,
              hasOlder: false,
              newerHref: null,
              olderHref: null,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "No wiki history yet" }),
    ).toBeVisible();
    expect(screen.getAllByText("Newer")[0]).toHaveAttribute(
      "aria-disabled",
      "true",
    );

    rerender(
      <RepositoryWikiHistoryPage
        historyResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "Repository wiki was not found.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "History unavailable" }),
    ).toBeVisible();
    expect(screen.getByText("Repository wiki was not found.")).toBeVisible();
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
  });
});
