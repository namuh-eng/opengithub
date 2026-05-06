import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryWikiRevisionPage } from "@/components/RepositoryWikiRevisionPage";
import type { RepositoryOverview, RepositoryWikiRevisionView } from "@/lib/api";

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

function revisionView(
  overrides: Partial<RepositoryWikiRevisionView> = {},
): RepositoryWikiRevisionView {
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
      path: "Home.md",
      href: "/namuh-eng/opengithub/wiki",
      revision: {
        id: "rev-1",
        author: {
          id: "user-1",
          login: "mona",
          displayName: "Mona",
          avatarUrl: null,
          href: "/mona",
        },
        message: "Publish original wiki home",
        commitOid: "abcdef1234567890",
        shortOid: "abcdef1",
        createdAt: "2026-05-05T09:00:00Z",
        href: "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
      },
      markdown: "# Home\n\nOriginal content.",
      html: '<h1 id="home">Home</h1><p>Original content.</p>',
      contentSha: "sha",
      outline: [{ id: "home", level: 1, text: "Home", href: "#home" }],
      editHref: null,
      historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
    },
    revisionContext: {
      selectedRevision: {
        id: "rev-1",
        author: {
          id: "user-1",
          login: "mona",
          displayName: "Mona",
          avatarUrl: null,
          href: "/mona",
        },
        message: "Publish original wiki home",
        commitOid: "abcdef1234567890",
        shortOid: "abcdef1",
        createdAt: "2026-05-05T09:00:00Z",
        href: "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
      },
      latestHref: "/namuh-eng/opengithub/wiki",
      historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
      previousRevisionHref: null,
      nextRevisionHref: "/namuh-eng/opengithub/wiki/Home/_history/bcdef123",
      isLatest: false,
    },
    pages: [
      {
        id: "page-1",
        title: "Home",
        slug: "Home",
        href: "/namuh-eng/opengithub/wiki",
        active: true,
        hasOutline: true,
        updatedAt: "2026-05-06T10:00:00Z",
      },
    ],
    links: {
      homeHref: "/namuh-eng/opengithub/wiki",
      pagesHref: "/namuh-eng/opengithub/wiki/_pages",
    },
    ...overrides,
  };
}

describe("RepositoryWikiRevisionPage", () => {
  it("renders a read-only historical snapshot with concrete navigation", () => {
    const { container } = render(
      <RepositoryWikiRevisionPage
        repository={repositoryOverview()}
        revisionResult={{ ok: true, revision: revisionView() }}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getAllByRole("heading", { name: "Home" })[0]).toBeVisible();
    expect(screen.getByText("Read-only snapshot")).toBeVisible();
    expect(screen.getByText("abcdef1")).toBeVisible();
    expect(screen.getByText("Original content.")).toBeVisible();
    expect(screen.getByRole("link", { name: "Latest" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki",
    );
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history",
    );
    expect(screen.getByRole("link", { name: "Pages" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/_pages",
    );
    expect(screen.getByRole("link", { name: "Next Revision" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history/bcdef123",
    );
    expect(
      screen.queryByRole("link", { name: "Edit" }),
    ).not.toBeInTheDocument();
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("renders unavailable state without placeholder links", () => {
    const { container } = render(
      <RepositoryWikiRevisionPage
        repository={repositoryOverview()}
        revisionResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "Repository wiki revision was not found.",
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Revision unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText("Repository wiki revision was not found."),
    ).toBeVisible();
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
  });
});
