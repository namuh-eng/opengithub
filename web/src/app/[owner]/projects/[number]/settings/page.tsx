import { AppShell } from "@/components/AppShell";
import { ProjectSettingsPage } from "@/components/ProjectSettingsPage";
import {
  getSessionAndShellContext,
  getUserProjectSettings,
} from "@/lib/server-session";

type UserProjectSettingsRouteProps = {
  params: Promise<{ owner: string; number: string }>;
};

export default async function UserProjectSettingsRoute({
  params,
}: UserProjectSettingsRouteProps) {
  const [{ owner, number }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getUserProjectSettings(ownerLogin, projectNumber)
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project settings could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectSettingsPage
          owner={ownerLogin}
          scope="user"
          settings={result.settings}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project settings unavailable</div>
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
