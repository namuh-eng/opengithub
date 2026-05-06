import { AppShell } from "@/components/AppShell";
import { RepositoryActionsCachesPage as RepositoryActionsCachesView } from "@/components/RepositoryActionsCachesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import type { RepositoryActionsCaches } from "@/lib/api";
import {
  getRepository,
  getRepositoryActionsCaches,
  getSessionAndShellContext,
} from "@/lib/server-session";

type ActionsCachesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
};

export default async function ActionsCachesPage({
  params,
}: ActionsCachesPageProps) {
  const { owner, repo } = await params;
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const { session, shellContext } = await getSessionAndShellContext();
  const [repository, caches] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryActionsCaches(ownerLogin, repositoryName),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && !("error" in caches) ? (
        <RepositoryActionsCachesView detail={caches} repository={repository} />
      ) : repository && "error" in caches ? (
        <RepositoryActionsCachesView
          detail={emptyCaches(ownerLogin, repositoryName)}
          repository={repository}
          validationError={caches}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}

function emptyCaches(
  ownerLogin: string,
  repositoryName: string,
): RepositoryActionsCaches {
  return {
    repository: {
      id: "unavailable",
      ownerLogin,
      name: repositoryName,
      visibility: "public",
      defaultBranch: "main",
    },
    viewerPermission: null,
    caches: {
      items: [],
      total: 0,
      page: 1,
      pageSize: 30,
    },
    totalSizeBytes: 0,
    limitBytes: 10 * 1024 * 1024 * 1024,
    canDelete: false,
  };
}
