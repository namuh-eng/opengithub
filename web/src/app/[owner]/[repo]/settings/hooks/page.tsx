import { AppShell } from "@/components/AppShell";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { WebhooksSettingsPage } from "@/components/WebhooksSettingsPage";
import {
  getRepository,
  getRepositoryWebhooks,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositorySettingsHooksPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function RepositorySettingsHooksPage({
  params,
}: RepositorySettingsHooksPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [{ session, shellContext }, repository, catalog] = await Promise.all([
    getSessionAndShellContext(),
    getRepository(ownerLogin, repositoryName),
    getRepositoryWebhooks(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="hooks"
          repository={repository}
          title="Webhooks"
        >
          <WebhooksSettingsPage
            catalog={catalog}
            endpointBase={`/api/repos/${encodeURIComponent(ownerLogin)}/${encodeURIComponent(repositoryName)}/hooks`}
            ownerLabel={`${ownerLogin} / ${repositoryName}`}
          />
        </RepositorySettingsShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
