import { OrganizationTeamDetailPage } from "@/components/OrganizationTeamDetailPage";
import { PlaceholderPage } from "@/components/PlaceholderPage";
import { ProfileOrgShell } from "@/components/ProfileOrgShell";
import {
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
  organizationTeamsHref,
} from "@/lib/navigation";
import {
  getOrganizationTeamDetail,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type TeamPageProps = {
  params: Promise<{ org: string; teamSlug: string }>;
};

export default async function TeamPage({ params }: TeamPageProps) {
  const [{ org, teamSlug }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const teamLogin = decodeURIComponent(teamSlug);
  const [profile, detail] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationTeamDetail(orgLogin, teamLogin),
  ]);

  if (detail) {
    return (
      <ProfileOrgShell
        actions={[
          { href: organizationProjectHref(orgLogin), label: "Projects" },
          { href: organizationSettingsHref(orgLogin), label: "Settings" },
        ]}
        activeTab="teams"
        eyebrow="Organization"
        hrefForTab={(value) => organizationTabHref(orgLogin, value)}
        identityLabel={profile?.identity.name ?? detail.organization.name}
        message="Team membership, repository access, mention delivery, and nested permissions are connected to this organization."
        session={session}
        shellContext={shellContext}
        tabLabel="Organization sections"
        tabs={ORGANIZATION_TABS}
        title={profile?.identity.name ?? detail.organization.name}
      >
        <OrganizationTeamDetailPage detail={detail} org={orgLogin} />
      </ProfileOrgShell>
    );
  }

  return (
    <PlaceholderPage
      actions={[
        { href: organizationTeamsHref(orgLogin), label: "All teams" },
        {
          href: organizationSettingsHref(orgLogin),
          label: "Organization settings",
        },
      ]}
      eyebrow="Team"
      message={`${teamLogin} in ${orgLogin} could not be loaded. Team details require organization membership and team visibility access.`}
      session={session}
      shellContext={shellContext}
      title={`${orgLogin} / ${teamLogin}`}
    />
  );
}
