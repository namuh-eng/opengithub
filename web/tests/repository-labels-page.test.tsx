import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryLabelsPage } from "@/components/RepositoryLabelsPage";
import type {
  RepositoryLabelSummary,
  RepositoryLabelsView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryLabelsHref } from "@/lib/navigation";

const refreshMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: refreshMock }),
}));

beforeEach(() => {
  refreshMock.mockReset();
  vi.stubGlobal(
    "confirm",
    vi.fn(() => true),
  );
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => ({
      ok: true,
      json: async () => ({
        eventId: "event-1",
        label: labelSummary({ id: "label-new", name: "triage" }),
      }),
    })),
  );
});

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Label test repository",
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
    ...overrides,
  };
}

function labelSummary(
  overrides: Partial<RepositoryLabelSummary> = {},
): RepositoryLabelSummary {
  return {
    id: "label-1",
    name: "bug",
    color: "b85c38",
    description: "Something is not working",
    isDefault: true,
    counts: {
      openIssues: 3,
      openPullRequests: 1,
      discussions: 2,
      totalIssueCount: 4,
    },
    issuesHref:
      "/mona/octo-app/issues?q=is%3Aissue+state%3Aopen+label%3Abug&labels=bug",
    pullRequestsHref:
      "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3Abug&labels=bug",
    discussionsHref: "/mona/octo-app/discussions?label=bug",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

function labelsView(
  overrides: Partial<RepositoryLabelsView> = {},
): RepositoryLabelsView {
  const items = overrides.items ?? [
    labelSummary(),
    labelSummary({
      id: "label-2",
      name: "good first issue",
      color: "3f7a5b",
      description: "Ready for new contributors",
      counts: {
        openIssues: 8,
        openPullRequests: 0,
        discussions: 0,
        totalIssueCount: 8,
      },
      issuesHref:
        "/mona/octo-app/issues?q=is%3Aissue+state%3Aopen+label%3A%22good+first+issue%22&labels=good+first+issue",
      pullRequestsHref:
        "/mona/octo-app/pulls?q=is%3Apr+state%3Aopen+label%3A%22good+first+issue%22&labels=good+first+issue",
      discussionsHref: "/mona/octo-app/discussions?label=good%20first%20issue",
    }),
  ];
  return {
    items,
    total: overrides.total ?? items.length,
    page: 1,
    pageSize: 100,
    filters: {
      query: null,
      sort: "name",
      direction: "asc",
    },
    viewer: {
      authenticated: true,
      role: "write",
      canRead: true,
      canWrite: true,
      canAdmin: false,
    },
    repository: {
      id: "repo-1",
      owner: "mona",
      name: "octo-app",
      visibility: "public",
      isArchived: false,
    },
    ...overrides,
  };
}

function renderLabels(
  view: RepositoryLabelsView = labelsView(),
  repository: RepositoryOverview = repositoryOverview(),
) {
  return render(
    <RepositoryLabelsPage
      labels={view}
      query={{
        q: view.filters.query,
        sort: view.filters.sort,
        direction: view.filters.direction,
      }}
      repository={repository}
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

describe("RepositoryLabelsPage", () => {
  it("renders searchable label rows with concrete issue and pull request links", () => {
    const { container } = renderLabels();

    expect(screen.getByRole("heading", { name: "Labels" })).toBeVisible();
    expect(screen.getByPlaceholderText("Search all labels")).toBeVisible();
    expect(screen.getByText("2 labels")).toBeVisible();
    expect(screen.getByText("bug")).toBeVisible();
    expect(screen.getByText("Something is not working")).toBeVisible();
    expect(screen.getByRole("link", { name: /3 open issues/ })).toHaveAttribute(
      "href",
      expect.stringContaining("/mona/octo-app/issues"),
    );
    expect(
      screen.getByRole("link", { name: /1 open pull requests/ }),
    ).toHaveAttribute("href", expect.stringContaining("/mona/octo-app/pulls"));
    expectNoDeadControls(container);
  });

  it("preserves query state in search and sort links", () => {
    renderLabels(
      labelsView({
        filters: {
          query: "docs",
          sort: "total_issue_count",
          direction: "desc",
        },
      }),
    );

    expect(screen.getByPlaceholderText("Search all labels")).toHaveValue(
      "docs",
    );
    fireEvent.click(screen.getByText("Sort"));
    expect(screen.getByRole("menuitemradio", { name: /Name/ })).toHaveAttribute(
      "href",
      repositoryLabelsHref("mona", "octo-app", {
        q: "docs",
        sort: "name",
        direction: "asc",
      }),
    );
  });

  it("creates labels through the same-origin proxy and refreshes", async () => {
    renderLabels();

    fireEvent.click(screen.getByRole("button", { name: "New label" }));
    fireEvent.change(screen.getByLabelText("Label name"), {
      target: { value: "needs review" },
    });
    fireEvent.change(screen.getByLabelText("Label description"), {
      target: { value: "Needs maintainer attention" },
    });
    fireEvent.change(screen.getByLabelText("Label color"), {
      target: { value: "7f6a42" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save label" }));

    await waitFor(() => expect(refreshMock).toHaveBeenCalled());
    expect(fetch).toHaveBeenCalledWith("/mona/octo-app/labels/actions", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        name: "needs review",
        description: "Needs maintainer attention",
        color: "7f6a42",
      }),
    });
  });

  it("edits and deletes labels with confirmation", async () => {
    renderLabels();
    const row = screen.getByText("bug").closest("article");
    expect(row).not.toBeNull();

    fireEvent.click(
      within(row as HTMLElement).getByRole("button", { name: "Edit" }),
    );
    fireEvent.change(screen.getByLabelText("Label description"), {
      target: { value: "Confirmed defect" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save label" }));

    await waitFor(() =>
      expect(fetch).toHaveBeenCalledWith(
        "/mona/octo-app/labels/actions/label-1",
        expect.objectContaining({ method: "PATCH" }),
      ),
    );

    fireEvent.click(
      within(row as HTMLElement).getByRole("button", { name: "Delete" }),
    );

    await waitFor(() =>
      expect(fetch).toHaveBeenCalledWith(
        "/mona/octo-app/labels/actions/label-1",
        {
          method: "DELETE",
        },
      ),
    );
    expect(confirm).toHaveBeenCalledWith("Delete label bug?");
  });

  it("hides writer controls for readers and wraps long label names", () => {
    const longName = "label-with-a-very-long-name-that-must-wrap-in-the-row";
    const { container } = renderLabels(
      labelsView({
        items: [labelSummary({ id: "label-long", name: longName })],
        viewer: {
          authenticated: true,
          role: "read",
          canRead: true,
          canWrite: false,
          canAdmin: false,
        },
      }),
      repositoryOverview({ viewerPermission: "read" }),
    );

    expect(screen.queryByRole("button", { name: "New label" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Edit" })).toBeNull();
    expect(screen.getByText(longName)).toHaveClass("break-words");
    expectNoDeadControls(container);
  });
});
