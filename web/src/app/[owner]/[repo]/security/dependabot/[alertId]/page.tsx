import { AppShell } from "@/components/AppShell";
import { RepositoryDependabotAlertDetailPage } from "@/components/RepositoryDependabotAlertDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDependabotAlertDetail,
  getSession,
} from "@/lib/server-session";

type RepositoryDependabotAlertPageProps = {
  params: Promise<{ owner: string; repo: string; alertId: string }>;
};

export default async function RepositoryDependabotAlertPage({
  params,
}: RepositoryDependabotAlertPageProps) {
  const [{ owner, repo, alertId }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const decodedAlertId = decodeURIComponent(alertId);
  const [repository, detailResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDependabotAlertDetail(
      ownerLogin,
      repositoryName,
      decodedAlertId,
    ),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryDependabotAlertDetailPage
          detailResult={detailResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
