import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDiscussionDetailPage } from "@/components/RepositoryDiscussionDetailPage";
import type {
  RepositoryDiscussionDetailView,
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
    visibility: "public",
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
      forksCount: 1,
      releasesCount: 0,
      deploymentsCount: 2,
      contributorsCount: 3,
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

function discussionDetail(
  overrides: Partial<RepositoryDiscussionDetailView> = {},
): RepositoryDiscussionDetailView {
  const author = {
    id: "user-2",
    login: "ashley",
    displayName: "Ashley",
    avatarUrl: null,
  };
  const maintainer = {
    id: "user-3",
    login: "mira",
    displayName: "Mira",
    avatarUrl: null,
  };
  const category = {
    id: "cat-1",
    slug: "q-a",
    name: "Q&A",
    emoji: "?",
    description: "Questions with accepted answers.",
    count: 4,
    openCount: 3,
    href: "/namuh-eng/opengithub/discussions/categories/q-a",
    active: true,
  };
  const labels = [
    {
      id: "label-1",
      name: "api",
      color: "var(--accent)",
      description: "API design",
      count: 2,
    },
  ];
  const comment = {
    kind: "comment" as const,
    id: "comment-1",
    author: maintainer,
    body: {
      markdown: "Use cursor pagination.",
      html: '<div class="markdown-body"><p>Use <strong>cursor</strong> pagination.</p></div>',
    },
    reactions: [{ content: "+1", count: 3, viewerReacted: true }],
    replies: [
      {
        id: "reply-1",
        author,
        body: {
          markdown: "That works for import previews.",
          html: "<p>That works for import previews.</p>",
        },
        reactions: [{ content: "heart", count: 1, viewerReacted: false }],
        href: "#reply-1",
        edited: false,
        deleted: false,
        deletedReason: null,
        createdAt: "2026-05-04T01:00:00Z",
        updatedAt: "2026-05-04T01:00:00Z",
      },
    ],
    answer: true,
    href: "#comment-1",
    edited: true,
    deleted: false,
    deletedReason: null,
    createdAt: "2026-05-04T00:00:00Z",
    updatedAt: "2026-05-04T00:20:00Z",
  };
  const event = {
    kind: "event" as const,
    id: "event-1",
    eventType: "answer_marked",
    actor: maintainer,
    payload: { commentId: "comment-1" },
    createdAt: "2026-05-04T00:30:00Z",
  };

  return {
    repository: {
      id: "repo-1",
      owner: "namuh-eng",
      name: "opengithub",
      visibility: "public",
      isArchived: false,
      href: "/namuh-eng/opengithub",
      discussionsHref: "/namuh-eng/opengithub/discussions",
    },
    viewer: {
      authenticated: true,
      permission: "write",
      canRead: true,
      canComment: true,
      canReact: true,
      canSubscribe: true,
      canMarkAnswer: true,
      canModerate: true,
      viewerVoted: true,
    },
    enabled: true,
    disabledReason: null,
    discussion: {
      id: "discussion-1",
      number: 42,
      title:
        "How should import previews handle extremely large dependency manifests?",
      state: "open",
      answered: true,
      locked: false,
      commentsCount: 2,
      votesCount: 12,
      href: "/namuh-eng/opengithub/discussions/42",
      createdAt: "2026-05-03T00:00:00Z",
      updatedAt: "2026-05-04T00:30:00Z",
      lastActivityAt: "2026-05-04T00:30:00Z",
    },
    author,
    category,
    labels,
    body: {
      markdown: "Can we stream the manifest parse?",
      html: '<div class="markdown-body"><p>Can we stream the <code>manifest</code> parse?</p><script>alert("x")</script></div>',
    },
    formAnswers: [
      {
        fieldId: "area",
        fieldLabel: "Area",
        value: "Repository import",
      },
    ],
    poll: {
      id: "poll-1",
      question: "Which strategy should ship first?",
      allowsMultiple: false,
      options: [
        { id: "option-1", position: 1, label: "Cursor batches" },
        { id: "option-2", position: 2, label: "Background job" },
      ],
    },
    answer: {
      commentId: "comment-1",
      markedBy: maintainer,
      markedAt: "2026-05-04T00:30:00Z",
      href: "#comment-1",
    },
    reactions: [{ content: "+1", count: 5, viewerReacted: false }],
    subscription: {
      state: "subscribed",
      reason: "manual",
      subscribed: true,
      canChange: false,
    },
    moderation: {
      globalPin: null,
      categoryPin: null,
      lockAllowsReactions: true,
      closedReason: null,
    },
    sidebar: {
      category,
      labels,
      categoryOptions: [
        {
          ...category,
          acceptsAnswers: true,
          isPoll: false,
          formHref: "/namuh-eng/opengithub/discussions/new?category=q-a",
        },
        {
          id: "cat-2",
          slug: "ideas",
          name: "Ideas",
          emoji: "!",
          description: "Product ideas.",
          acceptsAnswers: false,
          isPoll: false,
          count: 1,
          openCount: 1,
          href: "/namuh-eng/opengithub/discussions/categories/ideas",
          formHref: "/namuh-eng/opengithub/discussions/new?category=ideas",
        },
      ],
      labelOptions: [
        ...labels,
        {
          id: "label-2",
          name: "imports",
          color: "var(--accent)",
          description: "Import workflows",
          count: 1,
        },
      ],
      participants: [author, maintainer],
      events: [event],
    },
    timeline: [comment, event],
    sort: "oldest",
    page: 1,
    pageSize: 30,
    totalComments: 2,
    hasNextPage: false,
    ...overrides,
  };
}

describe("RepositoryDiscussionDetailPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders detail metadata, answer summary, timeline, replies, and sidebar", () => {
    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: /How should import previews handle extremely large dependency manifests/,
      }),
    ).toBeVisible();
    expect(screen.getAllByText("Answered").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "View full answer" }),
    ).toHaveAttribute("href", "#comment-1");
    expect(screen.getByText(/Can we stream the/)).toBeVisible();
    expect(screen.getByText(/Use/)).toBeVisible();
    expect(screen.getByText("cursor")).toBeVisible();
    expect(screen.getByText("That works for import previews.")).toBeVisible();
    expect(screen.getByText("Which strategy should ship first?")).toBeVisible();
    expect(screen.getByText("Repository import")).toBeVisible();

    const sidebar = screen.getByRole("complementary");
    expect(within(sidebar).getByText("Subscribed")).toBeVisible();
    expect(within(sidebar).getByText("Q&A")).toBeVisible();
    expect(within(sidebar).getAllByText("api").length).toBeGreaterThan(0);
  });

  it("keeps sort and permalink anchors concrete and composer controls disabled for Phase 2", () => {
    const { container } = render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Newest" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/42?sort=newest",
    );
    expect(screen.getByRole("link", { name: "Top" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/42?sort=top",
    );
    expect(
      screen.getAllByRole("link", { name: "Permalink" })[0],
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/42");
    expect(screen.getByRole("button", { name: "Comment" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Attach files" })).toBeDisabled();
    expect(container.querySelector('[href="#"]')).toBeNull();
  });

  it("does not introduce GitHub visual values or inert inline handlers", () => {
    const { container } = render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|#82071e|#f6f8fa|#1f2328|#d0d7de|#59636e|Octicon|@primer\//i,
    );
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
  });

  it("renders maintainer answer, state, category, and label controls", () => {
    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("button", { name: "Unmark answer" })).toBeVisible();
    expect(screen.getByRole("button", { name: "resolved" })).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Change discussion category" }),
    ).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Moderation category" }),
    ).toBeVisible();
    expect(screen.getByLabelText("imports")).toBeVisible();
  });

  it("submits pin, lock, close, and category moderation payloads", async () => {
    const nextDetail = discussionDetail({
      moderation: {
        globalPin: {
          target: "global",
          categorySlug: null,
          customTitle: "Pinned import guidance",
          customBody: "Read this before changing import previews.",
          position: 1,
        },
        categoryPin: null,
        lockAllowsReactions: false,
        closedReason: "resolved",
      },
    });
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue(nextDetail),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Pin discussion" }));
    fireEvent.change(screen.getByLabelText("Custom title"), {
      target: { value: "Pinned import guidance" },
    });
    fireEvent.change(screen.getByLabelText("Pinned note"), {
      target: { value: "Read this before changing import previews." },
    });
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "Pin discussion" })).getByRole(
        "button",
        { name: "Pin discussion" },
      ),
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/pin",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({
            target: "global",
            title: "Pinned import guidance",
            body: "Read this before changing import previews.",
          }),
        }),
      ),
    );

    fireEvent.click(screen.getByRole("button", { name: "Lock conversation" }));
    fireEvent.click(screen.getByLabelText("Allow reactions while locked"));
    fireEvent.click(screen.getByRole("button", { name: "Lock" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/lock",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({ allowReactions: false }),
        }),
      ),
    );

    fireEvent.click(screen.getByRole("button", { name: "resolved" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/state",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({ state: "closed", reason: "resolved" }),
        }),
      ),
    );

    fireEvent.change(
      screen.getByRole("combobox", { name: "Moderation category" }),
      {
        target: { value: "ideas" },
      },
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/category",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ categorySlug: "ideas" }),
        }),
      ),
    );
  });

  it("shows server moderation errors and reader unavailable state", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: vi.fn().mockResolvedValue({
        error: { message: "at most four global discussion pins are allowed" },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Pin discussion" }));
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "Pin discussion" })).getByRole(
        "button",
        { name: "Pin discussion" },
      ),
    );
    expect(
      await screen.findByText(
        "at most four global discussion pins are allowed",
      ),
    ).toBeVisible();

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail({
          viewer: {
            ...discussionDetail().viewer,
            permission: "read",
            canMarkAnswer: false,
            canModerate: false,
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByText(/Triage, write, and admin members can moderate/),
    ).toBeVisible();
  });
});
