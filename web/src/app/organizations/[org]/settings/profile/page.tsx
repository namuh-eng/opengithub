import Link from "next/link";
import { OrganizationProfileSettingsForm } from "@/components/OrganizationProfileSettingsForm";
import { OrganizationSettingsShell } from "@/components/OrganizationSettingsShell";
import { PlaceholderPage } from "@/components/PlaceholderPage";
import { organizationHref } from "@/lib/navigation";
import {
  getOrganizationProfileSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProfileSettingsPageProps = {
  params: Promise<{ org: string }>;
};

export default async function OrganizationProfileSettingsPage({
  params,
}: OrganizationProfileSettingsPageProps) {
  const [{ org }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const settingsResult = await getOrganizationProfileSettings(orgLogin);

  if (!settingsResult.ok) {
    const forbidden = settingsResult.status === 403;
    return (
      <PlaceholderPage
        actions={[
          { href: organizationHref(orgLogin), label: "Organization profile" },
          { href: "/dashboard", label: "Dashboard" },
        ]}
        eyebrow="Organization settings"
        message={
          forbidden
            ? "Only organization owners can view or change organization settings."
            : settingsResult.message
        }
        session={session}
        shellContext={shellContext}
        title={
          forbidden
            ? "Owner access required"
            : `${orgLogin} settings could not load`
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
      activeSection="profile"
      session={session}
      settings={settingsResult.settings}
      shellContext={shellContext}
      title="Profile"
    >
      <div className="mb-5 flex flex-wrap items-center gap-2">
        <Link
          className="btn sm"
          href={organizationHref(settingsResult.settings.organization.slug)}
        >
          View organization
        </Link>
        <span className="chip soft">
          {settingsResult.settings.viewerState.role}
        </span>
      </div>
      <OrganizationProfileSettingsForm settings={settingsResult.settings} />
    </OrganizationSettingsShell>
  );
}
