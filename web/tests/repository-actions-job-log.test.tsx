import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryActionsJobLogPage } from "@/components/RepositoryActionsJobLogPage";
import type {
  RepositoryActionsJobLogDetail,
  RepositoryOverview,
} from "@/lib/api";

const pushMock = vi.fn();
const refreshMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: pushMock, refresh: refreshMock }),
}));

beforeEach(() => {
  pushMock.mockClear();
  refreshMock.mockClear();
});

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions job log test repository",
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

function jobLogDetail(
  overrides: Partial<RepositoryActionsJobLogDetail> = {},
): RepositoryActionsJobLogDetail {
  const detail: RepositoryActionsJobLogDetail = {
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
      sourceBranch: "main",
      sourceSha: "workflow-source-sha",
      sourceHref: "/mona/octo-app/blob/main/.github/workflows/ci.yml",
    },
    run: {
      id: "run-1",
      workflowId: "workflow-1",
      workflowName: "CI",
      workflowPath: ".github/workflows/ci.yml",
      runNumber: 42,
      displayTitle: "Validate run detail",
      status: "completed",
      conclusion: "failure",
      statusCategory: "failure",
      event: "pull_request",
      actor: {
        id: "user-1",
        login: "mona",
        displayName: "Mona",
        avatarUrl: null,
      },
      headBranch: "feature/actions",
      headSha: "abcdef0123456789",
      shortSha: "abcdef0",
      pullRequest: {
        id: "pr-1",
        number: 17,
        title: "Improve Actions",
      },
      commitMessage: "Add run detail fixture",
      jobSummary: {
        total: 2,
        queued: 0,
        inProgress: 0,
        completed: 2,
        cancelled: 0,
        success: 1,
        failure: 1,
        skipped: 0,
        timedOut: 0,
      },
      durationSeconds: 180,
      isLive: false,
      startedAt: "2026-05-01T00:00:00Z",
      completedAt: "2026-05-01T00:03:00Z",
      createdAt: "2026-05-01T00:00:00Z",
      updatedAt: "2026-05-01T00:03:00Z",
    },
    jobs: [
      {
        id: "job-1",
        name: "unit / web",
        groupName: "Checks",
        attemptNumber: 2,
        status: "completed",
        conclusion: "failure",
        runnerLabel: "ubuntu-latest",
        durationSeconds: 120,
        logAvailable: true,
        logDeletedAt: null,
        steps: [],
        startedAt: "2026-05-01T00:04:00Z",
        completedAt: "2026-05-01T00:06:00Z",
        createdAt: "2026-05-01T00:04:00Z",
        updatedAt: "2026-05-01T00:06:00Z",
      },
      {
        id: "job-2",
        name: "deploy preview",
        groupName: "Deploy",
        attemptNumber: 2,
        status: "completed",
        conclusion: "success",
        runnerLabel: "ubuntu-latest",
        durationSeconds: 60,
        logAvailable: false,
        logDeletedAt: "2026-05-01T00:07:00Z",
        steps: [],
        startedAt: "2026-05-01T00:04:00Z",
        completedAt: "2026-05-01T00:05:00Z",
        createdAt: "2026-05-01T00:04:00Z",
        updatedAt: "2026-05-01T00:05:00Z",
      },
    ],
    job: {
      id: "job-1",
      name: "unit / web",
      groupName: "Checks",
      attemptNumber: 2,
      status: "completed",
      conclusion: "failure",
      runnerLabel: "ubuntu-latest",
      durationSeconds: 120,
      logAvailable: true,
      logDeletedAt: null,
      steps: [],
      startedAt: "2026-05-01T00:04:00Z",
      completedAt: "2026-05-01T00:06:00Z",
      createdAt: "2026-05-01T00:04:00Z",
      updatedAt: "2026-05-01T00:06:00Z",
    },
    steps: [
      {
        id: "step-1",
        number: 1,
        name: "Install dependencies",
        status: "completed",
        conclusion: "success",
        durationSeconds: 20,
        startedAt: "2026-05-01T00:04:00Z",
        completedAt: "2026-05-01T00:04:20Z",
        matchCount: 1,
        lines: {
          items: [
            {
              lineNumber: 1,
              timestamp: "2026-05-01T00:04:00Z",
              content: "Installing dependencies after cache error",
              anchor: "L1",
            },
          ],
          page: 1,
          pageSize: 30,
          total: 1,
        },
      },
      {
        id: "step-2",
        number: 2,
        name: "Run tests",
        status: "completed",
        conclusion: "failure",
        durationSeconds: 100,
        startedAt: "2026-05-01T00:04:20Z",
        completedAt: "2026-05-01T00:06:00Z",
        matchCount: 1,
        lines: {
          items: [
            {
              lineNumber: 3,
              timestamp: "2026-05-01T00:06:00Z",
              content: "error: Expected string, found number",
              anchor: "L3",
            },
          ],
          page: 1,
          pageSize: 30,
          total: 1,
        },
      },
    ],
    annotations: [
      {
        id: "annotation-1",
        jobId: "job-1",
        stepId: "step-2",
        level: "failure",
        path: "web/src/app/page.tsx",
        startLine: 42,
        endLine: 42,
        title: "Type error",
        message: "Expected string, found number",
        rawDetails: "tsc failed",
        createdAt: "2026-05-01T00:06:00Z",
      },
    ],
    logState: {
      available: true,
      status: 200,
      reason: null,
      deletedAt: null,
      isLive: false,
      nextCursor: 3,
    },
    search: {
      query: "error",
      totalMatches: 2,
      selectedMatch: 1,
      matches: [
        {
          lineNumber: 1,
          stepId: "step-1",
          stepNumber: 1,
          anchor: "L1",
          preview: "Installing dependencies after cache error",
        },
        {
          lineNumber: 3,
          stepId: "step-2",
          stepNumber: 2,
          anchor: "L3",
          preview: "error: Expected string, found number",
        },
      ],
    },
    options: {
      showTimestamps: false,
      rawLogs: false,
      wrapLines: true,
    },
    downloadHref: "/api/repos/mona/octo-app/actions/jobs/job-1/logs/download",
    runArchiveHref: "/api/repos/mona/octo-app/actions/runs/run-1/logs/archive",
  };

  return { ...detail, ...overrides };
}

