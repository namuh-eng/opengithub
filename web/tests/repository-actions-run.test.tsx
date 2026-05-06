import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryActionsRunPage } from "@/components/RepositoryActionsRunPage";
import type { RepositoryActionsRunDetail, RepositoryOverview } from "@/lib/api";

const refresh = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh }),
}));

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions run detail test repository",
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

function runDetail(
  overrides: Partial<RepositoryActionsRunDetail> = {},
): RepositoryActionsRunDetail {
  const detail: RepositoryActionsRunDetail = {
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
    runtimePolicy: {
      secretCount: 1,
      variableCount: 2,
      blockedSecretCount: 1,
      blockedVariableCount: 0,
      scopes: [{ scope: "repository", secrets: 1, variables: 2 }],
      blockedReasons: ["fork_pull_request"],
      redactionMarker: "::add-mask::***",
    },
    attempts: [
      {
        id: null,
        attemptNumber: 1,
        status: "completed",
        conclusion: "failure",
        triggerKind: "initial",
        actor: {
          id: "user-1",
          login: "mona",
          displayName: "Mona",
          avatarUrl: null,
        },
        startedAt: "2026-05-01T00:00:00Z",
        completedAt: "2026-05-01T00:03:00Z",
        createdAt: "2026-05-01T00:00:00Z",
      },
      {
        id: "attempt-2",
        attemptNumber: 2,
        status: "completed",
        conclusion: "failure",
        triggerKind: "rerun_failed",
        actor: null,
        startedAt: "2026-05-01T00:04:00Z",
        completedAt: "2026-05-01T00:06:00Z",
        createdAt: "2026-05-01T00:04:00Z",
      },
    ],
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
          },
        ],
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
    artifacts: [
      {
        id: "artifact-1",
        name: "playwright-report",
        digest: "sha256:abc123",
        sizeBytes: 2048,
        retentionDays: 14,
        expiredAt: null,
        downloadAvailable: true,
        deleteAvailable: true,
        createdAt: "2026-05-01T00:06:00Z",
        updatedAt: "2026-05-01T00:06:00Z",
      },
    ],
    actionState: {
      canRerun: true,
      canRerunFailed: true,
      canCancel: false,
      canDeleteLogs: true,
      disabledReason: null,
    },
  };

  return { ...detail, ...overrides };
}

