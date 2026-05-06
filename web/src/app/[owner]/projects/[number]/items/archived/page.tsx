import { AppShell } from "@/components/AppShell";
import { ProjectArchivedItemsPage } from "@/components/ProjectArchivedItemsPage";
import {
  getProjectArchivedItems,
  getSessionAndShellContext,
  getUserProjectWorkspace,
} from "@/lib/server-session";

type UserProjectArchivedItemsRouteProps = {
  params: Promise<{ owner: string; number: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function itemTypeParam(value: string | string[] | undefined) {
  const raw = firstParam(value);
  return raw === "draft_issue" || raw === "issue" || raw === "pull_request"
    ? raw
    : undefined;
}

export default async function UserProjectArchivedItemsRoute({
  params,
  searchParams,
}: UserProjectArchivedItemsRouteProps) {
  const [{ owner, number }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const ownerLogin = decodeURIComponent(owner);
  const projectNumber = Number.parseInt(number, 10);
  const view = firstParam(queryParams?.view) ?? "1";
  const viewNumber = Number.parseInt(view, 10);
  const workspaceResult = Number.isFinite(projectNumber)
    ? await getUserProjectWorkspace(ownerLogin, projectNumber, { view })
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project workspace could not be found.",
      };
  const archivedResult = workspaceResult.ok
    ? await getProjectArchivedItems(workspaceResult.workspace.project.id, {
        q: firstParam(queryParams?.q),
        itemType: itemTypeParam(queryParams?.itemType),
      })
    : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {workspaceResult.ok && archivedResult?.ok ? (
        <ProjectArchivedItemsPage
          initialItems={archivedResult.archived.items}
          owner={ownerLogin}
          scope="user"
          total={archivedResult.archived.total}
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
            <div className="t-label mb-2">Project archive unavailable</div>
            <h1 className="t-h2">Archived items cannot be opened.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {workspaceResult.ok
                ? archivedResult && !archivedResult.ok
                  ? archivedResult.message
                  : "Archived project items could not be loaded."
                : workspaceResult.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
