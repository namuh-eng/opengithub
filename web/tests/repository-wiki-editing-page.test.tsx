import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryWikiEditor } from "@/components/RepositoryWikiEditor";
import { RepositoryWikiPagesIndex } from "@/components/RepositoryWikiPagesIndex";
import type {
  RepositoryOverview,
  RepositoryWikiEditView,
  RepositoryWikiPagesIndex as RepositoryWikiPagesIndexContract,
} from "@/lib/api";

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

function pagesIndex(
  overrides: Partial<RepositoryWikiPagesIndexContract> = {},
): RepositoryWikiPagesIndexContract {
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
    pages: [
      {
        id: "page-b",
        title: "Zebra Notes",
        slug: "Zebra Notes",
        href: "/namuh-eng/opengithub/wiki/Zebra%20Notes",
        active: false,
        hasOutline: false,
        updatedAt: "2026-05-03T00:00:00Z",
      },
      {
        id: "page-a",
        title: "Architecture Guide",
        slug: "Architecture Guide",
        href: "/namuh-eng/opengithub/wiki/Architecture%20Guide",
        active: false,
        hasOutline: true,
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    links: {
      homeHref: "/namuh-eng/opengithub/wiki",
      newPageHref: "/namuh-eng/opengithub/wiki/_new",
    },
    ...overrides,
  };
}

function editView(overrides: Partial<RepositoryWikiEditView> = {}) {
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
      id: "page-a",
      title: "Architecture Guide",
      slug: "Architecture Guide",
      path: "Architecture Guide.md",
      markdown: "# Architecture Guide\n\nInitial services map.",
      latestRevisionId: "revision-current-123456",
      editMode: "markdown" as const,
    },
    supportedFormats: [
      { mode: "markdown" as const, label: "Markdown", extension: ".md" },
    ],
    ...overrides,
  };
}

