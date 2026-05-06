import { AppShell } from "@/components/AppShell";
import { RepositoryPullRequestChecksPage } from "@/components/RepositoryPullRequestChecksPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryPullRequestChecks,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PullRequestChecksPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
  }>;
};

export default async function PullRequestChecksRoute({
  params,
}: PullRequestChecksPageProps) {
  const { owner, repo, number } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const pullNumber = Number.parseInt(decodeURIComponent(number), 10);
  const [{ session, shellContext }, repository, checks] = await Promise.all([
    getSessionAndShellContext(),
    getRepository(ownerLogin, repositoryName),
    Number.isFinite(pullNumber)
      ? getRepositoryPullRequestChecks(ownerLogin, repositoryName, pullNumber)
      : Promise.resolve(null),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && checks && !("error" in checks) ? (
        <RepositoryPullRequestChecksPage
          checks={checks}
          repository={repository}
          viewerAuthenticated={session.authenticated}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
