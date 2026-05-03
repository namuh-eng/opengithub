import { AppShell } from "@/components/AppShell";
import { RepositoryReleaseDetailPage } from "@/components/RepositoryReleasesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryReleaseDetail,
  getSession,
} from "@/lib/server-session";

type ReleaseTagPageProps = {
  params: Promise<{ owner: string; repo: string; tag: string }>;
};

export default async function ReleaseTagPage({ params }: ReleaseTagPageProps) {
  const [{ owner, repo, tag }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const releaseTag = decodeURIComponent(tag);
  const repository = await getRepository(ownerLogin, repositoryName);
  const release = repository
    ? await getRepositoryReleaseDetail(ownerLogin, repositoryName, releaseTag)
    : null;

  return (
    <AppShell session={session}>
      {repository && release ? (
        <RepositoryReleaseDetailPage
          release={release}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
