import { AppShell } from "@/components/AppShell";
import { ProjectsListPage } from "@/components/ProjectsListPage";
import { RepositoryShell } from "@/components/RepositoryShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryProjects,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryProjectsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function numberParam(value: string | string[] | undefined) {
  const raw = firstParam(value);
  if (!raw) {
    return undefined;
  }
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : undefined;
}

export default async function RepositoryProjectsPage({
  params,
  searchParams,
}: RepositoryProjectsPageProps) {
  const [{ owner, repo }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, projectResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryProjects(ownerLogin, repositoryName, {
      q: firstParam(queryParams?.q),
      state: firstParam(queryParams?.state),
      tab: firstParam(queryParams?.tab),
      sort: firstParam(queryParams?.sort),
      page: numberParam(queryParams?.page),
      pageSize: numberParam(queryParams?.pageSize),
    }),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && projectResult.ok ? (
        <RepositoryShell
          activePath={`/${ownerLogin}/${repositoryName}/projects`}
          frameClassName="max-w-[1240px]"
          repository={repository}
        >
          <ProjectsListPage
            list={projectResult.projects}
            scopeLabel={`${repository.owner_login}/${repository.name} projects`}
          />
        </RepositoryShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
