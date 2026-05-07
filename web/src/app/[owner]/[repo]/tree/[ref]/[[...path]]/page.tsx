import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { RepositoryTreeView } from "@/components/RepositoryPathViews";
import {
  getRepository,
  getRepositoryPath,
  getSession,
} from "@/lib/server-session";

type RepositoryTreePageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    ref: string;
    path?: string[];
  }>;
  searchParams: Promise<{
    page?: string;
    pageSize?: string;
  }>;
};

export default async function RepositoryTreePage({
  params,
  searchParams,
}: RepositoryTreePageProps) {
  const [{ owner, repo, ref, path = [] }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const refName = decodeURIComponent(ref);
  const repositoryPath = path.map(decodeURIComponent).join("/");
  const page = Number.parseInt(query.page ?? "1", 10);
  const pageSize = Number.parseInt(query.pageSize ?? "30", 10);
  const overview =
    session.authenticated && session.user
      ? await getRepositoryPath(
          ownerLogin,
          repositoryName,
          refName,
          repositoryPath,
          {
            page: Number.isFinite(page) ? page : 1,
            pageSize: Number.isFinite(pageSize) ? pageSize : 30,
          },
        )
      : null;
  const recoveryRepository =
    !overview && session.authenticated && session.user
      ? await getRepository(ownerLogin, repositoryName)
      : null;

  return (
    <AppShell session={session}>
      {overview ? (
        <RepositoryTreeView overview={overview} />
      ) : (
        <section className="mx-auto max-w-6xl px-6 py-8">
          <div className="rounded-md border border-[var(--line)] bg-[var(--surface)] p-5">
            <h1 className="text-2xl font-semibold tracking-normal text-[var(--ink-1)]">
              Path unavailable
            </h1>
            <p
              className="mt-2 t-sm leading-6 text-[var(--ink-3)]"
              role="status"
            >
              The requested folder could not be loaded for this session.
            </p>
            <Link
              className="btn mt-4 inline-flex h-9 items-center px-4 text-[var(--accent)] hover:bg-[var(--surface-2)]"
              href={`/${ownerLogin}/${repositoryName}`}
            >
              Back to repository
            </Link>
            {recoveryRepository ? (
              <Link
                className="btn accent ml-2 mt-4 inline-flex h-9 items-center px-4 text-[var(--accent-ink)] hover:bg-[var(--accent-hover)]"
                href={`/${ownerLogin}/${repositoryName}/tree/${recoveryRepository.default_branch}`}
              >
                Open default branch
              </Link>
            ) : null}
          </div>
        </section>
      )}
    </AppShell>
  );
}
