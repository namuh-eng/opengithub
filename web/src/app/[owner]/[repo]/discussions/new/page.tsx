import Link from "next/link";
import { redirect } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { RepositoryShell } from "@/components/RepositoryShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import {
  repositoryDiscussionChooseCategoryHref,
  repositoryDiscussionsHref,
} from "@/lib/navigation";
import {
  getRepository,
  getRepositoryDiscussionCreation,
  getSessionAndShellContext,
} from "@/lib/server-session";

type RepositoryNewDiscussionPageProps = {
  params: Promise<{ owner: string; repo: string }>;
  searchParams: Promise<{ category?: string; q?: string; next?: string }>;
};

export default async function RepositoryNewDiscussionPage({
  params,
  searchParams,
}: RepositoryNewDiscussionPageProps) {
  const [{ owner, repo }, query, { session, shellContext }] = await Promise.all(
    [params, searchParams, getSessionAndShellContext()],
  );
  const ownerLogin = decodeURIComponent(owner);
  const repositoryName = decodeURIComponent(repo);
  const category = query.category?.trim();

  if (!category) {
    redirect(
      repositoryDiscussionChooseCategoryHref(ownerLogin, repositoryName),
    );
  }

  const [repository, creation] = await Promise.all([
    getRepository(ownerLogin, repositoryName),
    getRepositoryDiscussionCreation(ownerLogin, repositoryName, { category }),
  ]);

  const selected =
    creation && !("error" in creation) ? creation.selectedCategory : null;

  return (
    <AppShell session={session} shellContext={shellContext}>
      {repository && creation && !("error" in creation) ? (
        <RepositoryShell
          activePath={`/${ownerLogin}/${repositoryName}/discussions`}
          frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
          repository={repository}
        >
          <main className="min-w-0 space-y-5">
            <section className="card p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                New discussion
              </p>
              <h1 className="t-h2 mt-1 break-words">
                {selected
                  ? `${selected.emoji} ${selected.name}`
                  : "Choose a category"}
              </h1>
              <p
                className="t-sm mt-2 max-w-2xl"
                style={{ color: "var(--ink-3)" }}
              >
                {selected?.description ??
                  "Select one of the repository discussion categories before starting a thread."}
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                {selected?.acceptsAnswers ? (
                  <span className="chip ok">Answers enabled</span>
                ) : null}
                {selected?.isPoll ? (
                  <span className="chip warn">Poll</span>
                ) : null}
                <Link
                  className="chip soft hover:underline"
                  href={repositoryDiscussionChooseCategoryHref(
                    ownerLogin,
                    repositoryName,
                  )}
                >
                  Choose a different category
                </Link>
              </div>
            </section>
          </main>
          <aside className="space-y-4">
            <section className="card p-4">
              <h2 className="t-h3">Similar discussions</h2>
              <Link
                className="t-sm mt-3 inline-block hover:underline"
                href={
                  creation.similarSearch.href ||
                  repositoryDiscussionsHref(ownerLogin, repositoryName, {
                    q: query.q,
                  })
                }
              >
                Search before posting
              </Link>
            </section>
          </aside>
        </RepositoryShell>
      ) : (
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      )}
    </AppShell>
  );
}
