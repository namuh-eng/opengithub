import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositorySecurityOverviewPage } from "@/components/RepositorySecurityOverviewPage";
import type {
  RepositoryOverview,
  RepositorySecurityOverviewView,
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
    viewerPermission: "read",
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
    ...overrides,
  };
}

function securityOverview(
  overrides: Partial<RepositorySecurityOverviewView> = {},
): RepositorySecurityOverviewView {
  return {
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
      permission: "read",
      canRead: true,
      canWrite: false,
      canEditPolicy: false,
      canViewPrivateAlertCounts: false,
    },
    policy: {
      exists: true,
      path: "SECURITY.md",
      ref: "main",
      blobOid: "blob-1",
      contentSha: "sha-1",
      html: '<h1 id="security-policy">Security policy</h1><p>Please email <a href="mailto:security@example.com">security</a>.</p>',
      sourceHref: "/namuh-eng/opengithub/blob/main/SECURITY.md",
      rawHref: "/namuh-eng/opengithub/raw/main/SECURITY.md",
      historyHref: "/namuh-eng/opengithub/commits/main/SECURITY.md",
      editHref: null,
      updatedAt: "2026-05-05T00:00:00Z",
      emptyState: "",
    },
    features: [
      {
        key: "dependabot",
        label: "Dependabot",
        status: "enabled",
        summary: "Dependency alerts are monitored.",
        alertCount: null,
        privateCount: null,
        href: "/namuh-eng/opengithub/security/dependabot",
        configHref: null,
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        key: "code_scanning",
        label: "Code scanning",
        status: "needs_setup",
        summary: "No code scanning workflow is configured.",
        alertCount: null,
        privateCount: null,
        href: "/namuh-eng/opengithub/security/code-scanning",
        configHref: "/namuh-eng/opengithub/security/code-scanning/setup",
        updatedAt: "2026-05-04T00:00:00Z",
      },
      {
        key: "secret_scanning",
        label: "Secret scanning",
        status: "disabled",
        summary: "Secret scanning is not enabled.",
        alertCount: null,
        privateCount: null,
        href: "/namuh-eng/opengithub/security/secret-scanning",
        configHref: null,
        updatedAt: null,
      },
    ],
    advisories: [
      {
        id: "adv-1",
        identifier: "GHSA-demo-2026",
        severity: "high",
        status: "published",
        title: "Demo package vulnerability",
        summary: "A vulnerable parser accepts unsafe input.",
        packageName: "@demo/parser",
        vulnerableRange: "< 2.0.0",
        href: "/namuh-eng/opengithub/security/advisories/GHSA-demo-2026",
        publishedAt: "2026-05-03T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    links: {
      overviewHref: "/namuh-eng/opengithub/security",
      policyHref: "/namuh-eng/opengithub/security/policy",
      advisoriesHref: "/namuh-eng/opengithub/security/advisories",
      dependabotHref: "/namuh-eng/opengithub/security/dependabot",
      codeScanningHref: "/namuh-eng/opengithub/security/code-scanning",
      secretScanningHref: "/namuh-eng/opengithub/security/secret-scanning",
    },
    ...overrides,
  };
}

