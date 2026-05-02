import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  activeOrganizationTab,
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationPageProps = {
  params: Promise<{ org: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function OrganizationPage({
  params,
  searchParams,
}: OrganizationPageProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const activeTab = activeOrganizationTab(firstParam(queryParams?.tab));
  const profile = await getPublicOrganizationProfile(orgLogin);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab={activeTab}
        profile={profile}
        session={session}
        shellContext={shellContext}
      />
    );
  }

  const activeTabLabel =
    ORGANIZATION_TABS.find((tab) => tab.value === activeTab)?.label ??
    "Overview";

  return (
    <ProfileOrgShell
      actions={[
        { href: organizationProjectHref(orgLogin), label: "Projects" },
        { href: organizationSettingsHref(orgLogin), label: "Settings" },
      ]}
      activeTab={activeTab}
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={`${activeTabLabel} for ${orgLogin} will show organization repositories, people, teams, and packages when the organization features are implemented. The skeleton keeps organization navigation grounded today.`}
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={orgLogin}
    />
  );
}
