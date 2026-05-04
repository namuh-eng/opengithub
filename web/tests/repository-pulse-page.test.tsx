import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryPulsePage } from "@/components/RepositoryPulsePage";
import type { RepositoryOverview, RepositoryPulseView } from "@/lib/api";

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

function pulseView(
  overrides: Partial<RepositoryPulseView> = {},
): RepositoryPulseView {
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
    },
    metrics: [
      {
        key: "merged_pull_requests",
        label: "Merged pull requests",
        count: 4,
        href: "/namuh-eng/opengithub/pulls?state=merged&from=2026-05-01",
      },
      {
        key: "open_pull_requests",
        label: "Open pull requests",
        count: 2,
        href: "/namuh-eng/opengithub/pulls?state=open",
      },
      {
        key: "closed_issues",
        label: "Closed issues",
        count: 8,
        href: "/namuh-eng/opengithub/issues?state=closed",
      },
      {
        key: "new_issues",
        label: "New issues",
        count: 3,
        href: "/namuh-eng/opengithub/issues?state=open&sort=created-desc&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
      },
    ],
    summary: {
      sentence:
        "2 authors pushed 12 commits touching 18 files with 420 additions and 90 deletions in the 1w window.",
      commits: 12,
      filesChanged: 18,
      additions: 420,
      deletions: 90,
      authors: 2,
      mergedPullRequests: 4,
      openPullRequests: 2,
      closedIssues: 8,
      newIssues: 3,
      openIssues: 6,
      releases: 1,
    },
    topCommitters: [
      {
        userId: "user-1",
        login: "mona",
        avatarUrl: null,
        commits: 9,
        filesChanged: 12,
        additions: 320,
        deletions: 45,
        profileHref: "/mona",
        commitsHref: "/namuh-eng/opengithub/commits/main?author=mona",
      },
      {
        userId: "user-2",
        login: "octo",
        avatarUrl: null,
        commits: 3,
        filesChanged: 6,
        additions: 100,
        deletions: 45,
        profileHref: "/octo",
        commitsHref: "/namuh-eng/opengithub/commits/main?author=octo",
      },
    ],
    releases: [
      {
        kind: "release",
        number: null,
        title: "Pulse preview",
        state: "published",
        authorLogin: "mona",
        authorAvatarUrl: null,
        href: "/namuh-eng/opengithub/releases/tag/v1.2.3",
        occurredAt: "2026-05-06T08:00:00Z",
      },
    ],
    mergedPullRequests: [
      {
        kind: "pull_request",
        number: 41,
        title: "Merge Pulse summary",
        state: "merged",
        authorLogin: "octo",
        authorAvatarUrl: null,
        href: "/namuh-eng/opengithub/pull/41",
        occurredAt: "2026-05-06T07:00:00Z",
      },
    ],
    issueActivity: [
      {
        kind: "issue",
        number: 9,
        title: "Track Pulse activity",
        state: "closed",
        authorLogin: "mona",
        authorAvatarUrl: null,
        href: "/namuh-eng/opengithub/issues/9",
        occurredAt: "2026-05-06T06:00:00Z",
      },
    ],
    snapshot: {
      cacheKey: "1w:202605010000:202605070000",
      computedAt: "2026-05-07T00:00:00Z",
      expiresAt: "2026-05-07T00:10:00Z",
      stale: false,
    },
    ...overrides,
  };
}

describe("RepositoryPulsePage", () => {
  it("renders the Insights shell, overview metrics, chart, and activity links", () => {
    const { container } = render(
      <RepositoryPulsePage
        pulseResult={{ ok: true, pulse: pulseView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Repository activity" }),
    ).toBeVisible();
    expect(screen.getByText("May 1, 2026 - May 7, 2026")).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Pulse Activity summary for the selected period",
      }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulse");
    expect(
      screen.getByRole("link", {
        name: "Contributors Contributor commit activity",
      }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/graphs/contributors");
    expect(
      screen.getByRole("button", { name: "Period: Last week" }),
    ).toHaveAttribute("aria-expanded", "false");
    expect(screen.getByText("Active period")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Commit history" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/commits/main");

    for (const label of [
      "Merged pull requests",
      "Open pull requests",
      "Closed issues",
      "New issues",
    ]) {
      expect(screen.getAllByText(label)[0]).toBeVisible();
    }
    expect(
      screen.getByRole("link", { name: /Merged pull requests/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/pulls?state=merged&from=2026-05-01",
    );
    expect(screen.getByRole("link", { name: /New issues/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/issues?state=open&sort=created-desc&since=2026-05-01T00%3A00%3A00Z&until=2026-05-07T00%3A00%3A00Z",
    );
    expect(
      screen.getByRole("img", { name: "Top committers bar chart" }),
    ).toBeVisible();

    const table = screen.getByRole("table", {
      name: "Top committers data table",
    });
    expect(within(table).getByText("mona")).toBeVisible();
    expect(screen.getByRole("link", { name: /mona/ })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getByRole("link", { name: "9 commits" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/main?author=mona",
    );
    expect(screen.getByRole("link", { name: /Pulse preview/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/releases/tag/v1.2.3",
    );
    expect(
      screen.getByRole("link", { name: /Merge Pulse summary/ }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pull/41");
    expect(
      screen.getByRole("link", { name: /Track Pulse activity/ }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/issues/9");

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.innerHTML).toContain("var(--accent)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
  });

  it("opens a URL-backed period menu and closes it with Escape", () => {
    render(
      <RepositoryPulsePage
        pulseResult={{
          ok: true,
          pulse: pulseView({
            period: {
              key: "3d",
              label: "Last 3 days",
              startedAt: "2026-05-04T00:00:00Z",
              endedAt: "2026-05-07T00:00:00Z",
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    const button = screen.getByRole("button", { name: "Period: Last 3 days" });
    fireEvent.click(button);

    expect(button).toHaveAttribute("aria-expanded", "true");
    const menu = screen.getByRole("menu", { name: "Pulse period" });
    expect(
      within(menu).getByRole("menuitem", { name: "Last 24 hours" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulse?period=24h");
    expect(
      within(menu).getByRole("menuitem", { name: /Last 3 days/ }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulse?period=3d");
    expect(
      within(menu).getByRole("menuitem", { name: "Last week" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulse");
    expect(
      within(menu).getByRole("menuitem", { name: "Last month" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulse?period=1m");

    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("menu", { name: "Pulse period" }),
    ).not.toBeInTheDocument();
  });

  it("renders truthful empty states and recovery links", () => {
    render(
      <RepositoryPulsePage
        pulseResult={{
          ok: true,
          pulse: pulseView({
            topCommitters: [],
            releases: [],
            mergedPullRequests: [],
            issueActivity: [],
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByText("No commits were indexed for this Pulse window."),
    ).toBeVisible();
    expect(
      screen.getAllByText(
        "No activity in this section for the selected period.",
      ),
    ).toHaveLength(3);
    expect(screen.getByRole("link", { name: "View releases" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/releases",
    );
    expect(
      screen.getByRole("link", { name: "View pull requests" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pulls?state=merged");
    expect(screen.getByRole("link", { name: "View issues" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/issues",
    );
  });

  it("renders API failures inside the Insights shell", () => {
    render(
      <RepositoryPulsePage
        pulseResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository Pulse is unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Pulse unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText("Repository Pulse is unavailable right now."),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Back to Code" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub",
    );
  });
});
