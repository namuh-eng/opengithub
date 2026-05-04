import Link from "next/link";
import { OrganizationMemberPrivilegesPage } from "@/components/OrganizationMemberPrivilegesPage";
import { OrganizationSettingsShell } from "@/components/OrganizationSettingsShell";
import { PlaceholderPage } from "@/components/PlaceholderPage";
import { organizationHref } from "@/lib/navigation";
import {
  getOrganizationMemberPrivileges,
  getOrganizationProfileSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationMemberPrivilegesRouteProps = {
  params: Promise<{ org: string }>;
};

export default async function OrganizationMemberPrivilegesRoute({
  params,
}: OrganizationMemberPrivilegesRouteProps) {
  const [{ org }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const [profileResult, privilegesResult] = await Promise.all([
    getOrganizationProfileSettings(orgLogin),
    getOrganizationMemberPrivileges(orgLogin),
  ]);

  if (!profileResult.ok) {
    const forbidden = profileResult.status === 403;
    return (
      <PlaceholderPage
        actions={[
          { href: organizationHref(orgLogin), label: "Organization profile" },
          { href: "/dashboard", label: "Dashboard" },
        ]}
        eyebrow="Organization settings"
        message={
          forbidden
            ? "Only organization owners can view or change member privilege settings."
            : profileResult.message
        }
        session={session}
        shellContext={shellContext}
        title={
          forbidden
            ? "Owner access required"
            : `${orgLogin} member privileges could not load`
        }
      >
        <div className="mt-4">
          <span className={`chip ${forbidden ? "warn" : "err"}`}>
            {forbidden ? "Restricted" : "Unavailable"}
          </span>
        </div>
      </PlaceholderPage>
    );
  }

  if (!privilegesResult.ok) {
    const status = privilegesResult.status;
    const message = privilegesResult.message;
    const forbidden = status === 403;
    return (
      <PlaceholderPage
        actions={[
          { href: organizationHref(orgLogin), label: "Organization profile" },
          { href: "/dashboard", label: "Dashboard" },
        ]}
        eyebrow="Organization settings"
        message={
          forbidden
            ? "Only organization owners can view or change member privilege settings."
            : message
        }
        session={session}
        shellContext={shellContext}
        title={
          forbidden
            ? "Owner access required"
            : `${orgLogin} member privileges could not load`
        }
      >
        <div className="mt-4">
          <span className={`chip ${forbidden ? "warn" : "err"}`}>
            {forbidden ? "Restricted" : "Unavailable"}
          </span>
        </div>
      </PlaceholderPage>
    );
  }

  return (
    <OrganizationSettingsShell
      activeSection="member-privileges"
      session={session}
      settings={profileResult.settings}
      shellContext={shellContext}
      title="Member Privileges"
    >
      <div className="mb-5 flex flex-wrap items-center gap-2">
        <Link
          className="btn sm"
          href={organizationHref(profileResult.settings.organization.slug)}
        >
          View organization
        </Link>
        <span className="chip soft">
          {privilegesResult.settings.viewerState.role}
        </span>
      </div>
      <OrganizationMemberPrivilegesPage settings={privilegesResult.settings} />
    </OrganizationSettingsShell>
  );
}
