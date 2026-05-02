import Link from "next/link";
import type { ReactNode } from "react";
import { RepositoryBlobViewer } from "@/components/RepositoryBlobViewer";
import { RepositoryTreeBrowser } from "@/components/RepositoryTreeBrowser";
import type {
  RepositoryBlameView,
  RepositoryBlobView,
  RepositoryCommitHistory,
  RepositoryPathBreadcrumb,
  RepositoryPathOverview,
} from "@/lib/api";

function Breadcrumbs({
  breadcrumbs,
}: {
  breadcrumbs: RepositoryPathBreadcrumb[];
}) {
  return (
    <nav
      aria-label="Breadcrumb"
      className="flex flex-wrap items-center gap-1 text-sm"
    >
      {breadcrumbs.map((breadcrumb, index) => (
        <span className="flex min-w-0 items-center gap-1" key={breadcrumb.href}>
          {index > 0 ? <span style={{ color: "var(--ink-3)" }}>/</span> : null}
          <Link
            className="max-w-48 truncate font-semibold hover:underline"
            href={breadcrumb.href}
            style={{ color: "var(--accent)" }}
          >
            {breadcrumb.name}
          </Link>
        </span>
      ))}
    </nav>
  );
}

function RepositoryPathHeader({
  owner,
  repo,
  visibility,
  children,
}: {
  owner: string;
  repo: string;
  visibility?: string;
  children: ReactNode;
}) {
  return (
    <header
      className="border-b px-6 py-5"
      style={{ borderColor: "var(--line)", background: "var(--surface-2)" }}
    >
      <div className="mx-auto max-w-7xl">
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {owner}
        </p>
        <div className="mt-1 flex flex-wrap items-center gap-2">
          <Link
            className="text-xl font-semibold tracking-normal hover:underline"
            href={`/${owner}/${repo}`}
            style={{ color: "var(--accent)" }}
          >
            {repo}
          </Link>
          {visibility ? (
            <span className="chip soft capitalize">{visibility}</span>
          ) : null}
        </div>
        <div className="mt-5">{children}</div>
      </div>
    </header>
  );
}

export function RepositoryTreeView({
  overview,
}: {
  overview: RepositoryPathOverview;
}) {
  return (
    <div>
      <RepositoryPathHeader
        owner={overview.owner_login}
        repo={overview.name}
        visibility={overview.visibility}
      >
        <Breadcrumbs breadcrumbs={overview.breadcrumbs} />
      </RepositoryPathHeader>
      <main className="mx-auto max-w-7xl space-y-4 px-6 py-6">
        <RepositoryTreeBrowser overview={overview} />
      </main>
    </div>
  );
}

export function RepositoryBlobViewPage({
  blob,
  initialBlame,
  initialMode,
  initialSymbolsOpen,
}: {
  blob: RepositoryBlobView;
  initialBlame?: RepositoryBlameView | null;
  initialMode?: "code" | "blame";
  initialSymbolsOpen?: boolean;
}) {
  return (
    <div>
      <RepositoryPathHeader
        owner={blob.owner_login}
        repo={blob.name}
        visibility={blob.visibility}
      >
        <Breadcrumbs breadcrumbs={blob.breadcrumbs} />
      </RepositoryPathHeader>
      <main className="mx-auto max-w-7xl space-y-4 px-6 py-6">
        <RepositoryBlobViewer
          blob={blob}
          initialBlame={initialBlame}
          initialMode={initialMode}
          initialSymbolsOpen={initialSymbolsOpen}
        />
      </main>
    </div>
  );
}

function formatCommitGroupDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Unknown date";
  }
  return new Intl.DateTimeFormat("en", {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function formatCommitMetaDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "recently";
  }
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function commitHistoryQueryString(
  history: RepositoryCommitHistory,
  overrides: Record<string, string | null>,
) {
  const params = new URLSearchParams();
  const next = {
    author: history.filters.author,
    since: history.filters.since,
    until: history.filters.until,
    ...overrides,
  };
  for (const [key, value] of Object.entries(next)) {
    if (value) {
      params.set(key, value);
    }
  }
  const query = params.toString();
  return query ? `?${query}` : "";
}

function commitHistoryBase(
  owner: string,
  repo: string,
  refName: string,
  path: string | null,
) {
  const base = `/${owner}/${repo}/commits/${encodeURIComponent(refName)}`;
  return path
    ? `${base}/${path.split("/").map(encodeURIComponent).join("/")}`
    : base;
}

export function RepositoryCommitHistoryView({
  owner,
  repo,
  path,
  history,
}: {
  owner: string;
  repo: string;
  path: string;
  history: RepositoryCommitHistory;
}) {
  const basePath = commitHistoryBase(
    owner,
    repo,
    history.filters.ref,
    path || null,
  );
  const clearHref = basePath;
  const grouped = history.items.reduce<
    Array<{ date: string; commits: typeof history.items }>
  >((groups, commit) => {
    const date = formatCommitGroupDate(commit.committedAt);
    const existing = groups.find((group) => group.date === date);
    if (existing) {
      existing.commits.push(commit);
    } else {
      groups.push({ date, commits: [commit] });
    }
    return groups;
  }, []);
  const activeFilterCount = [
    history.filters.author,
    history.filters.since,
    history.filters.until,
  ].filter(Boolean).length;

  return (
    <div>
      <RepositoryPathHeader owner={owner} repo={repo}>
        <div className="min-w-0">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Commit history
          </p>
          <h1 className="t-h2 mt-1" style={{ color: "var(--ink-1)" }}>
            {path || history.resolvedRef.shortName}
          </h1>
        </div>
      </RepositoryPathHeader>
      <main className="mx-auto max-w-7xl space-y-5 px-6 py-6">
        <section className="card p-4" aria-label="Commit history filters">
          <div className="flex flex-wrap items-end gap-3">
            <div className="min-w-56 flex-1">
              <p className="t-label">Branch or tag</p>
              <div className="mt-2 flex flex-wrap gap-2">
                {history.refs.slice(0, 8).map((ref) => (
                  <Link
                    className={`chip ${ref.active ? "active" : "soft"}`}
                    href={`${commitHistoryBase(owner, repo, ref.shortName, path || null)}${commitHistoryQueryString(history, {})}`}
                    key={ref.name}
                  >
                    {ref.shortName}
                  </Link>
                ))}
              </div>
            </div>
            <form action={basePath} className="flex flex-wrap items-end gap-3">
              <div>
                <label className="t-label block" htmlFor="commit-author-filter">
                  Author
                </label>
                <select
                  className="input mt-2 min-w-44"
                  defaultValue={history.filters.author ?? ""}
                  id="commit-author-filter"
                  name="author"
                >
                  <option value="">All authors</option>
                  {history.authors.map((author) => (
                    <option key={author.login} value={author.login}>
                      {author.login} ({author.count})
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="t-label block" htmlFor="commit-since-filter">
                  Since
                </label>
                <input
                  className="input mt-2"
                  defaultValue={history.filters.since ?? ""}
                  id="commit-since-filter"
                  name="since"
                  type="date"
                />
              </div>
              <div>
                <label className="t-label block" htmlFor="commit-until-filter">
                  Until
                </label>
                <input
                  className="input mt-2"
                  defaultValue={history.filters.until ?? ""}
                  id="commit-until-filter"
                  name="until"
                  type="date"
                />
              </div>
              <button className="btn primary" type="submit">
                Apply filters
              </button>
              {activeFilterCount > 0 ? (
                <Link className="btn ghost" href={clearHref}>
                  Clear
                </Link>
              ) : null}
            </form>
          </div>
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            Showing <span className="t-num">{history.items.length}</span> of{" "}
            <span className="t-num">{history.total}</span> commits on{" "}
            <span className="chip soft">{history.resolvedRef.kind}</span>{" "}
            <span className="t-mono-sm">{history.resolvedRef.shortName}</span>
            {path ? (
              <>
                {" "}
                for <span className="t-mono-sm">{path}</span>
              </>
            ) : null}
            .
          </p>
        </section>

        {grouped.length > 0 ? (
          <section className="space-y-4" aria-label="Grouped commits">
            {grouped.map((group) => (
              <div className="card overflow-hidden" key={group.date}>
                <div
                  className="border-b px-4 py-3"
                  style={{
                    borderColor: "var(--line)",
                    background: "var(--surface-2)",
                  }}
                >
                  <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
                    {group.date}
                  </h2>
                </div>
                <ul>
                  {group.commits.map((commit) => (
                    <li
                      className="border-b last:border-b-0"
                      key={commit.oid}
                      style={{ borderColor: "var(--line-soft)" }}
                    >
                      <div className="list-row grid grid-cols-[minmax(0,1fr)_auto] gap-4 px-4 py-4 max-md:grid-cols-1">
                        <div className="min-w-0">
                          <Link
                            className="block truncate font-semibold hover:underline"
                            href={commit.href}
                            style={{ color: "var(--ink-1)" }}
                          >
                            {commit.message}
                          </Link>
                          <p
                            className="t-sm mt-1"
                            style={{ color: "var(--ink-3)" }}
                          >
                            {commit.authorDisplayName ??
                              commit.authorLogin ??
                              "Unknown author"}
                            {commit.authorLogin ? (
                              <>
                                {" "}
                                authored on{" "}
                                {formatCommitMetaDate(commit.committedAt)}
                              </>
                            ) : null}
                          </p>
                        </div>
                        <div className="flex flex-wrap items-center justify-end gap-2 max-md:justify-start">
                          <Link className="chip soft" href={commit.statusHref}>
                            {commit.statusLabel}
                          </Link>
                          <Link className="btn sm ghost" href={commit.treeHref}>
                            Browse
                          </Link>
                          <Link
                            className="t-mono-sm hover:underline"
                            href={commit.href}
                            style={{ color: "var(--accent)" }}
                          >
                            {commit.shortOid}
                          </Link>
                        </div>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </section>
        ) : (
          <section className="card p-6" role="status">
            <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
              No commits match these filters
            </h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Clear the author or date filters, or choose another branch or tag.
            </p>
            <Link className="btn mt-4 inline-flex" href={clearHref}>
              Clear filters
            </Link>
          </section>
        )}
      </main>
    </div>
  );
}
