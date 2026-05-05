import { AppShell } from "@/components/AppShell";
import { RepositoryDependentsPage as RepositoryDependentsView } from "@/components/RepositoryDependentsPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryDependents,
  getSession,
} from "@/lib/server-session";

type RepositoryDependentsPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{
    package?: string;
    owner?: string;
  }>;
};

export default async function RepositoryDependentsPage({
  params,
  searchParams,
}: RepositoryDependentsPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, dependentsResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDependents(ownerLogin, repositoryName, {
      package: query.package,
      owner: query.owner,
    }),
  ]);

  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryDependentsView
          dependentsResult={dependentsResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
