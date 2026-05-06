import { AppShell } from "@/components/AppShell";
import { ProjectWorkflowSettingsPage } from "@/components/ProjectWorkflowSettingsPage";
import {
  getOrganizationProjectWorkflowSettings,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectWorkflowSettingsRouteProps = {
  params: Promise<{ org: string; number: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function OrganizationProjectWorkflowSettingsRoute({
  params,
  searchParams,
}: OrganizationProjectWorkflowSettingsRouteProps) {
  const [{ org, number }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getOrganizationProjectWorkflowSettings(orgLogin, projectNumber)
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project workflows could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectWorkflowSettingsPage
          owner={orgLogin}
          scope="organization"
          selectedWorkflowId={firstParam(queryParams?.workflow)}
          settings={result.settings}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project workflows unavailable</div>
            <h1 className="t-h2">This project cannot be automated.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {result.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
