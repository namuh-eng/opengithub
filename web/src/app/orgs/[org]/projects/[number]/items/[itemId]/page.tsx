import { AppShell } from "@/components/AppShell";
import { ProjectWorkspacePage } from "@/components/ProjectWorkspacePage";
import {
  getOrganizationProjectWorkspace,
  getProjectItemDetail,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectItemRouteProps = {
  params: Promise<{ org: string; number: string; itemId: string }>;
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

export default async function OrganizationProjectItemRoute({
  params,
  searchParams,
}: OrganizationProjectItemRouteProps) {
  const [{ org, number, itemId }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const view = firstParam(queryParams?.view) ?? "1";
  const viewNumber = Number.parseInt(view, 10);
  const workspaceResult = Number.isFinite(projectNumber)
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
  const detailResult = workspaceResult.ok
    ? await getProjectItemDetail(workspaceResult.workspace.project.id, itemId)
    : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {workspaceResult.ok ? (
        <ProjectWorkspacePage
          initialItemDetail={detailResult?.ok ? detailResult.detail : null}
          initialItemError={
            detailResult && !detailResult.ok ? detailResult.message : null
          }
          owner={orgLogin}
          scope="organization"
          viewNumber={
            Number.isFinite(viewNumber)
              ? viewNumber
              : workspaceResult.workspace.selectedView.number
          }
          workspace={workspaceResult.workspace}
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project workspace unavailable</div>
            <h1 className="t-h2">This project cannot be opened.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {workspaceResult.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
