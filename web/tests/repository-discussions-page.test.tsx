import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryDiscussionsPage } from "@/components/RepositoryDiscussionsPage";
import type { RepositoryDiscussionsView, RepositoryOverview } from "@/lib/api";

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
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/new");
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
      "/namuh-eng/opengithub/discussions/categories/general?q=is%3Aopen",
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
      "/namuh-eng/opengithub/discussions?q=is%3Aopen&answered=false&sort=top",
    );
    expect(
      screen.getByRole("link", { name: "Most commented" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions?q=is%3Aopen&label=help-wanted&answered=false&sort=most_commented",
    );
    expect(container.querySelector("button:not([type])")).toBeNull();
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
