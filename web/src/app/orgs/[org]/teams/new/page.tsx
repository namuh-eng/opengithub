import { OrganizationTeamCreatePage } from "@/components/OrganizationTeamCreatePage";
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
  getOrganizationTeams,
  getPublicOrganizationProfile,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationNewTeamRouteProps = {
  params: Promise<{ org: string }>;
};

export default async function OrganizationNewTeamRoute({
  params,
}: OrganizationNewTeamRouteProps) {
  const [{ org }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const [profile, directory] = await Promise.all([
    getPublicOrganizationProfile(orgLogin),
    getOrganizationTeams(orgLogin, { pageSize: 100 }),
  ]);

  if (directory?.viewerState.canCreateTeam) {
    return (
      <ProfileOrgShell
        actions={[
          { href: organizationProjectHref(orgLogin), label: "Projects" },
          { href: organizationSettingsHref(orgLogin), label: "Settings" },
        ]}
        activeTab="teams"
        eyebrow="Organization"
        hrefForTab={(value) => organizationTabHref(orgLogin, value)}
        identityLabel={profile?.identity.name ?? orgLogin}
        message="Create a visible or secret team for repository access, review ownership, and mention notifications."
        session={session}
        shellContext={shellContext}
        tabLabel="Organization sections"
        tabs={ORGANIZATION_TABS}
        title={profile?.identity.name ?? orgLogin}
      >
        <OrganizationTeamCreatePage directory={directory} org={orgLogin} />
      </ProfileOrgShell>
    );
  }

  return (
    <PlaceholderPage
      actions={[
        { href: organizationTeamsHref(orgLogin), label: "Back to teams" },
        { href: "/docs/api#organization-teams", label: "Learn more" },
      ]}
      eyebrow="Team setup"
      message={`Creating teams for ${orgLogin} requires organization owner, admin, or member team-creation access.`}
      session={session}
      shellContext={shellContext}
      title="Team creation unavailable"
    />
  );
}
