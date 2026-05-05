import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositorySecretScanningAlertDetailPage } from "@/components/RepositorySecretScanningAlertDetailPage";
import type {
  RepositoryOverview,
  RepositorySecretScanningAlertDetail,
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

function secretScanningDetail(
  overrides: Partial<RepositorySecretScanningAlertDetail> = {},
): RepositorySecretScanningAlertDetail {
  const base: RepositorySecretScanningAlertDetail = {
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
      message: "Secret scanning is monitoring committed content.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    alert: {
      id: "alert-1",
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
      redactedSecret: "github_pat_********",
      redactedContext: "TOKEN=github_pat_********",
      fingerprint: "fp-redacted-1",
      validity: {
        status: "active",
        provider: "GitHub",
        checkedAt: "2026-05-05T00:00:00Z",
        message: "Provider says the token is active.",
      },
      primaryLocation: {
        path: ".env",
        pathHref: "/namuh-eng/opengithub/blob/main/.env#L4",
        rawHref: "/namuh-eng/opengithub/raw/main/.env",
        commitHref: "/namuh-eng/opengithub/commit/abc123",
        refName: "refs/heads/main",
        branchName: "main",
        startLine: 4,
        endLine: null,
        redactedSnippet: "TOKEN=github_pat_********",
      },
      assignees: [
        { id: "user-1", login: "jaeyun", avatarUrl: null, href: "/jaeyun" },
      ],
      bypassed: true,
      href: "/namuh-eng/opengithub/security/secret-scanning/1",
      detectedAt: "2026-05-05T00:00:00Z",
      updatedAt: "2026-05-05T00:00:00Z",
    },
    pattern: {
      id: "pattern-1",
      slug: "github-pat",
      provider: "GitHub",
      secretType: "github_personal_access_token",
      displayName: "GitHub personal access token",
      resultKind: "provider",
      pushProtectionEnabled: true,
    },
    locations: [
      {
        path: ".env",
        pathHref: "/namuh-eng/opengithub/blob/main/.env#L4",
        rawHref: "/namuh-eng/opengithub/raw/main/.env",
        commitHref: "/namuh-eng/opengithub/commit/abc123",
        refName: "refs/heads/main",
        branchName: "main",
        startLine: 4,
        endLine: null,
        redactedSnippet: "TOKEN=github_pat_********",
      },
    ],
    validity: {
      status: "active",
      provider: "GitHub",
      checkedAt: "2026-05-05T00:00:00Z",
      message: "Provider says the token is active.",
    },
    bypasses: [
      {
        id: "bypass-1",
        actor: {
          id: "user-1",
          login: "jaeyun",
          avatarUrl: null,
          href: "/jaeyun",
        },
        reason: "Emergency rotation landed separately.",
        status: "accepted",
        refName: "refs/heads/main",
        commitOid: "abc123",
        path: ".env",
        redactedSnippet: "TOKEN=github_pat_********",
        createdAt: "2026-05-05T00:00:00Z",
      },
    ],
    timeline: [
      {
        id: "event-1",
        eventType: "created",
        message: "Secret scanning opened this alert from committed content.",
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
        selected: true,
      },
      {
        id: "user-2",
        kind: "user",
        login: "mona",
        avatarUrl: null,
        selected: false,
      },
    ],
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
  };
  return { ...base, ...overrides };
}

describe("RepositorySecretScanningAlertDetailPage", () => {
  it("renders redacted detail, location, bypass, timeline, and concrete links", () => {
    const { container } = render(
      <RepositorySecretScanningAlertDetailPage
        detailResult={{ ok: true, secretScanningAlert: secretScanningDetail() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "GitHub personal access token" }),
    ).toBeVisible();
    expect(container.textContent).toContain("github_pat_********");
    expect(
      screen.getByText("Provider says the token is active."),
    ).toBeVisible();
    expect(
      screen.getByText("Emergency rotation landed separately."),
    ).toBeVisible();
    expect(screen.getAllByRole("link", { name: /\.env:4/ })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/.env#L4",
    );
    expect(
      screen.getByRole("link", { name: "Back to alerts" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/secret-scanning");
    expect(
      screen.getByRole("list", { name: "Secret scanning alert timeline" }),
    ).toBeVisible();
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toContain("ghp_plaintext_secret");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("submits resolve, validity, and assignment mutations through concrete routes", async () => {
    const resolved = secretScanningDetail({
      alert: {
        ...secretScanningDetail().alert,
        state: "resolved",
        resolution: "revoked",
      },
    });
    const validityUpdated = secretScanningDetail({
      validity: {
        status: "inactive",
        provider: "manual",
        checkedAt: "2026-05-05T01:00:00Z",
        message: "Validity updated by a maintainer.",
      },
    });
    const assigned = secretScanningDetail({
      alert: {
        ...secretScanningDetail().alert,
        assignees: [
          { id: "user-2", login: "mona", avatarUrl: null, href: "/mona" },
        ],
      },
      assigneeOptions: [
        {
          id: "user-1",
          kind: "user",
          login: "jaeyun",
          avatarUrl: null,
          selected: false,
        },
        {
          id: "user-2",
          kind: "user",
          login: "mona",
          avatarUrl: null,
          selected: true,
        },
      ],
    });

    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(Response.json(resolved))
      .mockResolvedValueOnce(Response.json(validityUpdated))
      .mockResolvedValueOnce(Response.json(assigned));

    render(
      <RepositorySecretScanningAlertDetailPage
        detailResult={{ ok: true, secretScanningAlert: secretScanningDetail() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Resolve alert" }));
    await waitFor(() =>
      expect(screen.getByText("Resolution saved.")).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/namuh-eng/opengithub/security/secret-scanning/1/actions",
      expect.objectContaining({ method: "PATCH" }),
    );

    fireEvent.change(screen.getByLabelText("Token validity"), {
      target: { value: "inactive" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save validity" }));
    await waitFor(() =>
      expect(screen.getByText("Validity saved.")).toBeVisible(),
    );

    fireEvent.click(screen.getByLabelText("mona"));
    fireEvent.click(screen.getByRole("button", { name: "Save assignments" }));
    await waitFor(() =>
      expect(screen.getByText("Assignments saved.")).toBeVisible(),
    );
    fetchMock.mockRestore();
  });

  it("renders read-only and unavailable states without mutation controls", () => {
    const readOnly = secretScanningDetail({
      viewer: {
        permission: "read",
        canRead: true,
        canWrite: false,
        canEditPolicy: false,
        canViewPrivateAlertCounts: false,
      },
    });
    const { rerender } = render(
      <RepositorySecretScanningAlertDetailPage
        detailResult={{ ok: true, secretScanningAlert: readOnly }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Read-only access" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Resolve alert" }),
    ).not.toBeInTheDocument();

    rerender(
      <RepositorySecretScanningAlertDetailPage
        detailResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "secret scanning alert was not found",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Alert unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to Secret scanning alerts" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/secret-scanning");
  });
});
