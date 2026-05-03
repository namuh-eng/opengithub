import { AppShell } from "@/components/AppShell";
import { RepositoryPagesSettingsPage } from "@/components/RepositoryPagesSettingsPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryPagesSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsPagesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySettingsPagesPage({
  params,
}: RepositorySettingsPagesPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryPagesSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="pages"
          repository={repository}
          title="Pages"
        >
          <RepositoryPagesSettingsPage
            repository={repository}
            settingsResult={settingsResult}
          />
        </RepositorySettingsShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
