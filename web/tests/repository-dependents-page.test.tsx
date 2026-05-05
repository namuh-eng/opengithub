import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryDependentsPage } from "@/components/RepositoryDependentsPage";
import type { RepositoryDependentsView, RepositoryOverview } from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "public",
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
  };
}

function dependentsView(
  overrides: Partial<RepositoryDependentsView> = {},
): RepositoryDependentsView {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "public",
      viewerPermission: "read",
      href: "/namuh-eng/opengithub",
      treeHref: "/namuh-eng/opengithub/tree/main",
    },
    filters: {
      package: null,
      owner: null,
    },
    summary: {
      repositoryCount: 1,
      packageCount: 2,
      hiddenPrivateCount: 1,
      approximate: true,
    },
    packages: [
      {
        package: {
          id: "pkg-1",
          ecosystem: "npm",
          name: "@namuh/flow",
          href: "/packages/npm/%40namuh%2Fflow",
        },
        dependentCount: 1,
        selected: false,
      },
      {
        package: {
          id: "pkg-2",
          ecosystem: "cargo",
          name: "sqlx",
          href: "/packages/cargo/sqlx",
        },
        dependentCount: 0,
        selected: false,
      },
    ],
    dependents: [
      {
        repositoryId: "dep-repo-1",
        ownerLogin: "public-consumer",
        ownerAvatarUrl: null,
        name: "workflow-tools",
        description: "Uses the opengithub package in production.",
        visibility: "public",
        package: {
          id: "pkg-1",
          ecosystem: "npm",
          name: "@namuh/flow",
          href: "/packages/npm/%40namuh%2Fflow",
        },
        manifestPath: "package.json",
        detectedAt: "2026-05-05T00:00:00Z",
        starsCount: 12,
        forksCount: 3,
        openIssuesCount: 2,
        openPullRequestsCount: 1,
        href: "/public-consumer/workflow-tools",
        ownerHref: "/public-consumer",
        packageHref: "/packages/npm/%40namuh%2Fflow",
      },
    ],
    availability: {
      enabled: true,
      indexed: true,
      supportedEcosystems: ["npm", "cargo", "pip"],
      message: "Dependents are estimated from public indexed dependency usage.",
      unavailableReason: null,
    },
    links: {
      dependenciesHref: "/namuh-eng/opengithub/network/dependencies",
      dependentsHref: "/namuh-eng/opengithub/network/dependents",
      exportSbomHref:
        "/api/repos/namuh-eng/opengithub/network/dependencies/sbom",
    },
    freshness: {
      computedAt: "2026-05-05T00:00:00Z",
      expiresAt: "2026-05-06T00:00:00Z",
      stale: false,
      cadence: "daily",
    },
    ...overrides,
  };
}

describe("RepositoryDependentsPage", () => {
  it("renders package and owner filters, warning disclosure, and public dependent rows", () => {
    const { container } = render(
      <RepositoryDependentsPage
        dependentsResult={{ ok: true, dependents: dependentsView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Dependents" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Dependents" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: "Dependencies" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependencies",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Package: All packages" }),
    );
    expect(
      screen.getByRole("menuitem", { name: /npm:@namuh\/flow/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependents?package=npm%3A%40namuh%2Fflow",
    );
    fireEvent.change(screen.getByLabelText("Owner"), {
      target: { value: "public-consumer" },
    });
    expect(screen.getByRole("button", { name: "Apply owner" })).toBeVisible();
    expect(screen.getByText("Counts are approximate")).toBeVisible();
    fireEvent.click(screen.getByText("Counts are approximate"));
    expect(screen.getByText(/Private consumers are counted/)).toBeVisible();
    expect(container.querySelector(".chip.ok")).not.toBeNull();
    expect(container.querySelector(".chip.warn")).not.toBeNull();

    expect(screen.getByLabelText("Dependents summary metrics")).toBeVisible();
    const list = screen.getByRole("list", {
      name: "Repository dependents list",
    });
    expect(
      within(list).getByRole("link", {
        name: "public-consumer/workflow-tools",
      }),
    ).toHaveAttribute("href", "/public-consumer/workflow-tools");
    expect(within(list).getByRole("link", { name: "Owner" })).toHaveAttribute(
      "href",
      "/public-consumer",
    );
    expect(
      within(list).getByRole("link", { name: "npm:@namuh/flow" }),
    ).toHaveAttribute("href", "/packages/npm/%40namuh%2Fflow");
    expect(within(list).getByText("package.json")).toBeVisible();
    expect(within(list).getByText("Stars")).toBeVisible();

    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|#f1aeb5|#fff1f3|@primer\/|Octicon/i,
    );
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName();
    }
    const focusableLabels = Array.from(
      container.querySelectorAll<HTMLElement>("a[href], button, input"),
    ).map(
      (element) =>
        element.getAttribute("aria-label") || element.textContent?.trim(),
    );
    expect(focusableLabels).toContain("Dependencies");
    expect(focusableLabels).toContain("Dependents");
    expect(focusableLabels).toContain("Package: All packages");
    expect(screen.getByLabelText("Owner")).toBeVisible();
    expect(focusableLabels).toContain("Repository");
  });

  it("renders empty and unavailable dependents states without private repository names", () => {
    const { rerender } = render(
      <RepositoryDependentsPage
        dependentsResult={{
          ok: true,
          dependents: dependentsView({
            dependents: [],
            filters: { package: "npm:@namuh/flow", owner: "private-consumer" },
            summary: {
              repositoryCount: 0,
              packageCount: 1,
              hiddenPrivateCount: 1,
              approximate: true,
            },
          }),
        }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "No public dependents matched these filters.",
      }),
    ).toBeVisible();
    expect(screen.queryByText("private-consumer/private-repo")).toBeNull();
    expect(
      screen.getAllByRole("link", { name: "Clear filters" })[0],
    ).toHaveAttribute("href", "/namuh-eng/opengithub/network/dependents");

    rerender(
      <RepositoryDependentsPage
        dependentsResult={{
          ok: false,
          status: 422,
          code: "dependency_graph_unavailable",
          message: "Dependents are shown only for public source repositories.",
        }}
        repository={{ ...repositoryOverview(), visibility: "private" }}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Dependents unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText(
        "Dependents are shown only for public source repositories.",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Retry dependents" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/network/dependents");
  });
});
