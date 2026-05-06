import { AppShell } from "@/components/AppShell";
import { ProjectWorkflowSettingsPage } from "@/components/ProjectWorkflowSettingsPage";
import {
  getSessionAndShellContext,
  getUserProjectWorkflowSettings,
} from "@/lib/server-session";

type UserProjectWorkflowSettingsRouteProps = {
  params: Promise<{ owner: string; number: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function UserProjectWorkflowSettingsRoute({
  params,
  searchParams,
}: UserProjectWorkflowSettingsRouteProps) {
  const [{ owner, number }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getUserProjectWorkflowSettings(ownerLogin, projectNumber)
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
          owner={ownerLogin}
          scope="user"
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
