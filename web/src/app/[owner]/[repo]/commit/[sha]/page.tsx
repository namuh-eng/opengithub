import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { RepositoryCommitDetailPage } from "@/components/RepositoryCommitDetailPage";
import { getRepositoryCommitDetail, getSession } from "@/lib/server-session";

type RepositoryCommitPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    sha: string;
  }>;
};

export default async function RepositoryCommitPage({
  params,
}: RepositoryCommitPageProps) {
  const [{ owner, repo, sha }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const commitSha = decodeURIComponent(sha);
  const detail =
    session.authenticated && session.user
      ? await getRepositoryCommitDetail(ownerLogin, repositoryName, commitSha)
      : null;

  return (
    <AppShell session={session}>
      {detail ? (
        <RepositoryCommitDetailPage detail={detail} />
      ) : (
        <section className="mx-auto max-w-6xl px-6 py-8">
          <div className="rounded-md border border-[var(--line)] bg-[var(--surface)] p-5">
            <h1 className="t-h2">Commit unavailable</h1>
            <p
              className="mt-2 t-sm leading-6 text-[var(--ink-3)]"
              role="status"
            >
              The requested commit could not be loaded for this session.
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
