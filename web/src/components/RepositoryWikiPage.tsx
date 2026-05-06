import Link from "next/link";
import { CopyButton } from "@/components/CopyButton";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import { WikiPageList } from "@/components/WikiPageList";
import type {
  RepositoryOverview,
  RepositoryWikiFetchResult,
  RepositoryWikiHeading,
  RepositoryWikiRenderedBlock,
  RepositoryWikiView,
} from "@/lib/api";

type RepositoryWikiPageProps = {
  repository: RepositoryOverview;
  wikiResult: RepositoryWikiFetchResult;
};

function formatDate(value: string | null | undefined) {
  if (!value) return "Not recorded";
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function shortRevisionLabel(wiki: RepositoryWikiView) {
  const revision = wiki.page?.revision;
  if (!revision) return "No revision";
  return revision.shortOid ?? revision.message;
}

function OutlineLinks({
  headings,
  label,
}: {
  headings: RepositoryWikiHeading[];
  label: string;
}) {
  if (headings.length === 0) return null;

  return (
    <nav aria-label={label} className="mt-3 grid gap-2">
      {headings.map((heading) => (
        <Link
          className="t-sm break-words hover:underline"
          href={heading.href}
          key={heading.id}
          style={{
            color: "var(--ink-3)",
            paddingLeft: `${Math.max(heading.level - 1, 0) * 10}px`,
          }}
        >
          {heading.text}
        </Link>
      ))}
    </nav>
  );
}

function WikiRenderedBlock({
  block,
  label,
}: {
  block: RepositoryWikiRenderedBlock;
  label: string;
}) {
  return (
    <section className="card p-4">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {label}
      </p>
      <h3 className="t-h3 mt-2 break-words" style={{ color: "var(--ink-1)" }}>
        <Link className="hover:underline" href={block.href}>
          {block.title}
        </Link>
      </h3>
      <div className="mt-3">
        <MarkdownBody html={block.html} />
      </div>
    </section>
  );
}

function WikiUnavailableState({
  title,
  message,
  canEdit,
  newPageHref,
}: {
  title: string;
  message: string;
  canEdit: boolean;
  newPageHref: string | null;
}) {
  return (
    <section className="card p-5">
      <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository wiki
          </p>
          <h1
            className="t-h1 mt-2 break-words"
            style={{ color: "var(--ink-1)" }}
          >
            {title}
          </h1>
          <p className="t-sm mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {message}
          </p>
        </div>
        {canEdit && newPageHref ? (
          <Link className="btn primary" href={newPageHref}>
            New Page
          </Link>
        ) : (
          <span className="chip soft">Reader view</span>
        )}
      </div>
    </section>
  );
}

function WikiReader({ wiki }: { wiki: RepositoryWikiView }) {
  const page = wiki.page;
  const canEdit = wiki.viewer.canEditWiki;

  if (wiki.state.kind !== "ready" || !page) {
    const title =
      wiki.state.kind === "disabled"
        ? "Wiki is disabled"
        : wiki.state.kind === "missing_page"
          ? "Wiki page not found"
          : "No wiki pages yet";
    return (
      <WikiUnavailableState
        canEdit={canEdit}
        message={wiki.state.message}
        newPageHref={wiki.links.newPageHref}
        title={title}
      />
    );
  }

  return (
    <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_300px]">
      <main className="min-w-0">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository wiki
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              id="repository-wiki-title"
              style={{ color: "var(--ink-1)" }}
            >
              {page.title}
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Updated {formatDate(page.revision.createdAt)}
              {page.revision.author ? (
                <> by {page.revision.author.login}</>
              ) : null}
              {" at "}
              <Link
                className="t-mono-sm hover:underline"
                href={page.revision.href}
              >
                {shortRevisionLabel(wiki)}
              </Link>
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            {page.historyHref ? (
              <Link className="btn sm" href={page.historyHref}>
                History
              </Link>
            ) : null}
            {canEdit && page.editHref ? (
              <Link className="btn primary" href={page.editHref}>
                Edit
              </Link>
            ) : null}
            {canEdit && wiki.links.newPageHref ? (
              <Link className="btn" href={wiki.links.newPageHref}>
                New Page
              </Link>
            ) : null}
          </div>
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
            <MarkdownBody html={page.html} labelledBy="repository-wiki-title" />
          </div>
        </article>

        {wiki.footer ? (
          <div className="mt-5">
            <WikiRenderedBlock block={wiki.footer} label="Wiki footer" />
          </div>
        ) : null}
      </main>

      <aside className="grid content-start gap-4">
        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Pages
          </p>
          <WikiPageList
            currentOutline={page.outline}
            owner={wiki.repository.ownerLogin}
            pages={wiki.pages}
            repo={wiki.repository.name}
          />
        </section>

        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            On this page
          </p>
          <OutlineLinks headings={page.outline} label="Wiki page headings" />
        </section>

        {wiki.sidebar ? (
          <WikiRenderedBlock block={wiki.sidebar} label="Custom sidebar" />
        ) : null}

        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Clone this wiki locally
          </p>
          <div className="mt-3 flex min-w-0 items-center gap-2">
            <code
              className="t-mono-sm min-w-0 flex-1 overflow-x-auto rounded-md border px-3 py-2"
              style={{
                borderColor: "var(--line)",
                background: "var(--surface-2)",
                color: "var(--ink-2)",
              }}
            >
              {wiki.clone.httpsUrl}
            </code>
            <CopyButton
              className="btn sm"
              copiedLabel="Copied URL"
              label="Copy"
              value={wiki.clone.httpsUrl}
            />
          </div>
        </section>
      </aside>
    </div>
  );
}

export function RepositoryWikiPage({
  repository,
  wikiResult,
}: RepositoryWikiPageProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/wiki`}
      repository={repository}
    >
      {wikiResult.ok ? (
        <WikiReader wiki={wikiResult.wiki} />
      ) : (
        <WikiUnavailableState
          canEdit={false}
          message={wikiResult.message}
          newPageHref={null}
          title="Wiki unavailable"
        />
      )}
    </RepositoryShell>
  );
}
