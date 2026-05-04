import { AppShell } from "@/components/AppShell";
import { RepositoryForksPage as RepositoryForksView } from "@/components/RepositoryForksPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryForks,
  getSession,
} from "@/lib/server-session";

type RepositoryForksPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    period?: string;
    type?: string;
    repositoryType?: string;
    sort?: string;
  }>;
};

export default async function RepositoryForksPage({
  params,
  searchParams,
}: RepositoryForksPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const repositoryType = query.type ?? query.repositoryType;
  const [repository, forksResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryForks(ownerLogin, repositoryName, {
      period: query.period,
      repositoryType,
      sort: query.sort,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryForksView
          forksResult={forksResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
