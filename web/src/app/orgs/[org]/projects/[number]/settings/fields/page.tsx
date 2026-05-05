import { AppShell } from "@/components/AppShell";
import { ProjectFieldSettingsPage } from "@/components/ProjectFieldSettingsPage";
import {
  getOrganizationProjectFieldSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectFieldSettingsRouteProps = {
  params: Promise<{ org: string; number: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function OrganizationProjectFieldSettingsRoute({
  params,
  searchParams,
}: OrganizationProjectFieldSettingsRouteProps) {
  const [{ org, number }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getOrganizationProjectFieldSettings(orgLogin, projectNumber)
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project field settings could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectFieldSettingsPage
          owner={orgLogin}
          scope="organization"
          selectedFieldId={firstParam(queryParams?.field)}
          settings={result.settings}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">
              Project field settings unavailable
            </div>
            <h1 className="t-h2">This project cannot be configured.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {result.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
