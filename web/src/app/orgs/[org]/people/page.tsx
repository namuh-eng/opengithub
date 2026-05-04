import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import type { OrganizationPeopleAdminTabParam } from "@/lib/api";
import {
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationPeople,
  getOrganizationPeopleAdmin,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationPeopleRouteProps = {
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

function adminTabParam(
  value: string | string[] | undefined,
): OrganizationPeopleAdminTabParam | undefined {
  const raw = firstParam(value);
  if (
    raw === "members" ||
    raw === "outside_collaborators" ||
    raw === "pending_collaborators" ||
    raw === "invitations" ||
    raw === "failed_invitations" ||
    raw === "security_managers"
  ) {
    return raw;
  }
  return undefined;
}

export default async function OrganizationPeopleRoute({
  params,
  searchParams,
}: OrganizationPeopleRouteProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const peopleQuery = {
    q: firstParam(queryParams?.q),
    page: numberParam(queryParams?.page),
    pageSize: numberParam(queryParams?.pageSize),
  };
  const [profile, peopleList, adminPeople] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationPeople(orgLogin, peopleQuery),
    getOrganizationPeopleAdmin(orgLogin, {
      ...peopleQuery,
      tab: adminTabParam(queryParams?.tab),
    }),
  ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab="people"
        adminPeople={adminPeople}
        peopleList={peopleList}
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
      activeTab="people"
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={`People for ${orgLogin} will show organization members when this organization is available.`}
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={orgLogin}
    />
  );
}
