import { AppShell } from "@/components/AppShell";
import { RepositoryShell } from "@/components/RepositoryShell";
import { RepositoryUnavailablePage } from "@/components/RepositoryUnavailablePage";
import { RepositoryWikiEditor } from "@/components/RepositoryWikiEditor";
import { RepositoryWikiHistoryPage } from "@/components/RepositoryWikiHistoryPage";
import { RepositoryWikiPage as RepositoryWikiView } from "@/components/RepositoryWikiPage";
import { RepositoryWikiPagesIndex } from "@/components/RepositoryWikiPagesIndex";
import { RepositoryWikiRevisionPage } from "@/components/RepositoryWikiRevisionPage";
import {
  getRepository,
  getRepositoryWiki,
  getRepositoryWikiEdit,
  getRepositoryWikiHistory,
  getRepositoryWikiPages,
  getRepositoryWikiRevision,
  getSession,
} from "@/lib/server-session";

type RepositoryWikiSlugPageProps = {
  params: Promise<{ owner: string; repo: string; slug: string[] }>;
  searchParams: Promise<{ page?: string; pageSize?: string }>;
};

function decodePathSegment(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function wikiSlugFromParams(slug: string[]) {
  return slug.map(decodePathSegment).filter(Boolean).join("/");
}

function positiveNumber(value: string | undefined) {
  if (!value) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : null;
}

export default async function RepositoryWikiSlugPage({
  params,
  searchParams,
}: RepositoryWikiSlugPageProps) {
  const [{ owner, repo, slug }, query, session] = await Promise.all([
    params,
    searchParams,
    getSession(),
  ]);
  const ownerLogin = decodePathSegment(owner);
  const repositoryName = decodePathSegment(repo);
  const wikiSlug = wikiSlugFromParams(slug);
  const repository = await getRepository(ownerLogin, repositoryName);

  if (!repository) {
    return (
      <AppShell session={session}>
        <RepositoryUnavailablePage owner={ownerLogin} repo={repositoryName} />
      </AppShell>
    );
  }

  if (wikiSlug === "_pages") {
    const pagesIndex = await getRepositoryWikiPages(ownerLogin, repositoryName)
      .then((value) => ({ ok: true as const, value }))
      .catch((error) => ({
        ok: false as const,
        message:
          error instanceof Error
            ? error.message
            : "Repository wiki pages failed to load.",
      }));

    return (
      <AppShell session={session}>
        <RepositoryWikiPagesIndex
          pagesIndex={pagesIndex}
          repository={repository}
        />
      </AppShell>
    );
  }

  if (wikiSlug === "_history" || wikiSlug.endsWith("/_history")) {
    const historySlug = wikiSlug.endsWith("/_history")
      ? wikiSlug.slice(0, -"/_history".length) || null
      : null;
    const historyResult = await getRepositoryWikiHistory(
      ownerLogin,
      repositoryName,
      historySlug,
      positiveNumber(query.page),
      positiveNumber(query.pageSize),
    );

    return (
      <AppShell session={session}>
        <RepositoryWikiHistoryPage
          historyResult={historyResult}
          repository={repository}
        />
      </AppShell>
    );
  }

  const revisionMarker = "/_history/";
  if (wikiSlug.includes(revisionMarker)) {
    const [revisionSlug, revisionRef] = wikiSlug.split(revisionMarker);
    const revisionResult =
      revisionSlug && revisionRef
        ? await getRepositoryWikiRevision(
            ownerLogin,
            repositoryName,
            revisionSlug,
            revisionRef,
          )
        : {
            ok: false as const,
            status: 404,
            code: "not_found",
            message: "Repository wiki revision was not found.",
          };

    return (
      <AppShell session={session}>
        <RepositoryWikiRevisionPage
          repository={repository}
          revisionResult={revisionResult}
        />
      </AppShell>
    );
  }

  if (wikiSlug === "_new") {
    const pagesIndex = await getRepositoryWikiPages(ownerLogin, repositoryName)
      .then((value) => ({ ok: true as const, value }))
      .catch(() => null);

    return (
      <AppShell session={session}>
        <RepositoryShell
          activePath={`/${repository.owner_login}/${repository.name}/wiki`}
          repository={repository}
        >
          {pagesIndex?.ok ? (
            <RepositoryWikiEditor
              pagesIndex={pagesIndex.value}
              repository={repository}
            />
          ) : (
            <section className="card p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Repository wiki
              </p>
              <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
                Editor unavailable
              </h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                Repository wiki pages failed to load.
              </p>
            </section>
          )}
        </RepositoryShell>
      </AppShell>
    );
  }

  if (wikiSlug.endsWith("/_edit")) {
    const editSlug = wikiSlug.slice(0, -"/_edit".length);
    const [pagesIndex, editView] = await Promise.all([
      getRepositoryWikiPages(ownerLogin, repositoryName)
        .then((value) => ({ ok: true as const, value }))
        .catch(() => null),
      getRepositoryWikiEdit(ownerLogin, repositoryName, editSlug).catch(
        () => null,
      ),
    ]);

    return (
      <AppShell session={session}>
        <RepositoryShell
          activePath={`/${repository.owner_login}/${repository.name}/wiki`}
          repository={repository}
        >
          {pagesIndex?.ok && editView ? (
            <RepositoryWikiEditor
              editView={editView}
              pagesIndex={pagesIndex.value}
              repository={repository}
            />
          ) : (
            <section className="card p-5">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Repository wiki
              </p>
              <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
                Editor unavailable
              </h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                The selected wiki page could not be opened for editing.
              </p>
            </section>
          )}
        </RepositoryShell>
      </AppShell>
    );
  }

  const wikiResult = await getRepositoryWiki(
    ownerLogin,
    repositoryName,
    wikiSlug,
  );

  return (
    <AppShell session={session}>
      <RepositoryWikiView repository={repository} wikiResult={wikiResult} />
    </AppShell>
  );
}
