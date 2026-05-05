import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionCategoryChooser } from "@/components/RepositoryDiscussionCategoryChooser";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDiscussionCreation,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryDiscussionChoosePageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositoryDiscussionChoosePage({
  params,
}: RepositoryDiscussionChoosePageProps) {
  const [{ owner, repo }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, creation] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussionCreation(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && creation && !("error" in creation) ? (
        <RepositoryDiscussionCategoryChooser
          creation={creation}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
