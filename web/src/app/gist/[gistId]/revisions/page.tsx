import Link from "next/link";
import { notFound } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import {
  getGistRevisions,
  getSessionAndShellContext,
} from "@/lib/server-session";

type PageProps = { params: Promise<{ gistId: string }> };

export default async function GistRevisionsPage({ params }: PageProps) {
  const [{ gistId }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const data = await getGistRevisions(gistId);
  if (!data) notFound();
  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Gist revisions
              </p>
              <h1 className="t-h1 mt-2">
                {data.gist.description || data.gist.id}
              </h1>
            </div>
            <Link className="btn" href={data.gist.href}>
              Back to gist
            </Link>
          </header>
          <section className="card overflow-hidden">
            {data.revisions.map((revision) => (
              <article className="list-row p-4" key={revision.id}>
                <div className="flex flex-wrap justify-between gap-3">
                  <h2 className="t-h3">Revision {revision.version}</h2>
                  <span className="chip">{revision.files.length} files</span>
                </div>
                <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  {revision.author.login} ·{" "}
                  {new Date(revision.createdAt).toLocaleString()}
                </p>
              </article>
            ))}
          </section>
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
