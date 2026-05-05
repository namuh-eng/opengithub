import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDependabotAlertDetailPage } from "@/components/RepositoryDependabotAlertDetailPage";
import type {
  RepositoryDependabotAlertDetail,
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

function detail(
  overrides: Partial<RepositoryDependabotAlertDetail> = {},
): RepositoryDependabotAlertDetail {
  const base: RepositoryDependabotAlertDetail = {
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
      message: "Dependabot alerts are monitored.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    alert: {
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
      assignees: [],
      href: "/namuh-eng/opengithub/security/dependabot/1",
      detectedAt: "2026-05-05T00:00:00Z",
      updatedAt: "2026-05-05T00:00:00Z",
    },
    advisory: {
      identifier: "GHSA-demo-0001",
      severity: "high",
      title: "Demo parser accepts unsafe input",
      href: "/advisories/GHSA-demo-0001",
      vulnerableRange: "< 2.0.0",
      publishedAt: "2026-05-04T00:00:00Z",
    },
    dependency: {
      package: {
        id: "pkg-1",
        ecosystem: "npm",
        name: "@testing-library/react",
        href: "/packages/npm/%40testing-library%2Freact",
      },
      manifestPath: "package.json",
      manifestHref: "/namuh-eng/opengithub/blob/main/package.json",
      lockfilePath: "package-lock.json",
      lockfileHref: "/namuh-eng/opengithub/blob/main/package-lock.json",
      currentVersion: "1.0.0",
      relationship: "direct",
    },
    timeline: [
      {
        id: "event-1",
        eventType: "created",
        message: "Dependabot opened this alert from the dependency graph.",
        actor: null,
        createdAt: "2026-05-05T00:00:00Z",
      },
    ],
    assigneeOptions: [
      {
        id: "user-1",
        kind: "user",
        login: "jaeyun",
        avatarUrl: null,
        selected: false,
      },
    ],
    securityUpdate: {
      supported: true,
      status: "available",
      href: "/api/repos/namuh-eng/opengithub/security/dependabot/1/security-update",
      pullRequestHref: null,
      message:
        "A security update pull request can be prepared for this manifest.",
    },
    links: {
      listHref: "/namuh-eng/opengithub/security/dependabot",
      openHref: "/namuh-eng/opengithub/security/dependabot?state=open",
      closedHref: "/namuh-eng/opengithub/security/dependabot?state=closed",
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
  };
  return { ...base, ...overrides };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RepositoryDependabotAlertDetailPage", () => {
  it("renders detail data, timeline, and Editorial triage controls", () => {
    const { container } = render(
      <RepositoryDependabotAlertDetailPage
        detailResult={{ ok: true, dependabotAlert: detail() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Demo parser accepts unsafe input" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "package.json" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/package.json",
    );
    expect(screen.getByRole("button", { name: "Dismiss alert" })).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Save assignments" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Create security update PR" }),
    ).toBeVisible();
    expect(
      within(
        screen.getByRole("list", { name: "Dependabot alert timeline" }),
      ).getByText("Dependabot opened this alert from the dependency graph."),
    ).toBeVisible();
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("submits dismiss and assignment mutations through the same-origin action route", async () => {
    const fetchMock = vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: async () =>
        detail({
          alert: { ...detail().alert, state: "dismissed" },
          timeline: [
            ...detail().timeline,
            {
              id: "event-2",
              eventType: "dismissed",
              message: "Dismissed this alert as not_used.",
              actor: null,
              createdAt: "2026-05-05T01:00:00Z",
            },
          ],
        }),
    } as Response);

    render(
      <RepositoryDependabotAlertDetailPage
        detailResult={{ ok: true, dependabotAlert: detail() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Dismiss reason"), {
      target: { value: "not_used" },
    });
    fireEvent.change(screen.getByLabelText("Optional comment"), {
      target: { value: "Fixture dependency only" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Dismiss alert" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/security/dependabot/1/actions",
      expect.objectContaining({
        body: JSON.stringify({
          action: "dismiss",
          dismissalComment: "Fixture dependency only",
          dismissalReason: "not_used",
        }),
        method: "PATCH",
      }),
    );
    expect(await screen.findByText("Dismiss saved.")).toBeVisible();
  });

  it("creates a security update pull request through the same-origin route", async () => {
    const fetchMock = vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: async () => ({
        status: "created",
        branch: "dependabot/npm/testing-library-react-1",
        commitOid: "abc123",
        pullRequestHref: "/namuh-eng/opengithub/pull/12",
        message: "Security update pull request created.",
      }),
    } as Response);

    render(
      <RepositoryDependabotAlertDetailPage
        detailResult={{ ok: true, dependabotAlert: detail() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Create security update PR" }),
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    expect(fetchMock).toHaveBeenCalledWith(
      "/namuh-eng/opengithub/security/dependabot/1/security-update",
      expect.objectContaining({ method: "POST" }),
    );
    expect(
      await screen.findByText("Security update pull request created."),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Open security update PR" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pull/12");
  });

  it("renders read-only and unavailable states without mutation controls", () => {
    const readonly = detail({
      viewer: {
        permission: "read",
        canRead: true,
        canWrite: false,
        canEditPolicy: false,
        canViewPrivateAlertCounts: false,
      },
    });
    const { rerender } = render(
      <RepositoryDependabotAlertDetailPage
        detailResult={{ ok: true, dependabotAlert: readonly }}
        repository={repositoryOverview()}
      />,
    );
    expect(screen.getByText("Read-only access")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Dismiss alert" })).toBeNull();

    rerender(
      <RepositoryDependabotAlertDetailPage
        detailResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "dependabot alert was not found",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Alert unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to Dependabot alerts" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/dependabot");
  });
});
