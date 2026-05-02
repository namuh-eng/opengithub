import { OrganizationOverviewPage } from "@/components/OrganizationOverviewPage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  activeOrganizationTab,
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationOverview,
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
  const organization = await getOrganizationOverview(orgLogin);

  if (organization && activeTab === "overview") {
    return (
      <OrganizationOverviewPage
        activeTab={activeTab}
        organization={organization}
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
      message={
        organization
          ? `${activeTabLabel} for ${organization.displayName} is linked from the organization overview and will expand as project, package, people, and team routes are completed.`
          : `${activeTabLabel} for ${orgLogin} will show organization repositories, people, teams, and packages when the organization APIs return data for this slug.`
      }
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={organization?.displayName ?? orgLogin}
    />
  );
}
