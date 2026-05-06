import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryPullRequestChecksPage } from "@/components/RepositoryPullRequestChecksPage";
import type { PullRequestChecksView, RepositoryOverview } from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Checks repository",
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
      contributorsCount: 1,
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
}

function checksView(): PullRequestChecksView {
  return {
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      defaultBranch: "main",
    },
    pullRequest: {
      id: "pull-1",
      number: 42,
      title: "Wire check runs",
      state: "open",
      headRef: "feature/check-runs",
      baseRef: "main",
      headSha: "abcdef1234567890",
      href: "/mona/octo-app/pull/42",
    },
    summary: {
      status: "completed",
      conclusion: "failure",
      totalCount: 2,
      completedCount: 2,
      failedCount: 1,
    },
    requiredStatusChecks: ["ci/test"],
    canRerun: true,
    checkRuns: [
      {
        id: "check-1",
        name: "ci/test",
        status: "completed",
        conclusion: "failure",
        required: true,
        startedAt: "2026-05-07T00:00:00Z",
        completedAt: "2026-05-07T00:02:00Z",
        outputTitle: "ci/test",
        outputSummary: "Job failed. Review annotations and logs.",
        annotationsCount: 1,
        detailsHref: "/mona/octo-app/actions/runs/run-1/jobs/job-1",
        rerunHref: "/mona/octo-app/pull/42/checks/check-1/rerun",
        annotations: [
          {
            id: "annotation-1",
            path: "src/app.ts",
            startLine: 12,
            endLine: null,
            level: "failure",
            message: "Expected route to return 200.",
            createdAt: "2026-05-07T00:01:00Z",
          },
        ],
      },
      {
        id: "check-2",
        name: "lint",
        status: "completed",
        conclusion: "success",
        required: false,
        startedAt: "2026-05-07T00:00:00Z",
        completedAt: "2026-05-07T00:01:00Z",
        outputTitle: "lint",
        outputSummary: "Job completed successfully.",
        annotationsCount: 0,
        detailsHref: "/mona/octo-app/actions/runs/run-1/jobs/job-2",
        rerunHref: "/mona/octo-app/pull/42/checks/check-2/rerun",
        annotations: [],
      },
    ],
  };
}

describe("RepositoryPullRequestChecksPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders check runs, annotations, required badges, and concrete links", () => {
    const { container } = render(
      <RepositoryPullRequestChecksPage
        checks={checksView()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Checks for #42" }),
    ).toBeVisible();
    expect(screen.getByText("1 check failed.")).toHaveClass("chip", "err");
    expect(screen.getAllByText("ci/test").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Required")).toHaveClass("chip", "warn");
    expect(screen.getByText("src/app.ts:12")).toBeVisible();
    expect(screen.getByText("Expected route to return 200.")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "View details" })[0],
    ).toHaveAttribute("href", "/mona/octo-app/actions/runs/run-1/jobs/job-1");
    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("queues a job rerun with visible feedback", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ ok: true }),
    });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <RepositoryPullRequestChecksPage
        checks={checksView()}
        repository={repositoryOverview()}
        viewerAuthenticated={true}
      />,
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Re-run job" })[0]);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/pull/42/checks/check-1/rerun",
        { method: "POST" },
      ),
    );
    expect(await screen.findByText("Check re-run queued.")).toHaveAttribute(
      "role",
      "status",
    );
  });
});
