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
  getOrganizationWebhookSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationSettingsHooksPageProps = {
  params: Promise<{ org: string }>;
  searchParams: Promise<{ new?: string | string[] }>;
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

export default async function OrganizationSettingsHooksPage({
  params,
  searchParams,
}: OrganizationSettingsHooksPageProps) {
  const [{ org }, query, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const newValue = Array.isArray(query.new) ? query.new[0] : query.new;
  const [profileResult, hooksResult] = await Promise.all([
    getOrganizationProfileSettings(orgLogin),
    getOrganizationWebhookSettings(orgLogin),
  ]);

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
        title={`${orgLogin} webhooks could not load`}
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
        intent={newValue === "webhook" ? "new" : undefined}
        repository={organizationAsRepository(orgLogin)}
        scopeLabel="Organization webhooks"
        scopeNoun="organization"
        settingsResult={repositoryLikeSettings(hooksResult)}
      />
    </OrganizationSettingsShell>
  );
}
