import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositorySecretScanningAlertsPage } from "@/components/RepositorySecretScanningAlertsPage";
import type {
  RepositoryOverview,
  RepositorySecretScanningAlertsView,
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

function secretScanningView(
  overrides: Partial<RepositorySecretScanningAlertsView> = {},
): RepositorySecretScanningAlertsView {
  const base: RepositorySecretScanningAlertsView = {
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
      pushProtectionEnabled: true,
      message:
        "Secret scanning alerts are indexed from committed content and protected pushes.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    filters: {
      state: "open",
      query: null,
      provider: null,
      secretType: null,
      validity: null,
      resolution: null,
      bypassed: null,
      team: null,
      topic: null,
      sort: "recently_detected",
    },
    counts: {
      open: 2,
      resolved: 1,
      provider: 1,
      generic: 1,
      bypassed: 1,
      total: 3,
      visible: 2,
    },
    alerts: [
      {
        id: "secret-alert-1",
        number: 1,
        state: "open",
        resolution: null,
        pattern: {
          id: "pattern-1",
          slug: "github-pat",
          provider: "GitHub",
          secretType: "github_personal_access_token",
          displayName: "GitHub personal access token",
          resultKind: "provider",
          pushProtectionEnabled: true,
        },
        redactedSecret: "ghp_************",
        redactedContext: "token=ghp_************",
        fingerprint: "fp_secret_001",
        validity: {
          status: "active",
          provider: "GitHub",
          checkedAt: "2026-05-05T00:00:00Z",
          message: "Provider reported the credential is active.",
        },
        primaryLocation: {
          path: ".env",
          pathHref: "/namuh-eng/opengithub/blob/main/.env#L12",
          rawHref: "/namuh-eng/opengithub/raw/main/.env",
          commitHref: "/namuh-eng/opengithub/commit/abc123",
          refName: "refs/heads/main",
          branchName: "main",
          startLine: 12,
          endLine: null,
          redactedSnippet: "TOKEN=ghp_************",
        },
        assignees: [
          {
            id: "user-1",
            login: "jaeyun",
            avatarUrl: null,
            href: "/jaeyun",
          },
        ],
        bypassed: true,
        href: "/namuh-eng/opengithub/security/secret-scanning/1",
        detectedAt: "2026-05-05T00:00:00Z",
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "secret-alert-2",
        number: 2,
        state: "open",
        resolution: null,
        pattern: {
          id: "pattern-2",
          slug: "generic-api-key",
          provider: "Generic",
          secretType: "generic_api_key",
          displayName: "Generic API key",
          resultKind: "generic",
          pushProtectionEnabled: false,
        },
        redactedSecret: "sk_************",
        redactedContext: null,
        fingerprint: "fp_secret_002",
        validity: {
          status: "unknown",
          provider: "Generic",
          checkedAt: null,
          message: "No provider validity check is configured.",
        },
        primaryLocation: {
          path: "docs/example.env",
          pathHref: "/namuh-eng/opengithub/blob/main/docs%2Fexample.env#L3",
          rawHref: "/namuh-eng/opengithub/raw/main/docs%2Fexample.env",
          commitHref: null,
          refName: "refs/heads/main",
          branchName: "main",
          startLine: 3,
          endLine: null,
          redactedSnippet: "API_KEY=sk_************",
        },
        assignees: [],
        bypassed: false,
        href: "/namuh-eng/opengithub/security/secret-scanning/2",
        detectedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    providers: [
      { provider: "GitHub", openCount: 1, selected: false },
      { provider: "Generic", openCount: 1, selected: false },
    ],
    secretTypes: [
      {
        secretType: "github_personal_access_token",
        displayName: "GitHub personal access token",
        provider: "GitHub",
        resultKind: "provider",
        openCount: 1,
        selected: false,
      },
      {
        secretType: "generic_api_key",
        displayName: "Generic API key",
        provider: "Generic",
        resultKind: "generic",
        openCount: 1,
        selected: false,
      },
    ],
    pushProtection: {
      enabled: true,
      protectedPatternCount: 1,
      bypassCount: 1,
      pendingReviewCount: 1,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    links: {
      listHref: "/namuh-eng/opengithub/security/secret-scanning",
      providerHref:
        "/namuh-eng/opengithub/security/secret-scanning?state=open&result_kind=provider",
      genericHref:
        "/namuh-eng/opengithub/security/secret-scanning?state=open&result_kind=generic",
      openHref: "/namuh-eng/opengithub/security/secret-scanning?state=open",
      resolvedHref:
        "/namuh-eng/opengithub/security/secret-scanning?state=resolved",
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    freshness: {
      computedAt: "2026-05-05T00:00:00Z",
      cadence: "per push",
    },
  };

  return { ...base, ...overrides };
}

describe("RepositorySecretScanningAlertsPage", () => {
  it("renders the Editorial Secret scanning list with active navigation, tabs, rows, and concrete links", () => {
    const { container } = render(
      <RepositorySecretScanningAlertsPage
        repository={repositoryOverview()}
        secretScanningResult={{
          ok: true,
          secretScanning: secretScanningView(),
        }}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Secret scanning Credential exposure findings",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { name: "Secret scanning alerts" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: /Open 2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&sort=recently_detected",
    );
    expect(screen.getByRole("link", { name: /Resolved 1/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=resolved&sort=recently_detected",
    );
    expect(
      screen.getByRole("link", { name: /Provider and default 1/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&result_kind=provider",
    );

    const rows = screen.getByRole("list", { name: "Secret scanning alerts" });
    expect(
      within(rows).getByText("GitHub personal access token"),
    ).toBeVisible();
    expect(within(rows).getByText("ghp_************")).toBeVisible();
    expect(within(rows).getByRole("link", { name: ".env:12" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/.env#L12",
    );
    expect(
      within(rows).getAllByRole("link", { name: "Raw file" })[0],
    ).toHaveAttribute("href", "/namuh-eng/opengithub/raw/main/.env");
    expect(within(rows).getByRole("link", { name: "Commit" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commit/abc123",
    );
    expect(
      within(rows).getAllByRole("link", { name: "View alert" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning/1",
    );
    expect(
      screen.getByRole("link", { name: "Push protection" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");
    expect(
      screen.getByRole("heading", {
        name: "Protected pushes and bypass outcomes",
      }),
    ).toBeVisible();
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toContain("super-secret-value");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("builds URL-backed filter and sort destinations", () => {
    render(
      <RepositorySecretScanningAlertsPage
        repository={repositoryOverview()}
        secretScanningResult={{
          ok: true,
          secretScanning: secretScanningView(),
        }}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Provider: All providers" }),
    );
    expect(screen.getByRole("menuitem", { name: /GitHub/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&provider=GitHub&sort=recently_detected",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Secret type: All secret types" }),
    );
    expect(
      screen.getByRole("menuitem", {
        name: /GitHub personal access token · GitHub/,
      }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&secret_type=github_personal_access_token&sort=recently_detected",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Validity: All validity states" }),
    );
    expect(screen.getByRole("menuitem", { name: "Active" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&validity=active&sort=recently_detected",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Sort: Recently detected" }),
    );
    expect(screen.getByRole("menuitem", { name: "Provider" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning?state=open&sort=provider",
    );
    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/secret-scanning",
    );
  });

  it("selects visible rows without mutation controls in the list phase", () => {
    render(
      <RepositorySecretScanningAlertsPage
        repository={repositoryOverview()}
        secretScanningResult={{
          ok: true,
          secretScanning: secretScanningView(),
        }}
      />,
    );

    const summary = screen.getByRole("region", {
      name: "Secret scanning alert summary",
    });
    expect(within(summary).getByText("Selected")).toBeVisible();
    expect(within(summary).getByText("0")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Select all visible" }));
    expect(screen.getByRole("button", { name: "Clear visible" })).toBeVisible();
    expect(screen.getByText("2 selected")).toBeVisible();
  });

  it("renders disabled and unavailable states with concrete recovery links", () => {
    const disabled = secretScanningView({
      availability: {
        enabled: false,
        indexed: false,
        pushProtectionEnabled: false,
        message: "Secret scanning is not enabled for this repository.",
        disabledReason:
          "Enable secret scanning to detect accidental sensitive information in committed code.",
        settingsHref: "/namuh-eng/opengithub/settings/security",
      },
      alerts: [],
      counts: {
        open: 0,
        resolved: 0,
        provider: 0,
        generic: 0,
        bypassed: 0,
        total: 0,
        visible: 0,
      },
      providers: [],
      secretTypes: [],
    });

    const { rerender } = render(
      <RepositorySecretScanningAlertsPage
        repository={repositoryOverview()}
        secretScanningResult={{ ok: true, secretScanning: disabled }}
      />,
    );
    expect(
      screen.getByRole("heading", { level: 1, name: "Secret scanning alerts" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Enable secret scanning" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/security");
    expect(screen.queryByText("2 matching alerts")).not.toBeInTheDocument();

    rerender(
      <RepositorySecretScanningAlertsPage
        repository={repositoryOverview()}
        secretScanningResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Secret scanning alerts are unavailable right now.",
        }}
      />,
    );
    expect(
      screen.getByRole("heading", {
        name: "Secret scanning alerts unavailable",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to security overview" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security");
  });
});
