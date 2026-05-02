import { AppShell } from "@/components/AppShell";
import { RepositoryAccessSettingsPage } from "@/components/RepositoryAccessSettingsPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryAccessSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsAccessPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ q?: string | string[] }>;
};

export default async function RepositorySettingsAccessPage({
  params,
  searchParams,
}: RepositorySettingsAccessPageProps) {
  const { owner, repo } = await params;
  const query = (await searchParams).q;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryAccessSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="access"
          repository={repository}
          title="Access"
        >
          <RepositoryAccessSettingsPage
            query={Array.isArray(query) ? query[0] : query}
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
