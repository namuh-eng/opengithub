import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryTrafficPage } from "@/components/RepositoryTrafficPage";
import type { RepositoryOverview, RepositoryTrafficView } from "@/lib/api";

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

function trafficView(
  overrides: Partial<RepositoryTrafficView> = {},
): RepositoryTrafficView {
  return {
    repository: {
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "private",
      viewerPermission: "admin",
      href: "/namuh-eng/opengithub",
    },
    window: {
      key: "14d",
      label: "Last 14 days",
      startedOn: "2026-04-24",
      endedOn: "2026-05-07",
      timezone: "UTC",
      dayCount: 14,
      clonesUpdateCadence: "hourly",
      visitorsUpdateCadence: "hourly",
      referrersUpdateCadence: "daily",
      popularContentUpdateCadence: "daily",
    },
    summaries: {
      clonesTotal: 42,
      clonesUnique: 12,
      visitorsTotal: 220,
      visitorsUnique: 87,
      referrersTotal: 2,
      popularContentTotal: 2,
    },
    clones: [
      { date: "2026-05-05", total: 10, unique: 3 },
      { date: "2026-05-06", total: 14, unique: 5 },
      { date: "2026-05-07", total: 18, unique: 4 },
    ],
    visitors: [
      { date: "2026-05-05", total: 70, unique: 31 },
      { date: "2026-05-06", total: 80, unique: 32 },
      { date: "2026-05-07", total: 70, unique: 24 },
    ],
    referrers: [
      {
        referrer: "https://search.opengithub.local/results?q=traffic",
        href: "https://search.opengithub.local/results?q=traffic",
        totalViews: 120,
        uniqueVisitors: 44,
      },
      {
        referrer: "https://example.com/docs",
        href: "https://example.com/docs",
        totalViews: 18,
        uniqueVisitors: 9,
      },
    ],
    popularContent: [
      {
        path: "README.md",
        title: "README",
        href: "/namuh-eng/opengithub/blob/main/README.md",
        totalViews: 60,
        uniqueVisitors: 28,
      },
      {
        path: "docs/traffic report.md",
        title: "Traffic report",
        href: "/namuh-eng/opengithub/blob/main/docs/traffic%20report.md",
        totalViews: 16,
        uniqueVisitors: 7,
      },
    ],
    snapshot: {
      cacheKey: "traffic:repo-1:20260424:20260507",
      computedAt: "2026-05-07T01:00:00Z",
      expiresAt: "2026-05-07T02:00:00Z",
      stale: false,
    },
    ...overrides,
  };
}

describe("RepositoryTrafficPage", () => {
  it("renders the Insights shell, summary cards, charts, and safe links", () => {
    const { container } = render(
      <RepositoryTrafficPage
        repository={repositoryOverview()}
        trafficResult={{ ok: true, traffic: trafficView() }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Traffic analytics" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Traffic Clone and visitor analytics",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.getByText(/Apr 24, 2026 - May 7, 2026/)).toBeVisible();
    expect(screen.getAllByText("Last 14 days").length).toBeGreaterThanOrEqual(
      1,
    );
    expect(screen.getByText("Fresh snapshot")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Commit history" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/commits/main");

    const metrics = screen.getByLabelText("Traffic summary metrics");
    expect(within(metrics).getByText("Full clones")).toBeVisible();
    expect(within(metrics).getByText("Visitors")).toBeVisible();
    expect(within(metrics).getAllByText("42")[0]).toBeVisible();
    expect(within(metrics).getAllByText("220")[0]).toBeVisible();
    expect(
      screen.getByRole("img", { name: "Clones line chart" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", {
        name: "Clones May 7, 2026: 18 clones, 4 unique cloners",
      }),
    ).toBeVisible();
    expect(screen.getAllByText("Selected point").length).toBeGreaterThan(0);
    expect(
      screen.getByText("May 5, 2026: 10 clones, 3 unique cloners"),
    ).toBeVisible();
    expect(
      screen.getByRole("table", { name: "Clones data table" }),
    ).toBeVisible();
    expect(
      screen.getByRole("img", { name: "Visitors line chart" }),
    ).toBeVisible();
    expect(
      screen.getByRole("table", { name: "Visitors data table" }),
    ).toBeVisible();

    const referrer = screen.getByRole("link", {
      name: "https://search.opengithub.local/results?q=traffic",
    });
    expect(referrer).toHaveAttribute(
      "href",
      "https://search.opengithub.local/results?q=traffic",
    );
    expect(referrer).toHaveAttribute("target", "_blank");
    expect(referrer).toHaveAttribute("rel", "noopener noreferrer");
    expect(
      screen.getByRole("link", { name: "Traffic report" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/docs/traffic%20report.md",
    );

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(6);
    expect(container.querySelector(".chip.ok")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--accent)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|#f1aeb5|#fff1f3|@primer\/|Octicon/i,
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("reveals exact traffic values when chart points receive focus or hover", () => {
    render(
      <RepositoryTrafficPage
        repository={repositoryOverview()}
        trafficResult={{ ok: true, traffic: trafficView() }}
      />,
    );

    const clonePoint = screen.getByRole("button", {
      name: "Clones May 7, 2026: 18 clones, 4 unique cloners",
    });
    fireEvent.focus(clonePoint);
    expect(
      screen.getByText("May 7, 2026: 18 clones, 4 unique cloners"),
    ).toBeVisible();
    expect(clonePoint).toHaveAttribute(
      "aria-describedby",
      "clones-traffic-point-details",
    );

    const visitorPoint = screen.getByRole("button", {
      name: "Visitors May 6, 2026: 80 views, 32 unique visitors",
    });
    fireEvent.mouseEnter(visitorPoint);
    expect(
      screen.getByText("May 6, 2026: 80 views, 32 unique visitors"),
    ).toBeVisible();
  });

  it("renders permission denial without leaking traffic counts", () => {
    const { container } = render(
      <RepositoryTrafficPage
        repository={repositoryOverview({ viewerPermission: "read" })}
        trafficResult={{
          ok: false,
          status: 403,
          code: "traffic_access_required",
          message: "Repository traffic is available to users with push access.",
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Traffic unavailable" }),
    ).toBeVisible();
    expect(screen.getAllByText(/push access/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByRole("link", { name: "Back to Code" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub",
    );
    expect(container).not.toHaveTextContent("42");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });

  it("renders empty traffic states with truthful copy", () => {
    render(
      <RepositoryTrafficPage
        repository={repositoryOverview()}
        trafficResult={{
          ok: true,
          traffic: trafficView({
            summaries: {
              clonesTotal: 0,
              clonesUnique: 0,
              visitorsTotal: 0,
              visitorsUnique: 0,
              referrersTotal: 0,
              popularContentTotal: 0,
            },
            clones: [
              { date: "2026-05-06", total: 0, unique: 0 },
              { date: "2026-05-07", total: 0, unique: 0 },
            ],
            visitors: [
              { date: "2026-05-06", total: 0, unique: 0 },
              { date: "2026-05-07", total: 0, unique: 0 },
            ],
            referrers: [],
            popularContent: [],
          }),
        }}
      />,
    );

    expect(
      screen.getByText("No external referrers were recorded for this window."),
    ).toBeVisible();
    expect(
      screen.getByText("No repository paths were viewed during this window."),
    ).toBeVisible();
  });
});
