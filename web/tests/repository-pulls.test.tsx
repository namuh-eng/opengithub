import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryPullsPage } from "@/components/RepositoryPullsPage";
import type {
  PullRequestListItem,
  PullRequestListView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryPullRequestClearFilterHref,
  repositoryPullRequestCompareHref,
  repositoryPullRequestDetailHref,
  repositoryPullRequestNoAssigneeHref,
  repositoryPullRequestNoMilestoneHref,
  repositoryPullRequestPageHref,
  repositoryPullRequestSetChecksHref,
  repositoryPullRequestSetLabelHref,
  repositoryPullRequestSetMilestoneHref,
  repositoryPullRequestSetReviewHref,
  repositoryPullRequestSetUserFilterHref,
  repositoryPullRequestSortHref,
  repositoryPullRequestStateHref,
  repositoryPullRequestsHref,
} from "@/lib/navigation";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Pull request test repository",
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
  return { ...base, ...overrides };
}

function pullRequestItem(
  overrides: Partial<PullRequestListItem> = {},
): PullRequestListItem {
  const base: PullRequestListItem = {
    id: "pull-1",
    repositoryId: "repo-1",
    repositoryOwner: "mona",
    repositoryName: "octo-app",
    number: 17,
    title: "Add signed-in dashboard feed",
    body: "Ready for review",
    state: "open",
    isDraft: false,
    author: {
      id: "user-2",
      login: "hubot",
      displayName: "Hubot",
      avatarUrl: null,
    },
    authorRole: "write",
    labels: [
      {
        id: "label-1",
        name: "review",
        color: "var(--accent)",
        description: "Needs review",
      },
    ],
    milestone: null,
    commentCount: 4,
    linkedIssues: [
      {
        number: 12,
        state: "open",
        title: "Track dashboard work",
        href: "/mona/octo-app/issues/12",
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
      totalCount: 3,
      completedCount: 3,
      failedCount: 0,
    },
    taskProgress: {
      completed: 2,
      total: 5,
    },
    headRef: "dashboard-feed",
    baseRef: "main",
    href: "/mona/octo-app/pull/17",
    checksHref: "/mona/octo-app/pull/17/checks",
    reviewsHref: "/mona/octo-app/pull/17#reviews",
    commentsHref: "/mona/octo-app/pull/17#comments",
    linkedIssuesHref: "/mona/octo-app/pull/17#linked-issues",
    createdAt: "2026-04-30T00:00:00Z",
    updatedAt: "2026-04-30T01:00:00Z",
    closedAt: null,
    mergedAt: null,
  };
  return { ...base, ...overrides };
}

function pullRequestListView(
  overrides: Partial<PullRequestListView> = {},
): PullRequestListView {
  const items = overrides.items ?? [pullRequestItem()];
  const base: PullRequestListView = {
    items,
    total: items.length,
    page: 1,
    pageSize: 30,
    openCount: items.filter((item) => item.state === "open").length,
    closedCount: items.filter((item) => item.state === "closed").length,
    mergedCount: items.filter((item) => item.state === "merged").length,
    counts: {
      open: items.filter((item) => item.state === "open").length,
      closed: items.filter((item) => item.state === "closed").length,
      merged: items.filter((item) => item.state === "merged").length,
    },
    filters: {
      query: "is:pr is:open",
      state: "open",
      author: null,
      labels: [],
      milestone: null,
      noMilestone: false,
      assignee: null,
      noAssignee: false,
      project: null,
      review: null,
      checks: null,
      sort: "updated-desc",
    },
    filterOptions: {
      users: [
        {
          id: "user-2",
          login: "hubot",
          displayName: "Hubot",
          avatarUrl: null,
        },
        {
          id: "user-3",
          login: "mona",
          displayName: "Mona",
          avatarUrl: null,
        },
      ],
      labels: [
        {
          id: "label-1",
          name: "review",
          color: "var(--accent)",
          description: "Needs review",
        },
      ],
      milestones: [
        {
          id: "milestone-1",
          title: "Review queue",
          state: "open",
        },
      ],
      projects: [
        {
          id: "projects-unavailable",
          name: "No repository projects",
          description: "Project links are not modeled for pull requests yet.",
          count: 0,
          disabledReason:
            "Project filters will activate when project links exist.",
        },
      ],
      reviewStates: [
        "none",
        "required",
        "approved",
        "changes_requested",
        "reviewed_by_me",
        "not_reviewed_by_me",
        "review_requested",
        "team_review_requested",
      ],
      checkStates: ["success", "failure", "pending", "running"],
      sortOptions: [
        "best-match",
        "updated-desc",
        "updated-asc",
        "created-desc",
        "created-asc",
        "comments-desc",
        "comments-asc",
        "reactions-desc",
        "reactions-thumbs_up-desc",
        "reactions-thumbs_down-desc",
        "reactions-laugh-desc",
        "reactions-hooray-desc",
        "reactions-confused-desc",
        "reactions-heart-desc",
        "reactions-rocket-desc",
        "reactions-eyes-desc",
      ],
    },
    viewerPermission: "owner",
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    preferences: {
      dismissedContributorBanner: false,
      dismissedContributorBannerAt: null,
    },
  };
  return { ...base, ...overrides };
}

describe("RepositoryPullsPage", () => {
  it("renders the default open pull request list with real row metadata", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView()}
        query={{ q: "is:pr is:open", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Pull requests" }),
    ).toBeVisible();
    expect(screen.getByLabelText("pull-query")).toHaveValue("is:pr is:open");
    for (const link of screen.getAllByRole("link", {
      name: "New pull request",
    })) {
      expect(link).toHaveAttribute("href", "/mona/octo-app/compare");
    }
    expect(screen.getByRole("link", { name: /Open/ })).toHaveAttribute(
      "aria-current",
      "page",
    );
    const row = screen.getByRole("article");
    expect(
      within(row).getByRole("link", { name: "Add signed-in dashboard feed" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/17");
    expect(within(row).getByText("#17")).toBeVisible();
    expect(within(row).getByText("dashboard-feed")).toBeVisible();
    expect(within(row).getByText("main")).toBeVisible();
    expect(within(row).getByText("write")).toBeVisible();
    expect(
      within(row).getByRole("link", { name: "3 passing" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/17/checks");
    expect(within(row).getByRole("link", { name: "Approved" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17#reviews",
    );
    expect(within(row).getByRole("link", { name: "1 linked" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17#linked-issues",
    );
    expect(within(row).getByText("2/5 tasks")).toBeVisible();
    expect(screen.queryByRole("link", { name: "Clear query" })).toBeNull();
  });

  it("renders empty state and concrete controls without inert links", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView({ items: [], total: 0 })}
        query={{ q: "is:pr is:open missing", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByText("No pull requests matched this query"),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Clear query" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pulls?q=is%3Apr+is%3Aopen&state=open",
    );
    for (const link of screen.getAllByRole("link", {
      name: "New pull request",
    })) {
      expect(link).toHaveAttribute("href", "/mona/octo-app/compare");
    }
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("renders URL-backed filter menus, selected chips, and reset links", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView({
          filters: {
            query:
              'is:pr state:open author:hubot label:review milestone:"Review queue" assignee:mona review:approved checks:success',
            state: "open",
            author: "hubot",
            labels: ["review"],
            milestone: "Review queue",
            noMilestone: false,
            assignee: "mona",
            noAssignee: false,
            project: null,
            review: "approved",
            checks: "success",
            sort: "comments-desc",
          },
        })}
        query={{
          q: 'is:pr state:open author:hubot label:review milestone:"Review queue" assignee:mona review:approved checks:success',
          state: "open",
          author: "hubot",
          labels: ["review"],
          milestone: "Review queue",
          noMilestone: false,
          assignee: "mona",
          review: "approved",
          checks: "success",
          sort: "comments-desc",
        }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Author/ }));
    expect(
      screen.getByRole("dialog", { name: "Pull request author filter" }),
    ).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Filter authors" }),
    ).toHaveFocus();
    expect(screen.getByRole("option", { name: /hubot/ })).toHaveAttribute(
      "href",
      expect.stringContaining("author%3Ahubot"),
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("dialog", { name: "Pull request author filter" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Labels/ }));
    expect(
      screen.getByRole("dialog", { name: "Pull request labels filter" }),
    ).toBeVisible();
    expect(screen.getByRole("option", { name: /review/ })).toHaveAttribute(
      "href",
      expect.stringContaining("label%3Areview"),
    );
    fireEvent.keyDown(document, { key: "Escape" });

    fireEvent.click(screen.getByRole("button", { name: /Assignee/ }));
    expect(screen.getByRole("option", { name: /mona/ })).toHaveAttribute(
      "href",
      expect.stringContaining("assignee%3Amona"),
    );
    expect(screen.getByRole("option", { name: /No assignee/ })).toHaveAttribute(
      "href",
      expect.stringContaining("no%3Aassignee"),
    );
    fireEvent.keyDown(document, { key: "Escape" });

    fireEvent.click(screen.getByRole("button", { name: /Milestones/ }));
    expect(
      screen.getByRole("option", { name: /Review queue/ }),
    ).toHaveAttribute(
      "href",
      expect.stringContaining("milestone%3A%22Review+queue%22"),
    );
    expect(
      screen.getByRole("option", { name: /No milestone/ }),
    ).toHaveAttribute("href", expect.stringContaining("no%3Amilestone"));
    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("dialog", { name: "Pull request milestones filter" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Projects/ }));
    expect(
      screen.getByRole("dialog", { name: "Pull request projects filter" }),
    ).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Filter projects" }),
    ).toHaveFocus();
    expect(
      screen.getByRole("option", { name: /No repository projects/ }),
    ).toHaveAttribute("aria-disabled", "true");
    fireEvent.pointerDown(document.body);
    expect(
      screen.queryByRole("dialog", { name: "Pull request projects filter" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Reviews/ }));
    expect(
      screen.getByRole("menuitemradio", { name: /No reviews/ }),
    ).toHaveFocus();
    expect(
      screen.getByRole("menuitemradio", { name: /Approved review/ }),
    ).toHaveAttribute("href", expect.stringContaining("review%3Aapproved"));
    expect(
      screen.getAllByRole("menuitemradio", {
        name: /Awaiting review from you/,
      })[0],
    ).toHaveAttribute(
      "href",
      expect.stringContaining("review%3Areview_requested"),
    );
    expect(
      screen.getByRole("menuitemradio", {
        name: /Awaiting review from you or your team/,
      }),
    ).toHaveAttribute(
      "href",
      expect.stringContaining("review%3Ateam_review_requested"),
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      screen.queryByRole("menu", { name: "Pull request reviews filter" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Checks/ }));
    expect(
      screen.getByRole("option", { name: /Checks passing/ }),
    ).toHaveAttribute("href", expect.stringContaining("checks%3Asuccess"));
    fireEvent.keyDown(document, { key: "Escape" });

    fireEvent.click(screen.getByRole("button", { name: /Sort by/ }));
    expect(
      screen.getByRole("menu", { name: "Sort pull requests" }),
    ).toBeVisible();
    expect(
      screen.getByRole("menuitemradio", { name: /Best match/ }),
    ).toHaveFocus();
    expect(
      screen.getByRole("menuitemradio", { name: /Best match/ }),
    ).toHaveAttribute("href", expect.stringContaining("sort=best-match"));
    expect(
      screen.getByRole("menuitemradio", { name: /Most commented/ }),
    ).toHaveAttribute("href", expect.stringContaining("sort=comments-desc"));
    expect(
      screen.getByRole("menuitemradio", { name: /Recently updated/ }),
    ).toHaveAttribute("aria-checked", "false");
    expect(
      screen.getByRole("menuitemradio", { name: /Most reactions/ }),
    ).toHaveAttribute("href", expect.stringContaining("sort=reactions-desc"));
    expect(
      screen.getByRole("menuitemradio", { name: /Most \+1/ }),
    ).toHaveAttribute(
      "href",
      expect.stringContaining("sort=reactions-thumbs_up-desc"),
    );
    expect(
      screen.getByRole("menuitemradio", { name: /Most rocket/ }),
    ).toHaveAttribute(
      "href",
      expect.stringContaining("sort=reactions-rocket-desc"),
    );

    expect(screen.getByRole("link", { name: "label:review" })).toHaveAttribute(
      "href",
      expect.not.stringContaining("labels=review"),
    );
    expect(screen.getByRole("link", { name: "author:hubot" })).toHaveAttribute(
      "href",
      expect.not.stringContaining("author=hubot"),
    );
    expect(
      screen.getByRole("link", { name: "milestone:Review queue" }),
    ).toHaveAttribute("href", expect.not.stringContaining("milestone="));
    expect(screen.getByRole("link", { name: "assignee:mona" })).toHaveAttribute(
      "href",
      expect.not.stringContaining("assignee=mona"),
    );
    expect(
      screen.getByRole("link", { name: "review:Approved review" }),
    ).toHaveAttribute("href", expect.not.stringContaining("review="));
    expect(
      screen.getByRole("link", { name: "checks:success" }),
    ).toHaveAttribute("href", expect.not.stringContaining("checks="));
  }, 10_000);

  it("preserves invalid advanced query text while offering real recovery actions", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView({
          items: [],
          total: 0,
          filters: {
            query: "is:pr state:closed project:roadmap",
            state: "closed",
            author: null,
            labels: [],
            milestone: null,
            noMilestone: false,
            assignee: null,
            noAssignee: false,
            project: "roadmap",
            review: null,
            checks: null,
            sort: "updated-desc",
          },
        })}
        query={{
          q: "is:pr state:closed project:roadmap",
          state: "closed",
          project: "roadmap",
        }}
        repository={repositoryOverview()}
        validationError={{
          error: {
            code: "validation_failed",
            message: "Invalid pull request filter.",
          },
          status: 422,
          details: {
            reason:
              "invalid issue filter: project filters are not available until repository project links are modeled",
          },
        }}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "project filters are not available",
    );
    expect(screen.getByLabelText("pull-query")).toHaveValue(
      "is:pr state:closed project:roadmap",
    );
    expect(
      screen.getByRole("link", { name: "Clear invalid query" }),
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/pulls?q=is%3Apr+is%3Aopen&state=open",
    );
    expect(
      screen.getByRole("link", { name: "project:roadmap" }),
    ).toHaveAttribute("href", expect.not.stringContaining("project="));
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });

  it("dismisses the contributor banner through the preferences endpoint", async () => {
    const fetchMock = vi.fn(
      async () =>
        new Response(
          JSON.stringify({
            dismissedContributorBanner: true,
            dismissedContributorBannerAt: "2026-05-01T00:00:00Z",
          }),
          { status: 200 },
        ),
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullsPage
        pulls={pullRequestListView()}
        query={{ q: "is:pr is:open", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("region", { name: "Contributor guidance" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("region", { name: "Contributor guidance" }),
      ).toBeNull(),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/pulls/preferences",
      expect.objectContaining({
        body: JSON.stringify({ dismissedContributorBanner: true }),
        method: "PATCH",
      }),
    );

    vi.unstubAllGlobals();
  });

  it("shows pull contributor banner save failures without hiding the guidance", async () => {
    const fetchMock = vi.fn(async () => new Response("nope", { status: 502 }));
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryPullsPage
        pulls={pullRequestListView()}
        query={{ q: "is:pr is:open", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));

    expect(
      await screen.findByText("This preference could not be saved. Try again."),
    ).toBeVisible();
    expect(
      screen.getByRole("region", { name: "Contributor guidance" }),
    ).toBeVisible();

    vi.unstubAllGlobals();
  });

  it("hides the contributor banner when the server preference is already dismissed", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView({
          preferences: {
            dismissedContributorBanner: true,
            dismissedContributorBannerAt: "2026-05-01T00:00:00Z",
          },
        })}
        query={{ q: "is:pr is:open", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.queryByRole("region", { name: "Contributor guidance" }),
    ).toBeNull();
  });

  it("builds pull request list navigation hrefs", () => {
    expect(repositoryPullRequestsHref("mona", "octo-app")).toBe(
      "/mona/octo-app/pulls",
    );
    expect(
      repositoryPullRequestStateHref(
        "mona",
        "octo-app",
        { q: "is:pr is:open label:review", labels: ["review"] },
        "closed",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+label%3Areview+state%3Aclosed&state=closed&labels=review",
    );
    expect(
      repositoryPullRequestPageHref(
        "mona",
        "octo-app",
        { q: "is:pr is:open", state: "open" },
        2,
      ),
    ).toBe("/mona/octo-app/pulls?q=is%3Apr+is%3Aopen&state=open&page=2");
    expect(repositoryPullRequestDetailHref("mona", "octo-app", 17)).toBe(
      "/mona/octo-app/pull/17",
    );
    expect(repositoryPullRequestCompareHref("mona", "octo-app")).toBe(
      "/mona/octo-app/compare",
    );
    expect(
      repositoryPullRequestSetLabelHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open", state: "open" },
        "needs review",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3A%22needs+review%22&state=open&labels=needs+review",
    );
    expect(
      repositoryPullRequestSetUserFilterHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open", state: "open" },
        "author",
        "hubot",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+author%3Ahubot&state=open&author=hubot",
    );
    expect(
      repositoryPullRequestSetUserFilterHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open no:assignee", state: "open", noAssignee: true },
        "assignee",
        "hubot",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+assignee%3Ahubot&state=open&assignee=hubot",
    );
    expect(
      repositoryPullRequestNoAssigneeHref("mona", "octo-app", {
        q: "is:pr state:open assignee:hubot",
        state: "open",
        assignee: "hubot",
      }),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+no%3Aassignee&state=open&noAssignee=true",
    );
    expect(
      repositoryPullRequestSetMilestoneHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open", state: "open" },
        "Review queue",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+milestone%3A%22Review+queue%22&state=open&milestone=Review+queue",
    );
    expect(
      repositoryPullRequestNoMilestoneHref("mona", "octo-app", {
        q: 'is:pr state:open milestone:"Review queue"',
        state: "open",
        milestone: "Review queue",
      }),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+no%3Amilestone&state=open&noMilestone=true",
    );
    expect(
      repositoryPullRequestSetReviewHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open", state: "open" },
        "review_requested",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+review%3Areview_requested&state=open&review=review_requested",
    );
    expect(
      repositoryPullRequestSetChecksHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open", state: "open" },
        "success",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+checks%3Asuccess&state=open&checks=success",
    );
    expect(
      repositoryPullRequestSortHref(
        "mona",
        "octo-app",
        { q: "is:pr state:open sort:updated-desc", state: "open" },
        "comments-desc",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen&state=open&sort=comments-desc",
    );
    expect(
      repositoryPullRequestSortHref(
        "mona",
        "octo-app",
        {
          q: "is:pr state:open label:review sort:comments-desc",
          labels: ["review"],
          state: "open",
        },
        "reactions-rocket-desc",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3Areview&state=open&labels=review&sort=reactions-rocket-desc",
    );
    expect(
      repositoryPullRequestClearFilterHref(
        "mona",
        "octo-app",
        {
          q: "is:pr state:open label:bug label:review",
          labels: ["bug", "review"],
        },
        "labels",
        "bug",
      ),
    ).toBe(
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3Areview&labels=review",
    );
  });

  it("keeps final guardrail controls accessible and detail anchors concrete", () => {
    render(
      <RepositoryPullsPage
        pulls={pullRequestListView({
          items: [
            pullRequestItem({
              checksHref: "/mona/octo-app/pull/17/checks",
              commentsHref: "/mona/octo-app/pull/17#comments",
              linkedIssuesHref: "/mona/octo-app/pull/17#linked-issues",
              reviewsHref: "/mona/octo-app/pull/17#reviews",
            }),
          ],
        })}
        query={{ q: "is:pr is:open", state: "open" }}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "3 passing" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17/checks",
    );
    expect(screen.getByRole("link", { name: "Approved" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17#reviews",
    );
    expect(screen.getByRole("link", { name: "1 linked" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17#linked-issues",
    );
    expect(screen.getByRole("link", { name: "4" })).toHaveAttribute(
      "href",
      "/mona/octo-app/pull/17#comments",
    );
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName();
    }
  });
});
