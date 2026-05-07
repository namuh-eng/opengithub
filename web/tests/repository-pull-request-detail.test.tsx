import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PullRequestFilesChangedPage } from "@/components/PullRequestFilesChangedPage";
import { RepositoryPullRequestDetailPage } from "@/components/RepositoryPullRequestDetailPage";
import type {
  PullRequestDetailView,
  PullRequestDiffReviewView,
  PullRequestTimelineItem,
  RepositoryOverview,
} from "@/lib/api";
import { apiEndpointDocs } from "@/lib/api-docs";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Pull request detail test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 2,
    tagCount: 0,
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
      starred: false,
      watching: false,
      forkedRepositoryHref: null,
    },
    cloneUrls: {
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
}

function pullRequestDetail(
  overrides: Partial<PullRequestDetailView> = {},
): PullRequestDetailView {
  const base: PullRequestDetailView = {
    id: "pull-1",
    issueId: "issue-1",
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    number: 42,
    title: "Split repository routes",
    body: "Routes are split by resource.",
    bodyHtml: "<p>Routes are split by resource.</p>",
    state: "open",
    isDraft: false,
    author: {
      id: "user-2",
      login: "hubot",
      displayName: "Hubot",
      avatarUrl: null,
    },
    authorRole: "owner",
    headRef: "hubot/split-routes",
    baseRef: "main",
    labels: [
      {
        id: "label-1",
        name: "review",
        color: "var(--accent)",
        description: "Needs review",
      },
    ],
    milestone: { id: "mile-1", title: "Review queue", state: "open" },
    assignees: [
      {
        id: "user-3",
        login: "mira",
        displayName: "Mira",
        avatarUrl: null,
      },
    ],
    requestedReviewers: [],
    latestReviews: [
      {
        reviewer: {
          id: "user-4",
          login: "ashley",
          displayName: "Ashley",
          avatarUrl: null,
        },
        state: "approved",
        submittedAt: "2026-05-01T00:05:00Z",
      },
    ],
    linkedIssues: [
      {
        number: 12,
        state: "open",
        title: "Track route work",
        href: "/mona/octo-app/issues/12",
      },
    ],
    participants: [
      {
        id: "user-2",
        login: "hubot",
        displayName: "Hubot",
        avatarUrl: null,
      },
      {
        id: "user-4",
        login: "ashley",
        displayName: "Ashley",
        avatarUrl: null,
      },
    ],
    review: {
      state: "approved",
      required: true,
      requestedReviewers: [],
      reviewerCount: 1,
    },
    checks: {
      status: "completed",
      conclusion: "success",
      totalCount: 4,
      completedCount: 4,
      failedCount: 0,
    },
    taskProgress: { completed: 2, total: 3 },
    stats: {
      commits: 8,
      files: 6,
      additions: 120,
      deletions: 32,
      comments: 3,
    },
    subscription: {
      subscribed: true,
      reason: "participating",
      customEvents: [],
      canCustomize: true,
    },
    mergeability: {
      state: "ready",
      canMerge: true,
      canClose: true,
      canReopen: false,
      canMarkReady: false,
      defaultMethod: "squash",
      methods: ["squash", "merge_commit", "rebase"],
      branchProtection: {
        protected: false,
        pattern: null,
        requiredApprovingReviewCount: 0,
        requiresUpToDateBranch: false,
        requiredStatusChecks: [],
      },
      blockers: [],
      summary:
        "Ready to merge: approved review state, 4 of 4 checks complete, 6 changed files.",
    },
    metadataOptions: {
      labels: [
        {
          id: "label-1",
          name: "review",
          color: "var(--accent)",
          description: "Needs review",
        },
        {
          id: "label-2",
          name: "docs",
          color: "var(--ok)",
          description: "Documentation",
        },
      ],
      assignees: [
        {
          id: "user-3",
          login: "mira",
          displayName: "Mira",
          avatarUrl: null,
        },
        {
          id: "user-5",
          login: "jaeyun",
          displayName: "Jaeyun",
          avatarUrl: null,
        },
      ],
      milestones: [
        { id: "mile-1", title: "Review queue", state: "open" },
        { id: "mile-2", title: "Release train", state: "open" },
      ],
    },
    href: "/mona/octo-app/pull/42",
    commitsHref: "/mona/octo-app/pull/42/commits",
    checksHref: "/mona/octo-app/pull/42/checks",
    filesHref: "/mona/octo-app/pull/42/files",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-01T00:10:00Z",
    closedAt: null,
    mergedAt: null,
    viewerPermission: "owner",
  };
  return { ...base, ...overrides };
}

