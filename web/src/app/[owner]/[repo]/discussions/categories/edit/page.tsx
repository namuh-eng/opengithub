import { AppShell } from "@/components/AppShell";
import { RepositoryDiscussionCategorySettingsPage } from "@/components/RepositoryDiscussionCategorySettingsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDiscussionCategorySettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryDiscussionCategoryEditPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositoryDiscussionCategoryEditPage({
  params,
}: RepositoryDiscussionCategoryEditPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settings] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussionCategorySettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositoryDiscussionCategorySettingsPage
          repository={repository}
          settings={settings}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
