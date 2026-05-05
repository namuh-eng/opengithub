import { AppShell } from "@/components/AppShell";
import { RepositorySecretScanningAlertDetailPage } from "@/components/RepositorySecretScanningAlertDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecretScanningAlertDetail,
  getSession,
} from "@/lib/server-session";

type RepositorySecretScanningAlertPageProps = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

export default async function RepositorySecretScanningAlertPage({
  params,
}: RepositorySecretScanningAlertPageProps) {
  const [{ owner, repo, alertId }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedAlertId = decodeURIComponent(alertId);
  const [repository, detailResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecretScanningAlertDetail(
      ownerLogin,
      repositoryName,
      decodedAlertId,
    ),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecretScanningAlertDetailPage
          detailResult={detailResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
