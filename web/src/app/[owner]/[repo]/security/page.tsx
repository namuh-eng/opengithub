import { AppShell } from "@/components/AppShell";
import { RepositorySecurityOverviewPage as RepositorySecurityOverviewView } from "@/components/RepositorySecurityOverviewPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecurityOverview,
  getSession,
} from "@/lib/server-session";

type RepositorySecurityPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySecurityPage({
  params,
}: RepositorySecurityPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, securityResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecurityOverview(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecurityOverviewView
          repository={repository}
          securityResult={securityResult}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
