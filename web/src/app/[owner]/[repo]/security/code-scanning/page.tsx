import { AppShell } from "@/components/AppShell";
import { RepositoryCodeScanningAlertsPage as RepositoryCodeScanningAlertsView } from "@/components/RepositoryCodeScanningAlertsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryCodeScanningAlerts,
  getSession,
} from "@/lib/server-session";

type RepositoryCodeScanningAlertsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    state?: string;
    q?: string;
    severity?: string;
    security_severity?: string;
    tool?: string;
    branch?: string;
    ref?: string;
    tag?: string;
    application_code?: string;
    sort?: string;
  }>;
};

export default async function RepositoryCodeScanningAlertsPage({
  params,
  searchParams,
}: RepositoryCodeScanningAlertsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, codeScanningResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryCodeScanningAlerts(ownerLogin, repositoryName, {
      state: query.state,
      query: query.q,
      severity: query.severity,
      securitySeverity: query.security_severity,
      tool: query.tool,
      branch: query.branch,
      ref: query.ref,
      tag: query.tag,
      applicationCode: query.application_code,
      sort: query.sort,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryCodeScanningAlertsView
          codeScanningResult={codeScanningResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
