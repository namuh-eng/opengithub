import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryCodeScanningAlertsPage } from "@/components/RepositoryCodeScanningAlertsPage";
import type {
  RepositoryCodeScanningAlertsView,
  RepositoryOverview,
} from "@/lib/api";

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

function codeScanningView(
  overrides: Partial<RepositoryCodeScanningAlertsView> = {},
): RepositoryCodeScanningAlertsView {
  const base: RepositoryCodeScanningAlertsView = {
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
        "Code scanning alerts are normalized from SARIF analysis and Actions runs.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    filters: {
      state: "open",
      query: null,
      severity: null,
      securitySeverity: null,
      tool: null,
      branch: null,
      ref: null,
      tag: null,
      applicationCode: null,
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
        ruleId: "js/sql-injection",
        ruleName: "Unsanitized SQL query",
        message: "User-controlled data reaches a SQL sink.",
        severity: "warning",
        securitySeverity: "high",
        toolName: "CodeQL",
        path: "crates/api/src/routes/search.rs",
        pathHref:
          "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs#L42",
        startLine: 42,
        endLine: 45,
        refName: "refs/heads/main",
        branchName: "main",
        isDefaultBranch: true,
        linkedIssue: {
          id: "issue-1",
          number: 22,
          title: "Track SQL sink hardening",
          href: "/namuh-eng/opengithub/issues/22",
        },
        assignees: [
          {
            id: "user-1",
            login: "jaeyun",
            avatarUrl: null,
            href: "/jaeyun",
          },
        ],
        href: "/namuh-eng/opengithub/security/code-scanning/1",
        detectedAt: "2026-05-05T00:00:00Z",
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "alert-2",
        number: 2,
        state: "open",
        ruleId: "rust/path-traversal",
        ruleName: "Path traversal in archive reader",
        message: "Archive entries are joined without path normalization.",
        severity: "error",
        securitySeverity: "critical",
        toolName: "Semgrep",
        path: "crates/api/src/domain/archive.rs",
        pathHref:
          "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Fdomain%2Farchive.rs#L88",
        startLine: 88,
        endLine: null,
        refName: "refs/heads/main",
        branchName: "main",
        isDefaultBranch: true,
        linkedIssue: null,
        assignees: [],
        href: "/namuh-eng/opengithub/security/code-scanning/2",
        detectedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    tools: [
      {
        name: "CodeQL",
        version: "2.17.0",
        status: "completed",
        alertCount: 1,
        latestRunAt: "2026-05-05T00:00:00Z",
      },
      {
        name: "Semgrep",
        version: null,
        status: "completed",
        alertCount: 1,
        latestRunAt: "2026-05-04T00:00:00Z",
      },
    ],
    branches: [
      { name: "main", openCount: 2, selected: false },
      { name: "release/next", openCount: 1, selected: false },
    ],
    links: {
      listHref: "/namuh-eng/opengithub/security/code-scanning",
      openHref: "/namuh-eng/opengithub/security/code-scanning?state=open",
      closedHref: "/namuh-eng/opengithub/security/code-scanning?state=closed",
      uploadHref: "/api/repos/namuh-eng/opengithub/code-scanning/sarifs",
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    freshness: {
      computedAt: "2026-05-05T00:00:00Z",
      cadence: "per upload",
    },
  };

  return { ...base, ...overrides };
}

describe("RepositoryCodeScanningAlertsPage", () => {
  it("renders the Editorial Code scanning list with active navigation, tabs, rows, and concrete links", () => {
    const { container } = render(
      <RepositoryCodeScanningAlertsPage
        codeScanningResult={{ ok: true, codeScanning: codeScanningView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Code scanning Static analysis findings",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { name: "Code scanning alerts" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: /Open 2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning?state=open&sort=most_important",
    );
    expect(screen.getByRole("link", { name: /Closed 1/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning?state=closed&sort=most_important",
    );

    const rows = screen.getByRole("list", { name: "Code scanning alerts" });
    expect(within(rows).getByText("Unsanitized SQL query")).toBeVisible();
    expect(
      within(rows).getByRole("link", {
        name: "crates/api/src/routes/search.rs:42",
      }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs#L42",
    );
    expect(
      within(rows).getByRole("link", { name: "Linked issue #22" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/issues/22");
    expect(
      within(rows).getAllByRole("link", { name: "View alert" })[0],
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/code-scanning/1");
    expect(screen.getByRole("link", { name: "Upload SARIF" })).toHaveAttribute(
      "href",
      "/api/repos/namuh-eng/opengithub/code-scanning/sarifs",
    );
    expect(
      screen.getByRole("link", { name: "Code security settings" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("builds URL-backed filter and sort destinations", () => {
    render(
      <RepositoryCodeScanningAlertsPage
        codeScanningResult={{ ok: true, codeScanning: codeScanningView() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Tool: All tools" }));
    expect(
      screen.getByRole("menuitem", { name: /CodeQL 2\.17\.0/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning?state=open&tool=CodeQL&sort=most_important",
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Security severity: All security severities",
      }),
    );
    expect(screen.getByRole("menuitem", { name: "Critical" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning?state=open&security_severity=critical&sort=most_important",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Sort: Most important" }),
    );
    expect(
      screen.getByRole("menuitem", { name: "Recently detected" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning?state=open&sort=recently_detected",
    );
    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning",
    );
  });

  it("selects visible rows without mutation controls in the list phase", () => {
    render(
      <RepositoryCodeScanningAlertsPage
        codeScanningResult={{ ok: true, codeScanning: codeScanningView() }}
        repository={repositoryOverview()}
      />,
    );

    const summary = screen.getByRole("region", {
      name: "Code scanning alert summary",
    });
    expect(within(summary).getByText("Selected")).toBeVisible();
    expect(within(summary).getByText("0")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Select all visible" }));
    expect(screen.getByRole("button", { name: "Clear visible" })).toBeVisible();
    expect(screen.getByText("2 selected")).toBeVisible();
  });

  it("renders disabled and unavailable states with concrete recovery links", () => {
    const disabled = codeScanningView({
      availability: {
        enabled: false,
        indexed: false,
        message: "Code scanning is not enabled for this repository.",
        disabledReason: "Enable code scanning to analyze SARIF uploads.",
        settingsHref: "/namuh-eng/opengithub/settings/security",
      },
      alerts: [],
      counts: { open: 0, closed: 0, total: 0, visible: 0 },
      tools: [],
      branches: [],
    });

    const { rerender } = render(
      <RepositoryCodeScanningAlertsPage
        codeScanningResult={{ ok: true, codeScanning: disabled }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Code scanning is not enabled." }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Enable code scanning" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");
    expect(screen.queryByText("2 matching alerts")).not.toBeInTheDocument();

    rerender(
      <RepositoryCodeScanningAlertsPage
        codeScanningResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Code scanning alerts are unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", {
        name: "Code scanning alerts unavailable",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to security overview" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security");
  });
});
