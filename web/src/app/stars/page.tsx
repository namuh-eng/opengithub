import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { ProfileRepositoryTabs } from "@/components/ProfileRepositoryTabs";
import {
  getApiUser,
  getProfileStars,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PageProps = {
  searchParams?: Promise<Record<string, string | string[] | undefined>>;
};

function first(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value;
}

function positive(value: string | string[] | undefined) {
  const parsed = Number.parseInt(first(value) ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

export default async function StarsPage({ searchParams }: PageProps) {
  const [query, { session, shellContext }, user] = await Promise.all([
    searchParams,
    getSessionAndShellContext(),
    getApiUser(),
  ]);

  if (!user) {
    return (
      <AppShell session={session} shellContext={shellContext}>
        <AppShellFrame>
          <section className="card mx-auto max-w-xl p-6">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Stars
            </p>
            <h1 className="t-h1 mt-2">
              Sign in to review your starred repositories.
            </h1>
            <Link className="btn primary mt-5" href="/login?next=%2Fstars">
              Sign in
            </Link>
          </section>
        </AppShellFrame>
      </AppShell>
    );
  }

  const list = await getProfileStars(user.login, {
    q: first(query?.q),
    language: first(query?.language),
    sort: first(query?.sort),
    page: positive(query?.page),
    pageSize: positive(query?.pageSize),
  });

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Personal library
              </p>
              <h1 className="t-h1 mt-2">Your stars</h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                Search and sort repositories you have starred.
              </p>
            </div>
            <Link
              className="btn"
              href={`/${encodeURIComponent(user.login)}?tab=stars`}
            >
              Public stars tab
            </Link>
          </header>
          {list ? (
            <ProfileRepositoryTabs list={list} owner={user.login} />
          ) : (
            <section className="card p-6">
              <p className="t-body" style={{ color: "var(--ink-3)" }}>
                Starred repositories could not be loaded.
              </p>
            </section>
          )}
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
