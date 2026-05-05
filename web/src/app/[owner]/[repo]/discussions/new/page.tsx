import { redirect } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionCreatePage } from "@/components/RepositoryDiscussionCreatePage";
import { RepositoryShell } from "@/components/RepositoryShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { repositoryDiscussionChooseCategoryHref } from "@/lib/navigation";
import {
  getRepository,
  getRepositoryDiscussionCreation,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryNewDiscussionPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ category?: string; q?: string; next?: string }>;
};

export default async function RepositoryNewDiscussionPage({
  params,
  searchParams,
}: RepositoryNewDiscussionPageProps) {
  const [{ owner, repo }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const category = query.category?.trim();

  if (!category) {
    redirect(
      repositoryDiscussionChooseCategoryHref(ownerLogin, repositoryName),
    );
  }

  const [repository, creation] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussionCreation(ownerLogin, repositoryName, { category }),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && creation && !("error" in creation) ? (
        <RepositoryShell
          activePath={`/${ownerLogin}/${repositoryName}/discussions`}
          frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
          repository={repository}
        >
          <RepositoryDiscussionCreatePage
            creation={creation}
            owner={ownerLogin}
            repo={repositoryName}
          />
        </RepositoryShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
