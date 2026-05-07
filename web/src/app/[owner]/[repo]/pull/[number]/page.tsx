import { AppShell } from "@/components/AppShell";
import { RepositoryPullRequestDetailPage } from "@/components/RepositoryPullRequestDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getPullRequestAiSummary,
  getRepository,
  getRepositoryPullRequest,
  getRepositoryPullRequestTimeline,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PullRequestPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
  }>;
};

export default async function PullRequestPage({
  params,
}: PullRequestPageProps) {
  const { owner, repo, number } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const pullNumber = Number.parseInt(decodeURIComponent(number), 10);
  const [
    { session, shellContext },
    repository,
    pullRequest,
    timeline,
    aiSummary,
  ] = await Promise.all([
    getSessionAndShellContext(),
    getRepository(ownerLogin, repositoryName),
    Number.isFinite(pullNumber)
      ? getRepositoryPullRequest(ownerLogin, repositoryName, pullNumber)
      : Promise.resolve(null),
    Number.isFinite(pullNumber)
      ? getRepositoryPullRequestTimeline(ownerLogin, repositoryName, pullNumber)
      : Promise.resolve([]),
    Number.isFinite(pullNumber)
      ? getPullRequestAiSummary(ownerLogin, repositoryName, pullNumber)
      : Promise.resolve(null),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository &&
      pullRequest &&
      !("error" in pullRequest) &&
      !("error" in timeline) ? (
        <RepositoryPullRequestDetailPage
          pullRequest={pullRequest}
          aiSummary={aiSummary}
          repository={repository}
          timeline={timeline}
          viewerAuthenticated={session.authenticated}
        />
      ) : (
        <>
          {Number.isFinite(pullNumber) ? (
            <section className="mx-auto max-w-6xl px-6 pt-8">
              <h1 className="t-label">Pull request #{pullNumber}</h1>
            </section>
          ) : null}
          <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
        </>
      )}
    </AppShell>
  );
}
