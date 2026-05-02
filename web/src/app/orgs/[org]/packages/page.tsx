import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationPackages,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationPackagesPageProps = {
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

export default async function OrganizationPackagesPage({
  params,
  searchParams,
}: OrganizationPackagesPageProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const [profile, packageList] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationPackages(orgLogin, {
      q: firstParam(queryParams?.q),
      type: firstParam(queryParams?.type),
      visibility: firstParam(queryParams?.visibility),
      sort: firstParam(queryParams?.sort),
      artifactTab: firstParam(queryParams?.artifactTab),
      page: numberParam(queryParams?.page),
      pageSize: numberParam(queryParams?.pageSize),
    }),
  ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab="packages"
        packageList={packageList}
        profile={profile}
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
      activeTab="packages"
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={`Packages for ${orgLogin} are unavailable. The organization may not exist yet, or the organization API could not be reached.`}
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={orgLogin}
    />
  );
}
