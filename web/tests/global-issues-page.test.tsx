import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GlobalIssuesPage } from "@/components/GlobalIssuesPage";
import type { GlobalIssueListView, IssueListItem } from "@/lib/api";

function issue(overrides: Partial<IssueListItem> = {}): IssueListItem {
  return {
    id: "issue-1",
    repositoryId: "repo-1",
    repositoryOwner: "mona",
    repositoryName: "octo-app",
    number: 18,
    title: "Polish issue queue",
    body: "Make global issue triage work across repositories.",
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
        name: "triage",
        color: "var(--accent)",
        description: "Needs triage",
      },
    ],
    milestone: {
      id: "milestone-1",
      title: "Work queue",
      state: "open",
    },
    assignees: [
      {
        id: "user-2",
        login: "ashley",
        displayName: "Ashley",
        avatarUrl: null,
      },
    ],
    commentCount: 5,
    linkedPullRequest: null,
    href: "/mona/octo-app/issues/18",
    locked: false,
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-06T00:00:00Z",
    closedAt: null,
    ...overrides,
  };
}

function view(
  overrides: Partial<GlobalIssueListView> = {},
): GlobalIssueListView {
  const base: GlobalIssueListView = {
    items: [issue()],
    total: 1,
    page: 1,
    pageSize: 30,
    counts: {
      created: 4,
      assigned: 2,
      mentioned: 1,
    },
    filters: {
      scope: "assigned",
      query: "is:issue state:open",
      state: "open",
      repository: null,
      labels: [],
      milestone: null,
      project: null,
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
          name: "triage",
          color: "var(--accent)",
          description: "Needs triage",
        },
      ],
      milestones: [
        {
          id: "milestone-1",
          title: "Work queue",
          state: "open",
        },
      ],
      projects: [
        {
          id: "project-1",
          name: "Roadmap",
          description: "Planning project",
          count: 1,
          disabledReason: null,
        },
      ],
      sortOptions: ["updated-desc", "comments-desc", "created-desc"],
    },
  };
  return { ...base, ...overrides };
}

describe("GlobalIssuesPage", () => {
  it("renders signed-in global issue tabs, filters, and concrete rows", () => {
    render(<GlobalIssuesPage issues={view()} />);

    expect(screen.getByRole("heading", { name: "Issues" })).toBeVisible();
    expect(screen.getByRole("link", { name: /Assigned2/ })).toHaveAttribute(
      "href",
      expect.stringContaining("scope=assigned"),
    );
    expect(
      screen.getByRole("link", { name: "Polish issue queue" }),
    ).toHaveAttribute("href", "/mona/octo-app/issues/18");
    expect(screen.getByRole("link", { name: "mona/octo-app" })).toHaveAttribute(
      "href",
      "/mona/octo-app",
    );
    expect(screen.getByText("@ashley")).toBeVisible();

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
    expect(
      within(form as HTMLFormElement).getByLabelText("Project"),
    ).toHaveValue("");
    expect(within(form as HTMLFormElement).getByLabelText("Sort")).toHaveValue(
      "updated-desc",
    );
  });

  it("shows a working sign-in path when the API rejects the session", () => {
    render(
      <GlobalIssuesPage
        issues={{
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
      "/login?next=/issues",
    );
  });
});
