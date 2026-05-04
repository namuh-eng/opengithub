import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { RepositoryCommitHistoryPage } from "@/components/RepositoryCommitHistoryPage";
import { getRepositoryCommitHistory, getSession } from "@/lib/server-session";

type RepositoryCommitsPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    refPath: string[];
  }>;
  searchParams?: Promise<{
    author?: string;
    until?: string;
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

export default async function RepositoryCommitsPage({
  params,
  searchParams,
}: RepositoryCommitsPageProps) {
  const [{ owner, repo, refPath }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const [encodedRef = "main", ...encodedPath] = refPath;
  const refName = decodeURIComponent(encodedRef);
  const repositoryPath = encodedPath.map(decodeURIComponent).join("/");
  const history =
    session.authenticated && session.user
      ? await getRepositoryCommitHistory(
          ownerLogin,
          repositoryName,
          refName,
          repositoryPath,
          {
            author: query?.author ?? null,
            until: query?.until ?? null,
            page: numberParam(query?.page),
            pageSize: numberParam(query?.pageSize),
          },
        )
      : null;

  return (
    <AppShell session={session}>
      {history ? (
        <RepositoryCommitHistoryPage history={history} />
      ) : (
        <section className="mx-auto max-w-6xl px-6 py-8">
          <div className="rounded-md border border-[var(--line)] bg-[var(--surface)] p-5">
            <h1 className="text-2xl font-semibold tracking-normal text-[var(--ink-1)]">
              Commit history unavailable
            </h1>
            <p
              className="mt-2 t-sm leading-6 text-[var(--ink-3)]"
              role="status"
            >
              The requested commit history could not be loaded for this session.
            </p>
            <Link
              className="btn mt-4 inline-flex h-9 items-center px-4 text-[var(--accent)] hover:bg-[var(--surface-2)]"
              href={`/${ownerLogin}/${repositoryName}`}
            >
              Back to repository
            </Link>
          </div>
        </section>
      )}
    </AppShell>
  );
}
