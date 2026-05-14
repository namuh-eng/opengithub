import { AppShell } from "@/components/AppShell";
import { RepositoryActionsRunPage as RepositoryActionsRunView } from "@/components/RepositoryActionsRunPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type {
  ActionsJobLog,
  RepositoryActionsJobLogDetail,
  RepositoryActionsRunDetail,
} from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsJobLogDetail,
  getRepositoryActionsRunDetail,
  getSessionAndShellContext,
} from "@/lib/server-session";

type ActionRunPageProps = {
  params: Promise<{ owner: string; repo: string; runId: string }>;
};

export default async function ActionRunPage({ params }: ActionRunPageProps) {
  const { owner, repo, runId } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedRunId = decodeURIComponent(runId);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, detail] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsRunDetail(ownerLogin, repositoryName, decodedRunId),
  ]);
  const initialJobLog =
    !("error" in detail) && detail.jobs[0]?.logAvailable
      ? await getInitialJobLog(
          ownerLogin,
          repositoryName,
          decodedRunId,
          detail.jobs[0].id,
        )
      : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && !("error" in detail) ? (
        <RepositoryActionsRunView
          detail={detail}
          initialJobLog={initialJobLog}
          repository={repository}
        />
      ) : repository && "error" in detail ? (
        <RepositoryActionsRunView
          detail={emptyRunDetail(ownerLogin, repositoryName, decodedRunId)}
          initialJobLog={null}
          repository={repository}
          validationError={detail}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

async function getInitialJobLog(
  ownerLogin: string,
  repositoryName: string,
  runId: string,
  jobId: string,
): Promise<ActionsJobLog | null> {
  const detail = await getRepositoryActionsJobLogDetail(
    ownerLogin,
    repositoryName,
    runId,
    jobId,
    { pageSize: 100 },
  );
  if ("error" in detail) {
    return null;
  }
  return jobLogFromDetail(detail);
}

function jobLogFromDetail(
  detail: RepositoryActionsJobLogDetail,
): ActionsJobLog {
  const lines = detail.steps
    .flatMap((step) => step.lines.items)
    .sort((left, right) => left.lineNumber - right.lineNumber);

  return {
    job: {
      id: detail.job.id,
      runId: detail.run.id,
      name: detail.job.name,
      status: detail.job.status,
      conclusion: detail.job.conclusion,
      logDeletedAt: detail.job.logDeletedAt,
    },
    lines,
    total: lines.length,
    page: 1,
    pageSize: Math.max(lines.length, 1),
    query: null,
    downloadHref: detail.downloadHref,
  };
}

function emptyRunDetail(
  ownerLogin: string,
  repositoryName: string,
  runId: string,
): RepositoryActionsRunDetail {
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
    runtimePolicy: {
      secretCount: 0,
      variableCount: 0,
      blockedSecretCount: 0,
      blockedVariableCount: 0,
      scopes: [],
      blockedReasons: [],
      redactionMarker: "::add-mask::***",
    },
    attempts: [],
    jobs: [],
    annotations: [],
    artifacts: [],
    actionState: {
      canRerun: false,
      canRerunFailed: false,
      canCancel: false,
      canDeleteLogs: false,
      disabledReason: "Workflow run details could not be loaded.",
    },
  };
}
