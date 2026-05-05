import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionDetailPage as RepositoryDiscussionDetailView } from "@/components/RepositoryDiscussionDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDiscussionDetail,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryDiscussionDetailPageProps = {
  params: Promise<{ owner: string; repo: string; number: string }>;
  searchParams: Promise<{
    sort?: string;
    page?: string;
    page_size?: string;
  }>;
};

export default async function RepositoryDiscussionDetailPage({
  params,
  searchParams,
}: RepositoryDiscussionDetailPageProps) {
  const [{ owner, repo, number }, query, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const discussionNumber = Number.parseInt(decodeURIComponent(number), 10);
  const page = Number.parseInt(query.page ?? "1", 10);
  const pageSize = Number.parseInt(query.page_size ?? "30", 10);

  const [repository, discussion] = Number.isFinite(discussionNumber)
    ? await Promise.all([
        getRepository(ownerLogin, repositoryName),
        getRepositoryDiscussionDetail(
          ownerLogin,
          repositoryName,
          discussionNumber,
          {
            sort: query.sort,
            page: Number.isFinite(page) ? page : 1,
            pageSize: Number.isFinite(pageSize) ? pageSize : 30,
          },
        ),
      ])
    : [null, null];

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && discussion && !("error" in discussion) ? (
        <RepositoryDiscussionDetailView
          detail={discussion}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
