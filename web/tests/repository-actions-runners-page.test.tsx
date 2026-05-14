import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryActionsRunnersPage } from "@/components/RepositoryActionsRunnersPage";
import type {
  RepositoryActionsRunnerSettings,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Actions runners test repository",
    visibility: "private",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "admin",
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

function settings(
  overrides: Partial<RepositoryActionsRunnerSettings> = {},
): RepositoryActionsRunnerSettings {
  return {
    canManageRunners: true,
    queue: {
      busyRunners: 1,
      cancelInProgress: false,
      concurrencyLimit: 4,
      offlineRunners: 1,
      onlineRunners: 2,
      queuedJobs: 3,
    },
    workflowPermissions: {
      allowPullRequestApproval: false,
      githubTokenPermission: "read",
      githubTokenScopes: ["contents:read", "metadata:read", "packages:read"],
    },
    environments: [
      {
        deploymentBranchPolicy: {},
        id: "env-1",
        name: "production",
        protectionRulesEnabled: true,
        requiredReviewers: [],
        updatedAt: "2026-05-07T00:00:00Z",
      },
    ],
    repository: {
      defaultBranch: "main",
      id: "repo-1",
      name: "octo-app",
      ownerLogin: "mona",
      visibility: "private",
    },
    runners: [
      {
        busySince: "2026-05-07T00:02:00Z",
        createdAt: "2026-05-07T00:00:00Z",
        currentJob: {
          jobId: "job-1",
          jobName: "build",
          runId: "run-1",
          runNumber: 42,
          startedAt: "2026-05-07T00:02:00Z",
          workflowName: "CI",
        },
        id: "runner-1",
        labels: ["self-hosted", "ubuntu-latest"],
        lastHeartbeat: "2026-05-07T00:03:00Z",
        name: "linux-build-1",
        status: "busy",
        updatedAt: "2026-05-07T00:03:00Z",
      },
    ],
    setup: {
      dockerCommand:
        "docker run --rm -e OPENGITHUB_RUNNER_TOKEN=ogr_test opengithub/runner:latest --repo mona/octo-app",
      expiresInMinutes: 60,
      registrationToken: "ogr_test",
    },
    viewerPermission: "admin",
    ...overrides,
  };
}

describe("RepositoryActionsRunnersPage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders runner health, queue depth, setup command, and real actions", () => {
    render(
      <RepositoryActionsRunnersPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    expect(screen.getByText("Queued jobs")).toBeVisible();
    expect(screen.getByText("linux-build-1")).toBeVisible();
    expect(screen.getByText("busy")).toBeVisible();
    expect(screen.getByText("ubuntu-latest")).toBeVisible();
    expect(screen.getByText(/Running CI #42: build/)).toBeVisible();
    expect(screen.getByText(/docker run --rm/)).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Assign queued jobs" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Register runner" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Save scheduling settings" }),
    ).toBeEnabled();
    expect(screen.getByText("GITHUB_TOKEN policy")).toBeVisible();
    expect(screen.getByText("Secret release approval")).toBeVisible();
    expect(
      screen.getByLabelText(
        "Require reviewer approval before environment secrets are released",
      ),
    ).toBeChecked();
    expect(
      screen.getByLabelText("Read repository contents and metadata"),
    ).toBeChecked();
    expect(screen.getByText("contents:read")).toBeVisible();
  });

  it("posts create, scheduling, and concurrency mutations through the concrete action route", async () => {
    const fetchMock = vi.fn((_url: string, init?: RequestInit) => {
      const body = JSON.parse(String(init?.body));
      if (body.action === "schedule-jobs") {
        return Promise.resolve(
          new Response(JSON.stringify({ assigned: [{}], queuedJobs: 2 }), {
            status: 200,
          }),
        );
      }
      return Promise.resolve(
        new Response(JSON.stringify(settings({ runners: [] })), {
          status: 200,
        }),
      );
    });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <RepositoryActionsRunnersPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings({ runners: [] }) }}
      />,
    );

    fireEvent.change(screen.getByLabelText("Runner name"), {
      target: { value: "gpu-runner" },
    });
    fireEvent.change(screen.getByLabelText("Labels"), {
      target: { value: "self-hosted, gpu" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Register runner" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/settings/actions/runners/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "create-runner",
            labels: ["self-hosted", "gpu"],
            name: "gpu-runner",
          }),
        }),
      ),
    );

    fireEvent.change(screen.getByLabelText("Concurrency limit"), {
      target: { value: "8" },
    });
    fireEvent.click(
      screen.getByLabelText(
        "Cancel older in-progress runs in the same concurrency group",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Save scheduling settings" }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/settings/actions/runners/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "update-settings",
            allowPullRequestApproval: false,
            cancelInProgress: true,
            concurrencyLimit: 8,
            environment: "production",
            environmentProtectionRulesEnabled: true,
            githubTokenPermission: "read",
          }),
        }),
      ),
    );

    fireEvent.click(
      screen.getByLabelText(
        "Read and write repository contents, checks, packages, issues, and pull requests",
      ),
    );
    fireEvent.click(
      screen.getByLabelText(
        "Allow Actions to create and approve pull requests",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Save workflow permissions" }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/settings/actions/runners/actions",
        expect.objectContaining({
          body: JSON.stringify({
            action: "update-settings",
            allowPullRequestApproval: true,
            cancelInProgress: true,
            concurrencyLimit: 8,
            environment: "production",
            environmentProtectionRulesEnabled: true,
            githubTokenPermission: "write",
          }),
        }),
      ),
    );

    fireEvent.click(screen.getByRole("button", { name: "Assign queued jobs" }));
    await screen.findByText("1 queued job assigned. 2 still queued.");
    expect(
      within(document.body).queryByRole("link", { name: "#" }),
    ).not.toBeInTheDocument();
  });
});
