import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryCodeScanningAlertDetailPage } from "@/components/RepositoryCodeScanningAlertDetailPage";
import type {
  RepositoryCodeScanningAlertDetail,
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

function codeScanningDetail(
  overrides: Partial<RepositoryCodeScanningAlertDetail> = {},
): RepositoryCodeScanningAlertDetail {
  const base: RepositoryCodeScanningAlertDetail = {
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
      message: "Code scanning alerts are normalized from SARIF analysis.",
      disabledReason: null,
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
    alert: {
      id: "alert-1",
      number: 1,
      state: "open",
      ruleId: "js/sql-injection",
      ruleName: "Unsanitized SQL query",
      message: "User-controlled data reaches a SQL sink.",
      severity: "warning",
      securitySeverity: "high",
      toolName: "CodeQL",
      path: "crates/api/src/routes/search.rs",
      pathHref:
        "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs#L42",
      startLine: 42,
      endLine: 45,
      refName: "refs/heads/main",
      branchName: "main",
      isDefaultBranch: true,
      linkedIssue: null,
      assignees: [
        { id: "user-1", login: "jaeyun", avatarUrl: null, href: "/jaeyun" },
      ],
      href: "/namuh-eng/opengithub/security/code-scanning/1",
      detectedAt: "2026-05-05T00:00:00Z",
      updatedAt: "2026-05-05T00:00:00Z",
    },
    location: {
      path: "crates/api/src/routes/search.rs",
      pathHref:
        "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs#L42",
      rawHref:
        "/namuh-eng/opengithub/raw/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs",
      startLine: 42,
      endLine: 45,
      codeSnippet: "let query = request.q;\nsqlx::query(&query);",
      refName: "refs/heads/main",
      commitOid: "abc123",
    },
    rule: {
      id: "js/sql-injection",
      name: "Unsanitized SQL query",
      description: "Untrusted data reaches a SQL sink.",
      helpMarkdown: "Use parameterized queries before executing user input.",
      helpUri: "https://codeql.github.com",
    },
    timeline: [
      {
        id: "event-1",
        eventType: "created",
        message: "CodeQL opened this alert from SARIF analysis.",
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
    linkedIssue: {
      issue: null,
      canLink: true,
      createHref:
        "/api/repos/namuh-eng/opengithub/security/code-scanning/1/issue",
    },
    links: {
      listHref: "/namuh-eng/opengithub/security/code-scanning",
      openHref: "/namuh-eng/opengithub/security/code-scanning?state=open",
      closedHref: "/namuh-eng/opengithub/security/code-scanning?state=closed",
      uploadHref: "/api/repos/namuh-eng/opengithub/code-scanning/sarifs",
      settingsHref: "/namuh-eng/opengithub/settings/security",
    },
  };
  return { ...base, ...overrides };
}

describe("RepositoryCodeScanningAlertDetailPage", () => {
  it("renders detail, source location, remediation, timeline, and concrete links", () => {
    const { container } = render(
      <RepositoryCodeScanningAlertDetailPage
        detailResult={{ ok: true, codeScanningAlert: codeScanningDetail() }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Unsanitized SQL query" }),
    ).toBeVisible();
    expect(
      screen.getByText("User-controlled data reaches a SQL sink."),
    ).toBeVisible();
    expect(screen.getByText("js/sql-injection")).toBeVisible();
    expect(screen.getByText(/sqlx::query/)).toBeVisible();
    expect(
      screen.getAllByRole("link", {
        name: /crates\/api\/src\/routes\/search\.rs:42/,
      })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/crates%2Fapi%2Fsrc%2Froutes%2Fsearch.rs#L42",
    );
    expect(
      screen.getByRole("link", { name: "Back to alerts" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/code-scanning");
    expect(
      screen.getByRole("list", { name: "Code scanning alert timeline" }),
    ).toBeVisible();
    expect(container.innerHTML).not.toContain('href="#"');
    expect(container.innerHTML).not.toContain("dangerouslySetInnerHTML");
    expect(container.innerHTML).not.toMatch(
      /#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer\/|Octicon/i,
    );
  });

  it("submits dismiss, assignment, and issue-link mutations through concrete routes", async () => {
    const dismissed = codeScanningDetail({
      alert: { ...codeScanningDetail().alert, state: "dismissed" },
      timeline: [
        ...codeScanningDetail().timeline,
        {
          id: "event-2",
          eventType: "dismissed",
          message: "Dismissed this alert as false_positive.",
          actor: {
            id: "user-1",
            login: "jaeyun",
            avatarUrl: null,
            href: "/jaeyun",
          },
          createdAt: "2026-05-05T01:00:00Z",
        },
      ],
    });
    const assigned = codeScanningDetail({
      alert: {
        ...codeScanningDetail().alert,
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
    const linked = codeScanningDetail({
      linkedIssue: {
        issue: {
          id: "issue-1",
          number: 44,
          title: "Code scanning: Unsanitized SQL query",
          href: "/namuh-eng/opengithub/issues/44",
        },
        canLink: true,
        createHref:
          "/api/repos/namuh-eng/opengithub/security/code-scanning/1/issue",
      },
      alert: {
        ...codeScanningDetail().alert,
        linkedIssue: {
          id: "issue-1",
          number: 44,
          title: "Code scanning: Unsanitized SQL query",
          href: "/namuh-eng/opengithub/issues/44",
        },
      },
    });

    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(Response.json(dismissed))
      .mockResolvedValueOnce(Response.json(assigned))
      .mockResolvedValueOnce(Response.json(linked));

    render(
      <RepositoryCodeScanningAlertDetailPage
        detailResult={{ ok: true, codeScanningAlert: codeScanningDetail() }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Dismiss alert" }));
    await waitFor(() =>
      expect(screen.getByText("Dismiss saved.")).toBeVisible(),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/namuh-eng/opengithub/security/code-scanning/1/actions",
      expect.objectContaining({ method: "PATCH" }),
    );

    fireEvent.click(screen.getByLabelText("mona"));
    fireEvent.click(screen.getByRole("button", { name: "Save assignments" }));
    await waitFor(() =>
      expect(screen.getByText("Assignments saved.")).toBeVisible(),
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Create linked issue" }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("link", { name: "Open linked issue #44" }),
      ).toHaveAttribute("href", "/namuh-eng/opengithub/issues/44"),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      "/namuh-eng/opengithub/security/code-scanning/1/issue",
      expect.objectContaining({ method: "POST" }),
    );
    fetchMock.mockRestore();
  });

  it("renders read-only and unavailable states without mutation controls", () => {
    const readOnly = codeScanningDetail({
      viewer: {
        permission: "read",
        canRead: true,
        canWrite: false,
        canEditPolicy: false,
        canViewPrivateAlertCounts: false,
      },
      linkedIssue: { issue: null, canLink: false, createHref: null },
    });
    const { rerender } = render(
      <RepositoryCodeScanningAlertDetailPage
        detailResult={{ ok: true, codeScanningAlert: readOnly }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Read-only access" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Dismiss alert" }),
    ).not.toBeInTheDocument();

    rerender(
      <RepositoryCodeScanningAlertDetailPage
        detailResult={{
          ok: false,
          status: 404,
          code: "not_found",
          message: "code scanning alert was not found",
        }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Alert unavailable" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to Code scanning alerts" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/security/code-scanning");
  });
});
