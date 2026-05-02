import { AppShell } from "@/components/AppShell";
import { RepositoryReleasesPage } from "@/components/RepositoryReleasesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { getRepositoryReleaseByTag } from "@/lib/releases";
import { getRepository, getSession } from "@/lib/server-session";

type ReleaseTagPageProps = {
  params: Promise<{ owner: string; repo: string; tagName: string }>;
};

export default async function ReleaseTagPage({ params }: ReleaseTagPageProps) {
  const [{ owner, repo, tagName }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const releaseView = repository
    ? getRepositoryReleaseByTag(
        repository,
        session,
        decodeURIComponent(tagName),
      )
    : null;

  return (
    <AppShell session={session}>
      {repository && releaseView ? (
        <RepositoryReleasesPage
          mode="detail"
          releases={releaseView}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
