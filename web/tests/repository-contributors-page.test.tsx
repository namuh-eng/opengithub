import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryContributorsPage } from "@/components/RepositoryContributorsPage";
import type { RepositoryContributorsView, RepositoryOverview } from "@/lib/api";

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
      releasesCount: 1,
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

function contributorsView(
  overrides: Partial<RepositoryContributorsView> = {},
): RepositoryContributorsView {
  return {
    repository: {
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "private",
      viewerPermission: "admin",
      href: "/namuh-eng/opengithub",
    },
    period: {
      key: "1w",
      label: "Last week",
      startedAt: "2026-05-01T00:00:00Z",
      endedAt: "2026-05-07T00:00:00Z",
      bucketCount: 2,
    },
    threshold: {
      commitLimit: 5000,
      commitsConsidered: 12,
      lineCountsOmitted: false,
      message:
        "Line additions and deletions are included for this bounded commit range.",
    },
    totals: {
      commits: 12,
      authors: 2,
      additions: 420,
      deletions: 90,
    },
    weeks: [
      {
        weekStart: "2026-05-01T00:00:00Z",
        weekEnd: "2026-05-04T00:00:00Z",
        commits: 4,
        additions: 120,
        deletions: 30,
      },
      {
        weekStart: "2026-05-04T00:00:00Z",
        weekEnd: "2026-05-07T00:00:00Z",
        commits: 8,
        additions: 300,
        deletions: 60,
      },
    ],
    contributors: [
      {
        userId: "user-1",
        login: "mona",
        authorStatus: "active",
        isBot: false,
        avatarUrl: null,
        totalCommits: 9,
        totalAdditions: 320,
        totalDeletions: 45,
        profileHref: "/mona",
        commitsHref:
          "/namuh-eng/opengithub/commits/main?author=mona&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
        weeks: [
          {
            weekStart: "2026-05-01T00:00:00Z",
            commits: 3,
            additions: 100,
            deletions: 10,
          },
          {
            weekStart: "2026-05-04T00:00:00Z",
            commits: 6,
            additions: 220,
            deletions: 35,
          },
        ],
      },
      {
        userId: "user-2",
        login: "automation-runner[bot]",
        authorStatus: "bot",
        isBot: true,
        avatarUrl: null,
        totalCommits: 3,
        totalAdditions: 100,
        totalDeletions: 45,
        profileHref: "/automation-runner%5Bbot%5D",
        commitsHref:
          "/namuh-eng/opengithub/commits/main?author=automation-runner%5Bbot%5D&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
        weeks: [
          {
            weekStart: "2026-05-04T00:00:00Z",
            commits: 3,
            additions: 100,
            deletions: 45,
          },
        ],
      },
    ],
    snapshot: {
      cacheKey: "contributors:main:1w:202605010000:202605070000",
      computedAt: "2026-05-07T00:00:00Z",
      expiresAt: "2026-05-07T00:10:00Z",
      stale: false,
    },
    ...overrides,
  };
}

