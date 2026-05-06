import { AppShell } from "@/components/AppShell";
import { ProjectAccessSettingsPage } from "@/components/ProjectAccessSettingsPage";
import {
  getOrganizationProjectSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectAccessSettingsRouteProps = {
  params: Promise<{ org: string; number: string }>;
};

export default async function OrganizationProjectAccessSettingsRoute({
  params,
}: OrganizationProjectAccessSettingsRouteProps) {
  const [{ org, number }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getOrganizationProjectSettings(orgLogin, projectNumber)
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project access settings could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectAccessSettingsPage
          owner={orgLogin}
          scope="organization"
          settings={result.settings}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project access unavailable</div>
            <h1 className="t-h2">
              This project access list cannot be configured.
            </h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {result.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
