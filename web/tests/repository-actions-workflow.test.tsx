import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryActionsWorkflowPage } from "@/components/RepositoryActionsWorkflowPage";
import type {
  ActionsRunListItem,
  ActionsWorkflowRailItem,
  RepositoryActionsWorkflowDetail,
  RepositoryOverview,
} from "@/lib/api";

const pushMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock }),
}));

beforeEach(() => {
  pushMock.mockClear();
});

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions workflow test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
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
      contributorsCount: 1,
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

function workflow(
  overrides: Partial<ActionsWorkflowRailItem> = {},
): ActionsWorkflowRailItem {
  return {
    id: "workflow-1",
    name: "CI",
    path: ".github/workflows/ci.yml",
    state: "active",
    triggerEvents: ["push", "workflow_dispatch"],
    pinned: true,
    runCount: 1,
    latestRun: {
      id: "run-1",
      runNumber: 7,
      status: "completed",
      conclusion: "success",
      createdAt: "2026-05-01T00:00:00Z",
    },
    ...overrides,
  };
}

function run(overrides: Partial<ActionsRunListItem> = {}): ActionsRunListItem {
  return {
    id: "run-1",
    workflowId: "workflow-1",
    workflowName: "CI",
    workflowPath: ".github/workflows/ci.yml",
    runNumber: 7,
    displayTitle: "Run CI manually",
    status: "completed",
    conclusion: "success",
    statusCategory: "success",
    event: "workflow_dispatch",
    actor: {
      id: "user-1",
      login: "mona",
      displayName: "Mona",
      avatarUrl: null,
    },
    headBranch: "main",
    headSha: "abcdef1234567890",
    shortSha: "abcdef1",
    pullRequest: null,
    commitMessage: "Update workflow",
    jobSummary: {
      total: 3,
      queued: 0,
      inProgress: 0,
      completed: 3,
      cancelled: 0,
      success: 3,
      failure: 0,
      skipped: 0,
      timedOut: 0,
    },
    durationSeconds: 83,
    isLive: false,
    startedAt: "2026-05-01T00:00:00Z",
    completedAt: "2026-05-01T00:01:23Z",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-01T00:01:23Z",
    ...overrides,
  };
}

function detail(
  overrides: Partial<RepositoryActionsWorkflowDetail> = {},
): RepositoryActionsWorkflowDetail {
  const workflows = [
    workflow(),
    workflow({
      id: "workflow-2",
      name: "Release",
      path: ".github/workflows/release.yml",
      pinned: false,
      runCount: 0,
    }),
  ];
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    viewerPermission: "owner",
    workflow: {
      id: "workflow-1",
      name: "CI",
      path: ".github/workflows/ci.yml",
      state: "active",
      triggerEvents: ["push", "workflow_dispatch"],
      sourceBranch: "main",
      sourceSha: "workflow-source-sha",
      sourceBlobId: "blob-1",
      sourceHref: "/mona/octo-app/blob/main/.github/workflows/ci.yml",
      dispatch: {
        enabled: true,
        inputs: [],
      },
      yamlParseError: null,
      valid: true,
    },
    workflows,
    runs: {
      items: [run()],
      total: 1,
      page: 1,
      pageSize: 30,
    },
    filters: {
      actor: null,
      branch: null,
      event: null,
      page: 1,
      pageSize: 30,
      q: null,
      status: null,
      workflow: null,
    },
    filterOptions: {
      actors: [{ value: "user-1", label: "mona", count: 1 }],
      branches: [{ value: "main", label: "main", count: 1 }],
      events: [
        { value: "workflow_dispatch", label: "workflow dispatch", count: 1 },
      ],
      statuses: [{ value: "success", label: "success", count: 1 }],
      workflows: [],
    },
    refs: [
      {
        name: "refs/heads/main",
        shortName: "main",
        kind: "branch",
        sha: "abcdef",
      },
    ],
    emptyState: {
      hasWorkflows: true,
      hasRuns: true,
      message: "This workflow has not run yet.",
      newWorkflowHref: "/mona/octo-app/new/main/.github/workflows",
    },
    ...overrides,
  };
}