describe("RepositoryContributorsPage", () => {
  it("renders the Insights shell, charts, data table, and concrete links", () => {
    const { container } = render(
      <RepositoryContributorsPage
        contributorsResult={{ ok: true, contributors: contributorsView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Contributor analytics" }),
    ).toBeVisible();
    expect(screen.getByText(/Default branch scope:/)).toBeVisible();
    expect(screen.getByText("main")).toBeVisible();
    expect(screen.getByText(/May 1, 2026 - May 7, 2026/)).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Contributors Contributor commit activity",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("button", { name: "Period: Last week" }),
    ).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.getAllByRole("link", { name: "View as data table" })[0],
    ).toHaveAttribute("href", "#contributors-data-table-panel");
    expect(
      screen.getByRole("button", { name: "View as data table" }),
    ).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(screen.getByRole("button", { name: "View as data table" }));
    expect(
      screen.getByRole("link", { name: "Commit history" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/commits/main");
    expect(
      screen.getByRole("img", {
        name: "Repository commits over time chart",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("img", { name: "mona weekly commits chart" }),
    ).toBeVisible();
    expect(screen.getAllByRole("link", { name: "mona" })[0]).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getByRole("link", { name: "9 commits" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/main?author=mona&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
    );
    expect(screen.getByText("Bot")).toBeVisible();

    const table = screen.getByRole("table", {
      name: "Repository contributors data table",
    });
    expect(within(table).getAllByText("Repository")[0]).toBeVisible();
    expect(within(table).getAllByText("mona")[0]).toBeVisible();
    expect(within(table).getAllByText("May 4, 2026")[0]).toBeVisible();

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.innerHTML).toContain("var(--accent)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("builds URL-backed range controls and real CSV actions", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });

    render(
      <RepositoryContributorsPage
        contributorsResult={{ ok: true, contributors: contributorsView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("slider", { name: "Start week" })).toBeEnabled();
    fireEvent.change(screen.getByRole("slider", { name: "Start week" }), {
      target: { value: "1" },
    });
    expect(screen.getByRole("link", { name: "Apply" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/graphs/contributors?start=2026-05-04T00%3A00%3A00Z&end=2026-05-07T00%3A00%3A00Z",
    );
    expect(screen.getByRole("link", { name: "Clear" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/graphs/contributors",
    );

    fireEvent.click(screen.getByRole("button", { name: "Copy CSV" }));
    expect(writeText).toHaveBeenCalledWith(
      expect.stringContaining('"scope","week","commits"'),
    );
    expect(await screen.findByText("CSV copied")).toBeVisible();
    expect(screen.getByRole("link", { name: "Download CSV" })).toHaveAttribute(
      "download",
      "repository-contributors.csv",
    );
  });

  it("opens a URL-backed Contributors period menu and closes it with Escape", () => {
    render(
      <RepositoryContributorsPage
        contributorsResult={{
          ok: true,
          contributors: contributorsView({
            period: {
              key: "3d",
              label: "Last 3 days",
              startedAt: "2026-05-04T00:00:00Z",
              endedAt: "2026-05-07T00:00:00Z",
              bucketCount: 1,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    const button = screen.getByRole("button", { name: "Period: Last 3 days" });
    fireEvent.click(button);

    expect(button).toHaveAttribute("aria-expanded", "true");
    const menu = screen.getByRole("menu", { name: "Contributors period" });
    expect(
      within(menu).getByRole("menuitem", { name: "Last 24 hours" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/graphs/contributors?period=24h",
    );
    expect(
      within(menu).getByRole("menuitem", { name: /Last 3 days/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/graphs/contributors?period=3d",
    );
    expect(
      within(menu).getByRole("menuitem", { name: "Last week" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/graphs/contributors");
    expect(
      within(menu).getByRole("menuitem", { name: "Last month" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/graphs/contributors?period=1m",
    );

    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("menu", { name: "Contributors period" }),
    ).not.toBeInTheDocument();
  });

  it("renders empty and line-count omission states truthfully", () => {
    render(
      <RepositoryContributorsPage
        contributorsResult={{
          ok: true,
          contributors: contributorsView({
            threshold: {
              commitLimit: 5000,
              commitsConsidered: 6000,
              lineCountsOmitted: true,
              message:
                "Line additions and deletions are omitted because this range includes more than 5000 commits.",
            },
            totals: {
              commits: 0,
              authors: 0,
              additions: null,
              deletions: null,
            },
            weeks: [],
            contributors: [],
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getAllByText("Line counts omitted")[0]).toBeVisible();
    expect(
      screen.getByText("No commits were indexed for this contributors window."),
    ).toBeVisible();
    expect(screen.getByText("No contributor activity")).toBeVisible();
    expect(screen.getAllByText("omitted")[0]).toBeVisible();
  });

  it("renders API failures inside the Contributors shell", () => {
    render(
      <RepositoryContributorsPage
        contributorsResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository Contributors is unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Contributors unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText("Repository Contributors is unavailable right now."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Back to Code" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub",
    );
  });
});
