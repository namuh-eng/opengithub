import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositorySecurityPolicyPage } from "@/components/RepositorySecurityPolicyPage";
import type {
  RepositoryOverview,
  RepositorySecurityPolicyView,
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

function securityPolicy(
  overrides: Partial<RepositorySecurityPolicyView> = {},
): RepositorySecurityPolicyView {
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
      markdown:
        "# Security policy\n\nPlease email [security](mailto:security@example.com).",
      html: '<h1 id="security-policy"><a class="anchor" href="#security-policy" aria-label="Permalink: Security policy">#</a>Security policy</h1><p>Please email <a href="mailto:security@example.com">security</a>. See <a href="/namuh-eng/opengithub/blob/main/docs/security-guide.md">guide</a>.</p><h2 id="supported-versions"><a class="anchor" href="#supported-versions" aria-label="Permalink: Supported versions">#</a>Supported versions</h2>',
      outline: [
        {
          id: "security-policy",
          level: 1,
          text: "Security policy",
          href: "#security-policy",
        },
        {
          id: "supported-versions",
          level: 2,
          text: "Supported versions",
          href: "#supported-versions",
        },
      ],
      sourceHref: "/namuh-eng/opengithub/blob/main/SECURITY.md",
      rawHref: "/namuh-eng/opengithub/raw/main/SECURITY.md",
      historyHref: "/namuh-eng/opengithub/commits/main/SECURITY.md",
      editHref: null,
      latestCommit: {
        oid: "abcdef1234567890",
        shortOid: "abcdef1",
        message: "Publish security policy",
        committedAt: "2026-05-05T00:00:00Z",
        href: "/namuh-eng/opengithub/commit/abcdef1234567890",
      },
      updatedAt: "2026-05-05T00:00:00Z",
      emptyState: "",
    },
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

describe("RepositorySecurityPolicyPage", () => {
  it("renders the dedicated policy reader with anchors, mailto links, metadata, and file actions", () => {
    const { container } = render(
      <RepositorySecurityPolicyPage
        policyResult={{ ok: true, securityPolicy: securityPolicy() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Security policy Responsible disclosure guidance",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { level: 1, name: "Security policy" }),
    ).toBeVisible();
    expect(screen.getAllByText("SECURITY.md").length).toBeGreaterThan(1);
    expect(screen.getByText("main")).toBeVisible();
    expect(screen.getByRole("link", { name: "security" })).toHaveAttribute(
      "href",
      "mailto:security@example.com",
    );
    expect(screen.getByRole("link", { name: "guide" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/docs/security-guide.md",
    );

    expect(screen.getAllByRole("link", { name: "Source" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/SECURITY.md",
    );
    expect(screen.getAllByRole("link", { name: "Raw" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/raw/main/SECURITY.md",
    );
    expect(screen.getAllByRole("link", { name: "History" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/main/SECURITY.md",
    );

    const outline = screen.getByRole("navigation", {
      name: "Policy headings",
    });
    expect(
      within(outline).getByRole("link", { name: "Security policy" }),
    ).toHaveAttribute("href", "#security-policy");
    expect(
      within(outline).getByRole("link", { name: "Supported versions" }),
    ).toHaveAttribute("href", "#supported-versions");

    expect(
      screen.getByRole("link", { name: "Publish security policy" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/commit/abcdef1234567890");
    expect(screen.queryByRole("link", { name: "Edit policy" })).toBeNull();
    expect(container.innerHTML).not.toContain("<script");
    expect(container.querySelectorAll('a[href="#"]')).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAttribute("type", "button");
    }
  });

  it("shows maintainer setup when no policy exists", () => {
    render(
      <RepositorySecurityPolicyPage
        policyResult={{
          ok: true,
          securityPolicy: securityPolicy({
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
              markdown: null,
              html: null,
              outline: [],
              sourceHref: null,
              rawHref: null,
              historyHref: null,
              editHref: "/namuh-eng/opengithub/security/policy/edit",
              latestCommit: null,
              updatedAt: null,
              emptyState:
                "No SECURITY.md policy has been published. Maintainers can start setup.",
            },
          }),
        }}
        repository={repositoryOverview({ viewerPermission: "admin" })}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "No published policy" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Start setup" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/policy/edit",
    );
  });

  it("renders unavailable state without policy content", () => {
    render(
      <RepositorySecurityPolicyPage
        policyResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "repository was not found",
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Security policy unavailable" }),
    ).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(
      "repository was not found",
    );
    expect(screen.queryByText("Publish security policy")).toBeNull();
    expect(screen.getByText("404 · not_found")).toBeVisible();
  });
});
