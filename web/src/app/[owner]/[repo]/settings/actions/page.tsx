import { AppShell } from "@/components/AppShell";
import { RepositoryActionsRunnersPage } from "@/components/RepositoryActionsRunnersPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryActionsRunnerSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsActionsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySettingsActionsPage({
  params,
}: RepositorySettingsActionsPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsRunnerSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="actions"
          repository={repository}
          title="Actions"
        >
          <RepositoryActionsRunnersPage
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
