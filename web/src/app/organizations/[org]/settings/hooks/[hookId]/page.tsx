import { OrganizationSettingsShell } from "@/components/OrganizationSettingsShell";
import { PlaceholderPage } from "@/components/PlaceholderPage";
import { RepositoryWebhookSettingsPage } from "@/components/RepositoryWebhookSettingsPage";
import type {
  OrganizationWebhookSettingsFetchResult,
  RepositoryOverview,
  RepositoryWebhookSettingsFetchResult,
} from "@/lib/api";
import { organizationHref } from "@/lib/navigation";
import {
  getOrganizationProfileSettings,
  getOrganizationWebhookDeliveryDetail,
  getOrganizationWebhookDetail,
  getOrganizationWebhookSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationWebhookDetailPageProps = {
  params: Promise<{ hookId: string; org: string }>;
  searchParams: Promise<{
    delete?: string | string[];
    delivery?: string | string[];
    edit?: string | string[];
    redeliver?: string | string[];
    test?: string | string[];
  }>;
};

function organizationAsRepository(org: string): RepositoryOverview {
  return {
    name: org,
    owner_login: org,
  } as RepositoryOverview;
}

function repositoryLikeSettings(
  result: OrganizationWebhookSettingsFetchResult,
): RepositoryWebhookSettingsFetchResult {
  if (!result.ok) return result;
  return {
    ok: true,
    settings: {
      canEdit: result.settings.canEdit,
      eventDefinitions: result.settings.eventDefinitions,
      hooks: result.settings.hooks,
      name: result.settings.name,
      ownerLogin: result.settings.slug,
      repositoryId: result.settings.organizationId,
      viewerPermission: result.settings.viewerRole,
      visibility: "private",
    },
  };
}

export default async function OrganizationWebhookDetailPage({
  params,
  searchParams,
}: OrganizationWebhookDetailPageProps) {
  const [{ hookId, org }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const orgLogin = decodeURIComponent(org);
  const webhookId = decodeURIComponent(hookId);
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
  const [profileResult, hooksResult, detailResult] = await Promise.all([
    getOrganizationProfileSettings(orgLogin),
    getOrganizationWebhookSettings(orgLogin),
    getOrganizationWebhookDetail(orgLogin, webhookId),
  ]);
  const deliveryResult = selectedDelivery
    ? await getOrganizationWebhookDeliveryDetail(
        orgLogin,
        webhookId,
        selectedDelivery,
      )
    : null;

  if (!profileResult.ok) {
    return (
      <PlaceholderPage
        actions={[
          { href: organizationHref(orgLogin), label: "Organization profile" },
          { href: "/dashboard", label: "Dashboard" },
        ]}
        eyebrow="Organization webhooks"
        message={profileResult.message}
        session={session}
        shellContext={shellContext}
        title={`${orgLogin} webhook could not load`}
      />
    );
  }

  return (
    <OrganizationSettingsShell
      activeSection="hooks"
      session={session}
      settings={profileResult.settings}
      shellContext={shellContext}
      title="Webhooks"
    >
      <RepositoryWebhookSettingsPage
        basePath={`/organizations/${orgLogin}/settings/hooks`}
        deliveryResult={deliveryResult}
        detailResult={detailResult}
        intent={intent}
        repository={organizationAsRepository(orgLogin)}
        scopeLabel="Organization webhooks"
        scopeNoun="organization"
        settingsResult={repositoryLikeSettings(hooksResult)}
      />
    </OrganizationSettingsShell>
  );
}
