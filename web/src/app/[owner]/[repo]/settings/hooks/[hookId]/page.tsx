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
  searchParams: Promise<{
    delete?: string | string[];
    delivery?: string | string[];
    edit?: string | string[];
    redeliver?: string | string[];
    test?: string | string[];
  }>;
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
  const editValue = Array.isArray(query.edit) ? query.edit[0] : query.edit;
  const testValue = Array.isArray(query.test) ? query.test[0] : query.test;
  const deleteValue = Array.isArray(query.delete)
    ? query.delete[0]
    : query.delete;
  const redeliverValue = Array.isArray(query.redeliver)
    ? query.redeliver[0]
    : query.redeliver;
  const intent =
    editValue === "webhook"
      ? "edit"
      : testValue === "ping"
        ? "ping"
        : deleteValue === "confirm"
          ? "delete"
          : redeliverValue === "confirm"
            ? "redeliver"
            : undefined;
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
