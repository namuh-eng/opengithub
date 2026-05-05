import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDependabotAlertsPage } from "@/components/RepositoryDependabotAlertsPage";
import type {
  RepositoryDependabotAlertsView,
  RepositoryOverview,
} from "@/lib/api";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: vi.fn() }),
}));

afterEach(() => {
  vi.restoreAllMocks();
});

function repositoryOverview(): RepositoryOverview {
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
    viewerPermission: "write",
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
      forksCount: 2,
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
  };
}

function dependabotView(
  overrides: Partial<RepositoryDependabotAlertsView> = {},
): RepositoryDependabotAlertsView {
  const base: RepositoryDependabotAlertsView = {
    repository: {
      id: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      visibility: "private",
      defaultBranch: "main",
      securityHref: "/namuh-eng/opengithub/security",
      policyHref: "/namuh-eng/opengithub/security/policy",
      advisoriesHref: "/namuh-eng/opengithub/security/advisories",
    },
    viewer: {
      permission: "write",
      canRead: true,
      canWrite: true,
      canEditPolicy: true,
      canViewPrivateAlertCounts: true,
    },
    availability: {
      enabled: true,
      indexed: true,
      message:
        "Dependabot alerts are derived from indexed dependency manifests and advisories.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    filters: {
      state: "open",
      query: null,
      package: null,
      ecosystem: null,
      manifest: null,
      scope: null,
      severity: null,
      sort: "most_important",
    },
    counts: {
      open: 2,
      closed: 1,
      total: 3,
      visible: 2,
    },
    alerts: [
      {
        id: "alert-1",
        number: 1,
        state: "open",
        scope: "production",
        package: {
          id: "pkg-1",
          ecosystem: "npm",
          name: "@testing-library/react",
          href: "/packages/npm/%40testing-library%2Freact",
        },
        advisory: {
          id: "adv-1",
          identifier: "GHSA-demo-0001",
          severity: "high",
          title: "Demo parser accepts unsafe input",
          href: "/advisories/GHSA-demo-0001",
          publishedAt: "2026-05-04T00:00:00Z",
        },
        manifestPath: "package.json",
        manifestHref: "/namuh-eng/opengithub/blob/main/package.json",
        lockfilePath: "package-lock.json",
        lockfileHref: "/namuh-eng/opengithub/blob/main/package-lock.json",
        vulnerableRequirements: "< 2.0.0",
        currentVersion: "1.0.0",
        fixedVersion: "2.0.0",
        relationship: "direct",
        assignees: [
          {
            id: "user-1",
            login: "jaeyun",
            avatarUrl: null,
            href: "/jaeyun",
          },
        ],
        href: "/namuh-eng/opengithub/security/dependabot/1",
        detectedAt: "2026-05-05T00:00:00Z",
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "alert-2",
        number: 2,
        state: "open",
        scope: "development",
        package: {
          id: "pkg-2",
          ecosystem: "cargo",
          name: "sqlx",
          href: "/packages/cargo/sqlx",
        },
        advisory: {
          id: "adv-2",
          identifier: "GHSA-demo-0002",
          severity: "moderate",
          title: "SQLx demo advisory",
          href: "/advisories/GHSA-demo-0002",
          publishedAt: null,
        },
        manifestPath: "crates/api/Cargo.toml",
        manifestHref:
          "/namuh-eng/opengithub/blob/main/crates%2Fapi%2FCargo.toml",
        lockfilePath: null,
        lockfileHref: null,
        vulnerableRequirements: "< 0.8.2",
        currentVersion: "0.8.0",
        fixedVersion: null,
        relationship: "transitive",
        assignees: [],
        href: "/namuh-eng/opengithub/security/dependabot/2",
        detectedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    packages: [
      {
        package: {
          id: "pkg-1",
          ecosystem: "npm",
          name: "@testing-library/react",
          href: "/packages/npm/%40testing-library%2Freact",
        },
        openCount: 1,
        selected: false,
      },
    ],
    manifests: [
      {
        path: "package.json",
        ecosystem: "npm",
        href: "/namuh-eng/opengithub/blob/main/package.json",
        openCount: 1,
        selected: false,
      },
    ],
    links: {
      listHref: "/namuh-eng/opengithub/security/dependabot",
      openHref: "/namuh-eng/opengithub/security/dependabot?state=open",
      closedHref: "/namuh-eng/opengithub/security/dependabot?state=closed",
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    freshness: {
      computedAt: "2026-05-05T00:00:00Z",
      cadence: "daily",
    },
  };

  return { ...base, ...overrides };
}

describe("RepositoryDependabotAlertsPage", () => {
  it("renders the Editorial Dependabot list with active navigation, tabs, rows, and concrete links", () => {
    const { container } = render(
      <RepositoryDependabotAlertsPage
        dependabotResult={{ ok: true, dependabot: dependabotView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Dependabot Dependency alerts and updates",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { name: "Dependabot alerts" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: /Open 2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot?state=open&sort=most_important",
    );
    expect(screen.getByRole("link", { name: /Closed 1/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot?state=closed&sort=most_important",
    );

    const rows = screen.getByRole("list", {
      name: "Dependabot vulnerability alerts",
    });
    expect(within(rows).getByText("@testing-library/react")).toBeVisible();
    expect(
      within(rows).getByText("Demo parser accepts unsafe input"),
    ).toBeVisible();
    expect(
      within(rows).getByRole("link", { name: "package.json" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/blob/main/package.json");
    expect(
      within(rows).getAllByRole("link", { name: "View alert" })[0],
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/dependabot/1");
    expect(
      screen.getByRole("link", { name: "Vulnerability settings" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("builds URL-backed filter and sort destinations", () => {
    render(
      <RepositoryDependabotAlertsPage
        dependabotResult={{ ok: true, dependabot: dependabotView() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Package: All packages" }),
    );
    expect(
      screen.getByRole("menuitem", { name: /npm:@testing-library\/react/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot?state=open&package=npm%3A%40testing-library%2Freact&sort=most_important",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Sort: Most important" }),
    );
    expect(
      screen.getByRole("menuitem", { name: "Recently detected" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot?state=open&sort=recently_detected",
    );
    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot",
    );
  });

  it("selects visible rows and submits bulk dismiss through the same-origin route", async () => {
    const fetchMock = vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: async () => ({
        requestedCount: 2,
        updatedCount: 2,
        results: [],
        message: "2 Dependabot alerts updated.",
      }),
    } as Response);

    render(
      <RepositoryDependabotAlertsPage
        dependabotResult={{ ok: true, dependabot: dependabotView() }}
        repository={repositoryOverview()}
      />,
    );

    const summary = screen.getByRole("region", {
      name: "Dependabot alert summary",
    });
    expect(within(summary).getByText("Selected")).toBeVisible();
    expect(within(summary).getByText("0")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Select all visible" }));
    expect(screen.getByRole("button", { name: "Clear visible" })).toBeVisible();
    fireEvent.change(screen.getByLabelText("Comment"), {
      target: { value: "Batch triaged from the alerts queue" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Dismiss selected" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/security/dependabot/bulk",
      expect.objectContaining({
        body: JSON.stringify({
          action: "dismiss",
          alertIds: ["alert-1", "alert-2"],
          dismissalComment: "Batch triaged from the alerts queue",
          dismissalReason: "fix_started",
        }),
        method: "POST",
      }),
    );
    expect(
      await screen.findByText("2 Dependabot alerts updated."),
    ).toBeVisible();
  });

  it("renders disabled and unavailable states with concrete recovery links", () => {
    const disabled = dependabotView({
      availability: {
        enabled: false,
        indexed: false,
        message: "Dependabot alerts are disabled for this repository.",
        disabledReason: "Enable vulnerability alerts to monitor manifests.",
        settingsHref: "/namuh-eng/opengithub/settings/security",
      },
      alerts: [],
      counts: { open: 0, closed: 0, total: 0, visible: 0 },
    });

    const { rerender } = render(
      <RepositoryDependabotAlertsPage
        dependabotResult={{ ok: true, dependabot: disabled }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", {
        name: "Vulnerability alerts are disabled.",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Open vulnerability settings" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");

    rerender(
      <RepositoryDependabotAlertsPage
        dependabotResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Dependabot alerts are unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Dependabot alerts unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to security overview" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security");
  });
});
