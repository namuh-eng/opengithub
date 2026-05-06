import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryWikiPage } from "@/components/RepositoryWikiPage";
import type { RepositoryOverview, RepositoryWikiView } from "@/lib/api";

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
    ...overrides,
  };
}

function wikiView(
  overrides: Partial<RepositoryWikiView> = {},
): RepositoryWikiView {
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
    state: {
      kind: "ready",
      message: "Wiki page is ready.",
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
        message: "Publish wiki home",
        commitOid: "abcdef1234567890",
        shortOid: "abcdef1",
        createdAt: "2026-05-05T00:00:00Z",
        href: "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
      },
      markdown: "# Home",
      html: '<h1 id="home"><a class="anchor" href="#home" aria-label="Permalink: Home">#</a>Home</h1><p>Read the <a href="/namuh-eng/opengithub/wiki/Architecture">architecture guide</a>.</p>',
      contentSha: "sha-1",
      outline: [
        {
          id: "home",
          level: 1,
          text: "Home",
          href: "#home",
        },
      ],
      editHref: "/namuh-eng/opengithub/wiki/Home/_edit",
      historyHref: "/namuh-eng/opengithub/wiki/Home/_history",
    },
    pages: [
      {
        id: "page-1",
        title: "Home",
        slug: "Home",
        href: "/wiki",
        active: true,
        hasOutline: true,
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "page-2",
        title: "Architecture Guide",
        slug: "Architecture Guide",
        href: "/wiki/Architecture%20Guide",
        active: false,
        hasOutline: true,
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    sidebar: {
      title: "_Sidebar",
      slug: "_Sidebar",
      href: "/namuh-eng/opengithub/wiki/_Sidebar",
      html: '<p><a href="/namuh-eng/opengithub/wiki/Roadmap">Roadmap</a></p>',
      outline: [],
    },
    footer: {
      title: "_Footer",
      slug: "_Footer",
      href: "/namuh-eng/opengithub/wiki/_Footer",
      html: "<p>Maintained by platform engineering.</p>",
      outline: [],
    },
    clone: {
      httpsUrl: "https://opengithub.namuh.co/namuh-eng/opengithub.wiki.git",
    },
    links: {
      homeHref: "/namuh-eng/opengithub/wiki",
      newPageHref: "/namuh-eng/opengithub/wiki/_new",
    },
    ...overrides,
  };
}

describe("RepositoryWikiPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders the Editorial wiki Home reader with active tab, metadata, page links, sidebar, footer, and clone copy", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    const { container } = render(
      <RepositoryWikiPage
        repository={repositoryOverview()}
        wikiResult={{ ok: true, wiki: wikiView() }}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", { level: 1, name: "Home" }),
    ).toBeVisible();
    expect(screen.getByText(/Updated May 5, 2026 by mona at/)).toBeVisible();
    expect(screen.getByRole("link", { name: "abcdef1" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_history/abcdef1234567890",
    );
    expect(screen.getByRole("link", { name: "Edit" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home/_edit",
    );
    expect(screen.getByRole("link", { name: "New Page" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/_new",
    );

    const pages = screen.getByRole("navigation", { name: "Wiki pages" });
    expect(within(pages).getByRole("link", { name: "Home" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Home",
    );
    expect(
      within(pages).getByRole("link", { name: "Architecture Guide" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Architecture%20Guide",
    );
    expect(
      screen.getByRole("navigation", { name: "Wiki page headings" }),
    ).toBeVisible();
    expect(
      screen.getByText("Maintained by platform engineering."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Roadmap" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Roadmap",
    );

    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(
        "https://opengithub.namuh.co/namuh-eng/opengithub.wiki.git",
      );
    });
    expect(screen.getByRole("status")).toHaveTextContent("Copied URL");

    expect(container.innerHTML).not.toContain("<script");
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAttribute("type", "button");
    }
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("renders disabled and empty wiki states without edit controls for readers", () => {
    render(
      <RepositoryWikiPage
        repository={repositoryOverview({ viewerPermission: "read" })}
        wikiResult={{
          ok: true,
          wiki: wikiView({
            viewer: {
              permission: "read",
              canRead: true,
              canEditWiki: false,
            },
            state: {
              kind: "disabled",
              message: "Wiki is disabled for this repository.",
            },
            page: null,
            pages: [],
            sidebar: null,
            footer: null,
            links: {
              homeHref: "/namuh-eng/opengithub/wiki",
              newPageHref: null,
            },
          }),
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Wiki is disabled" }),
    ).toBeVisible();
    expect(
      screen.getByText("Wiki is disabled for this repository."),
    ).toBeVisible();
    expect(screen.queryByRole("link", { name: "New Page" })).toBeNull();
    expect(screen.getByText("Reader view")).toBeVisible();
  });

  it("keeps repository-safe fetch errors inside the repository shell", () => {
    render(
      <RepositoryWikiPage
        repository={repositoryOverview({ viewerPermission: "read" })}
        wikiResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "Repository wiki was not found.",
        }}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", { level: 1, name: "Wiki unavailable" }),
    ).toBeVisible();
    expect(screen.getByText("Repository wiki was not found.")).toBeVisible();
  });
});
