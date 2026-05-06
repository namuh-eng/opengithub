import { AppShell } from "@/components/AppShell";
import { RepositoryLabelsPage as RepositoryLabelsScreen } from "@/components/RepositoryLabelsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryLabels,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryLabelsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    q?: string;
    sort?: string;
    direction?: string;
    page?: string;
  }>;
};

export default async function RepositoryLabelsPage({
  params,
  searchParams,
}: RepositoryLabelsPageProps) {
  const [{ owner, repo }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const page = Number.parseInt(query.page ?? "1", 10);
  const labelsQuery = {
    q: query.q,
    sort: query.sort,
    direction: query.direction,
    page: Number.isFinite(page) ? page : 1,
    pageSize: 100,
  };
  const [repository, labels] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryLabels(ownerLogin, repositoryName, labelsQuery).catch(
      () => null,
    ),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && labels ? (
        <RepositoryLabelsScreen
          labels={labels}
          query={labelsQuery}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
