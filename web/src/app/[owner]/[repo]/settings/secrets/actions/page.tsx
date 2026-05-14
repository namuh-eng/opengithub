import { AppShell } from "@/components/AppShell";
import { RepositoryActionsSecretsPage } from "@/components/RepositoryActionsSecretsPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryActionsSecretsSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsActionsSecretsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ tab?: string | string[] }>;
};

export default async function RepositorySettingsActionsSecretsPage({
  params,
  searchParams,
}: RepositorySettingsActionsSecretsPageProps) {
  const { owner, repo } = await params;
  const tabValue = (await searchParams).tab;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const activeTab = Array.isArray(tabValue) ? tabValue[0] : tabValue;
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsSecretsSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="secrets"
          repository={repository}
          title="Actions secrets"
        >
          <RepositoryActionsSecretsPage
            activeTab={activeTab === "variables" ? "variables" : "secrets"}
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
