import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryMilestoneDetailPage } from "@/components/RepositoryMilestoneDetailPage";
import type { RepositoryMilestoneDetail, RepositoryOverview } from "@/lib/api";

const refreshMock = vi.fn();
const pushMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock, refresh: refreshMock }),
}));

beforeEach(() => {
  refreshMock.mockReset();
  pushMock.mockReset();
  vi.stubGlobal(
    "confirm",
    vi.fn(() => true),
  );
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => ({
      ok: true,
      json: async () => milestoneDetail(),
    })),
  );
});

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Planning repo",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "write",
    branchCount: 2,
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
      forksCount: 0,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 0,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
}

function milestoneDetail(
  overrides: Partial<RepositoryMilestoneDetail> = {},
): RepositoryMilestoneDetail {
  return {
    id: "milestone-1",
    title: "Launch readiness",
    description: "Track blockers before launch.",
    descriptionHtml: "<p>Track blockers before launch.</p>",
    state: "open",
    dueOn: "2026-05-20T00:00:00Z",
    closedAt: null,
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-06T00:00:00Z",
    progress: {
      openCount: 1,
      closedCount: 1,
      totalCount: 2,
      percentComplete: 50,
    },
    items: [
      {
        id: "issue-1",
        number: 42,
        title: "Stabilize importer",
        state: "open",
        isPullRequest: false,
        href: "/mona/octo-app/issues/42",
        commentCount: 3,
        labelNames: ["bug"],
        assigneeLogins: ["mona"],
        updatedAt: "2026-05-06T00:00:00Z",
      },
      {
        id: "issue-2",
        number: 43,
        title: "Merge docs",
        state: "closed",
        isPullRequest: true,
        href: "/mona/octo-app/pull/43",
        commentCount: 1,
        labelNames: ["docs"],
        assigneeLogins: [],
        updatedAt: "2026-05-05T00:00:00Z",
      },
    ],
    openIssuesHref: "/mona/octo-app/issues?q=milestone%3A%22Launch%22",
    closedIssuesHref: "/mona/octo-app/issues?q=milestone%3A%22Launch%22",
    href: "/mona/octo-app/milestones/milestone-1",
    viewer: {
      permission: "write",
      canEditMilestones: true,
    },
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      isArchived: false,
    },
    ...overrides,
  };
}

function renderDetail(
  milestone: RepositoryMilestoneDetail = milestoneDetail(),
  query: { state?: string | null } = { state: "open" },
) {
  return render(
    <RepositoryMilestoneDetailPage
      milestone={milestone}
      query={query}
      repository={repositoryOverview()}
    />,
  );
}

function expectNoDeadControls(container: HTMLElement) {
  expect(container.querySelectorAll('a[href="#"], a:not([href])')).toHaveLength(
    0,
  );
  for (const button of Array.from(container.querySelectorAll("button"))) {
    expect(button.textContent?.trim()).not.toEqual("");
    if (button.hasAttribute("disabled")) {
      expect(button).toHaveAttribute("aria-disabled", "true");
    }
  }
}

describe("RepositoryMilestoneDetailPage", () => {
  it("renders milestone progress, tabs, rows, and prefilled new issue links", () => {
    const { container } = renderDetail();

    expect(
      screen.getByRole("heading", { name: "Launch readiness" }),
    ).toBeVisible();
    expect(screen.getByRole("img", { name: "50% complete" })).toBeVisible();
    expect(screen.getByRole("link", { name: "New issue" })).toHaveAttribute(
      "href",
      "/mona/octo-app/issues/new?milestone=milestone-1",
    );
    expect(screen.getByRole("link", { name: /Open 1/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/milestones/milestone-1?state=open",
    );
    expect(screen.getByText("Stabilize importer")).toBeVisible();
    expect(screen.queryByText("Merge docs")).toBeNull();
    expectNoDeadControls(container);
  });

  it("shows closed PR-backed rows and selected count", () => {
    renderDetail(milestoneDetail(), { state: "closed" });

    expect(screen.getByText("Merge docs")).toBeVisible();
    fireEvent.click(screen.getByLabelText("Select Pull request 43"));
    expect(
      screen
        .getAllByText(/selected/)
        .some((node) => node.textContent?.includes("1 selected")),
    ).toBe(true);
  });

  it("closes, reopens, and deletes through same-origin milestone actions", async () => {
    renderDetail();

    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(refreshMock).toHaveBeenCalled());
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions/milestone-1",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ action: "close" }),
      }),
    );

    vi.mocked(fetch).mockClear();
    renderDetail(milestoneDetail({ state: "closed" }));
    fireEvent.click(screen.getByRole("button", { name: "Reopen" }));
    await waitFor(() => expect(refreshMock).toHaveBeenCalledTimes(2));
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions/milestone-1",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ action: "reopen" }),
      }),
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Delete" })[0]);
    await waitFor(() =>
      expect(pushMock).toHaveBeenCalledWith("/mona/octo-app/milestones"),
    );
  });
});
