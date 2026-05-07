import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import RepositoryTreePage from "@/app/[owner]/[repo]/tree/[ref]/[[...path]]/page";
import type { RepositoryPathOverview } from "@/lib/api";
import {
  getRepository,
  getRepositoryPath,
  getSession,
} from "@/lib/server-session";

vi.mock("@/components/AppShell", () => ({
  AppShell: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

vi.mock("@/components/RepositoryPathViews", () => ({
  RepositoryTreeView: ({ overview }: { overview: RepositoryPathOverview }) => (
    <section aria-label="mock tree view">
      {overview.resolvedRef.shortName}:{overview.pathName}
    </section>
  ),
}));

vi.mock("@/lib/server-session", () => ({
  getRepository: vi.fn(),
  getRepositoryPath: vi.fn(),
  getSession: vi.fn(),
}));

const mockedGetRepository = vi.mocked(getRepository);
const mockedGetRepositoryPath = vi.mocked(getRepositoryPath);
const mockedGetSession = vi.mocked(getSession);

function pathOverview(): RepositoryPathOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: null,
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    latestCommit: null,
    readme: null,
    refName: "feature/tree-nav",
    resolvedRef: {
      kind: "branch",
      shortName: "feature/tree-nav",
      qualifiedName: "refs/heads/feature/tree-nav",
      targetOid: "abc123",
      recoveryHref: "/mona/octo-app/tree/main",
    },
    defaultBranchHref: "/mona/octo-app/tree/main",
    recoveryHref: "/mona/octo-app/tree/feature%2Ftree-nav",
    total: 0,
    page: 1,
    pageSize: 30,
    hasMore: false,
    path: "",
    pathName: "octo-app",
    breadcrumbs: [
      {
        name: "octo-app",
        path: "",
        href: "/mona/octo-app/tree/feature%2Ftree-nav",
      },
    ],
    parentHref: null,
    entries: [],
    historyHref: "/mona/octo-app/commits/feature%2Ftree-nav",
  };
}

describe("RepositoryTreePage", () => {
  it("resolves root tree pages through the selected ref contents contract", async () => {
    mockedGetSession.mockResolvedValue({
      authenticated: true,
      user: {
        id: "user-1",
        email: "mona@example.com",
        display_name: "Mona",
        avatar_url: null,
      },
    });
    mockedGetRepositoryPath.mockResolvedValue(pathOverview());

    render(
      await RepositoryTreePage({
        params: Promise.resolve({
          owner: "mona",
          repo: "octo-app",
          ref: "feature%2Ftree-nav",
        }),
        searchParams: Promise.resolve({}),
      }),
    );

    expect(mockedGetRepositoryPath).toHaveBeenCalledWith(
      "mona",
      "octo-app",
      "feature/tree-nav",
      "",
      { page: 1, pageSize: 30 },
    );
    expect(mockedGetRepository).not.toHaveBeenCalled();
    expect(screen.getByLabelText("mock tree view")).toHaveTextContent(
      "feature/tree-nav:octo-app",
    );
  });
});
