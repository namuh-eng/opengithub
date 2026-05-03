import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryPagesSettingsPage } from "@/components/RepositoryPagesSettingsPage";
import type {
  PagesDeploymentSummary,
  RepositoryOverview,
  RepositoryPagesSettings,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: null,
    owner_organization_id: "org-1",
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
      releasesCount: 0,
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

function deployment(
  overrides: Partial<PagesDeploymentSummary> = {},
): PagesDeploymentSummary {
  return {
    id: "deployment-1",
    source: {
      branch: "gh-pages",
      folder: "/",
      kind: "branch",
      workflowArtifactName: null,
      workflowId: null,
    },
    status: "deployed",
    conclusion: "success",
    defaultUrl: "https://namuh-eng.opengithub.namuh.co/opengithub",
    customDomainUrl: "https://docs.namuh.co",
    workflowRunId: null,
    workflowArtifactId: null,
    failureReason: null,
    queuedAt: "2026-05-03T00:00:00Z",
    completedAt: "2026-05-03T00:02:00Z",
    createdAt: "2026-05-03T00:00:00Z",
    ...overrides,
  };
}

function settings(
  overrides: Partial<RepositoryPagesSettings> = {},
): RepositoryPagesSettings {
  return {
    repositoryId: "repo-1",
    ownerLogin: "namuh-eng",
    name: "opengithub",
    visibility: "private",
    viewerPermission: "admin",
    canEdit: true,
    site: {
      id: "site-1",
      source: {
        branch: "gh-pages",
        folder: "/",
        kind: "branch",
        workflowArtifactName: null,
        workflowId: null,
      },
      defaultSiteUrl: "https://namuh-eng.opengithub.namuh.co/opengithub",
      customDomain: "docs.namuh.co",
      domain: {
        challenge: {
          name: "_opengithub-pages.docs.namuh.co",
          recordType: "TXT",
          value: "og-pages-secret-token",
        },
        lastCheckedAt: "2026-05-03T00:05:00Z",
        status: "verified",
        warning: null,
      },
      httpsEnforced: true,
      certificateStatus: "issued",
      provisioningStatus: "deployed",
      cloudfrontAlias: "hidden-cloudfront-alias.example.net",
      latestDeploymentId: "deployment-1",
      unpublishedAt: null,
      updatedAt: "2026-05-03T00:06:00Z",
    },
    availableRefs: [
      {
        name: "main",
        targetOid: "abc123",
        updatedAt: "2026-05-03T00:00:00Z",
      },
      {
        name: "gh-pages",
        targetOid: "def456",
        updatedAt: "2026-05-03T00:00:00Z",
      },
    ],
    folderOptions: [
      { exists: true, label: "/(root)", value: "/" },
      { exists: true, label: "/docs", value: "/docs" },
    ],
    workflowSuggestions: [
      {
        artifactHint: "public",
        name: "Static HTML",
        path: ".github/workflows/pages.yml",
        workflowId: "workflow-1",
      },
    ],
    deployments: [deployment()],
    auditEvents: [],
    ...overrides,
  };
}

describe("repository Pages settings page", () => {
  it("renders branch publishing, live status, domain verification, and deployments", () => {
    const { container } = render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    expect(screen.getByText("Live")).toBeVisible();
    expect(screen.getByText("namuh-eng/opengithub Pages")).toBeVisible();
    expect(screen.getByRole("link", { name: "Visit site" })).toHaveAttribute(
      "href",
      "https://docs.namuh.co",
    );
    expect(screen.getAllByText("gh-pages · /(root)").length).toBeGreaterThan(0);
    expect(screen.getByDisplayValue("docs.namuh.co")).toBeVisible();
    expect(screen.getByText("_opengithub-pages.docs.namuh.co")).toBeVisible();
    expect(screen.getByText("og-pages-secret-token")).toBeVisible();
    expect(screen.getByText("Static HTML")).toBeVisible();
    expect(screen.getByText("public")).toBeVisible();
    expect(screen.getByRole("link", { name: /gh-pages ·/i })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/pages#deployment-deployment-1",
    );
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(0);
    expect(container.innerHTML).toContain("var(--ink-2)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("renders disabled Pages without inert anchors", () => {
    const disabled = settings({
      deployments: [],
      site: {
        ...settings().site,
        customDomain: null,
        domain: {
          challenge: null,
          lastCheckedAt: null,
          status: "not_configured",
          warning: null,
        },
        httpsEnforced: false,
        source: {
          branch: null,
          folder: null,
          kind: "none",
          workflowArtifactName: null,
          workflowId: null,
        },
        unpublishedAt: "2026-05-03T00:00:00Z",
      },
    });
    const { container } = render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: disabled }}
      />,
    );

    expect(screen.getByText("Not published")).toBeVisible();
    expect(screen.getByText("No deployments yet")).toBeVisible();
    expect(
      screen.getByText(
        "A DNS challenge appears after a custom domain is saved.",
      ),
    ).toBeVisible();
    for (const link of container.querySelectorAll("a")) {
      expect(link.getAttribute("href")).not.toBe("#");
    }
    for (const button of screen.getAllByRole("button")) {
      expect(button).toBeDisabled();
    }
  });

  it("renders Actions source suggestions and workflow deployment links", () => {
    const actionsDeployment = deployment({
      id: "deployment-actions",
      source: {
        branch: null,
        folder: null,
        kind: "actions",
        workflowArtifactName: "public",
        workflowId: "workflow-1",
      },
      status: "queued",
      workflowRunId: "run-1",
    });
    render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: true,
          settings: settings({
            deployments: [actionsDeployment],
            site: {
              ...settings().site,
              source: actionsDeployment.source,
            },
          }),
        }}
      />,
    );

    expect(
      screen.getAllByText("GitHub Actions · public").length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: /GitHub Actions · public/i }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/actions/runs/run-1");
  });

  it("renders forbidden and unavailable states without leaking private metadata", () => {
    const { container, rerender } = render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: false,
          status: 403,
          code: "permission_denied",
          message: "Permission denied",
        }}
      />,
    );

    expect(screen.getByText("Pages settings are restricted")).toBeVisible();
    expect(container.innerHTML).not.toContain("og-pages-secret-token");
    expect(container.innerHTML).not.toContain("hidden-cloudfront-alias");
    expect(
      screen.getByRole("link", { name: "Repository Code" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub");

    rerender(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: false,
          status: 503,
          code: "api_unavailable",
          message: "Repository Pages settings are unavailable right now.",
        }}
      />,
    );

    expect(screen.getByText("Pages settings could not load")).toBeVisible();
    expect(
      screen.getByText("Repository Pages settings are unavailable right now."),
    ).toBeVisible();
  });
});
