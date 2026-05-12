import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionsPage as RepositoryDiscussionsView } from "@/components/RepositoryDiscussionsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDiscussions,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryDiscussionCategoryPageProps = {
  params: Promise<{ owner: string; repo: string; slug: string }>;
  searchParams: Promise<{
    q?: string;
    discussions_q?: string;
    label?: string;
    state?: string;
    answered?: string;
    locked?: string;
    pinned?: string;
    sort?: string;
    page?: string;
    page_size?: string;
  }>;
};

function booleanParam(value: string | undefined) {
  if (value === "true") return true;
  if (value === "false") return false;
  return undefined;
}

export default async function RepositoryDiscussionCategoryPage({
  params,
  searchParams,
}: RepositoryDiscussionCategoryPageProps) {
  const [{ owner, repo, slug }, query, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const categorySlug = decodeURIComponent(slug);
  const page = Number.parseInt(query.page ?? "1", 10);
  const pageSize = Number.parseInt(query.page_size ?? "30", 10);
  const discussionQuery = {
    q: query.discussions_q ?? query.q,
    label: query.label,
    state: query.state,
    answered: booleanParam(query.answered),
    locked: booleanParam(query.locked),
    pinned: booleanParam(query.pinned),
    sort: query.sort,
    page: Number.isFinite(page) ? page : 1,
    pageSize: Number.isFinite(pageSize) ? pageSize : 30,
  };
  const [repository, discussions] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussions(
      ownerLogin,
      repositoryName,
      discussionQuery,
      categorySlug,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && discussions && !("error" in discussions) ? (
        <RepositoryDiscussionsView
          discussions={discussions}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
