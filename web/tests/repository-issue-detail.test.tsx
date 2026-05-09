import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryIssueDetailPage } from "@/components/RepositoryIssueDetailPage";
import type {
  IssueDetailView,
  IssueDiscussionConversionView,
  IssueTimelineItem,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Issue detail test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 1,
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
      contributorsCount: 1,
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
  return { ...base, ...overrides };
}

function issueDetail(
  overrides: Partial<IssueDetailView> = {},
): IssueDetailView {
  const base: IssueDetailView = {
    id: "issue-1",
    repositoryId: "repo-1",
    repositoryOwner: "mona",
    repositoryName: "octo-app",
    number: 42,
    title: "Fix `runner` queue backoff",
    body: "Investigate **retry** windows.",
    bodyHtml:
      '<div class="markdown-body"><p>Investigate <strong>retry</strong> windows.</p></div>',
    state: "open",
    author: {
      id: "user-1",
      login: "mona",
      displayName: "Mona",
      avatarUrl: null,
    },
    labels: [
      {
        id: "label-1",
        name: "bug",
        color: "var(--err)",
        description: "Something is not working",
      },
    ],
    milestone: {
      id: "milestone-1",
      title: "MVP",
      state: "open",
    },
    assignees: [
      {
        id: "user-2",
        login: "hubot",
        displayName: "Hubot",
        avatarUrl: null,
      },
    ],
    participants: [
      {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      {
        id: "user-2",
        login: "hubot",
        displayName: "Hubot",
        avatarUrl: null,
      },
    ],
    attachments: [
      {
        id: "attachment-1",
        fileName: "trace.txt",
        byteSize: 1536,
        contentType: "text/plain",
        storageStatus: "metadata_only",
        createdAt: "2026-04-30T00:05:00Z",
      },
    ],
    commentCount: 3,
    linkedPullRequest: {
      number: 7,
      state: "open",
      href: "/mona/octo-app/pull/7",
    },
    href: "/mona/octo-app/issues/42",
    locked: false,
    createdAt: "2026-04-30T00:00:00Z",
    updatedAt: "2026-04-30T01:00:00Z",
    closedAt: null,
    viewerPermission: "owner",
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
    },
    subscription: {
      subscribed: false,
      reason: "not_subscribed",
      customEvents: [],
      canCustomize: true,
    },
    reactions: [
      {
        content: "thumbs_up",
        count: 2,
        viewerReacted: true,
      },
    ],
    metadataOptions: {
      labels: [
        {
          id: "label-1",
          name: "bug",
          color: "var(--err)",
          description: "Something is not working",
        },
        {
          id: "label-2",
          name: "enhancement",
          color: "var(--accent)",
          description: "New feature or request",
        },
      ],
      assignees: [
        {
          id: "user-1",
          login: "mona",
          displayName: "Mona",
          avatarUrl: null,
        },
        {
          id: "user-2",
          login: "hubot",
          displayName: "Hubot",
          avatarUrl: null,
        },
      ],
      milestones: [
        {
          id: "milestone-1",
          title: "MVP",
          state: "open",
        },
        {
          id: "milestone-2",
          title: "Launch",
          state: "open",
        },
      ],
    },
  };
  return { ...base, ...overrides };
}

function issueTimeline(): IssueTimelineItem[] {
  return [
    {
      id: "event-opened",
      eventType: "opened",
      actor: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      comment: null,
      metadata: { number: 42 },
      createdAt: "2026-04-30T00:00:00Z",
    },
    {
      id: "event-comment",
      eventType: "commented",
      actor: {
        id: "user-2",
        login: "hubot",
        displayName: "Hubot",
        avatarUrl: null,
      },
      comment: {
        id: "comment-1",
        body: "I can reproduce this with `cargo test`.",
        bodyHtml:
          '<div class="markdown-body"><p>I can reproduce this with <code>cargo test</code>.</p></div>',
        isMinimized: false,
        reactions: [],
        createdAt: "2026-04-30T00:10:00Z",
        updatedAt: "2026-04-30T00:10:00Z",
      },
      metadata: { commentId: "comment-1" },
      createdAt: "2026-04-30T00:10:00Z",
    },
  ];
}

