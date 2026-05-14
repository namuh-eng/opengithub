import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { RepositoryFileFinderPage } from "@/components/RepositoryFileFinderPage";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  getRepository,
  getRepositoryFileFinder,
  getSession,
} from "@/lib/server-session";

type RepositoryFindPageProps = {
  params: Promise<{
    owner: string;
    repo: string;
    ref: string;
  }>;
};

export default async function RepositoryFindPage({
  params,
}: RepositoryFindPageProps) {
  const [{ owner, repo, ref }, session] = await Promise.all([
    params,
    getSession(),
  ]);
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const refName = decodeURIComponent(ref);
  const [repository, finder] =
    session.authenticated && session.user
      ? await Promise.all([
          getRepository(ownerLogin, repositoryName),
          getRepositoryFileFinder(ownerLogin, repositoryName, refName, "", {
            page: 1,
            pageSize: 5000,
            pathCache: true,
          }),
        ])
      : [null, null];

  return (
    <AppShell session={session}>
      {repository && finder ? (
        <RepositoryFileFinderPage finder={finder} repository={repository} />
      ) : repository ? (
        <section className="mx-auto max-w-6xl px-6 py-8">
          <div
            className="rounded-md p-5"
            style={{
              border: "1px solid var(--line)",
              background: "var(--surface)",
            }}
          >
            <h1 className="t-h2" style={{ color: "var(--ink-1)" }}>
              File finder unavailable
            </h1>
            <p className="mt-2 t-sm" role="status">
              The requested ref could not be loaded for this repository.
            </p>
            <Link
              className="btn mt-4 inline-flex h-9 items-center px-4"
              href={`/${ownerLogin}/${repositoryName}/find/${repository.default_branch}`}
            >
              Open default branch finder
            </Link>
          </div>
        </section>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
