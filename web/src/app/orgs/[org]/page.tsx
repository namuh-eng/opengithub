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
  getOrganizationPackages,
  getOrganizationPeople,
  getOrganizationPeopleAdmin,
  getOrganizationRepositories,
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

function numberParam(value: string | string[] | undefined) {
  const raw = firstParam(value);
  if (!raw) {
    return undefined;
  }
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : undefined;
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
  const peopleQuery = {
    q: firstParam(queryParams?.q),
    page: numberParam(queryParams?.page),
    pageSize: numberParam(queryParams?.pageSize),
  };
  const [profile, repositoryList, peopleList, adminPeople, packageList] =
    await Promise.all([
      getPublicOrganizationProfile(orgLogin),
      activeTab === "repositories"
        ? getOrganizationRepositories(orgLogin, {
            q: firstParam(queryParams?.q),
            type: firstParam(queryParams?.type),
            language: firstParam(queryParams?.language),
            sort: firstParam(queryParams?.sort),
            density: firstParam(queryParams?.density),
            page: numberParam(queryParams?.page),
            pageSize: numberParam(queryParams?.pageSize),
          })
        : Promise.resolve(null),
      activeTab === "people"
        ? getOrganizationPeople(orgLogin, peopleQuery)
        : Promise.resolve(null),
      activeTab === "people"
        ? getOrganizationPeopleAdmin(orgLogin, peopleQuery)
        : Promise.resolve(null),
      activeTab === "packages"
        ? getOrganizationPackages(orgLogin, {
            q: firstParam(queryParams?.q),
            type: firstParam(queryParams?.type),
            visibility: firstParam(queryParams?.visibility),
            sort: firstParam(queryParams?.sort),
            artifactTab: firstParam(queryParams?.artifactTab),
            page: numberParam(queryParams?.page),
            pageSize: numberParam(queryParams?.pageSize),
          })
        : Promise.resolve(null),
    ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab={activeTab}
        adminPeople={adminPeople}
        peopleList={peopleList}
        profile={profile}
        packageList={packageList}
        repositoryList={repositoryList}
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
