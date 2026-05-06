import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GlobalPullsPage } from "@/components/GlobalPullsPage";
import type { GlobalPullRequestListView, PullRequestListItem } from "@/lib/api";

function pull(
  overrides: Partial<PullRequestListItem> = {},
): PullRequestListItem {
  return {
    id: "pull-1",
    repositoryId: "repo-1",
    repositoryOwner: "mona",
    repositoryName: "octo-app",
    number: 42,
    title: "Refine review queue",
    body: "Make review queues work across repositories.",
    state: "open",
    isDraft: false,
    author: {
      id: "user-1",
      login: "mona",
      displayName: "Mona",
      avatarUrl: null,
    },
    authorRole: "owner",
    labels: [
      {
        id: "label-1",
        name: "review",
        color: "var(--accent)",
        description: "Needs review",
      },
    ],
    milestone: {
      id: "milestone-1",
      title: "Review queue",
      state: "open",
    },
    commentCount: 3,
    linkedIssues: [],
    review: {
      state: "required",
      required: true,
      requestedReviewers: [
        {
          id: "user-2",
          login: "ashley",
          displayName: "Ashley",
          avatarUrl: null,
        },
      ],
      reviewerCount: 1,
    },
    checks: {
      status: "completed",
      conclusion: "success",
      totalCount: 2,
      completedCount: 2,
      failedCount: 0,
    },
    taskProgress: { completed: 1, total: 2 },
    headRef: "feature/global-pulls",
    baseRef: "main",
    href: "/mona/octo-app/pull/42",
    checksHref: "/mona/octo-app/pull/42/checks",
    reviewsHref: "/mona/octo-app/pull/42#reviews",
    commentsHref: "/mona/octo-app/pull/42#comments",
    linkedIssuesHref: "/mona/octo-app/pull/42#linked-issues",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-06T00:00:00Z",
    closedAt: null,
    mergedAt: null,
    ...overrides,
  };
}

function view(
  overrides: Partial<GlobalPullRequestListView> = {},
): GlobalPullRequestListView {
  const base: GlobalPullRequestListView = {
    items: [pull()],
    total: 1,
    page: 1,
    pageSize: 30,
    counts: {
      created: 4,
      assigned: 2,
      mentioned: 1,
      review_requests: 3,
    },
    filters: {
      scope: "review_requests",
      query: "is:pr is:open",
      state: "open",
      repository: null,
      labels: [],
      milestone: null,
      sort: "updated-desc",
    },
    filterOptions: {
      repositories: [
        {
          id: "repo-1",
          ownerLogin: "mona",
          name: "octo-app",
          fullName: "mona/octo-app",
          count: 1,
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
      sortOptions: ["updated-desc", "comments-desc", "created-desc"],
    },
  };
  return { ...base, ...overrides };
}

describe("GlobalPullsPage", () => {
  it("renders signed-in global pull request tabs, filters, and concrete rows", () => {
    render(<GlobalPullsPage pulls={view()} />);

    expect(
      screen.getByRole("heading", { name: "Pull requests" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Review requests3/ }),
    ).toHaveAttribute("href", expect.stringContaining("scope=review_requests"));
    expect(
      screen.getByRole("link", { name: "Refine review queue" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/42");
    expect(screen.getByRole("link", { name: "mona/octo-app" })).toHaveAttribute(
      "href",
      "/mona/octo-app",
    );

    const form = screen.getByRole("button", { name: "Filter" }).closest("form");
    expect(form).not.toBeNull();
    expect(
      within(form as HTMLFormElement).getByLabelText("Repository"),
    ).toHaveValue("");
    expect(within(form as HTMLFormElement).getByLabelText("Label")).toHaveValue(
      "",
    );
    expect(
      within(form as HTMLFormElement).getByLabelText("Milestone"),
    ).toHaveValue("");
    expect(within(form as HTMLFormElement).getByLabelText("Sort")).toHaveValue(
      "updated-desc",
    );
  });

  it("shows a working sign-in path when the API rejects the session", () => {
    render(
      <GlobalPullsPage
        pulls={{
          error: {
            code: "unauthorized",
            message: "Authentication required.",
          },
          status: 401,
        }}
      />,
    );

    expect(
      screen.getByRole("heading", { name: /signed-in session/i }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Sign in" })).toHaveAttribute(
      "href",
      "/login?next=/pulls",
    );
  });
});
