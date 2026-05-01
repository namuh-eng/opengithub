import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryActionsPage } from "@/components/RepositoryActionsPage";
import type {
  ActionsRunListItem,
  ActionsWorkflowRailItem,
  RepositoryActionsDashboard,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions test repository",
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
  return { ...base, ...overrides };
}

function workflow(
  overrides: Partial<ActionsWorkflowRailItem> = {},
): ActionsWorkflowRailItem {
  return {
    id: "workflow-1",
    name: "CI",
    path: ".github/workflows/ci.yml",
    state: "active",
    triggerEvents: ["push", "pull_request"],
    pinned: true,
    runCount: 2,
    latestRun: {
      id: "run-1",
      runNumber: 42,
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
    runNumber: 42,
    displayTitle: "Build Editorial Actions page",
    status: "completed",
    conclusion: "success",
    statusCategory: "success",
    event: "push",
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
    commitMessage: "Build Editorial Actions page",
    jobSummary: {
      cancelled: 0,
      completed: 3,
      failure: 0,
      inProgress: 0,
      queued: 0,
      skipped: 0,
      success: 3,
      timedOut: 0,
      total: 3,
    },
    durationSeconds: 145,
    isLive: false,
    startedAt: "2026-05-01T00:00:00Z",
    completedAt: "2026-05-01T00:02:25Z",
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-01T00:02:25Z",
    ...overrides,
  };
}

function dashboard(
  overrides: Partial<RepositoryActionsDashboard> = {},
): RepositoryActionsDashboard {
  const workflows = overrides.workflows ?? [
    workflow(),
    workflow({
      id: "workflow-2",
      name: "Release",
      path: ".github/workflows/release.yml",
      pinned: false,
      runCount: 0,
      latestRun: null,
    }),
  ];
  const runs = overrides.runs?.items ?? [run()];
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    viewerPermission: "owner",
    workflows,
    runs: {
      items: runs,
      total: runs.length,
      page: 1,
      pageSize: 30,
      ...overrides.runs,
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
      actors: [],
      branches: [],
      events: [],
      statuses: [],
      workflows: [],
    },
    emptyState: {
      hasRuns: runs.length > 0,
      hasWorkflows: workflows.length > 0,
      message: workflows.length
        ? "No workflow runs match the current filters."
        : "This repository does not have any workflows yet.",
      newWorkflowHref: "/mona/octo-app/new/main/.github/workflows",
    },
    ...overrides,
  };
}

function renderActions(
  view: RepositoryActionsDashboard = dashboard(),
  query: Record<string, string | undefined> = {},
) {
  render(
    <RepositoryActionsPage
      dashboard={view}
      query={query}
      repository={repositoryOverview()}
    />,
  );
}

describe("RepositoryActionsPage", () => {
  it("renders the workflow rail, run count, filters, and concrete run links", () => {
    renderActions();

    expect(
      screen.getByRole("heading", { name: "All workflows" }),
    ).toBeVisible();
    const workflowNav = screen.getByRole("navigation", {
      name: "Actions workflows",
    });
    expect(
      within(workflowNav).getByRole("link", { name: /All workflows/ }),
    ).toHaveAttribute("href", "/mona/octo-app/actions");
    expect(
      within(workflowNav).getByRole("link", { name: /CI/ }),
    ).toHaveAttribute("href", "/mona/octo-app/actions?workflow=workflow-1");
    expect(screen.getByLabelText("Pinned workflow")).toBeInTheDocument();

    const runLink = screen.getByRole("link", {
      name: "Build Editorial Actions page",
    });
    expect(runLink).toHaveAttribute(
      "href",
      "/mona/octo-app/actions/runs/run-1",
    );
    expect(screen.getByText("3/3 jobs passed")).toBeVisible();
    expect(screen.getByLabelText("Success run")).toBeVisible();
    for (const filter of ["Workflow", "Event", "Status", "Branch", "Actor"]) {
      expect(screen.getByRole("link", { name: filter })).toHaveAttribute(
        "href",
        "/mona/octo-app/actions",
      );
    }
  });

  it("renders live and failed statuses with semantic Editorial chips", () => {
    renderActions(
      dashboard({
        runs: {
          items: [
            run({
              id: "run-live",
              runNumber: 43,
              displayTitle: "Deploy preview",
              status: "in_progress",
              conclusion: null,
              statusCategory: "in_progress",
              isLive: true,
              durationSeconds: null,
            }),
            run({
              id: "run-failed",
              runNumber: 44,
              displayTitle: "Release package",
              status: "completed",
              conclusion: "failure",
              statusCategory: "failure",
            }),
          ],
          total: 2,
          page: 1,
          pageSize: 30,
        },
      }),
    );

    expect(
      screen.getByRole("link", { name: "Deploy preview" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions/runs/run-live");
    expect(screen.getByText("In Progress")).toBeVisible();
    expect(screen.getByText("Live")).toBeVisible();
    expect(screen.getByLabelText("Failure run")).toBeVisible();
  });

  it("renders empty workflow templates and a working New workflow CTA", () => {
    renderActions(
      dashboard({
        workflows: [],
        runs: {
          items: [],
          total: 0,
          page: 1,
          pageSize: 30,
        },
      }),
    );

    expect(screen.getByText("Start automating this repository")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "New workflow" }).at(0),
    ).toHaveAttribute("href", "/mona/octo-app/new/main/.github/workflows");
    expect(screen.getByText("Rust").closest("a")).toHaveAttribute(
      "href",
      "/mona/octo-app/new/main/.github/workflows",
    );
  });

  it("does not render inert anchors or unnamed visible buttons", () => {
    const { container } = render(
      <RepositoryActionsPage
        dashboard={dashboard()}
        query={{}}
        repository={repositoryOverview()}
      />,
    );

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });
});
