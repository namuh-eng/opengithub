import { AppShell } from "@/components/AppShell";
import { RepositoryPlaceholderPage } from "@/components/RepositoryPlaceholderPage";
import { RepositoryReleasesPage } from "@/components/RepositoryReleasesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { getRepositoryLatestRelease } from "@/lib/releases";
import { getRepository, getSession } from "@/lib/server-session";

type LatestReleasePageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function LatestReleasePage({
  params,
}: LatestReleasePageProps) {
  const [{ owner, repo }, session] = await Promise.all([params, getSession()]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repository = await getRepository(ownerLogin, repositoryName);
  const releaseView = repository
    ? getRepositoryLatestRelease(repository, session)
    : null;

  return (
    <AppShell session={session}>
      {repository && releaseView ? (
        <RepositoryReleasesPage
          mode="detail"
          releases={releaseView}
          repository={repository}
        />
      ) : repository ? (
        <RepositoryPlaceholderPage
          activePath={`/${ownerLogin}/${repositoryName}/releases`}
          description="This repository has no stable release yet. Pre-releases remain available from the release history."
          repository={repository}
          title="No latest release"
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
