import { AppShell } from "@/components/AppShell";
import { RepositoryNetworkPage as RepositoryNetworkView } from "@/components/RepositoryNetworkPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryNetwork,
  getSession,
} from "@/lib/server-session";

type RepositoryNetworkPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositoryNetworkPage({
  params,
}: RepositoryNetworkPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, networkResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryNetwork(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryNetworkView
          networkResult={networkResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
