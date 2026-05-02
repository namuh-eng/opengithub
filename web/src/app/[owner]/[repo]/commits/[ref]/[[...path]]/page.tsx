import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { RepositoryCommitHistoryView } from "@/components/RepositoryPathViews";
import { getRepositoryCommitHistory, getSession } from "@/lib/server-session";

type RepositoryCommitsPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    ref: string;
    path?: string[];
  }>;
  searchParams: Promise<{
    author?: string;
    since?: string;
    until?: string;
    page?: string;
    pageSize?: string;
  }>;
};

export default async function RepositoryCommitsPage({
  params,
  searchParams,
}: RepositoryCommitsPageProps) {
  const [{ owner, repo, ref, path = [] }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const refName = decodeURIComponent(ref);
  const repositoryPath = path.map(decodeURIComponent).join("/");
  const history =
    session.authenticated && session.user
      ? await getRepositoryCommitHistory(
          ownerLogin,
          repositoryName,
          refName,
          repositoryPath,
          {
            author: query.author,
            since: query.since,
            until: query.until,
            page: query.page,
            pageSize: query.pageSize,
          },
        )
      : null;

  return (
    <AppShell session={session}>
      {history ? (
        <RepositoryCommitHistoryView
          history={history}
          owner={ownerLogin}
          path={repositoryPath}
          repo={repositoryName}
        />
      ) : (
        <section className="mx-auto max-w-6xl px-6 py-8">
          <div className="card p-5">
            <h1 className="t-h2" style={{ color: "var(--ink-1)" }}>
              Commit history unavailable
            </h1>
            <p
              className="mt-2 t-sm leading-6"
              role="status"
              style={{ color: "var(--ink-3)" }}
            >
              The requested commit history could not be loaded for this session.
            </p>
            <Link
              className="btn mt-4 inline-flex"
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
