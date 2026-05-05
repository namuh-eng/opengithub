import { AppShell } from "@/components/AppShell";
import { RepositorySecurityAdvisoriesPage as RepositorySecurityAdvisoriesView } from "@/components/RepositorySecurityAdvisoriesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecurityAdvisories,
  getSession,
} from "@/lib/server-session";

type RepositorySecurityAdvisoriesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    state?: string;
    q?: string;
    severity?: string;
    sort?: string;
    page?: string;
    page_size?: string;
  }>;
};

export default async function RepositorySecurityAdvisoriesPage({
  params,
  searchParams,
}: RepositorySecurityAdvisoriesPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, advisoriesResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecurityAdvisories(ownerLogin, repositoryName, {
      state: query.state,
      query: query.q,
      severity: query.severity,
      sort: query.sort,
      page: query.page,
      pageSize: query.page_size,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecurityAdvisoriesView
          advisoriesResult={advisoriesResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
