import { notFound } from "next/navigation";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { GistEditor } from "@/components/GistEditor";
import { getGist, getSessionAndShellContext } from "@/lib/server-session";

type PageProps = { params: Promise<{ gistId: string }> };

export default async function EditGistPage({ params }: PageProps) {
  const [{ gistId }, { session, shellContext }] = await Promise.all([
    params,
    getSessionAndShellContext(),
  ]);
  const gist = await getGist(gistId);
  if (!gist?.viewer.canEdit) notFound();
  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Edit gist
            </p>
            <h1 className="t-h1 mt-2">{gist.description || gist.id}</h1>
          </header>
          <GistEditor action="/gist/actions" gist={gist} />
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
