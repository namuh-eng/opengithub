import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryBranchesPage } from "@/components/RepositoryBranchesPage";
import type {
  RepositoryBranchDirectoryRow,
  RepositoryBranchesView,
  RepositoryOverview,
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
      zip: "/namuh-eng/opengithub/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function branchRow(
  overrides: Partial<RepositoryBranchDirectoryRow> = {},
): RepositoryBranchDirectoryRow {
  const name = overrides.name ?? "feature/editorial-branches";
  return {
    name,
    qualifiedName: `refs/heads/${name}`,
    classification: "active",
    isDefault: false,
    href: `/namuh-eng/opengithub/tree/${encodeURIComponent(name)}`,
    commitsHref: `/namuh-eng/opengithub/commits/${encodeURIComponent(name)}`,
    activityHref: `/namuh-eng/opengithub/branches/${encodeURIComponent(name)}`,
    latestCommit: {
      oid: "abc123456789",
      shortOid: "abc1234",
      subject: "Render branches overview",
      href: "/namuh-eng/opengithub/commit/abc123456789",
      committedAt: "2026-05-04T10:00:00Z",
      authorLogin: "mona",
      authorAvatarUrl: null,
    },
    checks: {
      status: "completed",
      conclusion: "success",
      totalCount: 2,
      completedCount: 2,
      failedCount: 0,
      href: "/namuh-eng/opengithub/actions?commit=abc123456789",
    },
    protection: {
      protected: false,
      matchingRuleCount: 0,
      matchingRulesetCount: 0,
      requiredStatusChecks: [],
      href: `/namuh-eng/opengithub/settings/branches?branch=${encodeURIComponent(name)}`,
    },
    ahead: 2,
    behind: 1,
    pullRequest: {
      number: 42,
      title: "Open branch directory",
      state: "open",
      draft: true,
      href: "/namuh-eng/opengithub/pull/42",
    },
    capabilities: {
      canCopy: true,
      canViewActivity: true,
      canViewRules: true,
      canDelete: false,
      deleteDisabledReason:
        "Protected branches require policy changes before deletion.",
      canRestore: false,
      restoreDisabledReason:
        "Branch restore is handled by a later mutation phase.",
    },
    updatedAt: "2026-05-04T10:00:00Z",
    ...overrides,
  };
}

function branchesView(
  overrides: Partial<RepositoryBranchesView> = {},
): RepositoryBranchesView {
  const defaultBranch = branchRow({
    name: "main",
    qualifiedName: "refs/heads/main",
    isDefault: true,
    classification: "default",
    href: "/namuh-eng/opengithub/tree/main",
    commitsHref: "/namuh-eng/opengithub/commits/main",
    activityHref: "/namuh-eng/opengithub/branches/main",
    protection: {
      protected: true,
      matchingRuleCount: 1,
      matchingRulesetCount: 1,
      requiredStatusChecks: ["ci"],
      href: "/namuh-eng/opengithub/settings/branches?branch=main",
    },
    ahead: 0,
    behind: 0,
    pullRequest: null,
  });
  const branch = branchRow();
  return {
    repository: {
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "private",
      viewerPermission: "admin",
    },
    tabs: { overview: 2, active: 1, stale: 1, all: 3, default: 1 },
    filters: { tab: "overview", query: null, staleCutoffDays: 90 },
    defaultBranch,
    branches: [branch],
    total: 1,
    page: 1,
    pageSize: 30,
    hasNextPage: false,
    hasPreviousPage: false,
    emptyState: {
      title: "No branches matched this search",
      message: "Adjust the branch tab or search query.",
      resetHref: "/namuh-eng/opengithub/branches",
    },
    ...overrides,
  };
}

