import { fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryBranchesPage } from "@/components/RepositoryBranchesPage";
import type { RepositoryOverview, RepositoryRefSummary } from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Branches test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "owner",
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
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
  return { ...base, ...overrides };
}

function branch(
  shortName: string,
  overrides: Partial<RepositoryRefSummary> = {},
): RepositoryRefSummary {
  return {
    name: `refs/heads/${shortName}`,
    shortName,
    kind: "branch",
    href: `/mona/octo-app/tree/${encodeURIComponent(shortName)}`,
    samePathHref: `/mona/octo-app/tree/${encodeURIComponent(shortName)}`,
    active: shortName === "main",
    targetShortOid: "abcdef1",
    updatedAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

const refs: RepositoryRefSummary[] = [
  branch("main"),
  branch("feature/actions", {
    href: "/mona/octo-app/tree/feature%2Factions",
    samePathHref: "/mona/octo-app/tree/feature%2Factions",
    updatedAt: "2026-04-20T00:00:00Z",
    targetShortOid: "1234567",
  }),
  branch("archive/old-ui", {
    href: "/mona/octo-app/tree/archive%2Fold-ui",
    samePathHref: "/mona/octo-app/tree/archive%2Fold-ui",
    updatedAt: "2025-10-01T00:00:00Z",
    targetShortOid: "7654321",
  }),
  {
    name: "refs/tags/v1.0.0",
    shortName: "v1.0.0",
    kind: "tag",
    href: "/mona/octo-app/tree/v1.0.0",
    samePathHref: "/mona/octo-app/tree/v1.0.0",
    active: false,
    targetShortOid: "tag0001",
    updatedAt: "2026-04-01T00:00:00Z",
  },
];

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-05-02T00:00:00Z"));
  Object.assign(navigator, {
    clipboard: { writeText: vi.fn() },
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("RepositoryBranchesPage", () => {
  it("renders overview with default and active branch sections from refs", () => {
    render(
      <RepositoryBranchesPage refs={refs} repository={repositoryOverview()} />,
    );

    expect(screen.getByRole("heading", { name: "Branches" })).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Default branch" }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Active branches" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "main" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tree/main",
    );
    expect(
      screen.getByRole("link", { name: "feature/actions" }),
    ).toHaveAttribute("href", "/mona/octo-app/tree/feature%2Factions");
    expect(screen.queryByRole("link", { name: "archive/old-ui" })).toBeNull();
    expect(screen.getAllByText("Default branch").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Checks not reported")).toHaveLength(2);
  });

  it("filters branches case-insensitively and excludes tag refs", () => {
    render(
      <RepositoryBranchesPage refs={refs} repository={repositoryOverview()} />,
    );

    fireEvent.change(
      screen.getByRole("searchbox", { name: "Search branches" }),
      {
        target: { value: "FEATURE" },
      },
    );

    expect(screen.getByRole("link", { name: "feature/actions" })).toBeVisible();
    expect(screen.queryByRole("link", { name: "main" })).toBeNull();
    expect(screen.queryByText("v1.0.0")).toBeNull();
  });

  it("switches to stale and all tabs without losing repository actions", () => {
    render(
      <RepositoryBranchesPage refs={refs} repository={repositoryOverview()} />,
    );

    fireEvent.click(screen.getByRole("tab", { name: "Stale" }));
    expect(
      screen.getByRole("heading", { name: "Stale branches" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "archive/old-ui" })).toBeVisible();
    expect(screen.queryByRole("link", { name: "feature/actions" })).toBeNull();

    fireEvent.click(screen.getByRole("tab", { name: "All" }));
    const row = screen
      .getByRole("link", { name: "archive/old-ui" })
      .closest("tr");
    expect(row).not.toBeNull();
    const rowScope = within(row as HTMLTableRowElement);
    expect(rowScope.getByRole("link", { name: "Find PRs" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pulls?q=head%3Amona%3Aarchive%2Fold-ui",
    );
    expect(rowScope.getByText("Actions")).toBeVisible();
    expect(rowScope.getByRole("link", { name: "View rules" })).toHaveAttribute(
      "href",
      "/mona/octo-app/settings/branches?pattern=archive%2Fold-ui",
    );
  });

  it("copies branch names through the browser clipboard", () => {
    render(
      <RepositoryBranchesPage refs={refs} repository={repositoryOverview()} />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Copy branch name main" }),
    );

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("main");
  });
});
