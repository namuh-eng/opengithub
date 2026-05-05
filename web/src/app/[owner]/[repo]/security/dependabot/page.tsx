import { AppShell } from "@/components/AppShell";
import { RepositoryDependabotAlertsPage as RepositoryDependabotAlertsView } from "@/components/RepositoryDependabotAlertsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDependabotAlerts,
  getSession,
} from "@/lib/server-session";

type RepositoryDependabotAlertsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    state?: string;
    q?: string;
    package?: string;
    ecosystem?: string;
    manifest?: string;
    scope?: string;
    severity?: string;
    sort?: string;
  }>;
};

export default async function RepositoryDependabotAlertsPage({
  params,
  searchParams,
}: RepositoryDependabotAlertsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, dependabotResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDependabotAlerts(ownerLogin, repositoryName, {
      state: query.state,
      query: query.q,
      package: query.package,
      ecosystem: query.ecosystem,
      manifest: query.manifest,
      scope: query.scope,
      severity: query.severity,
      sort: query.sort,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryDependabotAlertsView
          dependabotResult={dependabotResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
