import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryCodeOverview } from "@/components/RepositoryCodeOverview";
import { RepositoryHeaderActions } from "@/components/RepositoryHeaderActions";
import {
  RepositoryBlobViewPage,
  RepositoryCommitHistoryView,
  RepositoryTreeView,
} from "@/components/RepositoryPathViews";
import { RepositoryPlaceholderPage } from "@/components/RepositoryPlaceholderPage";
import type {
  RepositoryBlameView,
  RepositoryBlobView,
  RepositoryOverview,
  RepositoryPathOverview,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "A repository for testing the Code tab",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 2,
    tagCount: 1,
    defaultBranchRef: null,
    latestCommit: {
      oid: "abcdef1234567890",
      shortOid: "abcdef1",
      message: "Initial commit",
      href: "/mona/octo-app/commit/abcdef1234567890",
      committedAt: "2026-04-30T00:00:00Z",
    },
    rootEntries: [
      {
        kind: "folder",
        name: "src",
        path: "src",
        href: "/mona/octo-app/tree/main/src",
        byteSize: null,
        latestCommitMessage: "Initial commit",
        latestCommitHref: "/mona/octo-app/commit/abcdef1234567890",
        updatedAt: "2026-04-30T00:00:00Z",
      },
      {
        kind: "file",
        name: "README.md",
        path: "README.md",
        href: "/mona/octo-app/blob/main/README.md",
        byteSize: 42,
        latestCommitMessage: "Initial commit",
        latestCommitHref: "/mona/octo-app/commit/abcdef1234567890",
        updatedAt: "2026-04-30T00:00:00Z",
      },
    ],
    files: [
      {
        id: "file-1",
        repositoryId: "repo-1",
        commitId: "commit-1",
        path: "README.md",
        content: "# octo-app\n\nHello from the README.",
        oid: "readme-oid",
        byteSize: 42,
        createdAt: "2026-04-30T00:00:00Z",
      },
      {
        id: "file-2",
        repositoryId: "repo-1",
        commitId: "commit-1",
        path: "src/index.ts",
        content: "export const answer = 42;\n",
        oid: "index-oid",
        byteSize: 26,
        createdAt: "2026-04-30T00:00:00Z",
      },
    ],
    readme: {
      id: "file-1",
      repositoryId: "repo-1",
      commitId: "commit-1",
      path: "README.md",
      content: "# octo-app\n\nHello from the README.",
      oid: "readme-oid",
      byteSize: 42,
      createdAt: "2026-04-30T00:00:00Z",
    },
    sidebar: {
      about: "A repository for testing the Code tab",
      websiteUrl: null,
      topics: [],
      starsCount: 3,
      watchersCount: 2,
      forksCount: 1,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 1,
      languages: [
        {
          language: "TypeScript",
          color: "#3178c6",
          byteCount: 1200,
          percentage: 80,
        },
        {
          language: "Rust",
          color: "#dea584",
          byteCount: 300,
          percentage: 20,
        },
      ],
    },
    viewerState: {
      starred: false,
      watching: false,
      forkedRepositoryHref: null,
    },
    cloneUrls: {
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
  return { ...base, ...overrides };
}

afterEach(() => {
  vi.restoreAllMocks();
});

function pathOverview(): RepositoryPathOverview {
  const repository = repositoryOverview();
  return {
    ...repository,
    refName: "main",
    resolvedRef: {
      kind: "branch",
      shortName: "main",
      qualifiedName: "refs/heads/main",
      targetOid: "abcdef1234567890",
      recoveryHref: "/mona/octo-app/tree/main",
    },
    defaultBranchHref: "/mona/octo-app/tree/main",
    recoveryHref: "/mona/octo-app/tree/main/src",
    total: 1,
    page: 1,
    pageSize: 1,
    hasMore: false,
    path: "src",
    pathName: "src",
    breadcrumbs: [
      {
        name: "octo-app",
        path: "",
        href: "/mona/octo-app/tree/main",
      },
      {
        name: "src",
        path: "src",
        href: "/mona/octo-app/tree/main/src",
      },
    ],
    parentHref: "/mona/octo-app/tree/main",
    entries: [
      {
        kind: "file",
        name: "index.ts",
        path: "src/index.ts",
        href: "/mona/octo-app/blob/main/src/index.ts",
        byteSize: 31,
        latestCommitMessage: "Initial commit",
        latestCommitHref: "/mona/octo-app/commit/abcdef1234567890",
        updatedAt: "2026-04-30T00:00:00Z",
      },
    ],
    readme: {
      id: "file-2",
      repositoryId: "repo-1",
      commitId: "commit-1",
      path: "src/README.md",
      content: "# src docs",
      oid: "src-readme",
      byteSize: 10,
      createdAt: "2026-04-30T00:00:00Z",
    },
    historyHref: "/mona/octo-app/commits/main/src",
  };
}

function blobView(): RepositoryBlobView {
  const repository = repositoryOverview();
  return {
    ...repository,
    refName: "main",
    resolvedRef: {
      kind: "branch",
      shortName: "main",
      qualifiedName: "refs/heads/main",
      targetOid: "abcdef1234567890",
      recoveryHref: "/mona/octo-app/tree/main",
    },
    defaultBranchHref: "/mona/octo-app/tree/main",
    recoveryHref: "/mona/octo-app/tree/main/src",
    path: "src/index.ts",
    pathName: "index.ts",
    breadcrumbs: [
      {
        name: "octo-app",
        path: "",
        href: "/mona/octo-app/tree/main",
      },
      {
        name: "src",
        path: "src",
        href: "/mona/octo-app/tree/main/src",
      },
      {
        name: "index.ts",
        path: "src/index.ts",
        href: "/mona/octo-app/tree/main/src/index.ts",
      },
    ],
    parentHref: "/mona/octo-app/tree/main/src",
    file: {
      id: "file-3",
      repositoryId: "repo-1",
      commitId: "commit-1",
      path: "src/index.ts",
      content: "export const answer = 42;\n",
      oid: "index-oid",
      byteSize: 26,
      createdAt: "2026-04-30T00:00:00Z",
    },
    language: "TypeScript",
    isBinary: false,
    isLarge: false,
    lineCount: 1,
    locCount: 1,
    sizeLabel: "26 bytes",
    mimeType: "text/plain; charset=utf-8",
    renderMode: "text",
    displayContent: "export const answer = 42;\n",
    historyHref: "/mona/octo-app/commits/main/src/index.ts",
    latestPathCommit: {
      oid: "abcdef1234567890",
      shortOid: "abcdef1",
      message: "Add source",
      href: "/mona/octo-app/commit/abcdef1234567890",
      committedAt: "2026-04-30T00:00:00Z",
    },
    rawHref: "/mona/octo-app/src/index.ts?raw=1",
    downloadHref: "/mona/octo-app/src/index.ts?download=1",
    rawApiHref: "/api/repos/mona/octo-app/blobs/src/index.ts?ref=main&raw=1",
    downloadApiHref:
      "/api/repos/mona/octo-app/blobs/src/index.ts?ref=main&download=1",
    permalinkHref: "/mona/octo-app/blob/abcdef1234567890/src/index.ts",
    symbols: [
      {
        kind: "function",
        name: "answer",
        lineNumber: 1,
        preview: "export const answer = 42;",
      },
    ],
  };
}

function blameView(): RepositoryBlameView {
  const blob = blobView();
  return {
    ...blob,
    lines: [
      {
        lineNumber: 1,
        content: "export const answer = 42;",
        commit: {
          oid: "abcdef1234567890",
          shortOid: "abcdef1",
          message: "Add source",
          href: "/mona/octo-app/commit/abcdef1234567890",
          committedAt: "2026-04-30T00:00:00Z",
          authorLogin: "mona",
        },
      },
    ],
  };
}

describe("RepositoryCodeOverview", () => {
  it("renders the repository Code tab workspace with working navigation", () => {
    render(<RepositoryCodeOverview repository={repositoryOverview()} />);

    expect(screen.getByRole("heading", { name: "octo-app" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Code" })).toHaveAttribute(
      "href",
      "/mona/octo-app",
    );
    expect(screen.getByRole("link", { name: "Issues" })).toHaveAttribute(
      "href",
      "/mona/octo-app/issues",
    );
    expect(screen.getByRole("link", { name: "Actions" })).toHaveAttribute(
      "href",
      "/mona/octo-app/actions",
    );
    expect(screen.getByRole("link", { name: "Projects" })).toHaveAttribute(
      "href",
      "/mona/octo-app/projects",
    );
    expect(screen.getByRole("link", { name: "Wiki" })).toHaveAttribute(
      "href",
      "/mona/octo-app/wiki",
    );
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/mona/octo-app/settings",
    );
    expect(screen.getByRole("link", { name: "Code" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByLabelText(/Current ref main/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Go to file" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Create new file" }),
    ).toHaveAttribute("href", "/mona/octo-app/new/main");
    expect(screen.getByRole("link", { name: "Upload files" })).toHaveAttribute(
      "href",
      "/mona/octo-app/upload/main",
    );
    expect(screen.getByRole("button", { name: /Watch/ })).toBeVisible();
    expect(screen.getByRole("button", { name: /Fork/ })).toBeVisible();
    expect(screen.getByRole("button", { name: /Star/ })).toBeVisible();
    expect(screen.getByRole("link", { name: /src/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/main/src",
    );
    expect(screen.getByRole("link", { name: /README\.md/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/blob/main/README.md",
    );
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main",
    );
    expect(screen.getByText(/# octo-app/)).toBeVisible();
  });

  it("exposes clone commands and sidebar metadata without dead links", () => {
    const { container } = render(
      <RepositoryCodeOverview repository={repositoryOverview()} />,
    );

    expect(
      screen.getByText("A repository for testing the Code tab"),
    ).toBeVisible();
    expect(screen.getByText("3 stars")).toBeVisible();
    expect(screen.getByText("2 watching")).toBeVisible();
    expect(screen.getByText("1 forks")).toBeVisible();
    expect(screen.getByText("TypeScript")).toBeVisible();
    expect(screen.getAllByDisplayValue(/octo-app\.git$/)).toHaveLength(1);
    expect(
      screen.getByDisplayValue(/^https:\/\/.*octo-app\.git$/),
    ).toBeInTheDocument();
    expect(screen.queryByText("SSH")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Copy" })).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Download ZIP" })).toHaveAttribute(
      "href",
      "/mona/octo-app/archive/refs/heads/main.zip",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("renders a cached AI repository summary with a working regenerate action", () => {
    const { container } = render(
      <RepositoryCodeOverview
        aiSummary={{
          enabled: true,
          reason: null,
          output: {
            id: "ai-1",
            kind: "repo_summary",
            scopeType: "repository",
            scopeId: "repo-1",
            contentHash: "hash",
            promptVersion: "ai-001-v1",
            model: "gpt-4o-mini",
            output:
              "Purpose, notable files, and recent activity are summarized.",
            generatedAt: "2026-05-07T00:00:00Z",
            regeneratedCount: 0,
            cached: true,
          },
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByLabelText("AI repository summary")).toHaveTextContent(
      "Purpose, notable files",
    );
    expect(screen.getByRole("button", { name: "Regenerate" })).toBeEnabled();
    expect(container.querySelector("form")).toHaveAttribute(
      "action",
      "/mona/octo-app/ai/summary",
    );
  });

  it("searches files from the Go to file combobox and opens the selected result", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          items: [
            {
              path: "src/index.ts",
              name: "index.ts",
              kind: "file",
              href: "/mona/octo-app/blob/main/src/index.ts",
              byteSize: 26,
              language: "TypeScript",
            },
          ],
          total: 1,
          page: 1,
          pageSize: 20,
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );

    render(<RepositoryCodeOverview repository={repositoryOverview()} />);

    fireEvent.click(screen.getByRole("button", { name: "Go to file" }));
    fireEvent.change(screen.getByLabelText("Find a file"), {
      target: { value: "index" },
    });

    await waitFor(() => {
      expect(
        screen.getByRole("link", { name: /src\/index\.ts/ }),
      ).toHaveAttribute("href", "/mona/octo-app/blob/main/src/index.ts");
    });
  });

  it("searches refs and preserves the current tree path when switching branches", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          items: [
            {
              name: "refs/heads/feature/tree-nav",
              shortName: "feature/tree-nav",
              kind: "branch",
              href: "/mona/octo-app/tree/feature%2Ftree-nav/src",
              samePathHref: "/mona/octo-app/tree/feature%2Ftree-nav/src",
              active: false,
              targetShortOid: "def4567",
              updatedAt: "2026-04-30T00:00:00Z",
            },
          ],
          total: 1,
          page: 1,
          pageSize: 100,
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );

    render(<RepositoryTreeView overview={pathOverview()} />);

    fireEvent.click(
      screen.getByLabelText("Switch branches or tags. Current ref main"),
    );
    fireEvent.change(screen.getByLabelText("Search branches and tags"), {
      target: { value: "feature" },
    });

    await waitFor(() => {
      expect(
        screen.getByRole("link", { name: /feature\/tree-nav/ }),
      ).toHaveAttribute("href", "/mona/octo-app/tree/feature%2Ftree-nav/src");
    });
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/mona/octo-app/refs?activeRef=main&pageSize=100&q=feature&currentPath=src",
    );
  });

  it("renders quick setup for empty repositories", () => {
    const { container } = render(
      <RepositoryCodeOverview
        repository={repositoryOverview({
          rootEntries: [],
          files: [],
          readme: null,
          latestCommit: null,
        })}
      />,
    );

    expect(screen.getByRole("heading", { name: "Quick setup" })).toBeVisible();
    expect(screen.getByLabelText("HTTPS clone URL")).toHaveValue(
      "https://opengithub.namuh.co/mona/octo-app.git",
    );
    expect(
      screen.getByRole("button", { name: "Copy URL" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Copy commands" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/git clone/)).toBeVisible();
    expect(screen.getByText(/echo "# Getting started"/)).toBeVisible();
    expect(screen.getByText(/git push -u origin main/)).toBeVisible();
    expect(screen.getByRole("link", { name: "Git docs" })).toHaveAttribute(
      "href",
      "/docs/git",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("copies quick setup commands with status feedback", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, {
      clipboard: { writeText },
    });
    render(
      <RepositoryCodeOverview
        repository={repositoryOverview({
          rootEntries: [],
          files: [],
          readme: null,
          latestCommit: null,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Copy commands" }));
    await waitFor(() => {
      expect(writeText).toHaveBeenCalledWith(
        expect.stringContaining(
          "git clone https://opengithub.namuh.co/mona/octo-app.git",
        ),
      );
    });
    expect(screen.getByRole("status")).toHaveTextContent(
      "Quick setup commands copied",
    );
  });

  it("hides settings when the viewer cannot administer the repository", () => {
    render(
      <RepositoryCodeOverview
        repository={repositoryOverview({ viewerPermission: "read" })}
      />,
    );

    const repositoryNav = screen.getByRole("navigation", {
      name: "Repository",
    });
    expect(
      within(repositoryNav).queryByRole("link", { name: "Settings" }),
    ).toBeNull();
  });

  it("links repository social counts to stargazer and fork member lists", () => {
    render(<RepositoryCodeOverview repository={repositoryOverview()} />);

    expect(screen.getByRole("link", { name: "3 stars" })).toHaveAttribute(
      "href",
      "/mona/octo-app/stargazers",
    );
    expect(screen.getByRole("link", { name: "1 forks" })).toHaveAttribute(
      "href",
      "/mona/octo-app/network/members",
    );
  });

  it("renders repository placeholders inside the shared tab shell", () => {
    render(
      <RepositoryPlaceholderPage
        actions={[{ href: "/mona/octo-app", label: "Repository Code" }]}
        activePath="/mona/octo-app/actions/runs/123"
        description="Workflow run detail is not built yet."
        repository={repositoryOverview()}
        title="Workflow run"
      />,
    );

    expect(screen.getByRole("heading", { name: "Workflow run" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Actions" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("link", { name: "Repository Code" }),
    ).toHaveAttribute("href", "/mona/octo-app");
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("optimistically toggles the star control through a same-origin route", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockImplementation(async (input, init) => {
        const url = String(input);
        expect(url).toBe("/mona/octo-app/actions/star");
        expect(init?.method).toBe("PUT");
        return new Response(
          JSON.stringify({
            starred: true,
            watching: false,
            starsCount: 4,
            watchersCount: 2,
            forksCount: 1,
            forkedRepositoryHref: null,
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      });

    render(<RepositoryHeaderActions repository={repositoryOverview()} />);

    fireEvent.click(screen.getByRole("button", { name: /Star/ }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Unstar/ })).toHaveTextContent(
        "4",
      );
    });

    expect(fetchMock).toHaveBeenCalledWith("/mona/octo-app/actions/star", {
      method: "PUT",
    });
  });

  it("saves repository watch settings through the Editorial watch menu", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockImplementation(async (input, init) => {
        const url = String(input);
        expect(url).toBe("/mona/octo-app/actions/watch");
        if (init?.method === "GET") {
          return new Response(
            JSON.stringify({
              repositoryId: "repo-1",
              level: "participating",
              label: "Participating and @mentions",
              watching: true,
              watchersCount: 2,
              customEvents: [],
              availableEvents: [
                "issues",
                "pull_requests",
                "releases",
                "discussions",
                "actions",
                "security_alerts",
                "repository_invitations",
              ],
              ignoreWarning:
                "Ignoring this repository suppresses repository watch notifications until you choose another watch level.",
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        expect(init?.method).toBe("PATCH");
        expect(JSON.parse(String(init?.body))).toEqual({
          level: "custom",
          customEvents: ["issues", "pull_requests"],
        });
        return new Response(
          JSON.stringify({
            repositoryId: "repo-1",
            level: "custom",
            label: "Custom",
            watching: true,
            watchersCount: 3,
            customEvents: ["issues", "pull_requests"],
            availableEvents: [
              "issues",
              "pull_requests",
              "releases",
              "discussions",
              "actions",
              "security_alerts",
              "repository_invitations",
            ],
            ignoreWarning:
              "Ignoring this repository suppresses repository watch notifications until you choose another watch level.",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      });

    render(<RepositoryCodeOverview repository={repositoryOverview()} />);

    fireEvent.click(screen.getByRole("button", { name: /Watch/ }));
    expect(
      await screen.findByRole("menu", { name: "Repository watch settings" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("radio", { name: /Custom/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Issue activity" }));
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Pull request activity" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Custom/ })).toHaveTextContent(
        "3",
      );
    });

    expect(fetchMock).toHaveBeenCalledWith("/mona/octo-app/actions/watch", {
      method: "GET",
    });
    expect(fetchMock).toHaveBeenCalledWith("/mona/octo-app/actions/watch", {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        level: "custom",
        customEvents: ["issues", "pull_requests"],
      }),
    });
  });

  it("shows the ignore warning before saving repository watch settings", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          repositoryId: "repo-1",
          level: "participating",
          label: "Participating and @mentions",
          watching: true,
          watchersCount: 2,
          customEvents: [],
          availableEvents: ["issues"],
          ignoreWarning: "Ignoring hides repository watch notifications.",
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );

    render(<RepositoryHeaderActions repository={repositoryOverview()} />);

    fireEvent.click(screen.getByRole("button", { name: /Watch/ }));
    fireEvent.click(await screen.findByRole("radio", { name: /Ignore/ }));

    expect(
      screen.getByText("Ignoring hides repository watch notifications."),
    ).toBeVisible();
  });

  it("rolls back optimistic state and shows feedback when an action fails", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({ error: { code: "forbidden" }, status: 403 }),
        {
          status: 403,
        },
      ),
    );

    render(<RepositoryCodeOverview repository={repositoryOverview()} />);
    fireEvent.click(screen.getByRole("button", { name: /Star/ }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Star/ })).toBeVisible();
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Repository action could not be saved.",
      );
    });
  });

  it("renders nested tree navigation with breadcrumbs and history links", () => {
    render(<RepositoryTreeView overview={pathOverview()} />);

    expect(
      screen.getByRole("navigation", { name: "Breadcrumb" }),
    ).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Current directory" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Parent directory" }),
    ).toHaveAttribute("href", "/mona/octo-app/tree/main");
    expect(
      screen.getAllByRole("link", { name: /index\.ts/ }).at(-1),
    ).toHaveAttribute("href", "/mona/octo-app/blob/main/src/index.ts");
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main/src",
    );
    expect(screen.getByText("# src docs")).toBeVisible();
  });

  it("renders the split-pane tree browser with collapsible file tree and splitter keyboard resizing", () => {
    const { container } = render(
      <RepositoryTreeView overview={pathOverview()} />,
    );

    expect(
      screen.getByRole("region", { name: "Repository directory browser" }),
    ).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Repository file tree" }),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: "src" })).toBeVisible();
    expect(screen.getByText("Last commit message")).toBeVisible();
    expect(screen.getByText("Last commit date")).toBeVisible();
    expect(screen.getByRole("button", { name: "Go to file" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Create new file" }),
    ).toHaveAttribute("href", "/mona/octo-app/new/main");

    const splitter = screen.getByRole("separator", {
      name: "Resize file tree",
    });
    const pane = container.querySelector("aside");
    expect(pane).toHaveStyle({ width: "256px" });
    fireEvent.keyDown(splitter, { key: "ArrowRight" });
    expect(pane).toHaveStyle({ width: "280px" });
    fireEvent.keyDown(splitter, { key: "ArrowLeft" });
    expect(pane).toHaveStyle({ width: "256px" });

    fireEvent.click(screen.getByRole("button", { name: "Collapse file tree" }));
    expect(
      screen.queryByRole("navigation", { name: "Repository file tree" }),
    ).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Expand file tree" }));
    expect(
      screen.getByRole("navigation", { name: "Repository file tree" }),
    ).toBeVisible();
  });

  it("renders stable large-directory paging controls", () => {
    const overview = pathOverview();
    render(
      <RepositoryTreeView
        overview={{
          ...overview,
          total: 75,
          page: 1,
          pageSize: 30,
          hasMore: true,
          entries: Array.from({ length: 30 }, (_, index) => ({
            kind: "file",
            name: `example-${index.toString().padStart(3, "0")}.md`,
            path: `src/example-${index.toString().padStart(3, "0")}.md`,
            href: `/mona/octo-app/blob/main/src/example-${index
              .toString()
              .padStart(3, "0")}.md`,
            byteSize: 12,
            latestCommitMessage: "Initial commit",
            latestCommitHref: "/mona/octo-app/commit/abcdef1234567890",
            updatedAt: "2026-04-30T00:00:00Z",
          })),
        }}
      />,
    );

    expect(screen.getByText(/Showing/)).toHaveTextContent(
      "Showing 30 of 75 entries",
    );
    expect(
      screen.getByRole("link", { name: "Load more directory entries" }),
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/main/src?page=2&pageSize=30",
    );
    expect(
      screen.getAllByRole("link", { name: "Load more entries" })[0],
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/main/src?page=2&pageSize=30",
    );
  });

  it("renders blob previews with raw, download, parent, and history actions", () => {
    const { container } = render(<RepositoryBlobViewPage blob={blobView()} />);

    expect(screen.getByRole("heading", { name: "src/index.ts" })).toBeVisible();
    expect(screen.getAllByText("export const answer = 42;")).not.toHaveLength(
      0,
    );
    expect(screen.getByRole("link", { name: "Parent" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/main/src",
    );
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commits/main/src/index.ts",
    );
    expect(screen.getByRole("link", { name: "Raw" })).toHaveAttribute(
      "href",
      "/mona/octo-app/raw/main/src/index.ts",
    );
    expect(screen.getByRole("link", { name: "Download" })).toHaveAttribute(
      "href",
      "/mona/octo-app/download/main/src/index.ts",
    );
    expect(screen.getByRole("link", { name: "Blame" })).toHaveAttribute(
      "href",
      "/mona/octo-app/blob/main/src/index.ts?view=blame",
    );
    expect(screen.getByRole("button", { name: "Symbols" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Line 1" })).toHaveAttribute(
      "href",
      "#L1",
    );
    expect(screen.getByLabelText("Raw contents of src/index.ts")).toHaveValue(
      "export const answer = 42;\n",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("renders highlighted code safely and navigates through the symbol panel", () => {
    Element.prototype.scrollIntoView = vi.fn();
    window.history.pushState(null, "", "/mona/octo-app/blob/main/src/index.ts");
    render(
      <RepositoryBlobViewPage
        blob={{
          ...blobView(),
          displayContent: 'export const answer = "<script>";\n',
          file: {
            ...blobView().file,
            content: 'export const answer = "<script>";\n',
          },
        }}
        initialSymbolsOpen
      />,
    );

    expect(screen.getByLabelText("Raw contents of src/index.ts")).toHaveValue(
      'export const answer = "<script>";\n',
    );
    expect(document.querySelector("script")).toBeNull();
    expect(
      screen.getByRole("complementary", { name: "File symbols" }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: /answer/ })).toBeVisible();

    fireEvent.change(screen.getByLabelText("Filter symbols"), {
      target: { value: "ans" },
    });
    fireEvent.click(screen.getByRole("button", { name: /answer/ }));
    expect(window.location.hash).toBe("#L1");
    expect(
      screen.queryByRole("complementary", { name: "File symbols" }),
    ).toBeNull();
  });

  it("renders blame attribution and keeps code mode reachable", () => {
    render(
      <RepositoryBlobViewPage
        blob={blobView()}
        initialBlame={blameView()}
        initialMode="blame"
      />,
    );

    expect(screen.getByRole("link", { name: "Code" })).toHaveAttribute(
      "href",
      "/mona/octo-app/blob/main/src/index.ts",
    );
    expect(screen.getByRole("link", { name: "Blame" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: /abcdef1 mona/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/commit/abcdef1234567890",
    );
    expect(screen.getAllByText("Add source").length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "Line 1" })).toHaveAttribute(
      "href",
      "#L1",
    );
  });

  it("handles permalink and line-jump shortcuts without stealing input keys", async () => {
    Element.prototype.scrollIntoView = vi.fn();
    window.history.pushState(null, "", "/mona/octo-app/blob/main/src/index.ts");
    render(<RepositoryBlobViewPage blob={blobView()} />);

    fireEvent.keyDown(window, { key: "y" });
    expect(window.location.pathname).toBe(
      "/mona/octo-app/blob/abcdef1234567890/src/index.ts",
    );

    fireEvent.keyDown(window, { key: "l" });
    expect(screen.getByRole("form", { name: "Jump to line" })).toBeVisible();
    fireEvent.change(screen.getByRole("spinbutton", { name: "Jump to line" }), {
      target: { value: "1" },
    });
    fireEvent.submit(screen.getByRole("form", { name: "Jump to line" }));
    await waitFor(() => expect(window.location.hash).toBe("#L1"));

    const fileFinder = screen.getByRole("button", { name: "Go to file" });
    fireEvent.click(fileFinder);
    const input = screen.getByLabelText("Find a file");
    fireEvent.keyDown(input, { key: "l" });
    expect(screen.queryByRole("form", { name: "Jump to line" })).toBeNull();
  });

  it("copies blob raw content through the same-origin raw route", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => {
      return new Response("export const answer = 42;\n", { status: 200 });
    });

    render(<RepositoryBlobViewPage blob={blobView()} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy raw" }));

    await waitFor(() => {
      expect(globalThis.fetch).toHaveBeenCalledWith(
        "/mona/octo-app/raw/main/src/index.ts",
      );
      expect(writeText).toHaveBeenCalledWith("export const answer = 42;\n");
      expect(screen.getByRole("status")).toHaveTextContent(
        "Raw content copied",
      );
    });
  });

  it("renders non-renderable blob states with working raw and download actions", () => {
    render(
      <RepositoryBlobViewPage
        blob={{
          ...blobView(),
          isBinary: true,
          renderMode: "binary",
          displayContent: null,
          lineCount: 0,
          locCount: 0,
          sizeLabel: "1.0 MB",
          file: {
            ...blobView().file,
            path: "bin/app.bin",
            content: "\u0000\u0001",
            byteSize: 1_048_576,
          },
          path: "bin/app.bin",
          pathName: "app.bin",
          breadcrumbs: [
            {
              name: "octo-app",
              path: "",
              href: "/mona/octo-app/tree/main",
            },
            {
              name: "bin",
              path: "bin",
              href: "/mona/octo-app/tree/main/bin",
            },
            {
              name: "app.bin",
              path: "bin/app.bin",
              href: "/mona/octo-app/tree/main/bin/app.bin",
            },
          ],
          parentHref: "/mona/octo-app/tree/main/bin",
        }}
      />,
    );

    expect(
      screen.getByText("This binary file cannot be previewed inline."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Open raw file" })).toHaveAttribute(
      "href",
      "/mona/octo-app/raw/main/bin/app.bin",
    );
    expect(screen.getByRole("link", { name: "Download file" })).toHaveAttribute(
      "href",
      "/mona/octo-app/download/main/bin/app.bin",
    );
  });

  it("renders commit history rows with commit destinations", () => {
    render(
      <RepositoryCommitHistoryView
        history={{
          repository: {
            ownerLogin: "mona",
            name: "octo-app",
            defaultBranch: "main",
            visibility: "public",
          },
          resolvedRef: {
            shortName: "main",
            qualifiedName: "refs/heads/main",
            kind: "branch",
            targetOid: "abcdef1234567890",
            href: "/mona/octo-app/tree/main/src/index.ts",
          },
          filters: {
            path: "src/index.ts",
            author: null,
            until: null,
          },
          items: [],
          groups: [
            {
              date: "2026-04-30",
              commits: [
                {
                  oid: "abcdef1234567890",
                  shortOid: "abcdef1",
                  message: "Initial commit",
                  subject: "Initial commit",
                  body: null,
                  href: "/mona/octo-app/commit/abcdef1234567890",
                  browseHref: "/mona/octo-app/tree/abcdef1234567890",
                  committedAt: "2026-04-30T00:00:00Z",
                  authorLogin: "mona",
                  authorAvatarUrl: null,
                  pullRequests: [
                    {
                      number: 42,
                      title: "Ship first commit",
                      href: "/mona/octo-app/pull/42",
                      state: "merged",
                    },
                  ],
                  status: {
                    status: "completed",
                    conclusion: "success",
                    totalCount: 2,
                    completedCount: 2,
                    failedCount: 0,
                    href: "/mona/octo-app/actions?commit=abcdef1234567890",
                  },
                  verification: {
                    verified: true,
                    signatureState: "verified",
                    signatureSummary:
                      "Verified signature from an active GPG key.",
                  },
                },
              ],
            },
          ],
          total: 1,
          page: 1,
          pageSize: 30,
          hasNextPage: false,
          hasPreviousPage: false,
          authorOptions: [
            {
              login: "mona",
              avatarUrl: null,
              count: 1,
              active: false,
            },
          ],
        }}
      />,
    );

    expect(
      screen.getByRole("link", { name: /Initial commit/ }),
    ).toHaveAttribute("href", "/mona/octo-app/commit/abcdef1234567890");
    expect(screen.getByRole("link", { name: "abcdef1" })).toHaveAttribute(
      "href",
      "/mona/octo-app/commit/abcdef1234567890",
    );
    expect(screen.getByRole("link", { name: "#42" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42",
    );
    expect(
      screen.getByRole("link", { name: "2 checks passed" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions?commit=abcdef1234567890");
    expect(screen.getByText("Verified")).toBeVisible();
    expect(
      screen.getByText("Verified signature from an active GPG key."),
    ).toBeVisible();
  });
});
