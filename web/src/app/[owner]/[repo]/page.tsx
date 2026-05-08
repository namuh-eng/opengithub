import { AppShell } from "@/components/AppShell";
import { RepositoryCodeOverview } from "@/components/RepositoryCodeOverview";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryAiSummary,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryOverviewPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

export default async function RepositoryOverviewPage({
  params,
}: RepositoryOverviewPageProps) {
  const [{ owner, repo }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const aiSummary = repository
    ? await getRepositoryAiSummary(ownerLogin, repositoryName)
    : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositoryCodeOverview aiSummary={aiSummary} repository={repository} />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
