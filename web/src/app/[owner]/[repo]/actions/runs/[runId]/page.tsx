import { AppShell } from "@/components/AppShell";
import { RepositoryActionsRunPage as RepositoryActionsRunView } from "@/components/RepositoryActionsRunPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { ActionsJobLog, RepositoryActionsRunDetail } from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsJobLog,
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

  const initialJobLog = await initialVisibleJobLog(
    ownerLogin,
    repositoryName,
    detail,
  );

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
          repository={repository}
          validationError={detail}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
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

async function initialVisibleJobLog(
  ownerLogin: string,
  repositoryName: string,
  detail: RepositoryActionsRunDetail | { error: unknown },
): Promise<ActionsJobLog | null> {
  if ("error" in detail) {
    return null;
  }
  const job = detail.jobs[0];
  if (!job?.logAvailable) {
    return null;
  }
  const log = await getRepositoryActionsJobLog(
    ownerLogin,
    repositoryName,
    job.id,
  );
  return "error" in log ? null : log;
}
