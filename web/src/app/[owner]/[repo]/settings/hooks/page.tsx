import { AppShell } from "@/components/AppShell";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWebhookSettingsPage } from "@/components/RepositoryWebhookSettingsPage";
import {
  getRepository,
  getRepositoryWebhookSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsHooksPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ new?: string | string[] }>;
};

export default async function RepositorySettingsHooksPage({
  params,
  searchParams,
}: RepositorySettingsHooksPageProps) {
  const { owner, repo } = await params;
  const query = await searchParams;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const newValue = Array.isArray(query.new) ? query.new[0] : query.new;
  const intent = newValue === "webhook" ? "new" : undefined;
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryWebhookSettings(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="hooks"
          repository={repository}
          title="Webhooks"
        >
          <RepositoryWebhookSettingsPage
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
