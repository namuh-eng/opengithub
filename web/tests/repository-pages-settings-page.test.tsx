import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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
    artifactStorageKey: "pages/repo/deployment-1",
    artifactManifest: {
      artifactCount: 2,
      storageMode: "local_metadata",
      storagePrefix: "pages/repo/deployment-1",
      totalBytes: 512,
    },
    buildLogExcerpt:
      "Published 2 Pages artifact(s) to pages/repo/deployment-1 using local_metadata storage metadata.",
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
    policyLock: null,
    ...overrides,
  };
}

describe("repository Pages settings page", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

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
    expect(screen.getAllByText("Static HTML").length).toBeGreaterThan(0);
    expect(screen.getByText("public")).toBeVisible();
    expect(screen.getByRole("link", { name: /gh-pages ·/i })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/pages#deployment-deployment-1",
    );
    expect(screen.getByText(/Published 2 Pages artifact/)).toBeVisible();
    expect(
      screen.getAllByText(/pages\/repo\/deployment-1/).length,
    ).toBeGreaterThan(0);
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(0);
    expect(container.innerHTML).toContain("var(--ink-2)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("disables publishing source controls when organization policy locks Pages", () => {
    render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: true,
          settings: settings({
            policyLock: {
              field: "pagesPrivatePublishing",
              reason:
                "Organization policy prevents Pages publishing for private repositories.",
              settingsHref:
                "/organizations/namuh-eng/settings/member_privileges",
            },
          }),
        }}
      />,
    );

    expect(screen.getByText("Policy locked")).toBeVisible();
    expect(
      screen.getByRole("link", { name: /prevents Pages publishing/i }),
    ).toHaveAttribute(
      "href",
      "/organizations/namuh-eng/settings/member_privileges",
    );
    expect(screen.getByLabelText("Source")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save source" })).toBeDisabled();
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
    expect(screen.getByRole("button", { name: "Save source" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Save domain" })).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Remove domain" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Unpublish Pages" }),
    ).toBeDisabled();
  });

  it("saves branch source through the same-origin action and refreshes only from confirmed state", async () => {
    const nextSettings = settings({
      site: {
        ...settings().site,
        source: {
          branch: "main",
          folder: "/docs",
          kind: "branch",
          workflowArtifactName: null,
          workflowId: null,
        },
      },
      deployments: [
        deployment({
          id: "deployment-queued",
          source: {
            branch: "main",
            folder: "/docs",
            kind: "branch",
            workflowArtifactName: null,
            workflowId: null,
          },
          status: "queued",
        }),
      ],
    });
    const fetchMock = vi.fn().mockResolvedValue({
      json: () => Promise.resolve(nextSettings),
      ok: true,
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Branch"), {
      target: { value: "main" },
    });
    fireEvent.change(screen.getByLabelText("Folder"), {
      target: { value: "/docs" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save source" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/settings/pages/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "update-source",
            branch: "main",
            folder: "/docs",
            kind: "branch",
            workflowArtifactName: null,
            workflowId: null,
          }),
          method: "POST",
        }),
      ),
    );
    expect(
      await screen.findByText(
        "Branch source saved and a Pages deployment was queued.",
      ),
    ).toBeVisible();
    expect(screen.getAllByText("main · /docs").length).toBeGreaterThan(0);
    expect(screen.getAllByText("queued").length).toBeGreaterThan(0);
  });

  it("keeps invalid source changes local and surfaces API errors without replacing confirmed state", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      json: () =>
        Promise.resolve({
          error: {
            code: "validation_failed",
            message: "selected branch does not contain a /docs folder",
          },
          status: 422,
        }),
      ok: false,
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Folder"), {
      target: { value: "/docs" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save source" }));

    expect(
      await screen.findByText(
        "selected branch does not contain a /docs folder",
      ),
    ).toBeVisible();
    expect(screen.getAllByText("gh-pages · /(root)").length).toBeGreaterThan(0);
    expect(screen.queryByText("gh-pages · /docs")).not.toBeInTheDocument();
  });

  it("supports domain save, DNS recheck, HTTPS toggle, and confirmed unpublish", async () => {
    const pending = settings({
      site: {
        ...settings().site,
        customDomain: "docs.example.com",
        domain: {
          challenge: {
            name: "_opengithub-pages.docs.example.com",
            recordType: "TXT",
            value: "og-pages-next-token",
          },
          lastCheckedAt: null,
          status: "pending",
          warning: "DNS challenge has not propagated yet.",
        },
        httpsEnforced: false,
        certificateStatus: "pending",
      },
    });
    const verified = settings({
      site: {
        ...pending.site,
        domain: {
          ...pending.site.domain,
          lastCheckedAt: "2026-05-03T00:10:00Z",
          status: "verified",
          warning: null,
        },
        certificateStatus: "issued",
      },
    });
    const https = settings({
      site: {
        ...verified.site,
        httpsEnforced: true,
      },
    });
    const unpublished = settings({
      site: {
        ...https.site,
        source: {
          branch: null,
          folder: null,
          kind: "none",
          workflowArtifactName: null,
          workflowId: null,
        },
        provisioningStatus: "unpublished",
        unpublishedAt: "2026-05-03T00:12:00Z",
      },
    });
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({ json: () => Promise.resolve(pending), ok: true })
      .mockResolvedValueOnce({
        json: () => Promise.resolve(verified),
        ok: true,
      })
      .mockResolvedValueOnce({ json: () => Promise.resolve(https), ok: true })
      .mockResolvedValueOnce({
        json: () => Promise.resolve(unpublished),
        ok: true,
      });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPagesSettingsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Domain"), {
      target: { value: "Docs.Example.COM." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save domain" }));
    expect(
      await screen.findByText(
        "Custom domain saved. Add the DNS challenge before verification.",
      ),
    ).toBeVisible();
    expect(screen.getByDisplayValue("Docs.Example.COM.")).toBeVisible();
    expect(screen.getByText("og-pages-next-token")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Recheck DNS" }));
    expect(
      await screen.findByText("DNS verification rechecked from the Pages API."),
    ).toBeVisible();
    expect(screen.getByText(/Certificate:\s*issued/)).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Enforce HTTPS" }));
    expect(await screen.findByText("HTTPS enforcement enabled.")).toBeVisible();
    expect(screen.getByText("HTTPS enforced")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Unpublish Pages" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm unpublish" }));
    expect(
      await screen.findByText(
        "Pages unpublished. Repository files were preserved.",
      ),
    ).toBeVisible();
    expect(screen.getByText("Not published")).toBeVisible();

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/namuh-eng/opengithub/settings/pages/actions",
      expect.objectContaining({
        body: JSON.stringify({
          action: "save-domain",
          domain: "Docs.Example.COM.",
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/namuh-eng/opengithub/settings/pages/actions",
      expect.objectContaining({
        body: JSON.stringify({ action: "recheck-dns" }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      "/namuh-eng/opengithub/settings/pages/actions",
      expect.objectContaining({
        body: JSON.stringify({ action: "update-https", enforced: true }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      4,
      "/namuh-eng/opengithub/settings/pages/actions",
      expect.objectContaining({
        body: JSON.stringify({ action: "unpublish-pages" }),
      }),
    );
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
