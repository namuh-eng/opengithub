import { AppShell } from "@/components/AppShell";
import { ProjectWorkspacePage } from "@/components/ProjectWorkspacePage";
import {
  getOrganizationProjectWorkspace,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectWorkspaceRouteProps = {
  params: Promise<{ org: string; number: string; view: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function numberParam(value: string | string[] | undefined) {
  const raw = firstParam(value);
  if (!raw) return undefined;
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : undefined;
}

export default async function OrganizationProjectWorkspaceRoute({
  params,
  searchParams,
}: OrganizationProjectWorkspaceRouteProps) {
  const [{ org, number, view }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const viewNumber = Number.parseInt(view, 10);
  const result = Number.isFinite(projectNumber)
    ? await getOrganizationProjectWorkspace(orgLogin, projectNumber, {
        view,
        q: firstParam(queryParams?.q),
        sort: firstParam(queryParams?.sort),
        group: firstParam(queryParams?.group),
        slice: firstParam(queryParams?.slice),
        page: numberParam(queryParams?.page),
        pageSize: numberParam(queryParams?.pageSize),
      })
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project workspace could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectWorkspacePage
          owner={orgLogin}
          scope="organization"
          viewNumber={
            Number.isFinite(viewNumber)
              ? viewNumber
              : result.workspace.selectedView.number
          }
          workspace={result.workspace}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project workspace unavailable</div>
            <h1 className="t-h2">This project cannot be opened.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {result.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
