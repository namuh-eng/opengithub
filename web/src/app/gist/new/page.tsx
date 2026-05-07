import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { GistEditor } from "@/components/GistEditor";
import { getSessionAndShellContext } from "@/lib/server-session";

export default async function NewGistPage() {
  const { session, shellContext } = await getSessionAndShellContext();
  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                New gist
              </p>
              <h1 className="t-h1 mt-2">Capture a reusable snippet</h1>
            </div>
            <Link className="btn" href="/gist">
              Your gists
            </Link>
          </header>
          <GistEditor action="/gist/actions" />
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
