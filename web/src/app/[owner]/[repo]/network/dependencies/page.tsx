import { AppShell } from "@/components/AppShell";
import { RepositoryDependencyGraphPage as RepositoryDependencyGraphView } from "@/components/RepositoryDependencyGraphPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDependencies,
  getSession,
} from "@/lib/server-session";

type RepositoryDependencyGraphPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    q?: string;
    ecosystem?: string;
    relationship?: string;
  }>;
};

export default async function RepositoryDependencyGraphPage({
  params,
  searchParams,
}: RepositoryDependencyGraphPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, dependenciesResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDependencies(ownerLogin, repositoryName, {
      query: query.q,
      ecosystem: query.ecosystem,
      relationship: query.relationship,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryDependencyGraphView
          dependenciesResult={dependenciesResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
