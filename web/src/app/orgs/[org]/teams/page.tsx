import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationTeams,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationTeamsRouteProps = {
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

export default async function OrganizationTeamsRoute({
  params,
  searchParams,
}: OrganizationTeamsRouteProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const teamsQuery = {
    q: firstParam(queryParams?.q),
    visibility: firstParam(queryParams?.visibility),
    page: numberParam(queryParams?.page),
    pageSize: numberParam(queryParams?.pageSize),
  };
  const [profile, teamsDirectory] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationTeams(orgLogin, teamsQuery),
  ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab="teams"
        profile={profile}
        session={session}
        shellContext={shellContext}
        teamsDirectory={teamsDirectory}
      />
    );
  }

  return (
    <ProfileOrgShell
      actions={[
        { href: organizationProjectHref(orgLogin), label: "Projects" },
        { href: organizationSettingsHref(orgLogin), label: "Settings" },
      ]}
      activeTab="teams"
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={
        teamsDirectory
          ? `Teams for ${orgLogin} are available, but the public organization profile could not be loaded.`
          : `Teams for ${orgLogin} require organization membership or an available organization profile.`
      }
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={orgLogin}
    />
  );
}
