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

const pushMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock }),
}));

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
      allowsVoteChanges: true,
      totalVotes: 7,
      viewerCanVote: true,
      resultsVisible: true,
      viewerVoteOptionIds: [],
      unavailableReasons: [],
      options: [
        {
          id: "option-1",
          position: 1,
          label: "Cursor batches",
          votesCount: 3,
          percentage: 43,
        },
        {
          id: "option-2",
          position: 2,
          label: "Background job",
          votesCount: 4,
          percentage: 57,
        },
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
    pushMock.mockReset();
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
    expect(within(sidebar).getByRole("link", { name: /api/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions?label=api",
    );
  });

  it("uses the shared picker to update discussion labels", async () => {
    const updated = discussionDetail({
      labels: [
        ...discussionDetail().labels,
        {
          id: "label-2",
          name: "imports",
          color: "var(--accent)",
          description: "Import workflows",
          count: 1,
        },
      ],
    });
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue(updated),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    expect(
      screen.getByRole("dialog", { name: "Discussion label picker" }),
    ).toBeVisible();
    fireEvent.change(screen.getByLabelText("Search labels"), {
      target: { value: "imports" },
    });
    fireEvent.click(screen.getByLabelText(/imports/));
    fireEvent.click(screen.getByRole("button", { name: "Save labels" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/metadata",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ labelIds: ["label-1", "label-2"] }),
        }),
      ),
    );
    expect(
      await screen.findByText("Discussion metadata updated."),
    ).toBeVisible();
    expect(screen.getAllByText("imports").length).toBeGreaterThan(0);
  });

  it("submits poll votes and refreshes result bars", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue({
        discussionId: "discussion-1",
        discussionNumber: 42,
        changed: true,
        poll: {
          ...discussionDetail().poll,
          totalVotes: 8,
          viewerVoteOptionIds: ["option-2"],
          options: [
            {
              id: "option-1",
              position: 1,
              label: "Cursor batches",
              votesCount: 3,
              percentage: 38,
            },
            {
              id: "option-2",
              position: 2,
              label: "Background job",
              votesCount: 5,
              percentage: 62,
            },
          ],
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("radio", { name: /Background job/ }));
    fireEvent.click(screen.getByRole("button", { name: "Vote" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/poll/vote",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({ optionIds: ["option-2"] }),
        }),
      ),
    );
    expect(await screen.findByText("Poll vote updated.")).toBeVisible();
    expect(screen.getByText("8 total votes")).toBeVisible();
    expect(screen.getByText("62%")).toBeVisible();
  });

  it("supports multiple-choice poll vote updates and server errors", async () => {
    const basePoll = discussionDetail().poll;
    if (!basePoll) throw new Error("expected poll fixture");
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: vi.fn().mockResolvedValue({
        error: { message: "this poll does not allow vote changes" },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail({
          poll: {
            ...basePoll,
            allowsMultiple: true,
            viewerVoteOptionIds: ["option-1"],
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("checkbox", { name: /Background job/ }));
    fireEvent.click(screen.getByRole("button", { name: "Update vote" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/poll/vote",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify({ optionIds: ["option-1", "option-2"] }),
        }),
      ),
    );
    expect(
      await screen.findByText("this poll does not allow vote changes"),
    ).toBeVisible();
  });

  it("shows a concrete sign-in prompt instead of dead poll controls", () => {
    const basePoll = discussionDetail().poll;
    if (!basePoll) throw new Error("expected poll fixture");
    const { container } = render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail({
          poll: {
            ...basePoll,
            viewerCanVote: false,
            resultsVisible: false,
            unavailableReasons: ["Sign in to vote in this poll."],
          },
          viewer: {
            ...discussionDetail().viewer,
            authenticated: false,
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", { name: "Sign in to vote" }),
    ).toHaveAttribute("href", "/login");
    expect(screen.queryByRole("button", { name: "Vote" })).toBeNull();
    expect(container.querySelector('[href="#"]')).toBeNull();
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
    const labelsSection = screen
      .getByRole("heading", { name: "Labels" })
      .closest("section");
    expect(labelsSection).not.toBeNull();
    fireEvent.click(
      within(labelsSection as HTMLElement).getByRole("button", {
        name: "Edit",
      }),
    );
    expect(
      within(labelsSection as HTMLElement).getByText("imports"),
    ).toBeVisible();
  });

  it("submits pin, lock, close, and category moderation payloads", async () => {
    const normalDetail = discussionDetail({ poll: null });
    const nextDetail = discussionDetail({
      poll: null,
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
        detail={normalDetail}
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

  it("loads transfer targets and submits transfer and delete confirmations", async () => {
    const normalDetail = discussionDetail({ poll: null });
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: vi.fn().mockResolvedValue({
          currentRepository: discussionDetail().repository,
          discussionNumber: 42,
          targets: [
            {
              repositoryId: "repo-2",
              owner: "namuh-eng",
              name: "runtime",
              visibility: "private",
              href: "/namuh-eng/runtime",
              discussionsHref: "/namuh-eng/runtime/discussions",
              categoryOptions: [
                {
                  ...discussionDetail().sidebar.categoryOptions[0],
                  id: "category-target",
                  slug: "announcements",
                  name: "Announcements",
                },
              ],
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: vi.fn().mockResolvedValue({
          discussionId: "discussion-1",
          sourceHref: "/namuh-eng/opengithub/discussions",
          destinationHref: "/namuh-eng/runtime/discussions/7",
          destinationOwner: "namuh-eng",
          destinationRepo: "runtime",
          destinationNumber: 7,
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: vi.fn().mockResolvedValue({
          discussionId: "discussion-1",
          deleted: true,
          tombstoneId: "tombstone-1",
          discussionsHref: "/namuh-eng/opengithub/discussions",
        }),
      });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={normalDetail}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Transfer discussion" }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/transfer-targets",
      ),
    );
    fireEvent.change(
      await screen.findByRole("combobox", {
        name: "Transfer destination repository",
      }),
      { target: { value: "repo-2" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "Transfer" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/transfer",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            repositoryId: "repo-2",
            categorySlug: "announcements",
          }),
        }),
      ),
    );
    expect(pushMock).toHaveBeenCalledWith("/namuh-eng/runtime/discussions/7");

    fireEvent.click(screen.getByRole("button", { name: "Delete discussion" }));
    fireEvent.change(screen.getByLabelText("Reason"), {
      target: { value: "Spam cleanup" },
    });
    fireEvent.change(screen.getByLabelText("Type delete discussion 42"), {
      target: { value: "delete discussion 42" },
    });
    fireEvent.click(
      within(
        screen.getByRole("dialog", { name: "Delete this discussion" }),
      ).getByRole("button", { name: "Delete discussion" }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42/delete",
        expect.objectContaining({
          method: "DELETE",
          body: JSON.stringify({
            confirmation: "delete discussion 42",
            reason: "Spam cleanup",
          }),
        }),
      ),
    );
    expect(pushMock).toHaveBeenCalledWith("/namuh-eng/opengithub/discussions");
  });

  it("keeps poll discussions in poll-compatible categories", async () => {
    const pollCategory = {
      id: "cat-polls",
      slug: "polls",
      name: "Polls",
      emoji: "%",
      description: "Team polls.",
      count: 2,
      openCount: 2,
      href: "/namuh-eng/opengithub/discussions/categories/polls",
      active: true,
      acceptsAnswers: false,
      isPoll: true,
      formHref: "/namuh-eng/opengithub/discussions/new?category=polls",
    };
    const normalCategory = {
      ...discussionDetail().sidebar.categoryOptions[0],
      id: "cat-general",
      slug: "general",
      name: "General",
      isPoll: false,
      formHref: "/namuh-eng/opengithub/discussions/new?category=general",
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue({
        currentRepository: discussionDetail().repository,
        discussionNumber: 42,
        targets: [
          {
            repositoryId: "repo-2",
            owner: "namuh-eng",
            name: "runtime",
            visibility: "private",
            href: "/namuh-eng/runtime",
            discussionsHref: "/namuh-eng/runtime/discussions",
            categoryOptions: [normalCategory, pollCategory],
          },
        ],
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionDetailPage
        detail={discussionDetail({
          category: pollCategory,
          sidebar: {
            ...discussionDetail().sidebar,
            category: pollCategory,
            categoryOptions: [normalCategory, pollCategory],
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    const changeCategory = screen.getByRole("combobox", {
      name: "Change discussion category",
    });
    expect(within(changeCategory).getByText("% Polls")).toBeVisible();
    expect(within(changeCategory).queryByText("General")).toBeNull();
    expect(
      screen.getAllByText("Poll discussions must stay in poll categories.")
        .length,
    ).toBeGreaterThan(0);

    fireEvent.click(
      screen.getByRole("button", { name: "Transfer discussion" }),
    );
    fireEvent.change(
      await screen.findByRole("combobox", {
        name: "Transfer destination repository",
      }),
      { target: { value: "repo-2" } },
    );
    const transferCategory = screen.getByRole("combobox", {
      name: "Transfer destination category",
    });
    expect(within(transferCategory).getByText("% Polls")).toBeVisible();
    expect(within(transferCategory).queryByText("General")).toBeNull();
  });
});
