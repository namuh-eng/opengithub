import { AppShell } from "@/components/AppShell";
import { RepositoryBranchesPage } from "@/components/RepositoryBranchesPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryBranches,
  getSession,
} from "@/lib/server-session";

type BranchesPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams?: Promise<{
    tab?: string;
    q?: string;
    page?: string;
    pageSize?: string;
  }>;
};

function numberParam(value: string | undefined) {
  if (!value) {
    return null;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

export default async function BranchesPage({
  params,
  searchParams,
}: BranchesPageProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [repository, branchesResult] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryBranches(ownerLogin, repositoryName, {
      tab: query?.tab ?? null,
      query: query?.q ?? null,
      page: numberParam(query?.page),
      pageSize: numberParam(query?.pageSize),
    }),
  ]);
  return (
    <AppShell session={session}>
      {repository ? (
        <RepositoryBranchesPage
          branchesResult={branchesResult}
          repository={repository}
        />
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