describe("RepositoryActionsRunPage", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    refresh.mockClear();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        json: async () => ({
          job: {
            id: "job-1",
            runId: "run-1",
            name: "unit / web",
            status: "completed",
            conclusion: "failure",
            logDeletedAt: null,
          },
          lines: [
            {
              lineNumber: 1,
              timestamp: "2026-05-01T00:04:00Z",
              content: "Installing dependencies",
              anchor: "L1",
            },
            {
              lineNumber: 2,
              timestamp: "2026-05-01T00:05:00Z",
              content: "error: Expected string, found number",
              anchor: "L2",
            },
          ],
          total: 2,
          page: 1,
          pageSize: 30,
          query: null,
          downloadHref:
            "/api/repos/mona/octo-app/actions/jobs/job-1/logs/download",
        }),
        ok: true,
      }),
    );
  });

  it("renders run metadata, jobs, annotations, and artifacts", () => {
    render(
      <RepositoryActionsRunPage
        detail={runDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: /Validate run detail/ }),
    ).toBeVisible();
    expect(screen.getByText("#42")).toBeVisible();
    expect(
      screen.getAllByText((_content, element) =>
        Boolean(element?.textContent?.includes("Pull Request on")),
      ).length,
    ).toBeGreaterThan(0);
    expect(screen.getByText("feature/actions")).toBeVisible();
    expect(screen.getByText("abcdef0")).toBeVisible();
    expect(screen.getByText("1 available, 1 blocked")).toBeVisible();
    expect(screen.getByText("2 available")).toBeVisible();
    expect(screen.getByText("fork pull request")).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Workflow run jobs" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: /Attempt 2/ })).toHaveAttribute(
      "href",
      "/mona/octo-app/actions/runs/run-1?attempt=2",
    );
    expect(screen.getByText("Type error")).toBeVisible();
    expect(screen.getByText("Expected string, found number")).toBeVisible();
    expect(screen.getByText("playwright-report")).toBeVisible();
    expect(screen.getByText("sha256:abc123")).toBeVisible();
    expect(screen.getByText("2.0 KB")).toBeVisible();
    expect(screen.getByText("14 days")).toBeVisible();
  });

  it("focuses selected job details and shows deleted log state", () => {
    render(
      <RepositoryActionsRunPage
        detail={runDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("link", { name: /deploy preview/ }));
    const selected = screen
      .getByRole("heading", { name: "deploy preview" })
      .closest("section");
    expect(selected).not.toBeNull();
    expect(
      within(selected as HTMLElement).getByText("Logs deleted"),
    ).toBeVisible();
  });

  it("loads job logs, searches through the proxy, and copies artifact URLs", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockImplementation(async (input) => {
      const url = String(input);
      if (url.includes("/artifacts/")) {
        return {
          json: async () => ({
            artifactId: "artifact-1",
            name: "playwright-report",
            filename: "playwright-report.zip",
            downloadUrl:
              "/api/repos/mona/octo-app/actions/artifacts/artifact-1/download?token=dev-local",
            storageKey: "actions/artifacts/report.zip",
            expiresAt: "2026-05-01T00:16:00Z",
          }),
          ok: true,
        } as Response;
      }
      return {
        json: async () => ({
          job: {
            id: "job-1",
            runId: "run-1",
            name: "unit / web",
            status: "completed",
            conclusion: "failure",
            logDeletedAt: null,
          },
          lines: [
            {
              lineNumber: url.includes("q=error") ? 3 : 1,
              timestamp: "2026-05-01T00:06:00Z",
              content: url.includes("q=error")
                ? "error: Expected string, found number"
                : "Installing dependencies",
              anchor: url.includes("q=error") ? "L3" : "L1",
            },
          ],
          total: 1,
          page: 1,
          pageSize: 30,
          query: url.includes("q=error") ? "error" : null,
          downloadHref:
            "/api/repos/mona/octo-app/actions/jobs/job-1/logs/download",
        }),
        ok: true,
      } as Response;
    });

    render(
      <RepositoryActionsRunPage
        detail={runDetail()}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Search job log" }), {
      target: { value: "error" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));

    expect(
      await screen.findByText("error: Expected string, found number"),
    ).toBeVisible();
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/jobs/job-1/logs?q=error",
      { cache: "no-store" },
    );

    fireEvent.click(screen.getByRole("button", { name: "Copy URL" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Copied playwright-report.zip download URL.",
    );
    expect(writeText).toHaveBeenCalledWith(
      "/api/repos/mona/octo-app/actions/artifacts/artifact-1/download?token=dev-local",
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/artifacts/artifact-1/download?metadata=1",
      { cache: "no-store" },
    );

    const deleteButtons = screen.getAllByRole("button", { name: "Delete" });
    const artifactDelete = deleteButtons.at(-1);
    expect(artifactDelete).toBeDefined();
    fireEvent.click(artifactDelete as HTMLElement);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Artifact deleted.",
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/actions/artifacts/artifact-1/download",
      { cache: "no-store", method: "DELETE" },
    );
  });

  it("posts rerun, cancel, job rerun, and confirmed delete-log mutations", async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockImplementation(async (input) => {
      if (String(input).includes("/actions/jobs/")) {
        return {
          json: async () => ({
            job: {
              id: "job-1",
              runId: "run-1",
              name: "unit / web",
              status: "completed",
              conclusion: "failure",
              logDeletedAt: null,
            },
            lines: [],
            total: 0,
            page: 1,
            pageSize: 30,
            query: null,
            downloadHref:
              "/api/repos/mona/octo-app/actions/jobs/job-1/logs/download",
          }),
          ok: true,
        } as Response;
      }
      return {
        json: async () => runDetail(),
        ok: true,
      } as Response;
    });

    render(
      <RepositoryActionsRunPage
        detail={runDetail({
          actionState: {
            canRerun: true,
            canRerunFailed: true,
            canCancel: true,
            canDeleteLogs: true,
            disabledReason: null,
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Re-run all" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/actions/runs/run-1/rerun",
        expect.objectContaining({
          body: JSON.stringify({ mode: "all", jobId: null }),
          method: "POST",
        }),
      ),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: "Re-run failed" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/actions/runs/run-1/rerun",
        expect.objectContaining({
          body: JSON.stringify({ mode: "failed", jobId: null }),
          method: "POST",
        }),
      ),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalledTimes(2));

    fireEvent.click(screen.getByRole("button", { name: "Re-run job" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/actions/runs/run-1/rerun",
        expect.objectContaining({
          body: JSON.stringify({ mode: "job", jobId: "job-1" }),
          method: "POST",
        }),
      ),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalledTimes(3));

    fireEvent.click(screen.getByRole("button", { name: "Cancel run" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/actions/runs/run-1/cancel",
        expect.objectContaining({ method: "POST" }),
      ),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalledTimes(4));

    fireEvent.click(screen.getByRole("button", { name: "Delete logs" }));
    expect(screen.getByText(/Delete stored logs for this run/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Confirm delete" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/actions/runs/run-1/logs",
        expect.objectContaining({ method: "DELETE" }),
      ),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalledTimes(5));
  });

  it("does not render inert anchors or unnamed visible buttons", () => {
    const { container } = render(
      <RepositoryActionsRunPage
        detail={runDetail()}
        repository={repositoryOverview()}
      />,
    );

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });

  it("keeps unavailable artifacts non-downloadable and live runs cancelable", () => {
    render(
      <RepositoryActionsRunPage
        detail={runDetail({
          actionState: {
            canRerun: false,
            canRerunFailed: false,
            canCancel: true,
            canDeleteLogs: false,
            disabledReason: "Run is still in progress.",
          },
          artifacts: [
            {
              id: "artifact-expired",
              name: "coverage",
              digest: "sha256:expired",
              sizeBytes: 512,
              retentionDays: 1,
              expiredAt: "2026-05-01T00:16:00Z",
              downloadAvailable: false,
              deleteAvailable: false,
              createdAt: "2026-05-01T00:06:00Z",
              updatedAt: "2026-05-01T00:06:00Z",
            },
          ],
          run: {
            ...runDetail().run,
            conclusion: null,
            isLive: true,
            status: "in_progress",
            statusCategory: "in_progress",
          },
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("button", { name: "Cancel run" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Re-run all" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /^Download$/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Copy URL" })).toBeDisabled();
    expect(screen.queryByRole("link", { name: "Download" })).toBeNull();
    expect(screen.getByText("Expired")).toBeVisible();
  });

  it("shows backend validation errors without losing the run workspace", () => {
    render(
      <RepositoryActionsRunPage
        detail={runDetail({ jobs: [], annotations: [], artifacts: [] })}
        repository={repositoryOverview()}
        validationError={{
          error: {
            code: "not_found",
            message: "Workflow run could not be loaded.",
          },
          status: 404,
        }}
      />,
    );

    expect(screen.getByRole("status")).toHaveTextContent(
      "Workflow run could not be loaded.",
    );
    expect(
      screen.getByRole("heading", { name: /Validate run detail/ }),
    ).toBeVisible();
    expect(
      screen.getByText("No annotations were emitted for this run."),
    ).toBeVisible();
    expect(
      screen.getByText("This run did not upload artifacts."),
    ).toBeVisible();
  });
});