describe("RepositorySecurityOverviewPage", () => {
  it("renders the Editorial security workspace with policy, features, advisories, and concrete links", () => {
    const { container } = render(
      <RepositorySecurityOverviewPage
        repository={repositoryOverview()}
        securityResult={{ ok: true, security: securityOverview() }}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Overview Policy, feature state, and advisories",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { name: "Security overview" }),
    ).toBeVisible();
    expect(screen.getByText("Private counts hidden")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Security policy" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/policy");

    expect(screen.getByRole("heading", { name: "SECURITY.md" })).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Security policy" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "security" })).toHaveAttribute(
      "href",
      "mailto:security@example.com",
    );
    expect(screen.getByRole("link", { name: "Source" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/SECURITY.md",
    );
    expect(screen.getByRole("link", { name: "History" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/main/SECURITY.md",
    );
    expect(
      screen.getByRole("link", { name: "Open raw policy" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/raw/main/SECURITY.md");

    const features = screen.getByLabelText("Security feature cards");
    expect(
      within(features).getByRole("link", { name: "Dependabot" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/dependabot");
    expect(within(features).getAllByText("Hidden")).toHaveLength(6);
    expect(
      within(features).getByRole("link", { name: "Configure" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/code-scanning/setup",
    );

    expect(
      screen.getByRole("link", { name: "Demo package vulnerability" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories/GHSA-demo-2026",
    );
    expect(screen.getByText("GHSA-demo-2026")).toBeVisible();

    expect(container.innerHTML).not.toContain("<script");
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAttribute("type", "button");
    }
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|@primer\/|Octicon/i,
    );
  });

  it("shows maintainer setup and alert counts when the viewer can edit the missing policy", () => {
    render(
      <RepositorySecurityOverviewPage
        repository={repositoryOverview({ viewerPermission: "admin" })}
        securityResult={{
          ok: true,
          security: securityOverview({
            viewer: {
              permission: "admin",
              canRead: true,
              canWrite: true,
              canEditPolicy: true,
              canViewPrivateAlertCounts: true,
            },
            policy: {
              exists: false,
              path: null,
              ref: null,
              blobOid: null,
              contentSha: null,
              html: null,
              sourceHref: null,
              rawHref: null,
              historyHref: null,
              editHref: "/namuh-eng/opengithub/security/policy/edit",
              updatedAt: null,
              emptyState:
                "No SECURITY.md policy has been published. Maintainers can start setup.",
            },
            features: [
              {
                key: "dependabot",
                label: "Dependabot",
                status: "enabled",
                summary: "Dependency alerts are monitored.",
                alertCount: 7,
                privateCount: 2,
                href: "/namuh-eng/opengithub/security/dependabot",
                configHref: null,
                updatedAt: "2026-05-05T00:00:00Z",
              },
            ],
            advisories: [],
          }),
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "No published policy" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Start setup" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/policy/edit",
    );
    expect(screen.getByText("Private counts visible")).toBeVisible();
    const features = screen.getByLabelText("Security feature cards");
    expect(within(features).getByText("7")).toBeVisible();
    expect(within(features).getByText("2")).toBeVisible();
    expect(
      screen.getByText(
        "No published advisories are available for this repository.",
      ),
    ).toBeVisible();
  });

  it("keeps reader missing-policy state read-only and sidebar links keyboard focusable", () => {
    const { container } = render(
      <RepositorySecurityOverviewPage
        repository={repositoryOverview()}
        securityResult={{
          ok: true,
          security: securityOverview({
            policy: {
              exists: false,
              path: null,
              ref: null,
              blobOid: null,
              contentSha: null,
              html: null,
              sourceHref: null,
              rawHref: null,
              historyHref: null,
              editHref: null,
              updatedAt: null,
              emptyState:
                "No SECURITY.md policy has been published. Maintainers can start setup.",
            },
            advisories: [
              {
                id: "adv-long",
                identifier: "GHSA-long-policy-word",
                severity: "critical",
                status: "published",
                title:
                  "Extremely long advisory title wraps without overlapping the sidebar or action controls",
                summary:
                  "A very long vulnerability summary remains plain text and wraps cleanly for mobile screens.",
                packageName: "a-package-with-an-extremely-long-name",
                vulnerableRange: "< 999.999.999",
                href: "/namuh-eng/opengithub/security/advisories/GHSA-long-policy-word",
                publishedAt: "2026-05-03T00:00:00Z",
                updatedAt: "2026-05-04T00:00:00Z",
              },
            ],
          }),
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "No published policy" }),
    ).toBeVisible();
    expect(screen.getByText("Reader view")).toBeVisible();
    expect(screen.queryByRole("link", { name: "Start setup" })).toBeNull();
    expect(screen.queryByRole("link", { name: "Edit policy" })).toBeNull();

    const securityNav = screen.getByRole("complementary", {
      name: "Security and quality navigation",
    });
    for (const link of within(securityNav).getAllByRole("link")) {
      expect(link).toHaveAttribute(
        "href",
        expect.stringContaining("/security"),
      );
      link.focus();
      expect(link).toHaveFocus();
    }
    expect(
      screen.getByRole("link", {
        name: "Extremely long advisory title wraps without overlapping the sidebar or action controls",
      }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories/GHSA-long-policy-word",
    );
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
  });

  it("renders unavailable state without leaking private counts", () => {
    render(
      <RepositorySecurityOverviewPage
        repository={repositoryOverview()}
        securityResult={{
          ok: false,
          status: 403,
          code: "forbidden",
          message: "Repository Security is unavailable right now.",
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Security overview unavailable" }),
    ).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Repository Security is unavailable right now.",
    );
    expect(screen.queryByText("7")).not.toBeInTheDocument();
    expect(screen.getByText("403 · forbidden")).toBeVisible();
  });
});
