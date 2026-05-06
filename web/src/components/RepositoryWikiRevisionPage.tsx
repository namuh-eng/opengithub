import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import { WikiPageList } from "@/components/WikiPageList";
import type {
  RepositoryOverview,
  RepositoryWikiRevisionFetchResult,
  RepositoryWikiRevisionView,
} from "@/lib/api";

type RepositoryWikiRevisionPageProps = {
  repository: RepositoryOverview;
  revisionResult: RepositoryWikiRevisionFetchResult;
};

function formatDateTime(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

function authorLabel(revision: RepositoryWikiRevisionView) {
  const author = revision.page.revision.author;
  return author?.displayName ?? author?.login ?? "Unknown author";
}

function RevisionReader({
  revision,
}: {
  revision: RepositoryWikiRevisionView;
}) {
  const page = revision.page;
  const selected = revision.revisionContext.selectedRevision;

  return (
    <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_300px]">
      <main className="min-w-0">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Historical wiki revision
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              id="repository-wiki-revision-title"
              style={{ color: "var(--ink-1)" }}
            >
              {page.title}
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {selected.message} by {authorLabel(revision)} on{" "}
              <time dateTime={selected.createdAt}>
                {formatDateTime(selected.createdAt)}
              </time>
              .
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn sm" href={revision.revisionContext.latestHref}>
              Latest
            </Link>
            <Link
              className="btn sm"
              href={revision.revisionContext.historyHref}
            >
              History
            </Link>
            <Link className="btn sm" href={revision.links.pagesHref}>
              Pages
            </Link>
          </div>
        </section>

        <section
          className="card mt-5 p-4"
          aria-label="Historical revision notice"
        >
          <div className="flex flex-wrap items-center gap-2">
            <span className="chip warn">Read-only snapshot</span>
            <span className="chip soft t-mono-sm">
              {selected.shortOid ?? selected.id}
            </span>
            {revision.revisionContext.isLatest ? (
              <span className="chip ok">Latest revision</span>
            ) : null}
          </div>
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            This page shows the wiki content as it rendered at the selected
            revision.
          </p>
        </section>

        <article className="card mt-5 overflow-hidden">
          <div
            className="border-b px-5 py-4"
            style={{ borderColor: "var(--line)" }}
          >
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              {page.path}
            </p>
          </div>
          <div className="px-5 py-5">
            <MarkdownBody
              html={page.html}
              labelledBy="repository-wiki-revision-title"
            />
          </div>
        </article>

        <nav
          aria-label="Adjacent wiki revisions"
          className="mt-5 flex flex-wrap justify-between gap-2"
        >
          {revision.revisionContext.previousRevisionHref ? (
            <Link
              className="btn sm"
              href={revision.revisionContext.previousRevisionHref}
            >
              Previous Revision
            </Link>
          ) : (
            <span className="btn sm" aria-disabled="true">
              Previous Revision
            </span>
          )}
          {revision.revisionContext.nextRevisionHref ? (
            <Link
              className="btn sm"
              href={revision.revisionContext.nextRevisionHref}
            >
              Next Revision
            </Link>
          ) : (
            <span className="btn sm" aria-disabled="true">
              Next Revision
            </span>
          )}
        </nav>
      </main>

      <aside className="grid content-start gap-4">
        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Pages
          </p>
          <WikiPageList
            currentOutline={page.outline}
            owner={revision.repository.ownerLogin}
            pages={revision.pages}
            repo={revision.repository.name}
          />
        </section>
      </aside>
    </div>
  );
}

function WikiRevisionUnavailable({ message }: { message: string }) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Repository wiki
      </p>
      <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
        Revision unavailable
      </h1>
      <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
        {message}
      </p>
    </section>
  );
}

export function RepositoryWikiRevisionPage({
  repository,
  revisionResult,
}: RepositoryWikiRevisionPageProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/wiki`}
      repository={repository}
    >
      {revisionResult.ok ? (
        <RevisionReader revision={revisionResult.revision} />
      ) : (
        <WikiRevisionUnavailable message={revisionResult.message} />
      )}
    </RepositoryShell>
  );
}
