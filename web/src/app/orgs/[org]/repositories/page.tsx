import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationRepositories,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationRepositoriesRouteProps = {
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

export default async function OrganizationRepositoriesRoute({
  params,
  searchParams,
}: OrganizationRepositoriesRouteProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const [profile, repositoryList] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationRepositories(orgLogin, {
      q: firstParam(queryParams?.q),
      type: firstParam(queryParams?.type),
      language: firstParam(queryParams?.language),
      sort: firstParam(queryParams?.sort),
      density: firstParam(queryParams?.density),
      page: numberParam(queryParams?.page),
      pageSize: numberParam(queryParams?.pageSize),
    }),
  ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab="repositories"
        profile={profile}
        repositoryList={repositoryList}
        session={session}
        shellContext={shellContext}
      />
    );
  }

  return (
    <ProfileOrgShell
      actions={[
        { href: organizationProjectHref(orgLogin), label: "Projects" },
        { href: organizationSettingsHref(orgLogin), label: "Settings" },
      ]}
      activeTab="repositories"
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={`Repositories for ${orgLogin} will show organization-owned repositories when this organization is available.`}
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={orgLogin}
    />
  );
}