describe("RepositoryActionsJobLogPage", () => {
  it("renders job metadata, sibling navigation, annotations, and log steps", () => {
    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "unit / web" })).toBeVisible();
    expect(
      screen.getAllByText((_content, element) =>
        Boolean(element?.textContent?.includes("CI · Validate run detail")),
      ).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByRole("navigation", { name: "Workflow run jobs" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: /deploy preview/ }),
    ).toHaveAttribute("href", "/mona/octo-app/actions/runs/run-1/jobs/job-2");
    expect(screen.getByRole("textbox", { name: "Search log" })).toHaveValue(
      "error",
    );
    expect(screen.getByText("1 of 2 matches")).toBeVisible();
    expect(
      screen.getAllByText((_content, element) =>
        Boolean(
          element?.textContent?.includes(
            "Installing dependencies after cache error",
          ),
        ),
      ).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getAllByText((_content, element) =>
        Boolean(
          element?.textContent?.includes(
            "error: Expected string, found number",
          ),
        ),
      ).length,
    ).toBeGreaterThan(0);
    expect(screen.getByText("Type error")).toBeVisible();
    expect(screen.getByRole("link", { name: "Download log" })).toHaveAttribute(
      "href",
      "/mona/octo-app/actions/jobs/job-1/logs/download",
    );
  });

  it("collapses steps and toggles annotations without dead controls", () => {
    const { container } = render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail()}
        repository={repositoryOverview()}
      />,
    );

    const stepButton = screen.getByRole("button", {
      name: /Run tests/,
    });
    expect(stepButton).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(stepButton);
    expect(stepButton).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText("error: Expected string, found number"),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Hide annotations" }));
    expect(screen.queryByText("Problems in this job")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Show annotations" }));
    expect(screen.getByText("Problems in this job")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Log options" }));
    expect(screen.getByRole("menu")).toHaveTextContent("Raw logs");
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });

  it("saves log options, copies job permalinks, and exposes archive downloads", async () => {
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: () =>
          Promise.resolve({
            showTimestamps: true,
            rawLogs: true,
            wrapLines: true,
          }),
      }),
    );

    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("link", { name: "Download run archive" }),
    ).toHaveAttribute("href", "/mona/octo-app/actions/runs/run-1/logs/archive");

    fireEvent.click(screen.getByRole("button", { name: "Log options" }));
    fireEvent.click(screen.getByRole("menuitemcheckbox", { name: /Raw logs/ }));
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/actions/log-preferences",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          showTimestamps: false,
          rawLogs: true,
          wrapLines: true,
        }),
      }),
    );
    expect(await screen.findByText("Saved log options")).toBeVisible();
    expect(pushMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/runs/run-1/jobs/job-1?q=error&match=1&timestamps=false&raw=true",
    );

    fireEvent.click(
      screen.getByRole("menuitem", { name: "Copy job permalink" }),
    );
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
      expect.stringContaining("/mona/octo-app/actions/runs/run-1/jobs/job-1"),
    );
    vi.unstubAllGlobals();
  });

  it("submits URL-backed search and navigates between matches", () => {
    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Search log" }), {
      target: { value: "cache miss" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(pushMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/runs/run-1/jobs/job-1?q=cache+miss&match=1&timestamps=false",
    );

    fireEvent.click(screen.getByRole("button", { name: "Next result" }));
    expect(pushMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/runs/run-1/jobs/job-1?q=error&match=2&timestamps=false",
    );
    expect(
      screen.getByRole("button", { name: "Previous result" }),
    ).toBeDisabled();
  });

  it("renders highlighted matches and line permalink copy actions", async () => {
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });

    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getAllByText("error")[0].tagName.toLowerCase()).toBe("mark");
    fireEvent.click(
      screen.getByRole("button", { name: "Copy permalink for line 3" }),
    );
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
      expect.stringContaining(
        "/mona/octo-app/actions/runs/run-1/jobs/job-1#log-L3",
      ),
    );
    expect(await screen.findByText("Copied L3")).toBeVisible();
  });

  it("renders deleted or unavailable logs as a 410-style state", () => {
    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail({
          logState: {
            available: false,
            status: 410,
            reason: "workflow logs are unavailable",
            deletedAt: "2026-05-01T00:07:00Z",
            isLive: false,
            nextCursor: null,
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    const status = screen.getByRole("status");
    expect(within(status).getByText("410 unavailable")).toBeVisible();
    expect(
      within(status).getByText("workflow logs are unavailable"),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Download log" })).toBeDisabled();
  });

  it("shows backend validation errors while preserving the job shell", () => {
    render(
      <RepositoryActionsJobLogPage
        detail={jobLogDetail({
          annotations: [],
          logState: {
            available: false,
            status: 410,
            reason: "Workflow logs are unavailable for this job.",
            deletedAt: null,
            isLive: false,
            nextCursor: null,
          },
          steps: [],
        })}
        repository={repositoryOverview()}
        validationError={{
          error: {
            code: "not_found",
            message: "Workflow job could not be loaded.",
          },
          status: 404,
        }}
      />,
    );

    expect(screen.getAllByRole("status")[0]).toHaveTextContent(
      "Workflow job could not be loaded.",
    );
    expect(screen.getByRole("heading", { name: "unit / web" })).toBeVisible();
    expect(
      screen.getByText("Workflow logs are unavailable for this job."),
    ).toBeVisible();
  });
});
