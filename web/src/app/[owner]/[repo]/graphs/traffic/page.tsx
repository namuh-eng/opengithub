import { AppShell } from "@/components/AppShell";
import { RepositoryTrafficPage as RepositoryTrafficView } from "@/components/RepositoryTrafficPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryTraffic,
  getSession,
} from "@/lib/server-session";

type RepositoryTrafficPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositoryTrafficPage({
  params,
}: RepositoryTrafficPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, trafficResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryTraffic(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryTrafficView
          repository={repository}
          trafficResult={trafficResult}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
