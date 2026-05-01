import { AppShell } from "@/components/AppShell";
import { PullRequestFilesChangedPage } from "@/components/PullRequestFilesChangedPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { ApiErrorEnvelope } from "@/lib/api";
import {
  getRepository,
  getRepositoryPullRequestFiles,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PullRequestFilesPageProps = {
  params: Promise<{ owner: string; repo: string; number: string }>;
  searchParams: Promise<{
    view?: string;
    whitespace?: string;
    commit?: string;
    filter?: string;
    page?: string;
    pageSize?: string;
  }>;
};

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return Boolean(value && typeof value === "object" && "error" in value);
}

export default async function PullRequestFilesPage({
  params,
  searchParams,
}: PullRequestFilesPageProps) {
  const [{ owner, repo, number }, query, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const pullNumber = Number.parseInt(decodeURIComponent(number), 10);
  const [repository, diffReview] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    Number.isFinite(pullNumber)
      ? getRepositoryPullRequestFiles(ownerLogin, repositoryName, pullNumber, {
          view: query.view,
          whitespace: query.whitespace,
          commit: query.commit,
          filter: query.filter,
          page: query.page ? Number.parseInt(query.page, 10) : undefined,
          pageSize: query.pageSize
            ? Number.parseInt(query.pageSize, 10)
            : undefined,
        })
      : Promise.resolve(null),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && diffReview && !isApiError(diffReview) ? (
        <PullRequestFilesChangedPage
          diffReview={diffReview}
          repository={repository}
          viewerAuthenticated={Boolean(session?.user)}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
