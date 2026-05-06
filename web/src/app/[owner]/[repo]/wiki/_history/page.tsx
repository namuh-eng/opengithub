import { AppShell } from "@/components/AppShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWikiHistoryPage } from "@/components/RepositoryWikiHistoryPage";
import {
  getRepository,
  getRepositoryWikiHistory,
  getSession,
} from "@/lib/server-session";

type RepositoryWikiHistoryRouteProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ page?: string; pageSize?: string }>;
};

function decodePathSegment(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function positiveNumber(value: string | undefined) {
  if (!value) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : null;
}

export default async function RepositoryWikiHistoryRoute({
  params,
  searchParams,
}: RepositoryWikiHistoryRouteProps) {
  const [{ owner, repo }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodePathSegment(owner);
  const repositoryName = decodePathSegment(repo);
  const repository = await getRepository(ownerLogin, repositoryName);

  if (!repository) {
    return (
      <AppShell session={session}>
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      </AppShell>
    );
  }

  const historyResult = await getRepositoryWikiHistory(
    ownerLogin,
    repositoryName,
    null,
    positiveNumber(query.page),
    positiveNumber(query.pageSize),
  );

  return (
    <AppShell session={session}>
      <RepositoryWikiHistoryPage
        historyResult={historyResult}
        repository={repository}
      />
    </AppShell>
  );
}
