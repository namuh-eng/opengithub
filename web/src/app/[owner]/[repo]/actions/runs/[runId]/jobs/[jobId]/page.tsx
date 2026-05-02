import { AppShell } from "@/components/AppShell";
import { RepositoryActionsJobLogPage as RepositoryActionsJobLogView } from "@/components/RepositoryActionsJobLogPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { RepositoryActionsJobLogDetail } from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsJobLogDetail,
  getSessionAndShellContext,
} from "@/lib/server-session";

type ActionsJobLogPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    runId: string;
    jobId: string;
  }>;
};

export default async function ActionsJobLogPage({
  params,
}: ActionsJobLogPageProps) {
  const { owner, repo, runId, jobId } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedRunId = decodeURIComponent(runId);
  const decodedJobId = decodeURIComponent(jobId);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, detail] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsJobLogDetail(
      ownerLogin,
      repositoryName,
      decodedRunId,
      decodedJobId,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && !("error" in detail) ? (
        <RepositoryActionsJobLogView detail={detail} repository={repository} />
      ) : repository && "error" in detail ? (
        <RepositoryActionsJobLogView
          detail={emptyJobLogDetail(
            ownerLogin,
            repositoryName,
            decodedRunId,
            decodedJobId,
          )}
          repository={repository}
          validationError={detail}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

function emptyJobLogDetail(
  ownerLogin: string,
  repositoryName: string,
  runId: string,
  jobId: string,
): RepositoryActionsJobLogDetail {
  return {
    repository: {
      id: "unavailable",
      ownerLogin,
      name: repositoryName,
      visibility: "public",
      defaultBranch: "main",
    },
    viewerPermission: null,
    workflow: {
      id: "unavailable",
      name: "Workflow",
      path: ".github/workflows/workflow.yml",
      state: "active",
      sourceBranch: "main",
      sourceSha: null,
      sourceHref: `/${ownerLogin}/${repositoryName}/actions`,
    },
    run: {
      id: runId,
      workflowId: "unavailable",
      workflowName: "Workflow",
      workflowPath: ".github/workflows/workflow.yml",
      runNumber: 0,
      displayTitle: "Workflow run unavailable",
      status: "completed",
      conclusion: null,
      statusCategory: "completed",
      event: "workflow_dispatch",
      actor: null,
      headBranch: "main",
      headSha: null,
      shortSha: null,
      pullRequest: null,
      commitMessage: null,
      jobSummary: {
        total: 0,
        queued: 0,
        inProgress: 0,
        completed: 0,
        cancelled: 0,
        success: 0,
        failure: 0,
        skipped: 0,
        timedOut: 0,
      },
      durationSeconds: null,
      isLive: false,
      startedAt: null,
      completedAt: null,
      createdAt: new Date(0).toISOString(),
      updatedAt: new Date(0).toISOString(),
    },
    jobs: [],
    job: {
      id: jobId,
      name: "Workflow job unavailable",
      groupName: null,
      attemptNumber: 1,
      status: "completed",
      conclusion: null,
      runnerLabel: null,
      durationSeconds: null,
      logAvailable: false,
      logDeletedAt: null,
      steps: [],
      startedAt: null,
      completedAt: null,
      createdAt: new Date(0).toISOString(),
      updatedAt: new Date(0).toISOString(),
    },
    steps: [],
    annotations: [],
    logState: {
      available: false,
      status: 410,
      reason: "Workflow job logs could not be loaded.",
      deletedAt: null,
      isLive: false,
      nextCursor: null,
    },
    search: {
      query: null,
      totalMatches: 0,
      selectedMatch: null,
      matches: [],
    },
    options: {
      showTimestamps: true,
      rawLogs: false,
      wrapLines: true,
    },
    downloadHref: `/api/repos/${ownerLogin}/${repositoryName}/actions/jobs/${jobId}/logs/download`,
  };
}
