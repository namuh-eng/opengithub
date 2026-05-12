import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDiscussionsPage } from "@/components/RepositoryDiscussionsPage";
import type { RepositoryDiscussionsView, RepositoryOverview } from "@/lib/api";

const refreshMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: refreshMock }),
}));

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
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

function discussionsView(
  overrides: Partial<RepositoryDiscussionsView> = {},
): RepositoryDiscussionsView {
  const categories = [
    {
      id: "cat-1",
      slug: "general",
      name: "General",
      emoji: "💬",
      description: "General project conversation.",
      count: 2,
      openCount: 2,
      href: "/namuh-eng/opengithub/discussions/categories/general",
      active: false,
    },
    {
      id: "cat-2",
      slug: "ideas",
      name: "Ideas",
      emoji: "💡",
      description: "Shape product direction.",
      count: 1,
      openCount: 1,
      href: "/namuh-eng/opengithub/discussions/categories/ideas",
      active: false,
    },
  ];
  const labels = [
    {
      id: "label-1",
      name: "help-wanted",
      color: "var(--accent)",
      description: "Needs community input",
      count: 1,
    },
  ];
  const firstDiscussion = {
    id: "discussion-1",
    number: 12,
    title: "How should repository import previews handle very large manifests?",
    state: "open",
    answered: true,
    locked: false,
    pinned: true,
    category: categories[0],
    labels,
    author: {
      id: "user-2",
      login: "ashley",
      displayName: "Ashley",
      avatarUrl: null,
    },
    commentsCount: 8,
    votesCount: 14,
    viewerVoted: true,
    href: "/namuh-eng/opengithub/discussions/12",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-04T00:00:00Z",
    lastActivityAt: "2026-05-04T00:00:00Z",
  };
  const base: RepositoryDiscussionsView = {
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
      canVote: true,
      canCreate: true,
    },
    enabled: true,
    disabledReason: null,
    filters: {
      query: "is:open",
      label: null,
      state: "open",
      answered: null,
      locked: null,
      pinned: null,
      sort: "latest",
      category: null,
      page: 1,
      pageSize: 30,
    },
    categories,
    labels,
    pinned: [
      {
        discussion: firstDiscussion,
        position: 1,
        pinnedAt: "2026-05-04T00:00:00Z",
      },
    ],
    helpfulContributors: [
      {
        user: {
          id: "user-2",
          login: "ashley",
          displayName: "Ashley",
          avatarUrl: null,
        },
        commentsCount: 7,
        helpfulCount: 2,
      },
    ],
    communityLinks: [
      {
        id: "link-1",
        label: "Code of conduct",
        href: "/namuh-eng/opengithub/community/code-of-conduct",
        kind: "code_of_conduct",
      },
    ],
    items: [
      firstDiscussion,
      {
        ...firstDiscussion,
        id: "discussion-2",
        number: 13,
        title:
          "ExtremelyLongDiscussionTitleWithoutSpacesStillWrapsInsideTheEditorialListColumn",
        answered: false,
        pinned: false,
        viewerVoted: false,
        votesCount: 3,
        commentsCount: 1,
        href: "/namuh-eng/opengithub/discussions/13",
      },
    ],
    openCount: 2,
    closedCount: 1,
    total: 3,
    page: 1,
    pageSize: 30,
    hasNextPage: false,
  };
  return { ...base, ...overrides };
}

