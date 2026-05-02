import { AppShell } from "@/components/AppShell";
import { RepositoryBranchSettingsPage } from "@/components/RepositoryBranchSettingsPage";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryBranchSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsBranchesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ new?: string | string[] }>;
};

export default async function RepositorySettingsBranchesPage({
  params,
  searchParams,
}: RepositorySettingsBranchesPageProps) {
  const { owner, repo } = await params;
  const nextIntent = (await searchParams).new;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const intentValue = Array.isArray(nextIntent) ? nextIntent[0] : nextIntent;
  const intent =
    intentValue === "rule" || intentValue === "ruleset"
      ? intentValue
      : undefined;
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryBranchSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="branches"
          repository={repository}
          title="Branches"
        >
          <RepositoryBranchSettingsPage
            intent={intent}
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
