import { AppShell } from "@/components/AppShell";
import { RepositorySecretScanningAlertsPage as RepositorySecretScanningAlertsView } from "@/components/RepositorySecretScanningAlertsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecretScanningAlerts,
  getSession,
} from "@/lib/server-session";

type RepositorySecretScanningAlertsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    state?: string;
    q?: string;
    provider?: string;
    secret_type?: string;
    validity?: string;
    resolution?: string;
    bypassed?: string;
    team?: string;
    topic?: string;
    sort?: string;
  }>;
};

export default async function RepositorySecretScanningAlertsPage({
  params,
  searchParams,
}: RepositorySecretScanningAlertsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, secretScanningResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecretScanningAlerts(ownerLogin, repositoryName, {
      state: query.state,
      query: query.q,
      provider: query.provider,
      secretType: query.secret_type,
      validity: query.validity,
      resolution: query.resolution,
      bypassed: query.bypassed,
      team: query.team,
      topic: query.topic,
      sort: query.sort,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecretScanningAlertsView
          repository={repository}
          secretScanningResult={secretScanningResult}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
