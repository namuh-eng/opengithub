import { AppShell } from "@/components/AppShell";
import { ProjectInsightsPage } from "@/components/ProjectInsightsPage";
import {
  getOrganizationProjectInsights,
  getSessionAndShellContext,
} from "@/lib/server-session";

type OrganizationProjectInsightsRouteProps = {
  params: Promise<{ org: string; number: string }>;
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function firstParam(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

export default async function OrganizationProjectInsightsRoute({
  params,
  searchParams,
}: OrganizationProjectInsightsRouteProps) {
  const [{ org, number }, queryParams, { session, shellContext }] =
    await Promise.all([params, searchParams, getSessionAndShellContext()]);
  const orgLogin = decodeURIComponent(org);
  const projectNumber = Number.parseInt(number, 10);
  const result = Number.isFinite(projectNumber)
    ? await getOrganizationProjectInsights(orgLogin, projectNumber, {
        chart: firstParam(queryParams?.chart),
        range: firstParam(queryParams?.range),
        start: firstParam(queryParams?.start),
        end: firstParam(queryParams?.end),
        filter: firstParam(queryParams?.filter),
        table: firstParam(queryParams?.table) === "true",
      })
    : {
        ok: false as const,
        status: 404,
        code: "not_found",
        message: "Project Insights could not be found.",
      };

  return (
    <AppShell session={session} shellContext={shellContext}>
      {result.ok ? (
        <ProjectInsightsPage
          insights={result.insights}
          owner={orgLogin}
          scope="organization"
        />
      ) : (
        <main className="mx-auto max-w-[760px] px-6 py-16">
          <div className="card p-6">
            <div className="t-label mb-2">Project Insights unavailable</div>
            <h1 className="t-h2">This project cannot show charts.</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {result.message}
            </p>
          </div>
        </main>
      )}
    </AppShell>
  );
}
