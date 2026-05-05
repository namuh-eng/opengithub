import { fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDependencyGraphPage } from "@/components/RepositoryDependencyGraphPage";
import type { RepositoryDependenciesView, RepositoryOverview } from "@/lib/api";

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

function dependenciesView(
  overrides: Partial<RepositoryDependenciesView> = {},
): RepositoryDependenciesView {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "private",
      viewerPermission: "read",
      href: "/namuh-eng/opengithub",
      treeHref: "/namuh-eng/opengithub/tree/main",
    },
    filters: {
      query: null,
      ecosystem: null,
      relationship: null,
    },
    summary: {
      total: 2,
      directCount: 1,
      transitiveCount: 1,
      ecosystemCounts: [
        { ecosystem: "npm", count: 1 },
        { ecosystem: "cargo", count: 1 },
      ],
      manifestCount: 2,
      advisoryCount: 1,
    },
    manifests: [
      {
        id: "manifest-1",
        path: "package.json",
        ecosystem: "npm",
        lockfilePath: "package-lock.json",
        dependencyCount: 1,
        detectedAt: "2026-05-05T00:00:00Z",
        href: "/namuh-eng/opengithub/blob/main/package.json",
        lockfileHref: "/namuh-eng/opengithub/blob/main/package-lock.json",
      },
      {
        id: "manifest-2",
        path: "crates/api/Cargo.toml",
        ecosystem: "cargo",
        lockfilePath: null,
        dependencyCount: 1,
        detectedAt: "2026-05-05T00:00:00Z",
        href: "/namuh-eng/opengithub/blob/main/crates%2Fapi%2FCargo.toml",
        lockfileHref: null,
      },
    ],
    dependencies: [
      {
        id: "dep-1",
        package: {
          id: "pkg-1",
          ecosystem: "npm",
          name: "@testing-library/react",
          href: "/packages/npm/%40testing-library%2Freact",
        },
        version: "^16.0.0",
        relationship: "direct",
        license: "MIT",
        manifestPath: "package.json",
        manifestHref: "/namuh-eng/opengithub/blob/main/package.json",
        lockfilePath: "package-lock.json",
        lockfileHref: "/namuh-eng/opengithub/blob/main/package-lock.json",
        detectedAt: "2026-05-05T00:00:00Z",
        advisories: [
          {
            identifier: "GHSA-demo",
            severity: "high",
            title: "Demo advisory",
            href: "/advisories/GHSA-demo",
          },
        ],
        detailsHref:
          "/namuh-eng/opengithub/security/dependabot/1?package=%40testing-library%2Freact",
        advisoryHref:
          "/namuh-eng/opengithub/security/dependabot?package=%40testing-library%2Freact",
      },
      {
        id: "dep-2",
        package: {
          id: "pkg-2",
          ecosystem: "cargo",
          name: "sqlx",
          href: "/packages/cargo/sqlx",
        },
        version: "0.8.0",
        relationship: "transitive",
        license: null,
        manifestPath: "crates/api/Cargo.toml",
        manifestHref:
          "/namuh-eng/opengithub/blob/main/crates%2Fapi%2FCargo.toml",
        lockfilePath: null,
        lockfileHref: null,
        detectedAt: "2026-05-05T00:00:00Z",
        advisories: [],
        detailsHref: "/namuh-eng/opengithub/security/dependabot/2?package=sqlx",
        advisoryHref: null,
      },
    ],
    availability: {
      enabled: true,
      indexed: true,
      supportedEcosystems: ["npm", "cargo", "pip"],
      message:
        "Dependency graph is indexed from supported manifest and lock files.",
      unavailableReason: null,
    },
    export: {
      supported: true,
      href: "/api/repos/namuh-eng/opengithub/network/dependencies/sbom",
      latestStatus: null,
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

describe("RepositoryDependencyGraphPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders dependency filters, summaries, rows, manifests, and concrete links", () => {
    const { container } = render(
      <RepositoryDependencyGraphPage
        dependenciesResult={{ ok: true, dependencies: dependenciesView() }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Dependencies" })).toBeVisible();
    expect(
      screen.getByRole("link", {
        name: "Dependency graph Dependencies and dependents",
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Dependencies" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: "Dependents" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependents",
    );
    expect(screen.getByRole("button", { name: "Export SBOM" })).toBeEnabled();
    expect(screen.getByText("No export yet")).toBeVisible();

    expect(screen.getByLabelText("Search")).toHaveValue("");
    fireEvent.change(screen.getByLabelText("Search"), {
      target: { value: "sqlx" },
    });
    expect(screen.getByRole("button", { name: "Apply" })).toBeVisible();
    fireEvent.click(
      screen.getByRole("button", { name: "Ecosystem: All ecosystems" }),
    );
    expect(screen.getByRole("menuitem", { name: /npm/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependencies?q=sqlx&ecosystem=npm",
    );
    expect(screen.getByRole("link", { name: "Direct" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependencies?q=sqlx&relationship=direct",
    );

    expect(screen.getByLabelText("Dependency summary metrics")).toBeVisible();
    expect(screen.getByText("2 dependencies")).toBeVisible();
    const list = screen.getByRole("list", {
      name: "Repository dependencies list",
    });
    expect(
      within(list).getByRole("link", { name: "@testing-library/react" }),
    ).toHaveAttribute("href", "/packages/npm/%40testing-library%2Freact");
    expect(within(list).getByText("^16.0.0")).toBeVisible();
    expect(within(list).getByText("direct")).toBeVisible();
    expect(
      within(list).getByRole("link", { name: "package.json" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/blob/main/package.json");
    expect(
      within(list).getByRole("link", {
        name: "@testing-library/react package details",
      }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/security/dependabot/1?package=%40testing-library%2Freact",
    );
    expect(within(list).getByRole("link", { name: /GHSA-demo/ })).toHaveClass(
      "chip err",
    );

    const manifests = screen.getByRole("list", {
      name: "Indexed dependency manifests",
    });
    expect(manifests).toBeVisible();
    expect(
      within(manifests).getByRole("link", { name: "package-lock.json" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/package-lock.json",
    );

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.querySelector(".chip.ok")).not.toBeNull();
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

  it("starts a real SBOM export and exposes the signed download", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          id: "export-1",
          status: "ready",
          format: "spdx-json",
          artifactSha256: "sha256-demo",
          artifactByteSize: 512,
          downloadHref:
            "/namuh-eng/opengithub/network/dependencies/sbom/export-1",
          pollHref:
            "/api/repos/namuh-eng/opengithub/network/dependencies/sbom/export-1",
          expiresAt: "2026-05-06T00:00:00Z",
          createdAt: "2026-05-05T00:00:00Z",
          completedAt: "2026-05-05T00:00:01Z",
        }),
        { status: 201, headers: { "content-type": "application/json" } },
      ),
    );

    render(
      <RepositoryDependencyGraphPage
        dependenciesResult={{ ok: true, dependencies: dependenciesView() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Export SBOM" }));

    expect(
      await screen.findByRole("link", { name: "Download SBOM" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/network/dependencies/sbom/export-1",
    );
    expect(screen.getByText("Latest SBOM ready")).toBeVisible();
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/network/dependencies/sbom",
      { method: "POST" },
    );
  });

  it("renders empty and unavailable dependency graph states", () => {
    const empty = dependenciesView({
      dependencies: [],
      manifests: [],
      summary: {
        ...dependenciesView().summary,
        total: 0,
        directCount: 0,
        transitiveCount: 0,
        ecosystemCounts: [],
        manifestCount: 0,
        advisoryCount: 0,
      },
      availability: {
        ...dependenciesView().availability,
        indexed: false,
        message: "No supported dependency manifest was found.",
      },
      export: { ...dependenciesView().export, supported: false },
    });

    const { rerender } = render(
      <RepositoryDependencyGraphPage
        dependenciesResult={{ ok: true, dependencies: empty }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", {
        name: "No matching dependencies were found.",
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Browse source tree" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/tree/main");

    rerender(
      <RepositoryDependencyGraphPage
        dependenciesResult={{
          ok: false,
          status: 422,
          code: "dependency_graph_unavailable",
          message: "Dependency graph is disabled for this repository.",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Dependencies unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByText("Dependency graph is disabled for this repository."),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Retry dependencies" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/network/dependencies");
  });
});
