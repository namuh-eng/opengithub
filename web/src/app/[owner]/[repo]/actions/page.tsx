import { AppShell } from "@/components/AppShell";
import { RepositoryActionsPage as RepositoryActionsView } from "@/components/RepositoryActionsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { RepositoryActionsDashboardQuery } from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsDashboard,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryActionsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

export default async function RepositoryActionsPage({
  params,
  searchParams,
}: RepositoryActionsPageProps) {
  const { owner, repo } = await params;
  const rawSearchParams = await searchParams;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const normalizedQuery = normalizeActionsQuery(rawSearchParams);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, dashboard] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsDashboard(ownerLogin, repositoryName, normalizedQuery),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && !("error" in dashboard) ? (
        <RepositoryActionsView
          dashboard={dashboard}
          query={normalizedQuery}
          repository={repository}
        />
      ) : repository && "error" in dashboard ? (
        <RepositoryActionsView
          dashboard={emptyDashboard(ownerLogin, repositoryName)}
          query={normalizedQuery}
          repository={repository}
          validationError={dashboard}
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

function normalizeActionsQuery(
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
    workflow: firstParam(searchParams.workflow),
  };
}

function emptyDashboard(ownerLogin: string, repositoryName: string) {
  return {
    repository: {
      id: "unavailable",
      ownerLogin,
      name: repositoryName,
      visibility: "public" as const,
      defaultBranch: "main",
    },
    viewerPermission: null,
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
    emptyState: {
      hasRuns: false,
      hasWorkflows: false,
      message: "Repository Actions could not be loaded.",
      newWorkflowHref: `/${ownerLogin}/${repositoryName}/new/main/.github/workflows`,
    },
  };
}
