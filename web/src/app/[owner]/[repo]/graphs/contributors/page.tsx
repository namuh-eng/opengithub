import { AppShell } from "@/components/AppShell";
import { RepositoryContributorsPage as RepositoryContributorsView } from "@/components/RepositoryContributorsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryContributors,
  getSession,
} from "@/lib/server-session";

type RepositoryContributorsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{ period?: string; start?: string; end?: string }>;
};

export default async function RepositoryContributorsPage({
  params,
  searchParams,
}: RepositoryContributorsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, contributorsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryContributors(ownerLogin, repositoryName, {
      period: query?.period ?? null,
      start: query?.start ?? null,
      end: query?.end ?? null,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryContributorsView
          contributorsResult={contributorsResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
