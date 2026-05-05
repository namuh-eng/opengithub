import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositorySecurityAdvisoriesPage } from "@/components/RepositorySecurityAdvisoriesPage";
import type {
  RepositoryOverview,
  RepositorySecurityAdvisoriesView,
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

function advisoriesView(
  overrides: Partial<RepositorySecurityAdvisoriesView> = {},
): RepositorySecurityAdvisoriesView {
  const base: RepositorySecurityAdvisoriesView = {
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
    filters: {
      state: "published",
      severity: null,
      query: null,
      sort: "recently_updated",
      page: 1,
      pageSize: 10,
      total: 2,
      hasNextPage: true,
    },
    counts: {
      published: 2,
      draft: 1,
      withdrawn: 0,
    },
    advisories: [
      {
        id: "advisory-1",
        ghsaId: "GHSA-demo-2026",
        cveId: "CVE-2026-1234",
        severity: "high",
        state: "published",
        title: "Token scope bypass in repository import workflow",
        summary: "Repository imports could retain an overly broad token scope.",
        package: {
          ecosystem: "cargo",
          name: "opengithub-import",
          affectedVersions: "< 1.2.3",
          patchedVersions: ">= 1.2.3",
        },
        cvss: {
          vector: "CVSS:3.1/AV:N/AC:L/PR:L/UI:N/S:U/C:H/I:H/A:N",
          score: 8.1,
          metrics: {},
        },
        cwes: [{ id: "CWE-284", name: "Improper Access Control", href: null }],
        author: {
          id: "user-1",
          login: "jaeyun",
          avatarUrl: null,
          profileHref: "/jaeyun",
        },
        href: "/namuh-eng/opengithub/security/advisories/GHSA-demo-2026",
        publishedAt: "2026-05-05T00:00:00Z",
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "advisory-2",
        ghsaId: "GHSA-long-policy-word",
        cveId: null,
        severity: "medium",
        state: "published",
        title:
          "Extremely long advisory title wraps without breaking the Editorial layout column",
        summary:
          "Long package names and advisory summaries stay readable on narrow screens.",
        package: {
          ecosystem: "npm",
          name: "opengithub-advisory-authoring-with-a-very-long-name",
          affectedVersions: "< 4.0.0",
          patchedVersions: ">= 4.0.0",
        },
        cvss: null,
        cwes: [],
        author: null,
        href: "/namuh-eng/opengithub/security/advisories/GHSA-long-policy-word",
        publishedAt: "2026-05-04T00:00:00Z",
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    links: {
      listHref: "/namuh-eng/opengithub/security/advisories",
      newHref: "/namuh-eng/opengithub/security/advisories/new",
      publishedHref:
        "/namuh-eng/opengithub/security/advisories?state=published",
      draftHref: "/namuh-eng/opengithub/security/advisories?state=draft",
      withdrawnHref:
        "/namuh-eng/opengithub/security/advisories?state=withdrawn",
    },
  };

  return { ...base, ...overrides };
}

describe("RepositorySecurityAdvisoriesPage", () => {
  it("renders the Editorial advisories list with active navigation, tabs, rows, and concrete links", () => {
    const { container } = render(
      <RepositorySecurityAdvisoriesPage
        advisoriesResult={{ ok: true, advisories: advisoriesView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", {
        name: "Advisories Published security advisories",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("heading", { name: "Security advisories" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "New draft security advisory" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/advisories/new");
    expect(screen.getByRole("link", { name: /Published 2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories?state=published&sort=recently_updated&page=1&page_size=10",
    );
    expect(screen.getByRole("link", { name: /Draft 1/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories?state=draft&sort=recently_updated&page=1&page_size=10",
    );

    const rows = screen.getByRole("list", { name: "Security advisories" });
    expect(
      within(rows).getByText(
        "Token scope bypass in repository import workflow",
      ),
    ).toBeVisible();
    expect(within(rows).getByText("GHSA-demo-2026")).toBeVisible();
    expect(within(rows).getByText("CVE-2026-1234")).toBeVisible();
    expect(within(rows).getByText("cargo:opengithub-import")).toBeVisible();
    expect(
      within(rows).getAllByRole("link", { name: "View advisory" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories/GHSA-demo-2026",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories?state=published&sort=recently_updated&page=2&page_size=10",
    );
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("hides maintainer-only draft controls and counts from readers", () => {
    render(
      <RepositorySecurityAdvisoriesPage
        advisoriesResult={{
          ok: true,
          advisories: advisoriesView({
            viewer: {
              permission: "read",
              canRead: true,
              canWrite: false,
              canEditPolicy: false,
              canViewPrivateAlertCounts: false,
            },
            counts: { published: 1, draft: null, withdrawn: null },
            links: {
              listHref: "/namuh-eng/opengithub/security/advisories",
              newHref: null,
              publishedHref:
                "/namuh-eng/opengithub/security/advisories?state=published",
              draftHref: null,
              withdrawnHref: null,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.queryByRole("link", { name: "New draft security advisory" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: /Draft/ }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Draft")).not.toBeInTheDocument();
  });

  it("builds accessible filter menus and clear/apply URLs", () => {
    render(
      <RepositorySecurityAdvisoriesPage
        advisoriesResult={{
          ok: true,
          advisories: advisoriesView({
            filters: {
              state: "published",
              severity: "high",
              query: "token",
              sort: "severity",
              page: 2,
              pageSize: 10,
              total: 12,
              hasNextPage: false,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories",
    );

    fireEvent.click(screen.getByRole("button", { name: /Severity:/ }));
    expect(
      screen.getByRole("menu", { name: "Severity options" }),
    ).toBeVisible();
    expect(screen.getByRole("menuitem", { name: /Critical/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories?state=published&q=token&severity=critical&sort=severity&page=1&page_size=10",
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("menu", { name: "Severity options" }),
    ).not.toBeInTheDocument();
  });

  it("renders unavailable and empty states without leaking placeholder links", () => {
    const { container, rerender } = render(
      <RepositorySecurityAdvisoriesPage
        advisoriesResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository security advisories are unavailable right now.",
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Security advisories unavailable",
      }),
    ).toBeVisible();
    expect(container.innerHTML).not.toContain('href="#"');

    rerender(
      <RepositorySecurityAdvisoriesPage
        advisoriesResult={{
          ok: true,
          advisories: advisoriesView({
            advisories: [],
            filters: {
              state: "published",
              severity: null,
              query: "missing",
              sort: "recently_updated",
              page: 1,
              pageSize: 10,
              total: 0,
              hasNextPage: false,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(screen.getByText("No matching advisories.")).toBeVisible();
    expect(screen.getByRole("link", { name: "Start draft" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/advisories/new",
    );
  });
});