describe("RepositoryIssueDetailPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders the issue header, body, and sidebar read model", () => {
    render(
      <RepositoryIssueDetailPage
        issue={issueDetail()}
        repository={repositoryOverview()}
        timeline={issueTimeline()}
        viewerAuthenticated={true}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: /Fix `runner` queue backoff #42/,
      }),
    ).toBeVisible();
    expect(screen.getByText("Open")).toBeVisible();
    expect(screen.getByRole("link", { name: "All issues" })).toHaveAttribute(
      "href",
      "/mona/octo-app/issues?state=open",
    );
    expect(screen.getByRole("link", { name: "New issue" })).toHaveAttribute(
      "href",
      "/mona/octo-app/issues/new",
    );
    expect(screen.getByText("retry")).toBeVisible();

    expect(screen.getByRole("heading", { name: "Assignees" })).toBeVisible();
    expect(screen.getAllByText("hubot").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByRole("heading", { name: "Labels" })).toBeVisible();
    expect(screen.getByText("bug")).toHaveAttribute(
      "title",
      "Something is not working",
    );
    expect(screen.getByText("MVP")).toBeVisible();
    expect(screen.getByRole("link", { name: /PR #7/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/7",
    );
    expect(screen.getByText("trace.txt")).toBeVisible();
    expect(screen.getByText(/1.5 KB/)).toBeVisible();
    expect(screen.getByText("Not subscribed")).toBeVisible();
    expect(screen.getByRole("heading", { name: "Type" })).toBeVisible();
    expect(screen.getByText("Issue types are not configured.")).toBeVisible();
    expect(screen.getByRole("heading", { name: "Fields" })).toBeVisible();
    expect(
      screen.getByText("No custom issue fields are configured."),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: "Projects" })).toBeVisible();
    expect(screen.getByText("No project fields are connected.")).toBeVisible();
    expect(screen.getByText(/mona opened this issue/)).toBeVisible();
    expect(screen.getAllByText("hubot").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText(/I can reproduce this with/)).toBeVisible();
    expect(
      screen.getAllByRole("button", { name: /Close issue/ }).length,
    ).toBeGreaterThanOrEqual(1);
    expect(
      screen.getByRole("toolbar", { name: "Issue reactions" }),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: /thumbs up 2/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("textbox", { name: "Comment body" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Comment" })).toBeDisabled();
  });

  it("keeps controls concrete and shows honest empty sidebar states", () => {
    render(
      <RepositoryIssueDetailPage
        issue={issueDetail({
          assignees: [],
          labels: [],
          milestone: null,
          attachments: [],
          linkedPullRequest: null,
          participants: [],
          state: "closed",
          closedAt: "2026-04-30T03:00:00Z",
        })}
        repository={repositoryOverview()}
        timeline={[]}
        viewerAuthenticated={false}
      />,
    );

    expect(screen.getByText("Closed")).toBeVisible();
    expect(screen.getByText("No one assigned")).toBeVisible();
    expect(screen.getByText("No labels")).toBeVisible();
    expect(screen.getByText("No milestone")).toBeVisible();
    expect(screen.getByText("No linked pull requests")).toBeVisible();
    expect(screen.getByText("No attachments")).toBeVisible();
    expect(screen.getByText("No participants yet")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Sign in to subscribe" }),
    ).toHaveAttribute("href", "/login?next=%2Fmona%2Focto-app%2Fissues%2F42");
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      "/login?next=%2Fmona%2Focto-app%2Fissues%2F42",
    );
    for (const link of screen.getAllByRole("link")) {
      expect(link).toHaveAttribute("href");
      expect(link.getAttribute("href")).not.toBe("#");
    }
    for (const button of screen.queryAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });

  it("updates metadata through live sidebar controls", async () => {
    const updatedIssue = issueDetail({
      labels: [
        ...issueDetail().labels,
        {
          id: "label-2",
          name: "enhancement",
          color: "var(--accent)",
          description: "New feature or request",
        },
      ],
    });
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => updatedIssue,
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryIssueDetailPage
        issue={issueDetail()}
        repository={repositoryOverview()}
        timeline={issueTimeline()}
        viewerAuthenticated={true}
      />,
    );

    const labelSection = screen
      .getByRole("heading", { name: "Labels" })
      .closest("section");
    expect(labelSection).not.toBeNull();
    fireEvent.click(
      within(labelSection as HTMLElement).getByRole("button", {
        name: "Edit",
      }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Search labels" }), {
      target: { value: "enhance" },
    });
    fireEvent.click(screen.getByRole("checkbox", { name: /enhancement/i }));
    fireEvent.click(screen.getByRole("button", { name: "Save labels" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/issues/42/metadata",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({
            labelIds: ["label-1", "label-2"],
            assigneeUserIds: ["user-2"],
            milestoneId: "milestone-1",
          }),
        }),
      );
    });
    expect(await screen.findByText("Issue metadata updated.")).toBeVisible();
    expect(screen.getByText("enhancement")).toBeVisible();
  });

  it("previews markdown, posts comments, and toggles concrete issue actions", async () => {
    const createdComment: IssueTimelineItem = {
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
        body: "New **guardrail** comment",
        bodyHtml:
          '<div class="markdown-body"><p>New <strong>guardrail</strong> comment</p></div>',
        isMinimized: false,
        reactions: [],
        createdAt: "2026-04-30T02:00:00Z",
        updatedAt: "2026-04-30T02:00:00Z",
      },
      metadata: { commentId: "comment-2" },
      createdAt: "2026-04-30T02:00:00Z",
    };
    const closedIssue = issueDetail({ state: "closed" });
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
      if (url.endsWith("/state")) {
        return { ok: true, json: async () => closedIssue };
      }
      if (url.endsWith("/subscription")) {
        return {
          ok: true,
          json: async () => ({
            subscribed: true,
            reason: "subscribed",
            customEvents: ["closed", "reopened"],
            canCustomize: true,
          }),
        };
      }
      if (url.endsWith("/reactions")) {
        return {
          ok: true,
          json: async () => [
            { content: "thumbs_up", count: 3, viewerReacted: false },
          ],
        };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryIssueDetailPage
        issue={issueDetail()}
        repository={repositoryOverview()}
        timeline={issueTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Comment body" }), {
      target: { value: "New **guardrail** comment" },
    });
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    expect(await screen.findByText("Preview")).toBeVisible();
    expect(await screen.findByText("works")).toBeVisible();

    fireEvent.click(screen.getByRole("tab", { name: "Write" }));
    fireEvent.click(screen.getByRole("button", { name: "Comment" }));
    expect(await screen.findByText("Comment posted.")).toBeVisible();
    expect(await screen.findByText("guardrail")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Thumbs up 2" }));
    expect(
      await screen.findByRole("button", { name: "Thumbs up 3" }),
    ).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(screen.getByRole("button", { name: "Subscribe" }));
    expect(
      await screen.findByText("Subscribed to notifications."),
    ).toBeVisible();
    expect(screen.getByText("Subscribed: subscribed")).toBeVisible();
    expect(screen.getByText("Custom events: closed, reopened")).toBeVisible();

    fireEvent.click(screen.getAllByRole("button", { name: "Close issue" })[0]);
    expect(await screen.findByText("Issue closed.")).toBeVisible();
    expect(screen.getByText("Closed")).toBeVisible();
  });

  it("customizes issue thread notification events", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith("/subscription")) {
        return {
          ok: true,
          json: async () => ({
            subscribed: true,
            reason: "subscribed",
            customEvents: ["closed", "reopened"],
            canCustomize: true,
          }),
        };
      }
      throw new Error(`unexpected fetch ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryIssueDetailPage
        issue={issueDetail()}
        repository={repositoryOverview()}
        timeline={issueTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Customize" }));
    expect(
      screen.getByRole("heading", { name: "Customize updates" }),
    ).toBeVisible();
    expect(
      screen.queryByRole("checkbox", { name: /Merged/ }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("checkbox", { name: /Closed/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: /Reopened/ }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/issues/42/subscription",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({
            subscribed: true,
            customEvents: ["closed", "reopened"],
          }),
        }),
      );
    });
    expect(
      await screen.findByText("Subscribed to notifications."),
    ).toBeVisible();
    expect(
      screen.queryByRole("heading", { name: "Customize updates" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Custom events: closed, reopened")).toBeVisible();
  });

  it("loads conversion metadata and submits issue-to-discussion conversion", async () => {
    const conversionView: IssueDiscussionConversionView = {
      issueId: "issue-1",
      issueNumber: 42,
      alreadyConverted: false,
      convertedDiscussionNumber: null,
      convertedDiscussionHref: null,
      commentCount: 3,
      canConvert: true,
      disabledReason: null,
      categories: [
        {
          id: "cat-general",
          slug: "general",
          name: "General",
          emoji: "💬",
          description: "Open-ended discussion",
          disabledReason: null,
        },
        {
          id: "cat-polls",
          slug: "polls",
          name: "Polls",
          emoji: "📊",
          description: "Polls",
          disabledReason: "Poll categories cannot receive converted issues.",
        },
      ],
    };
    const assignMock = vi.fn();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { assign: assignMock },
    });
    const fetchMock = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.endsWith("/convert-to-discussion")) {
          if (init?.method === "POST") {
            return {
              ok: true,
              json: async () => ({
                issueId: "issue-1",
                issueNumber: 42,
                discussionId: "discussion-9",
                discussionNumber: 9,
                href: "/mona/octo-app/discussions/9",
                title: "Fix `runner` queue backoff",
                categorySlug: "general",
              }),
            };
          }
          return { ok: true, json: async () => conversionView };
        }
        throw new Error(`unexpected fetch ${url}`);
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryIssueDetailPage
        issue={issueDetail()}
        repository={repositoryOverview()}
        timeline={issueTimeline()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Convert to discussion" }),
    );
    expect(await screen.findByRole("dialog")).toBeVisible();
    expect(screen.getByText(/3 issue comments will be copied/)).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Discussion category" }),
    ).toHaveValue("general");
    fireEvent.click(screen.getByRole("button", { name: "Convert issue" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/issues/42/convert-to-discussion",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ categorySlug: "general" }),
        }),
      );
      expect(assignMock).toHaveBeenCalledWith("/mona/octo-app/discussions/9");
    });
  });
});
