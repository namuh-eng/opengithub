import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryActionsCachesPage } from "@/components/RepositoryActionsCachesPage";
import type { RepositoryActionsCaches, RepositoryOverview } from "@/lib/api";

const refresh = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh }),
}));

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions cache test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 2,
    tagCount: 0,
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
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
}

function cacheDetail(
  overrides: Partial<RepositoryActionsCaches> = {},
): RepositoryActionsCaches {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    viewerPermission: "owner",
    caches: {
      items: [
        {
          id: "cache-1",
          repositoryId: "repo-1",
          key: "node-linux-lock",
          version: "v1-main",
          scope: "refs/heads/main",
          sizeBytes: 2_097_152,
          lastUsedAt: "2026-05-01T00:10:00Z",
          createdAt: "2026-05-01T00:00:00Z",
          updatedAt: "2026-05-01T00:10:00Z",
        },
      ],
      total: 1,
      page: 1,
      pageSize: 30,
    },
    totalSizeBytes: 2_097_152,
    limitBytes: 10 * 1024 * 1024 * 1024,
    canDelete: true,
    ...overrides,
  };
}

beforeEach(() => {
  refresh.mockClear();
  vi.stubGlobal(
    "fetch",
    vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify({ id: "cache-1" }), { status: 200 }),
      ),
    ),
  );
});

describe("RepositoryActionsCachesPage", () => {
  it("renders cache usage, rows, concrete links, and delete action", async () => {
    const fetchMock = vi.mocked(fetch);
    render(
      <RepositoryActionsCachesPage
        detail={cacheDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Dependency caches" }),
    ).toBeVisible();
    expect(screen.getByText("node-linux-lock")).toBeVisible();
    expect(screen.getByText("v1-main")).toBeVisible();
    expect(screen.getAllByText("2.0 MB")).toHaveLength(2);
    expect(screen.getByRole("link", { name: "All workflows" })).toHaveAttribute(
      "href",
      "/mona/octo-app/actions",
    );
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs/api#actions-artifacts-caches",
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(refresh).toHaveBeenCalled());
    expect(screen.getByRole("status")).toHaveTextContent("Cache deleted.");
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/caches/cache-1",
      { cache: "no-store", method: "DELETE" },
    );
  });

  it("renders empty and read-only states without dead controls", () => {
    render(
      <RepositoryActionsCachesPage
        detail={cacheDetail({
          caches: { items: [], total: 0, page: 1, pageSize: 30 },
          canDelete: false,
          totalSizeBytes: 0,
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByText("No dependency caches yet")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Delete" })).toBeNull();
    expect(document.body.innerHTML).not.toContain('href="#"');
    expect(document.body.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e)/i,
    );
  });
});