describe("repository branches page", () => {
  it("renders overview sections, destinations, and Editorial primitives", () => {
    const { container } = render(
      <RepositoryBranchesPage
        branchesResult={{ ok: true, branches: branchesView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Branches" })).toBeVisible();
    expect(screen.getByText("Default branch")).toBeVisible();
    expect(screen.getByText("Active branches")).toBeVisible();
    for (const column of [
      "Branch",
      "Updated",
      "Check status",
      "Behind",
      "Ahead",
      "Pull request",
      "Actions",
    ]) {
      expect(screen.getAllByText(column)[0]).toBeVisible();
    }
    expect(screen.getAllByRole("link", { name: "main" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/main",
    );
    expect(
      screen.getByRole("link", { name: "feature/editorial-branches" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/feature%2Feditorial-branches",
    );
    expect(
      screen.getAllByRole("link", { name: "2 passed" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/actions?commit=abc123456789",
    );
    expect(screen.getByRole("link", { name: "Draft #42" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/pull/42",
    );
    expect(screen.getByRole("link", { name: "Protected" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/branches?branch=main",
    );

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(2);
    expect(container.querySelector(".tabs")).not.toBeNull();
    expect(container.querySelector(".input")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("renders branch tabs and search as URL-backed controls", () => {
    render(
      <RepositoryBranchesPage
        branchesResult={{
          ok: true,
          branches: branchesView({
            filters: { tab: "stale", query: "release", staleCutoffDays: 90 },
            page: 2,
            pageSize: 1,
            hasNextPage: true,
            hasPreviousPage: true,
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("tab", { name: /Overview 2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?q=release&pageSize=1",
    );
    expect(screen.getByRole("tab", { name: /Stale 1/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?tab=stale&q=release&pageSize=1",
    );
    expect(screen.getByLabelText("Search branches")).toHaveAttribute(
      "name",
      "q",
    );
    expect(screen.getByRole("button", { name: "Search" })).toHaveAttribute(
      "type",
      "submit",
    );
    expect(screen.getByText("Active filters")).toBeVisible();
    expect(screen.getByRole("link", { name: "Stale" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?q=release&pageSize=1",
    );
    expect(
      screen.getByRole("link", { name: "Search: release" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?tab=stale&pageSize=1",
    );
    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?tab=stale&q=release&pageSize=1",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches?tab=stale&q=release&page=3&pageSize=1",
    );
  });

  it("copies branch names with visible feedback", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(
      <RepositoryBranchesPage
        branchesResult={{ ok: true, branches: branchesView() }}
        repository={repositoryOverview()}
      />,
    );

    const row = screen
      .getByRole("link", { name: "feature/editorial-branches" })
      .closest("article");
    expect(row).not.toBeNull();
    fireEvent.click(
      within(row as HTMLElement).getByRole("button", {
        name: "Copy branch name feature/editorial-branches",
      }),
    );

    await waitFor(() =>
      expect(writeText).toHaveBeenCalledWith("feature/editorial-branches"),
    );
    expect(
      within(row as HTMLElement).getByRole("button", {
        name: "Copied branch name feature/editorial-branches",
      }),
    ).toBeVisible();
  });

  it("renders unavailable and empty states without dead controls", () => {
    const unavailable = render(
      <RepositoryBranchesPage
        branchesResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository branches are unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Branches unavailable" }),
    ).toBeVisible();
    unavailable.unmount();

    const { container } = render(
      <RepositoryBranchesPage
        branchesResult={{
          ok: true,
          branches: branchesView({
            filters: { tab: "stale", query: "release", staleCutoffDays: 90 },
            defaultBranch: null,
            branches: [],
            total: 0,
          }),
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "No branches matched this search" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Reset branch filters" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/branches");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("opens row action menus with concrete and disabled actions", () => {
    render(
      <RepositoryBranchesPage
        branchesResult={{ ok: true, branches: branchesView() }}
        repository={repositoryOverview()}
      />,
    );

    const row = screen
      .getByRole("link", { name: "feature/editorial-branches" })
      .closest("article");
    expect(row).not.toBeNull();
    fireEvent.click(
      within(row as HTMLElement).getByRole("button", { name: "Actions" }),
    );

    expect(
      within(row as HTMLElement)
        .getAllByRole("link", { name: "Activity" })
        .at(-1),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches/feature%2Feditorial-branches",
    );
    expect(
      within(row as HTMLElement).getByRole("link", { name: "Open tree" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/feature%2Feditorial-branches",
    );
    expect(
      within(row as HTMLElement).getByRole("button", {
        name: "Delete branch",
      }),
    ).toBeDisabled();
    expect(
      within(row as HTMLElement).getByRole("button", {
        name: "Restore branch",
      }),
    ).toBeDisabled();

    fireEvent.keyDown(
      within(row as HTMLElement).getByRole("button", { name: "Actions" }),
      { key: "Escape" },
    );
    expect(
      within(row as HTMLElement).queryAllByRole("link", { name: "Activity" }),
    ).toHaveLength(1);
  });

  it("keeps the final branch surface accessible, token-based, and safe", () => {
    const { container } = render(
      <RepositoryBranchesPage
        branchesResult={{ ok: true, branches: branchesView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("tablist")).toBeVisible();
    for (const tabName of ["Overview 2", "Active 1", "Stale 1", "All 3"]) {
      expect(screen.getByRole("tab", { name: tabName })).toHaveAttribute(
        "href",
        expect.stringContaining("/namuh-eng/opengithub/branches"),
      );
    }
    expect(screen.getByLabelText("Search branches")).toHaveClass("input");
    expect(
      screen.getByRole("button", {
        name: "Copy branch name feature/editorial-branches",
      }),
    ).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Activity" }).at(-1),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches/feature%2Feditorial-branches",
    );
    expect(
      screen.getAllByRole("link", { name: "Commits" }).at(-1),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/feature%2Feditorial-branches",
    );
    expect(screen.getAllByText("Open").at(-1)).toHaveClass("chip");
    expect(screen.getByRole("link", { name: "Draft #42" })).toHaveClass("chip");

    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
      expect(button.getAttribute("onclick")).toBeNull();
    }
    expect(container.querySelector("[dangerouslySetInnerHTML]")).toBeNull();
    expect(container.innerHTML).toContain("var(--ink-1)");
    expect(container.innerHTML).toContain("var(--line)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|#f1aeb5|#fff1f3|@primer\/|Octicon/i,
    );
  });
});