describe("RepositoryWikiPagesIndex", () => {
  it("renders sorted page rows with concrete reader and editor links", () => {
    const { container } = render(
      <RepositoryWikiPagesIndex
        pagesIndex={{ ok: true, value: pagesIndex() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("heading", { name: "Pages" })).toBeVisible();
    expect(screen.getByRole("link", { name: "New Page" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/_new",
    );
    const rows = screen
      .getAllByText(/Last updated/)
      .map((node) => node.closest(".list-row"));
    expect(rows).toHaveLength(2);
    expect(
      within(rows[0] as HTMLElement).getByRole("link", {
        name: "Architecture Guide",
      }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Architecture%20Guide",
    );
    expect(
      within(rows[0] as HTMLElement).getByRole("link", { name: "Edit" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/wiki/Architecture%20Guide/_edit",
    );
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("hides creation controls for readers and shows the empty state", () => {
    render(
      <RepositoryWikiPagesIndex
        pagesIndex={{
          ok: true,
          value: pagesIndex({
            viewer: {
              permission: "read",
              canRead: true,
              canEditWiki: false,
            },
            pages: [],
          }),
        }}
        repository={repositoryOverview({ viewerPermission: "read" })}
      />,
    );

    expect(screen.queryByRole("link", { name: "New Page" })).toBeNull();
    expect(screen.getByText("Reader view")).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "No wiki pages yet" }),
    ).toBeVisible();
  });
});

describe("RepositoryWikiEditor", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("builds a new-page draft, inserts image Markdown, and previews through the same-origin API", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        html: '<h1 id="new-page">New page</h1><p><img src="https://example.com/diagram.png" alt="Architecture diagram"></p>',
        contentSha: "sha-preview",
        outline: [
          {
            id: "new-page",
            level: 1,
            text: "New page",
            href: "#new-page",
          },
        ],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { container } = render(
      <RepositoryWikiEditor
        pagesIndex={pagesIndex()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Page title"), {
      target: { value: "Operations Guide" },
    });
    fireEvent.change(screen.getByLabelText("Image URL"), {
      target: { value: "https://example.com/diagram.png" },
    });
    fireEvent.change(screen.getByLabelText("Alt text"), {
      target: { value: "Architecture diagram" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Insert image/ }));
    expect(
      (screen.getByLabelText("Wiki page source") as HTMLTextAreaElement).value,
    ).toContain("![Architecture diagram](https://example.com/diagram.png)");

    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/wiki/preview",
        expect.objectContaining({
          method: "POST",
          body: expect.stringContaining("Architecture diagram"),
        }),
      );
    });
    expect(await screen.findByText("Preview rendered.")).toBeVisible();
    expect(container.innerHTML).not.toContain("<script");
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAttribute("type", "button");
    }
  });

  it("posts the save payload to the create endpoint and shows validation errors", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        error: { message: "Edit message is required." },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryWikiEditor
        pagesIndex={pagesIndex()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Page title"), {
      target: { value: "Operations Guide" },
    });
    fireEvent.change(screen.getByLabelText("Edit message"), {
      target: { value: "" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Page" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/wiki/pages",
        expect.objectContaining({
          method: "POST",
          body: expect.stringContaining("Operations Guide"),
        }),
      );
    });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Edit message is required.",
    );
  });

  it("hydrates an existing page, previews unsaved Markdown, and patches with the latest revision guard", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          html: '<h1 id="architecture-guide">Architecture Guide</h1><p>Updated services map.</p>',
          contentSha: "sha-preview",
          outline: [
            {
              id: "architecture-guide",
              level: 1,
              text: "Architecture Guide",
              href: "#architecture-guide",
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          redirectHref: "/namuh-eng/opengithub/wiki/Architecture%20Guide",
          page: {
            id: "page-a",
            title: "Architecture Guide",
            slug: "Architecture Guide",
            path: "Architecture Guide.md",
            href: "/namuh-eng/opengithub/wiki/Architecture%20Guide",
            revision: {
              id: "revision-next",
              author: null,
              message: "Refresh architecture wiki",
              commitOid: "abcdef1234567890",
              shortOid: "abcdef1",
              createdAt: "2026-05-06T00:00:00Z",
              href: "/namuh-eng/opengithub/wiki/Architecture%20Guide/_history/abcdef1",
            },
            markdown: "# Architecture Guide\n\nUpdated services map.",
            html: "<p>Updated services map.</p>",
            contentSha: "sha-next",
            outline: [],
            editHref: "/namuh-eng/opengithub/wiki/Architecture%20Guide/_edit",
            historyHref:
              "/namuh-eng/opengithub/wiki/Architecture%20Guide/_history",
          },
          gitCommit: {
            id: "commit-1",
            oid: "abcdef1234567890",
            shortOid: "abcdef1",
            branch: "main",
            message: "Refresh architecture wiki",
            storagePath: "/tmp/opengithub.wiki.git",
            createdAt: "2026-05-06T00:00:00Z",
          },
        }),
      });
    vi.stubGlobal("fetch", fetchMock);
    const assign = vi.fn();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { assign },
    });

    render(
      <RepositoryWikiEditor
        editView={editView()}
        pagesIndex={pagesIndex()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Edit Architecture Guide" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Page title")).toHaveValue(
      "Architecture Guide",
    );
    expect(screen.getByLabelText("Wiki page source")).toHaveValue(
      "# Architecture Guide\n\nInitial services map.",
    );

    fireEvent.change(screen.getByLabelText("Wiki page source"), {
      target: { value: "# Architecture Guide\n\nUpdated services map." },
    });
    fireEvent.change(screen.getByLabelText("Edit message"), {
      target: { value: "Refresh architecture wiki" },
    });
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/wiki/preview",
        expect.objectContaining({
          method: "POST",
          body: expect.stringContaining("Updated services map"),
        }),
      );
    });
    expect(await screen.findByText("Preview rendered.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Save Page" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/repos/namuh-eng/opengithub/wiki/Architecture%20Guide",
        expect.objectContaining({
          method: "PATCH",
          body: expect.stringContaining("revision-current-123456"),
        }),
      );
    });
    expect(assign).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/wiki/Architecture%20Guide",
    );
  });

  it("surfaces stale revision conflicts without dropping the editor draft", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        error: { message: "Wiki page has changed since you opened it." },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryWikiEditor
        editView={editView()}
        pagesIndex={pagesIndex()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Wiki page source"), {
      target: { value: "# Architecture Guide\n\nKeep this draft." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save Page" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Wiki page has changed since you opened it.",
    );
    expect(screen.getByLabelText("Wiki page source")).toHaveValue(
      "# Architecture Guide\n\nKeep this draft.",
    );
  });
});