describe("RepositoryDiscussionsPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    refreshMock.mockClear();
  });

  it("renders the Editorial discussions list with active tab, pinned cards, filters, rows, and rails", () => {
    const { container } = render(
      <RepositoryDiscussionsPage
        discussions={discussionsView()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Discussions" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("heading", { name: "Discussions" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "New discussion" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/new/choose");
    expect(screen.getByLabelText("discussion-query")).toHaveValue("is:open");

    const pinned = screen.getByRole("region", { name: "Pinned discussions" });
    expect(
      within(pinned).getByRole("link", { name: /repository import previews/i }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/12");

    const list = screen.getByRole("list", { name: "Repository discussions" });
    expect(
      within(list).getByRole("link", { name: /very large manifests/i }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/12");
    expect(within(list).getByText("Answered")).toBeVisible();
    expect(within(list).getAllByText("help-wanted")).toHaveLength(2);

    expect(screen.getByRole("link", { name: /General2/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/general?discussions_q=is%3Aopen",
    );
    expect(screen.getByText("Most helpful")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Code of conduct" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/community/code-of-conduct",
    );
    expect(container.querySelector('[href="#"]')).toBeNull();
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.innerHTML).not.toContain("@primer/");
  });

  it("optimistically upvotes and unvotes discussions with server reconciliation", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            discussionId: "discussion-2",
            discussionNumber: 13,
            viewerVoted: true,
            votesCount: 4,
          }),
          { status: 200 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            discussionId: "discussion-2",
            discussionNumber: 13,
            viewerVoted: false,
            votesCount: 3,
          }),
          { status: 200 },
        ),
      );

    render(
      <RepositoryDiscussionsPage
        discussions={discussionsView()}
        repository={repositoryOverview()}
      />,
    );

    const upvote = screen.getByRole("button", {
      name: "Upvote discussion 13",
    });
    fireEvent.click(upvote);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/13/vote",
        { method: "PUT" },
      ),
    );
    await waitFor(() => expect(upvote).toHaveAttribute("aria-pressed", "true"));
    expect(upvote).toHaveTextContent("4");
    expect(refreshMock).toHaveBeenCalledTimes(1);

    fireEvent.click(upvote);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/namuh-eng/opengithub/discussions/13/vote",
        { method: "DELETE" },
      ),
    );
    await waitFor(() =>
      expect(upvote).toHaveAttribute("aria-pressed", "false"),
    );
    expect(upvote).toHaveTextContent("3");
    expect(refreshMock).toHaveBeenCalledTimes(2);
  });

  it("rolls back failed discussion votes and shows signed-out affordances", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          error: {
            code: "validation_failed",
            message: "Repository discussions are disabled.",
          },
          status: 422,
        }),
        { status: 422 },
      ),
    );

    const { rerender } = render(
      <RepositoryDiscussionsPage
        discussions={discussionsView()}
        repository={repositoryOverview()}
      />,
    );

    const upvote = screen.getByRole("button", {
      name: "Upvote discussion 13",
    });
    fireEvent.click(upvote);
    await waitFor(() =>
      expect(upvote).toHaveAttribute("aria-pressed", "false"),
    );
    expect(upvote).toHaveTextContent("3");
    expect(
      screen.getByText("Repository discussions are disabled."),
    ).toBeVisible();

    rerender(
      <RepositoryDiscussionsPage
        discussions={discussionsView({
          viewer: {
            authenticated: false,
            permission: null,
            canRead: true,
            canVote: false,
            canCreate: false,
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getAllByRole("link", { name: /Sign in to upvote discussion/i })[0],
    ).toHaveAttribute("href", "/login?next=/namuh-eng/opengithub/discussions");
  });

  it("composes filter links and empty category CTAs without dead controls", () => {
    const categoryView = discussionsView({
      categories: discussionsView().categories.map((category) => ({
        ...category,
        active: category.slug === "ideas",
      })),
      filters: {
        ...discussionsView().filters,
        category: "ideas",
        label: "help-wanted",
        answered: false,
        sort: "top",
      },
      items: [],
      pinned: [],
      total: 0,
      hasNextPage: false,
    });

    const { container } = render(
      <RepositoryDiscussionsPage
        discussions={categoryView}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "💡 Ideas" })).toBeVisible();
    expect(screen.getByText("category:ideas")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "View all discussions" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions?discussions_q=is%3Aopen&label=help-wanted&answered=false&sort=top",
    );
    expect(
      screen.getByRole("link", { name: /Ideas1active category/ }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByText("No Ideas discussions match this view."),
    ).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "New discussion" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/new?category=ideas",
    );
    expect(screen.getByRole("link", { name: "Any label" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/ideas?discussions_q=is%3Aopen&answered=false&sort=top",
    );
    expect(
      screen.getByRole("link", { name: "Most commented" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/ideas?discussions_q=is%3Aopen&label=help-wanted&answered=false&sort=most_commented",
    );
    expect(screen.getByRole("link", { name: "Clear" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/ideas",
    );
    expect(container.querySelector("button:not([type])")).toBeNull();
  });

  it("renders the Polls category with poll row metadata and a concrete poll composer CTA", () => {
    const base = discussionsView();
    const pollCategory = {
      id: "cat-polls",
      slug: "polls",
      name: "Polls",
      emoji: "📊",
      description: "Vote on repository decisions.",
      count: 1,
      openCount: 1,
      href: "/namuh-eng/opengithub/discussions/categories/polls",
      active: true,
    };
    const pollView = discussionsView({
      categories: [...base.categories, pollCategory],
      filters: {
        ...base.filters,
        category: "polls",
      },
      pinned: [],
      items: [
        {
          ...base.items[0],
          id: "discussion-poll-1",
          number: 21,
          title: "Which branch protection policy should ship first?",
          category: pollCategory,
          categoryQualifier: "Poll",
          commentsCount: 4,
          pollSummary: {
            id: "poll-1",
            question: "Which branch protection policy should ship first?",
            allowsMultiple: false,
            optionCount: 3,
            totalVotes: 0,
          },
          viewerCanVote: false,
          resultsVisible: true,
          viewerVoteOptionIds: [],
          pollUnavailableReasons: [
            "Poll voting is not available for this discussion state.",
          ],
        },
      ],
      openCount: 1,
      closedCount: 0,
      total: 1,
    });

    const { container, rerender } = render(
      <RepositoryDiscussionsPage
        discussions={pollView}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "📊 Polls" })).toBeVisible();
    expect(screen.getByText("category:polls")).toBeVisible();
    expect(
      screen.getAllByText("Which branch protection policy should ship first?")
        .length,
    ).toBeGreaterThan(0);
    expect(container.textContent).toContain("3 options");
    expect(container.textContent).toContain("single choice");
    expect(container.textContent).toContain(
      "Poll voting is not available for this discussion state.",
    );
    expect(screen.getByText("Poll")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Start poll" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/new?category=polls",
    );
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.querySelector('[href="#"]')).toBeNull();

    rerender(
      <RepositoryDiscussionsPage
        discussions={discussionsView({
          categories: [...base.categories, pollCategory],
          filters: { ...base.filters, category: "polls" },
          items: [],
          pinned: [],
          total: 0,
        })}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getByText("No poll discussions match this view."),
    ).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Start poll" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/new?category=polls",
    );
  });

  it("renders disabled and mobile-safe empty states", () => {
    render(
      <div style={{ width: 360 }}>
        <RepositoryDiscussionsPage
          discussions={discussionsView({
            enabled: false,
            disabledReason:
              "Repository discussions are disabled by organization policy.",
            items: [],
            pinned: [],
            total: 0,
          })}
          repository={repositoryOverview()}
        />
      </div>,
    );

    expect(screen.getByText("Discussions disabled")).toBeVisible();
    expect(
      screen.getByText(
        "Repository discussions are disabled by organization policy.",
      ),
    ).toBeVisible();
    expect(screen.getByText("No discussions match this view.")).toBeVisible();
  });
});
