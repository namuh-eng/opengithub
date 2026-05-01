import { AppShell } from "@/components/AppShell";
import { RepositoryActionsWorkflowPage as RepositoryActionsWorkflowView } from "@/components/RepositoryActionsWorkflowPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type {
  RepositoryActionsDashboardQuery,
  RepositoryActionsWorkflowDetail,
} from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsWorkflowDashboard,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryActionsWorkflowPageProps = {
  params: Promise<{ owner: string; repo: string; workflowFile: string[] }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

export default async function RepositoryActionsWorkflowPage({
  params,
  searchParams,
}: RepositoryActionsWorkflowPageProps) {
  const { owner, repo, workflowFile } = await params;
  const rawSearchParams = await searchParams;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const workflowPath = workflowFile.map(decodeURIComponent).join("/");
  const normalizedQuery = normalizeWorkflowActionsQuery(rawSearchParams);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, detail] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsWorkflowDashboard(
      ownerLogin,
      repositoryName,
      workflowPath,
      normalizedQuery,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && !("error" in detail) ? (
        <RepositoryActionsWorkflowView
          detail={detail}
          query={normalizedQuery}
          repository={repository}
        />
      ) : repository && "error" in detail ? (
        <RepositoryActionsWorkflowView
          detail={emptyWorkflowDetail(ownerLogin, repositoryName, workflowPath)}
          query={normalizedQuery}
          repository={repository}
          validationError={detail}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function normalizeWorkflowActionsQuery(
  searchParams: Record<string, string | string[] | undefined>,
): RepositoryActionsDashboardQuery & Record<string, string | undefined> {
  return {
    actor: firstParam(searchParams.actor),
    branch: firstParam(searchParams.branch),
    event: firstParam(searchParams.event),
    page: firstParam(searchParams.page),
    pageSize: firstParam(searchParams.pageSize),
    q: firstParam(searchParams.q),
    showWorkflows: firstParam(searchParams.showWorkflows),
    status: firstParam(searchParams.status),
  };
}

function emptyWorkflowDetail(
  ownerLogin: string,
  repositoryName: string,
  workflowPath: string,
): RepositoryActionsWorkflowDetail {
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
      name: workflowPath.split("/").at(-1) ?? "Workflow",
      path: workflowPath,
      state: "active",
      triggerEvents: [],
      sourceBranch: "main",
      sourceSha: null,
      sourceBlobId: null,
      sourceHref: `/${ownerLogin}/${repositoryName}/blob/main/${workflowPath}`,
      dispatch: {
        enabled: false,
        inputs: [],
      },
      yamlParseError: null,
      yamlParsedAt: new Date(0).toISOString(),
      valid: true,
    },
    workflows: [],
    runs: {
      items: [],
      total: 0,
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
      actors: [],
      branches: [],
      events: [],
      statuses: [],
      workflows: [],
    },
    refs: [],
    emptyState: {
      hasRuns: false,
      hasWorkflows: true,
      message: "Workflow Actions could not be loaded.",
      newWorkflowHref: `/${ownerLogin}/${repositoryName}/new/main/.github/workflows`,
    },
  };
}
