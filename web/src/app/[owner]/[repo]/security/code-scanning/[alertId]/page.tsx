import { AppShell } from "@/components/AppShell";
import { RepositoryCodeScanningAlertDetailPage } from "@/components/RepositoryCodeScanningAlertDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryCodeScanningAlertDetail,
  getSession,
} from "@/lib/server-session";

type RepositoryCodeScanningAlertPageProps = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

export default async function RepositoryCodeScanningAlertPage({
  params,
}: RepositoryCodeScanningAlertPageProps) {
  const [{ owner, repo, alertId }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedAlertId = decodeURIComponent(alertId);
  const [repository, detailResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryCodeScanningAlertDetail(
      ownerLogin,
      repositoryName,
      decodedAlertId,
    ),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryCodeScanningAlertDetailPage
          detailResult={detailResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
