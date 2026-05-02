import { AppShell } from "@/components/AppShell";
import { RepositorySettingsShell } from "@/components/RepositorySettingsShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWebhookSettingsPage } from "@/components/RepositoryWebhookSettingsPage";
import {
  getRepository,
  getRepositoryWebhookDeliveryDetail,
  getRepositoryWebhookDetail,
  getRepositoryWebhookSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryWebhookDetailPageProps = {
  params: Promise<{ hookId: string; owner: string; repo: string }>;
  searchParams: Promise<{ delivery?: string | string[] }>;
};

export default async function RepositoryWebhookDetailPage({
  params,
  searchParams,
}: RepositoryWebhookDetailPageProps) {
  const { hookId, owner, repo } = await params;
  const query = await searchParams;
  const selectedDelivery = Array.isArray(query.delivery)
    ? query.delivery[0]
    : query.delivery;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const webhookId = decodeURIComponent(hookId);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, settingsResult, detailResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryWebhookSettings(ownerLogin, repositoryName),
    getRepositoryWebhookDetail(ownerLogin, repositoryName, webhookId),
  ]);
  const deliveryResult = selectedDelivery
    ? await getRepositoryWebhookDeliveryDetail(
        ownerLogin,
        repositoryName,
        webhookId,
        selectedDelivery,
      )
    : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository ? (
        <RepositorySettingsShell
          activeSection="hooks"
          repository={repository}
          title="Webhooks"
        >
          <RepositoryWebhookSettingsPage
            deliveryResult={deliveryResult}
            detailResult={detailResult}
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
