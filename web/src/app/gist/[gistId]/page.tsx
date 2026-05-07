import Link from "next/link";
import { notFound } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { getGist, getSessionAndShellContext } from "@/lib/server-session";

type PageProps = { params: Promise<{ gistId: string }> };

export default async function GistDetailPage({ params }: PageProps) {
  const [{ gistId }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const gist = await getGist(gistId);
  if (!gist) notFound();

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-6xl gap-6">
          <header className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                {gist.isPublic ? "Public gist" : "Secret gist"}
              </p>
              <h1 className="t-h1 mt-2">
                {gist.description || "Untitled gist"}
              </h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                by <Link href={gist.owner.href}>{gist.owner.login}</Link> ·{" "}
                {gist.files.length} files · {gist.commentsCount} comments
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <form action="/gist/actions" method="post">
                <input
                  name="intent"
                  type="hidden"
                  value={gist.viewer.isStarred ? "unstar" : "star"}
                />
                <input name="gistId" type="hidden" value={gist.id} />
                <button className="btn sm" type="submit">
                  {gist.viewer.isStarred ? "Unstar" : "Star"} ·{" "}
                  {gist.starsCount}
                </button>
              </form>
              <form action="/gist/actions" method="post">
                <input name="intent" type="hidden" value="fork" />
                <input name="gistId" type="hidden" value={gist.id} />
                <button className="btn sm" type="submit">
                  Fork · {gist.forksCount}
                </button>
              </form>
              {gist.viewer.canEdit ? (
                <Link className="btn sm" href={`/gist/${gist.id}/edit`}>
                  Edit
                </Link>
              ) : null}
              <Link className="btn sm" href={`/gist/${gist.id}/revisions`}>
                Revisions
              </Link>
            </div>
          </header>
          <aside className="card grid gap-2 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Share
            </p>
            <code className="t-mono-sm break-all">{`<script src="${gist.embedUrl}"></script>`}</code>
            <code className="t-mono-sm break-all">{`git clone ${gist.cloneUrl}`}</code>
          </aside>
          {gist.files.map((file) => (
            <section className="card overflow-hidden" key={file.id}>
              <header
                className="flex flex-wrap justify-between gap-3 border-b px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                <span className="t-mono-sm">{file.filename}</span>
                <span className="chip">
                  {file.language ?? "Text"} · {file.sizeBytes} bytes
                </span>
              </header>
              <pre className="overflow-x-auto p-4 t-mono-sm">
                <code>{file.content}</code>
              </pre>
            </section>
          ))}
          <section className="card p-5">
            <h2 className="t-h2">Comments</h2>
            {gist.comments.length ? (
              gist.comments.map((comment) => (
                <article className="list-row py-4" key={comment.id}>
                  <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                    {comment.author.login}
                  </p>
                  <p className="t-body mt-2">{comment.body}</p>
                </article>
              ))
            ) : (
              <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
                No comments yet.
              </p>
            )}
          </section>
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