function pullRequestTimeline(): PullRequestTimelineItem[] {
  return [
    {
      id: "event-opened",
      eventType: "opened",
      actor: {
        id: "user-2",
        login: "hubot",
        displayName: "Hubot",
        avatarUrl: null,
      },
      comment: null,
      metadata: {},
      createdAt: "2026-05-01T00:00:00Z",
    },
    {
      id: "event-comment-1",
      eventType: "commented",
      actor: {
        id: "user-4",
        login: "ashley",
        displayName: "Ashley",
        avatarUrl: null,
      },
      comment: {
        id: "comment-1",
        body: "Looks **ready** to review.",
        bodyHtml:
          '<div class="markdown-body"><p>Looks <strong>ready</strong> to review.</p></div>',
        isMinimized: false,
        reactions: [],
        createdAt: "2026-05-01T00:08:00Z",
        updatedAt: "2026-05-01T00:08:00Z",
      },
      metadata: { commentId: "comment-1" },
      createdAt: "2026-05-01T00:08:00Z",
    },
  ];
}

function pullRequestDiffReview(): PullRequestDiffReviewView {
  return {
    pullRequest: pullRequestDetail(),
    settings: {
      view: "unified",
      whitespace: "show",
      commit: null,
      filter: null,
      page: 1,
      pageSize: 50,
    },
    totalFiles: 2,
    page: 1,
    pageSize: 50,
    hasMore: false,
    fileTree: [
      {
        id: "file-1",
        path: "crates/api/src/routes/pulls.rs",
        status: "modified",
        additions: 80,
        deletions: 12,
        viewed: false,
        versionKey: "blob-1:80:12",
        href: "/mona/octo-app/pull/42/files#diff-crates-api-src-routes-pulls-rs",
      },
      {
        id: "file-2",
        path: "web/src/components/RepositoryPullRequestDetailPage.tsx",
        status: "added",
        additions: 40,
        deletions: 20,
        viewed: true,
        versionKey: "blob-2:40:20",
        href: "/mona/octo-app/pull/42/files#diff-web-src-components-repositorypullrequestdetailpage-tsx",
      },
    ],
    files: [
      {
        id: "file-1",
        path: "crates/api/src/routes/pulls.rs",
        status: "modified",
        additions: 80,
        deletions: 12,
        byteSize: 4000,
        blobOid: "blob-1",
        language: "Rust",
        viewed: false,
        viewedAt: null,
        versionKey: "blob-1:80:12",
        href: "/mona/octo-app/pull/42/files#diff-crates-api-src-routes-pulls-rs",
        hunks: [
          {
            id: "hunk-1",
            header: "@@ -1,3 +1,4 @@",
            oldStart: 1,
            oldLines: 3,
            newStart: 1,
            newLines: 4,
            lines: [
              {
                kind: "context",
                oldLine: 1,
                newLine: 1,
                content: "use axum::Router;",
                position: 1,
                commentCount: 0,
              },
              {
                kind: "added",
                oldLine: null,
                newLine: 2,
                content: "use axum::routing::patch;",
                position: 2,
                commentCount: 1,
              },
            ],
          },
        ],
        comments: [
          {
            id: "comment-1",
            author: {
              id: "user-4",
              login: "ashley",
              displayName: "Ashley",
              avatarUrl: null,
            },
            body: "Check this route.",
            bodyHtml: "<p>Check this route.</p>",
            path: "crates/api/src/routes/pulls.rs",
            side: "right",
            oldLine: null,
            newLine: 2,
            position: 2,
            state: "published",
            createdAt: "2026-05-01T00:08:00Z",
            updatedAt: "2026-05-01T00:08:00Z",
          },
        ],
      },
      {
        id: "file-2",
        path: "web/src/components/RepositoryPullRequestDetailPage.tsx",
        status: "added",
        additions: 40,
        deletions: 20,
        byteSize: 8000,
        blobOid: "blob-2",
        language: "TypeScript",
        viewed: true,
        viewedAt: "2026-05-01T00:12:00Z",
        versionKey: "blob-2:40:20",
        href: "/mona/octo-app/pull/42/files#diff-web-src-components-repositorypullrequestdetailpage-tsx",
        hunks: [],
        comments: [],
      },
    ],
    commits: [],
    pendingReview: {
      draftId: "draft-1",
      commentCount: 1,
      summaryBody: null,
      reviewState: "commented",
    },
  };
}

describe("RepositoryPullRequestDetailPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders the pull request conversation shell and sidebar metadata", () => {
    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    expect(
      screen.getByRole("heading", { name: /Split repository routes/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("Open")).toBeInTheDocument();
    expect(screen.getByText(/wants to merge/)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Files changed/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42/files",
    );
    expect(screen.getByRole("link", { name: ".diff" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/pulls/42.diff",
    );
    expect(screen.getByRole("link", { name: ".patch" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/pulls/42.patch",
    );
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs/api#pulls-raw-diff",
    );
    expect(screen.getByText("Routes are split by resource.")).toBeVisible();
    expect(screen.getAllByText("ashley").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("mira")).toBeVisible();
    expect(screen.getByText("Review queue")).toBeVisible();
    expect(screen.getByRole("link", { name: "#12 · open" })).toHaveAttribute(
      "href",
      "/mona/octo-app/issues/12",
    );
    expect(screen.getByText(/hubot opened this pull request/)).toBeVisible();
    expect(screen.getByText(/Looks/)).toBeVisible();
    expect(screen.getAllByText("ready").length).toBeGreaterThanOrEqual(1);
    expect(
      screen.getByRole("button", { name: "Open merge confirmation" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Close pull request" }),
    ).toBeEnabled();
    expect(screen.getByRole("textbox", { name: "Comment body" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Comment" })).toBeDisabled();
  });

  it("renders the AI review brief, risk files, reviewers, and regenerate action", () => {
    const { container } = render(
      <RepositoryPullRequestDetailPage
        aiSummary={{
          enabled: true,
          reason: null,
          output: {
            id: "ai-pr-1",
            kind: "pr_summary",
            scopeType: "pull_request",
            scopeId: "pr-1",
            contentHash: "hash",
            promptVersion: "ai-001-v1",
            model: "gpt-4o",
            output: "TL;DR: routes moved into smaller modules.",
            generatedAt: "2026-05-07T00:00:00Z",
            regeneratedCount: 0,
            cached: true,
          },
          filesOfInterest: [
            {
              path: "web/src/components/RepositoryPullRequestDetailPage.tsx",
              note: "modified with reviewer-visible UI changes",
            },
          ],
          suggestedReviewers: [{ login: "ashley", reason: "recent committer" }],
          inlineCommentSeed:
            "Ask whether diff coverage needs an integration test.",
        }}
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    expect(screen.getByLabelText("AI pull request summary")).toHaveTextContent(
      "TL;DR",
    );
    expect(screen.getByText("ashley · recent committer")).toBeVisible();
    expect(
      container.querySelector(
        'form[action="/mona/octo-app/pull/42/ai/summary"]',
      ),
    ).toHaveAttribute("method", "post");
  });

  it("opens merge confirmation, switches methods, and submits commit details", async () => {
    const mergedPullRequest = pullRequestDetail({
      state: "merged",
      mergedAt: "2026-05-01T00:20:00Z",
      mergeability: {
        ...pullRequestDetail().mergeability,
        state: "merged",
        canMerge: false,
        canClose: false,
        defaultMethod: "merge_commit",
        summary: "This pull request has been merged.",
        blockers: [
          {
            code: "already_merged",
            message: "This pull request has already been merged.",
            severity: "blocking",
          },
        ],
      },
    });
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith("/merge")) {
        return { ok: true, json: async () => mergedPullRequest };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Open merge confirmation" }),
    );
    expect(
      screen.getByRole("heading", { name: "Confirm merge" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Commit title")).toHaveValue(
      "Split repository routes (#42)",
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Create a merge commit" }),
    );
    expect(screen.getByLabelText("Commit title")).toHaveValue(
      "Merge pull request #42 from hubot/split-routes",
    );
    fireEvent.change(screen.getByLabelText("Commit title"), {
      target: { value: "Ship merge workflow" },
    });
    fireEvent.change(screen.getByLabelText("Commit body"), {
      target: { value: "Keeps the merge audit trail." },
    });
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /Delete head branch after merge/,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: "Confirm Create a merge commit",
      }),
    );

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/merge",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            method: "merge_commit",
            commitTitle: "Ship merge workflow",
            commitBody: "Keeps the merge audit trail.",
            deleteBranch: true,
          }),
        }),
      );
    });
    expect(await screen.findByText("Pull request merged.")).toBeVisible();
    expect(screen.queryByRole("heading", { name: "Confirm merge" })).toBeNull();
  });

  it("renders structured merge blockers returned by the API", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith("/merge")) {
        return {
          ok: false,
          json: async () => ({
            error: {
              code: "merge_blocked",
              message: "Pull request cannot merge.",
            },
            status: 409,
            details: {
              blockers: [
                {
                  code: "required_checks_failed",
                  message: "Required status checks have failed.",
                  severity: "blocking",
                },
              ],
            },
          }),
        };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Open merge confirmation" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm Squash and merge" }),
    );

    expect(await screen.findByText("Pull request cannot merge.")).toBeVisible();
    expect(
      await screen.findByText("Required status checks have failed."),
    ).toBeVisible();
  });

  it("shows a concrete sign-in CTA for anonymous public readers", () => {
    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={false}
      />,
    );

    expect(
      screen.getByRole("link", { name: "Sign in to participate" }),
    ).toHaveAttribute("href", "/login?next=%2Fmona%2Focto-app%2Fpull%2F42");
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      "/login?next=%2Fmona%2Focto-app%2Fpull%2F42",
    );
  });

  it("renders the files changed tab as a live comparison surface", () => {
    const { container } = render(
      <PullRequestFilesChangedPage
        diffReview={pullRequestDiffReview()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    expect(
      screen.getByRole("heading", { name: /Files changed/ }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Conversation" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42",
    );
    expect(
      screen.getByRole("button", { name: "Review changes" }),
    ).toBeEnabled();
    expect(screen.getByRole("link", { name: "Unified" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42/files",
    );
    expect(screen.getByRole("link", { name: "Split" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42/files?view=split",
    );
    expect(screen.getByRole("link", { name: ".diff" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/pulls/42.diff",
    );
    expect(screen.getByRole("link", { name: ".patch" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/pulls/42.patch",
    );
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs/api#pulls-raw-diff",
    );
    expect(screen.getByRole("textbox", { name: "File filter" })).toBeVisible();
    expect(
      screen.getAllByText("crates/api/src/routes/pulls.rs").length,
    ).toBeGreaterThanOrEqual(1);
    expect(
      screen.getAllByText(
        "web/src/components/RepositoryPullRequestDetailPage.tsx",
      ).length,
    ).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("@@ -1,3 +1,4 @@")).toBeVisible();
    expect(screen.getByText("use axum::routing::patch;")).toBeVisible();
    expect(screen.getByText("Check this route.")).toBeVisible();
    expect(
      screen.getByRole("button", {
        name: "Viewed?",
      }),
    ).toBeVisible();
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("updates viewed progress and file tree state after a viewed toggle persists", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith("/files/viewed")) {
        return { ok: true, json: async () => ({}) };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);
    const { container } = render(
      <PullRequestFilesChangedPage
        diffReview={pullRequestDiffReview()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    expect(container.textContent).toContain("1 viewed");
    fireEvent.click(screen.getByRole("button", { name: "Viewed?" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/files/viewed",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({
            fileId: "file-1",
            versionKey: "blob-1:80:12",
            viewed: true,
          }),
        }),
      );
    });
    expect(await screen.findByText("File marked as viewed.")).toBeVisible();
    expect(container.textContent).toContain("2 viewed");
    expect(screen.getAllByText("viewed").length).toBeGreaterThanOrEqual(2);
  });

  it("renders empty filter recovery and auth-gated review actions without inert controls", () => {
    const filteredReview = {
      ...pullRequestDiffReview(),
      settings: {
        ...pullRequestDiffReview().settings,
        filter: "missing",
      },
      totalFiles: 0,
      fileTree: [],
      files: [],
    };
    const { container } = render(
      <PullRequestFilesChangedPage
        diffReview={filteredReview}
        repository={repositoryOverview()}
        viewerAuthenticated={false}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "No changed files match this filter.",
      }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear filter" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/42/files",
    );
    expect(
      screen.getByRole("link", { name: "Clear file filter" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/42/files");
    expect(
      screen.getByRole("link", { name: "Review changes" }),
    ).toHaveAttribute(
      "href",
      "/login?next=%2Fmona%2Focto-app%2Fpull%2F42%2Ffiles",
    );
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("submits and abandons pull request reviews from the files changed dialog", async () => {
    const submittedReview = {
      id: "review-1",
      reviewer: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      state: "commented",
      body: "Ready for a maintainer pass.",
      submittedAt: "2026-05-01T00:20:00Z",
      publishedCommentCount: 1,
      pendingReview: {
        draftId: null,
        commentCount: 0,
        summaryBody: null,
        reviewState: "commented",
      },
    };
    const fetchMock = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url === "/markdown/preview") {
          return {
            ok: true,
            json: async () => ({
              html: "<p>Ready for a maintainer pass.</p>",
            }),
          };
        }
        if (url.endsWith("/files/reviews") && init?.method === "POST") {
          return { ok: true, json: async () => submittedReview };
        }
        if (url.endsWith("/files/reviews") && init?.method === "DELETE") {
          return {
            ok: true,
            json: async () => ({
              draftId: null,
              commentCount: 0,
              summaryBody: null,
              reviewState: "commented",
            }),
          };
        }
        throw new Error(`unexpected fetch ${url}`);
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <PullRequestFilesChangedPage
        diffReview={pullRequestDiffReview()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Review changes" }));
    expect(
      screen.getByRole("dialog", { name: /Review changes/ }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Abandon review" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/files/reviews",
        expect.objectContaining({ method: "DELETE" }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: "Review changes" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Review summary" }), {
      target: { value: "Ready for a maintainer pass." },
    });
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    expect(
      await screen.findByText("Ready for a maintainer pass."),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("tab", { name: "Write" }));
    fireEvent.click(screen.getByRole("radio", { name: /Comment/ }));
    fireEvent.click(screen.getByRole("button", { name: "Submit review" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/files/reviews",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            body: "Ready for a maintainer pass.",
            state: "commented",
          }),
        }),
      );
    });
    expect(await screen.findByText(/Review submitted/)).toBeVisible();
  });

  it("creates, edits, previews, and deletes pending inline review comments", async () => {
    const savedDraft = {
      id: "draft-comment-2",
      author: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      body: "Pending **line** note",
      bodyHtml: "<p>Pending <strong>line</strong> note</p>",
      path: "crates/api/src/routes/pulls.rs",
      side: "right",
      oldLine: null,
      newLine: 2,
      position: 2,
      state: "pending",
      createdAt: "2026-05-01T00:14:00Z",
      updatedAt: "2026-05-01T00:14:00Z",
    };
    const updatedDraft = {
      ...savedDraft,
      body: "Edited pending note",
      bodyHtml: "<p>Edited pending note</p>",
      updatedAt: "2026-05-01T00:15:00Z",
    };
    const fetchMock = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url === "/markdown/preview") {
          return {
            ok: true,
            json: async () => ({
              html: "<p>Pending <strong>line</strong> note</p>",
            }),
          };
        }
        if (
          url.endsWith("/review-comments/drafts") &&
          init?.method === "POST"
        ) {
          return { ok: true, json: async () => savedDraft };
        }
        if (
          url.endsWith("/review-comments/drafts/draft-comment-2") &&
          init?.method === "PATCH"
        ) {
          return { ok: true, json: async () => updatedDraft };
        }
        if (
          url.endsWith("/review-comments/drafts/draft-comment-2") &&
          init?.method === "DELETE"
        ) {
          return {
            ok: true,
            json: async () => ({
              draftId: "draft-1",
              commentCount: 0,
              summaryBody: null,
              reviewState: "commented",
            }),
          };
        }
        throw new Error(`unexpected fetch ${url}`);
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <PullRequestFilesChangedPage
        diffReview={pullRequestDiffReview()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Add comment at diff position 2" }),
    );
    fireEvent.change(
      screen.getByRole("textbox", {
        name: /Pending review comment for crates\/api\/src\/routes\/pulls.rs line 2/,
      }),
      { target: { value: "Pending **line** note" } },
    );
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/markdown/preview",
        expect.objectContaining({ method: "POST" }),
      );
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Save pending comment" }),
    );
    expect(
      await screen.findByText("left a pending review comment"),
    ).toBeVisible();
    expect(screen.getByText("pending", { exact: true })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    fireEvent.change(
      screen.getByRole("textbox", { name: "Edit pending review comment" }),
      { target: { value: "Edited pending note" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => {
      expect(
        screen.queryByRole("textbox", { name: "Edit pending review comment" }),
      ).not.toBeInTheDocument();
    });
    expect(screen.getByText("Edited pending note")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => {
      expect(screen.queryByText("Edited pending note")).not.toBeInTheDocument();
    });
  });

  it("previews markdown and posts pull request comments", async () => {
    const createdComment: PullRequestTimelineItem = {
      id: "event-comment-2",
      eventType: "commented",
      actor: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      comment: {
        id: "comment-2",
        body: "New **review** note",
        bodyHtml:
          '<div class="markdown-body"><p>New <strong>review</strong> note</p></div>',
        isMinimized: false,
        reactions: [],
        createdAt: "2026-05-01T00:12:00Z",
        updatedAt: "2026-05-01T00:12:00Z",
      },
      metadata: { commentId: "comment-2" },
      createdAt: "2026-05-01T00:12:00Z",
    };
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url === "/markdown/preview") {
        return {
          ok: true,
          json: async () => ({
            html: '<div class="markdown-body"><p>Preview <strong>works</strong></p></div>',
          }),
        };
      }
      if (url.endsWith("/comments")) {
        return { ok: true, json: async () => createdComment };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Comment body" }), {
      target: { value: "New **review** note" },
    });
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    expect(await screen.findByText("Preview")).toBeVisible();
    expect(await screen.findByText("works")).toBeVisible();

    fireEvent.click(screen.getByRole("tab", { name: "Write" }));
    fireEvent.click(screen.getByRole("button", { name: "Comment" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/comments",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ body: "New **review** note" }),
        }),
      );
    });
    expect(await screen.findByText("Comment posted.")).toBeVisible();
    expect(
      (await screen.findAllByText("review")).length,
    ).toBeGreaterThanOrEqual(1);
  });

  it("updates reviewers, metadata, draft state, and notification subscription", async () => {
    const updatedWithReviewer = pullRequestDetail({
      requestedReviewers: [
        {
          id: "user-5",
          login: "jaeyun",
          displayName: "Jaeyun",
          avatarUrl: null,
        },
      ],
    });
    const updatedMetadata = pullRequestDetail({
      labels: [
        {
          id: "label-2",
          name: "docs",
          color: "var(--ok)",
          description: "Documentation",
        },
      ],
      assignees: [
        {
          id: "user-5",
          login: "jaeyun",
          displayName: "Jaeyun",
          avatarUrl: null,
        },
      ],
      milestone: { id: "mile-2", title: "Release train", state: "open" },
    });
    const updatedDraft = pullRequestDetail({ isDraft: true });
    const fetchMock = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.endsWith("/review-requests")) {
          return { ok: true, json: async () => updatedWithReviewer };
        }
        if (url.endsWith("/metadata")) {
          return { ok: true, json: async () => updatedMetadata };
        }
        if (url.endsWith("/draft")) {
          return { ok: true, json: async () => updatedDraft };
        }
        if (url.endsWith("/subscription")) {
          return {
            ok: true,
            json: async () => ({
              subscribed: false,
              reason: "ignored",
              customEvents: [],
              canCustomize: true,
            }),
          };
        }
        throw new Error(`unexpected fetch ${url} ${init?.method ?? "GET"}`);
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]);
    fireEvent.click(screen.getByRole("button", { name: "Request jaeyun" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/review-requests",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ reviewerUserIds: ["user-5"] }),
        }),
      );
    });
    expect(await screen.findByText("Review requests updated.")).toBeVisible();

    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[1]);
    fireEvent.click(screen.getByRole("button", { name: "Assign jaeyun" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/metadata",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({
            labelIds: ["label-1"],
            assigneeUserIds: ["user-3", "user-5"],
            milestoneId: "mile-1",
          }),
        }),
      );
    });
    expect(
      await screen.findByText("Pull request metadata updated."),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Convert to draft" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/draft",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ isDraft: true }),
        }),
      );
    });
    expect(
      await screen.findByText("Pull request converted to draft."),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Unsubscribe" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/subscription",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ subscribed: false, customEvents: [] }),
        }),
      );
    });
    expect(await screen.findByText("Unsubscribed.")).toBeVisible();
    expect(screen.getByText("Not subscribed")).toBeVisible();
  });

  it("customizes pull request thread notification events", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith("/subscription")) {
        return {
          ok: true,
          json: async () => ({
            subscribed: true,
            reason: "subscribed",
            customEvents: ["merged", "closed"],
            canCustomize: true,
          }),
        };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail()}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Customize" }));
    expect(
      screen.getByRole("heading", { name: "Customize updates" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("checkbox", { name: /Merged/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: /Closed/ }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/subscription",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({
            subscribed: true,
            customEvents: ["merged", "closed"],
          }),
        }),
      );
    });
    expect(
      await screen.findByText("Subscribed to notifications."),
    ).toBeVisible();
    expect(screen.getByText("Custom events: merged, closed")).toBeVisible();
  });

  it("renders merge blockers and posts state and merge actions", async () => {
    const closed = pullRequestDetail({
      state: "closed",
      mergeability: {
        ...pullRequestDetail().mergeability,
        state: "closed",
        canMerge: false,
        canClose: false,
        canReopen: true,
        canMarkReady: false,
        defaultMethod: "squash",
        methods: ["squash", "merge_commit", "rebase"],
        blockers: [
          {
            code: "pull_request_closed",
            message:
              "Closed pull requests must be reopened before they can merge.",
            severity: "blocking",
          },
        ],
        summary: "Closed pull requests must be reopened before they can merge.",
      },
    });
    const reopened = pullRequestDetail();
    const merged = pullRequestDetail({
      state: "merged",
      mergeability: {
        ...pullRequestDetail().mergeability,
        state: "merged",
        canMerge: false,
        canClose: false,
        blockers: [
          {
            code: "already_merged",
            message: "This pull request has already been merged.",
            severity: "blocking",
          },
        ],
        summary: "This pull request has already been merged.",
      },
    });
    const fetchMock = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.endsWith("/state")) {
          return { ok: true, json: async () => reopened };
        }
        if (url.endsWith("/merge")) {
          return { ok: true, json: async () => merged };
        }
        throw new Error(`unexpected fetch ${url} ${init?.method ?? "GET"}`);
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullRequestDetailPage
        pullRequest={closed}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    expect(
      screen.getAllByText(
        "Closed pull requests must be reopened before they can merge.",
      ).length,
    ).toBeGreaterThanOrEqual(1);
    fireEvent.click(
      screen.getByRole("button", { name: "Reopen pull request" }),
    );
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/state",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ state: "open" }),
        }),
      );
    });
    expect(await screen.findByText("Pull request reopened.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Rebase and merge" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Open merge confirmation" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Confirm Rebase and merge" }),
    );
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/merge",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            method: "rebase",
            commitTitle: "Rebase pull request #42 onto main",
            commitBody: null,
            deleteBranch: false,
          }),
        }),
      );
    });
    expect(await screen.findByText("Pull request merged.")).toBeVisible();
    expect(screen.getByText("Merged")).toBeVisible();
  });

  it("renders repository merge policy methods and branch rule blockers", () => {
    render(
      <RepositoryPullRequestDetailPage
        pullRequest={pullRequestDetail({
          mergeability: {
            ...pullRequestDetail().mergeability,
            canMerge: false,
            defaultMethod: "squash",
            methods: ["squash"],
            branchProtection: {
              protected: true,
              pattern: "main",
              requiredApprovingReviewCount: 2,
              requiresUpToDateBranch: true,
              requiredStatusChecks: ["ci/test", "lint"],
              requiresLinearHistory: true,
              activeRuleCount: 1,
              activeRulesetCount: 1,
            },
            blockers: [
              {
                code: "required_approvals",
                message:
                  "2 approving reviews are required by branch protection.",
                severity: "blocking",
              },
              {
                code: "required_checks_missing",
                message:
                  "Required status checks have not reported yet: ci/test, lint.",
                severity: "blocking",
              },
            ],
            summary: "This pull request is blocked by branch protection.",
          },
        })}
        repository={repositoryOverview()}
        timeline={pullRequestTimeline()}
        viewerAuthenticated={true}
      />,
    );

    expect(screen.getByText("Protected branch")).toBeVisible();
    expect(screen.getAllByText("main").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText(/approving review required/)).toBeVisible();
    expect(screen.getByText(/Required checks:/)).toBeVisible();
    expect(screen.getAllByText(/ci\/test, lint/).length).toBeGreaterThanOrEqual(
      1,
    );
    expect(screen.getByText("Up-to-date branch required")).toBeVisible();
    expect(screen.getByText("Linear history required")).toBeVisible();
    expect(screen.getByText("2 policies combined")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Squash and merge" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Create a merge commit" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Rebase and merge" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Open merge confirmation" }),
    ).toBeDisabled();
  });

  it("documents pull request list, review, merge, diff, and patch endpoints", () => {
    const pullDocIds = apiEndpointDocs.map((endpoint) => endpoint.id);
    expect(pullDocIds).toEqual(
      expect.arrayContaining([
        "pulls-list",
        "pulls-create",
        "pulls-files",
        "pulls-submit-review",
        "pulls-merge",
        "pulls-raw-diff",
        "pulls-raw-patch",
      ]),
    );
    expect(
      apiEndpointDocs.find((endpoint) => endpoint.id === "pulls-raw-diff")
        ?.path,
    ).toBe("/api/repos/{owner}/{repo}/pulls/{number}.diff");
    expect(
      apiEndpointDocs.find((endpoint) => endpoint.id === "pulls-merge")?.notes,
    ).toContain(
      "Blocked merges return HTTP 409 with code merge_blocked and details.blockers.",
    );
  });
});
