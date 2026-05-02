import { AppShell } from "@/components/AppShell";
import { RepositoryGeneralSettingsPage } from "@/components/RepositoryGeneralSettingsPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySettingsPage({
  params,
}: RepositorySettingsPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="general"
          repository={repository}
          title="General"
        >
          <RepositoryGeneralSettingsPage
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
