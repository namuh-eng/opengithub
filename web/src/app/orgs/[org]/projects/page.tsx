import { OrganizationProfilePage } from "@/components/OrganizationProfilePage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  ORGANIZATION_TABS,
  organizationSettingsHref,
  organizationTabHref,
} from "@/lib/navigation";
import {
  getOrganizationProjects,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectsPageProps = {
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

export default async function OrganizationProjectsPage({
  params,
  searchParams,
}: OrganizationProjectsPageProps) {
  const [{ org }, queryParams, { session, shellContext }] = await Promise.all([
    params,
    searchParams,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const [profile, projectResult] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationProjects(orgLogin, {
      q: firstParam(queryParams?.q),
      state: firstParam(queryParams?.state),
      tab: firstParam(queryParams?.tab),
      sort: firstParam(queryParams?.sort),
      page: numberParam(queryParams?.page),
      pageSize: numberParam(queryParams?.pageSize),
    }),
  ]);

  if (profile) {
    return (
      <OrganizationProfilePage
        activeTab="projects"
        profile={profile}
        projectList={projectResult.ok ? projectResult.projects : null}
        session={session}
        shellContext={shellContext}
      />
    );
  }

  return (
    <ProfileOrgShell
      actions={[
        { href: organizationSettingsHref(orgLogin), label: "Settings" },
      ]}
      activeTab="projects"
      eyebrow="Organization"
      hrefForTab={(value) => organizationTabHref(orgLogin, value)}
      identityLabel={orgLogin}
      message={
        projectResult.ok
          ? `Projects for ${orgLogin} are available, but the organization profile shell could not be loaded.`
          : projectResult.message
      }
      session={session}
      shellContext={shellContext}
      tabLabel="Organization sections"
      tabs={ORGANIZATION_TABS}
      title={`${orgLogin} projects`}
    />
  );
}
