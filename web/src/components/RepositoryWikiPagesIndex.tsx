import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryOverview,
  RepositoryWikiPagesIndex as RepositoryWikiPagesIndexContract,
} from "@/lib/api";
import {
  repositoryWikiHref,
  repositoryWikiNewPageHref,
  repositoryWikiPagesHref,
} from "@/lib/navigation";

type RepositoryWikiPagesIndexProps = {
  repository: RepositoryOverview;
  pagesIndex:
    | { ok: true; value: RepositoryWikiPagesIndexContract }
    | { ok: false; message: string };
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

function DocumentIcon() {
  return (
    <svg
      aria-hidden="true"
      fill="none"
      height="18"
      viewBox="0 0 18 18"
      width="18"
    >
      <path
        d="M4.5 2.75h5.25L13.5 6.5v8.75H4.5V2.75Z"
        stroke="currentColor"
        strokeLinejoin="round"
        strokeWidth="1.35"
      />
      <path d="M9.75 2.9v3.7h3.55" stroke="currentColor" strokeWidth="1.35" />
    </svg>
  );
}

function UnavailableState({ message }: { message: string }) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Repository wiki
      </p>
      <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
        Pages unavailable
      </h1>
      <p className="t-sm mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {message}
      </p>
    </section>
  );
}

export function RepositoryWikiPagesIndex({
  repository,
  pagesIndex,
}: RepositoryWikiPagesIndexProps) {
  if (!pagesIndex.ok) {
    return (
      <RepositoryShell
        activePath={`/${repository.owner_login}/${repository.name}/wiki`}
        repository={repository}
      >
        <UnavailableState message={pagesIndex.message} />
      </RepositoryShell>
    );
  }

  const { value } = pagesIndex;
  const sortedPages = [...value.pages].sort((a, b) =>
    a.title.localeCompare(b.title, undefined, { sensitivity: "base" }),
  );
  const owner = value.repository.ownerLogin;
  const repo = value.repository.name;

  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/wiki`}
      repository={repository}
    >
      <div className="grid gap-5">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository wiki
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Pages
            </h1>
            <p
              className="t-sm mt-3 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              {sortedPages.length} wiki{" "}
              {sortedPages.length === 1 ? "page" : "pages"} in{" "}
              <Link
                className="t-mono-sm hover:underline"
                href={value.links.homeHref}
              >
                {owner}/{repo}
              </Link>
              .
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn" href={repositoryWikiHref(owner, repo)}>
              Wiki home
            </Link>
            {value.viewer.canEditWiki ? (
              <Link
                className="btn primary"
                href={repositoryWikiNewPageHref(owner, repo)}
              >
                New Page
              </Link>
            ) : (
              <span className="chip soft">Reader view</span>
            )}
          </div>
        </section>

        <section className="card overflow-hidden">
          <div
            className="border-b px-5 py-4"
            style={{ borderColor: "var(--line)" }}
          >
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              All pages
            </p>
          </div>
          {sortedPages.length > 0 ? (
            <div>
              {sortedPages.map((page) => (
                <div
                  className="list-row grid gap-3 px-5 py-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-center"
                  key={page.id}
                >
                  <div className="flex min-w-0 items-start gap-3">
                    <span
                      className="mt-0.5 shrink-0"
                      style={{ color: "var(--ink-4)" }}
                    >
                      <DocumentIcon />
                    </span>
                    <div className="min-w-0">
                      <Link
                        className="t-h3 break-words hover:underline"
                        href={repositoryWikiHref(owner, repo, page.slug)}
                        style={{ color: "var(--ink-1)" }}
                      >
                        {page.title}
                      </Link>
                      <p className="t-xs mt-1">
                        Last updated {formatDate(page.updatedAt)}
                      </p>
                    </div>
                  </div>
                  {value.viewer.canEditWiki ? (
                    <Link
                      className="btn sm"
                      href={`${repositoryWikiHref(owner, repo, page.slug)}/_edit`}
                    >
                      Edit
                    </Link>
                  ) : null}
                </div>
              ))}
            </div>
          ) : (
            <div className="px-5 py-8">
              <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
                No wiki pages yet
              </h2>
              <p
                className="t-sm mt-3 max-w-2xl"
                style={{ color: "var(--ink-3)" }}
              >
                Start with a Home page to publish internal notes, setup guides,
                or project context.
              </p>
              {value.viewer.canEditWiki ? (
                <Link
                  className="btn primary mt-4"
                  href={repositoryWikiNewPageHref(owner, repo)}
                >
                  New Page
                </Link>
              ) : null}
            </div>
          )}
        </section>

        <div className="flex flex-wrap gap-2">
          <Link className="btn sm" href={repositoryWikiPagesHref(owner, repo)}>
            Refresh pages
          </Link>
        </div>
      </div>
    </RepositoryShell>
  );
}
