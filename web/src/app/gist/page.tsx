import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { getGists, getSessionAndShellContext } from "@/lib/server-session";

export default async function GistsPage() {
  const [{ session, shellContext }, gists] = await Promise.all([
    getSessionAndShellContext(),
    getGists({ scope: "mine" }),
  ]);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Gists
              </p>
              <h1 className="t-h1 mt-2">Snippet library</h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                Create public or secret multi-file notes with revision history.
              </p>
            </div>
            <Link className="btn accent" href="/gist/new">
              New gist
            </Link>
          </header>
          <section className="card overflow-hidden">
            {(gists?.items ?? []).map((gist) => (
              <Link
                className="list-row block p-4"
                href={gist.href}
                key={gist.id}
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <p className="t-h3">
                      {gist.description ||
                        gist.files[0]?.filename ||
                        "Untitled gist"}
                    </p>
                    <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                      {gist.files.length} files ·{" "}
                      {gist.isPublic ? "Public" : "Secret"} · {gist.starsCount}{" "}
                      stars
                    </p>
                  </div>
                  <span className="chip">{gist.owner.login}</span>
                </div>
              </Link>
            ))}
            {!gists?.items.length ? (
              <div className="p-6">
                <p className="t-body" style={{ color: "var(--ink-3)" }}>
                  No gists yet.
                </p>
              </div>
            ) : null}
          </section>
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