function renderWorkflow(
  view: RepositoryActionsWorkflowDetail = detail(),
  query: Record<string, string | undefined> = {},
) {
  return render(
    <RepositoryActionsWorkflowPage
      detail={view}
      query={query}
      repository={repositoryOverview()}
    />,
  );
}

describe("RepositoryActionsWorkflowPage", () => {
  it("renders selected workflow context, source link, scoped runs, and no Workflow filter", () => {
    renderWorkflow();

    expect(screen.getByRole("heading", { name: "CI" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Workflow file" })).toHaveAttribute(
      "href",
      "/mona/octo-app/blob/main/.github/workflows/ci.yml",
    );
    expect(screen.getByRole("button", { name: "Run workflow" })).toBeDisabled();
    expect(
      screen.getByRole("navigation", { name: "Actions workflows" }),
    ).toBeVisible();
    const workflowNav = screen.getByRole("navigation", {
      name: "Actions workflows",
    });
    expect(
      within(workflowNav).getByRole("link", { name: /CI/ }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      within(workflowNav).getByRole("link", { name: /Release/ }),
    ).toHaveAttribute(
      "href",
      "/mona/octo-app/actions/workflows/.github/workflows/release.yml",
    );
    expect(
      screen.getByRole("link", { name: "Run CI manually" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions/runs/run-1");
    expect(screen.queryByRole("button", { name: "Workflow" })).toBeNull();
    for (const filter of ["Event", "Status", "Branch", "Actor"]) {
      expect(screen.getByRole("button", { name: filter })).toBeVisible();
    }
  });

  it("updates workflow page query params from search, filters, and selected chips", () => {
    renderWorkflow(
      detail({
        filters: {
          actor: null,
          branch: null,
          event: null,
          page: 1,
          pageSize: 30,
          q: "manual",
          status: "success",
          workflow: null,
        },
      }),
      { q: "manual", status: "success" },
    );

    fireEvent.change(
      screen.getByPlaceholderText("Filter this workflow's runs"),
      {
        target: { value: "release" },
      },
    );
    const searchForm = screen
      .getByRole("button", { name: "Search" })
      .closest("form");
    expect(searchForm).not.toBeNull();
    fireEvent.submit(searchForm as HTMLFormElement);
    expect(pushMock).toHaveBeenLastCalledWith(
      "/mona/octo-app/actions/workflows/.github/workflows/ci.yml?q=release&status=success",
    );

    fireEvent.click(screen.getByRole("button", { name: "Branch" }));
    fireEvent.click(screen.getByRole("menuitemradio", { name: /main/i }));
    expect(pushMock).toHaveBeenLastCalledWith(
      "/mona/octo-app/actions/workflows/.github/workflows/ci.yml?q=manual&status=success&branch=main",
    );

    fireEvent.click(screen.getByRole("button", { name: /Status: success/ }));
    expect(pushMock).toHaveBeenLastCalledWith(
      "/mona/octo-app/actions/workflows/.github/workflows/ci.yml?q=manual",
    );
  });

  it("renders invalid workflow state and empty runs with recovery links", () => {
    renderWorkflow(
      detail({
        workflow: {
          ...detail().workflow,
          dispatch: { enabled: false, inputs: [] },
          valid: false,
          yamlParseError: "mapping values are not allowed here",
        },
        runs: { items: [], total: 0, page: 1, pageSize: 30 },
      }),
    );

    expect(screen.getByText("Invalid workflow file")).toBeVisible();
    expect(
      screen.getByText("mapping values are not allowed here"),
    ).toBeVisible();
    expect(screen.queryByRole("button", { name: "Run workflow" })).toBeNull();
    expect(screen.getByText("No runs for this workflow yet")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to all workflows" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions");
  });

  it("does not render inert anchors or unnamed visible buttons", () => {
    const { container } = renderWorkflow();

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
    expect(
      within(
        screen.getByRole("navigation", { name: "Actions management" }),
      ).getByRole("link", { name: "API docs" }),
    ).toHaveAttribute("href", "/docs/api#actions-workflow-detail");
  });
});
