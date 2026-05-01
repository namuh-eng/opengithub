import { AppShell } from "@/components/AppShell";
import { PullRequestFilesChangedPage } from "@/components/PullRequestFilesChangedPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { ApiErrorEnvelope } from "@/lib/api";
import {
  getPullRequestCompare,
  getRepository,
  getRepositoryPullRequest,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PullRequestFilesPageProps = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return Boolean(value && typeof value === "object" && "error" in value);
}

export default async function PullRequestFilesPage({
  params,
}: PullRequestFilesPageProps) {
  const [{ owner, repo, number }, { session, shellContext }] =
    await Promise.all([params, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const pullNumber = Number.parseInt(decodeURIComponent(number), 10);
  const [repository, pullRequest] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    Number.isFinite(pullNumber)
      ? getRepositoryPullRequest(ownerLogin, repositoryName, pullNumber)
      : Promise.resolve(null),
  ]);
  const compare =
    repository && pullRequest && !isApiError(pullRequest)
      ? await getPullRequestCompare(
          ownerLogin,
          repositoryName,
          pullRequest.baseRef,
          pullRequest.headRef,
          { commits: 25, files: 100 },
        )
      : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && pullRequest && !isApiError(pullRequest) ? (
        <PullRequestFilesChangedPage
          compare={compare && !isApiError(compare) ? compare : null}
          pullRequest={pullRequest}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
