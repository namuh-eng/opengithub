import { AppShell } from "@/components/AppShell";
import { RepositorySecurityAdvisoryDetailPage as RepositorySecurityAdvisoryDetailView } from "@/components/RepositorySecurityAdvisoryDetailPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositorySecurityAdvisoryDetail,
  getSession,
} from "@/lib/server-session";

type RepositorySecurityAdvisoryDetailPageProps = {
  params: Promise<{ owner: string; repo: string; ghsaId: string }>;
};

export default async function RepositorySecurityAdvisoryDetailPage({
  params,
}: RepositorySecurityAdvisoryDetailPageProps) {
  const [{ owner, repo, ghsaId }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const advisoryId = decodeURIComponent(ghsaId);
  const [repository, advisoryResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositorySecurityAdvisoryDetail(ownerLogin, repositoryName, advisoryId),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositorySecurityAdvisoryDetailView
          advisoryResult={advisoryResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
