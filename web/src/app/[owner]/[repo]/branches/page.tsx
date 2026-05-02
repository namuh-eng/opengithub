import { AppShell } from "@/components/AppShell";
import { RepositoryBranchesPage } from "@/components/RepositoryBranchesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryRefs,
  getSession,
} from "@/lib/server-session";

type BranchesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function BranchesPage({ params }: BranchesPageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, refsEnvelope] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryRefs(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryBranchesPage
          refs={refsEnvelope?.items ?? []}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
