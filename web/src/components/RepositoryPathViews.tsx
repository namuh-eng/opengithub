import Link from "next/link";
import type { ReactNode } from "react";
import { RepositoryBlobViewer } from "@/components/RepositoryBlobViewer";
import { RepositoryTreeBrowser } from "@/components/RepositoryTreeBrowser";
import type {
  ListEnvelope,
  RepositoryBlameView,
  RepositoryBlobView,
  RepositoryCommitHistoryItem,
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

export function RepositoryCommitHistoryView({
  owner,
  repo,
  refName,
  path,
  history,
}: {
  owner: string;
  repo: string;
  refName: string;
  path: string;
  history: ListEnvelope<RepositoryCommitHistoryItem>;
}) {
  return (
    <div>
      <RepositoryPathHeader owner={owner} repo={repo}>
        <h1 className="t-h3" style={{ color: "var(--ink-1)" }}>
          Commit history for {path || refName}
        </h1>
      </RepositoryPathHeader>
      <main className="mx-auto max-w-7xl px-6 py-6">
        <div
          className="overflow-hidden rounded-md"
          style={{
            border: "1px solid var(--line)",
            background: "var(--surface)",
          }}
        >
          {history.items.map((commit) => (
            <Link
              className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 border-b px-4 py-3 text-sm last:border-b-0 hover:bg-[var(--surface-2)] max-md:grid-cols-1"
              href={commit.href}
              key={commit.oid}
              style={{ borderColor: "var(--line-soft)" }}
            >
              <span className="min-w-0">
                <span
                  className="block truncate font-semibold"
                  style={{ color: "var(--ink-1)" }}
                >
                  {commit.message}
                </span>
                <span
                  className="mt-1 block t-xs"
                  style={{ color: "var(--ink-3)" }}
                >
                  {commit.authorLogin ?? "Unknown author"}
                  {commit.signatureSummary ? (
                    <>
                      {" "}
                      <span aria-hidden="true">·</span>{" "}
                      <span
                        className={`chip ${commit.verified ? "ok" : "warn"}`}
                      >
                        {commit.verified ? "Verified" : "Unverified"}
                      </span>{" "}
                      <span>{commit.signatureSummary}</span>
                    </>
                  ) : null}
                </span>
              </span>
              <span className="t-mono-sm" style={{ color: "var(--accent)" }}>
                {commit.shortOid}
              </span>
            </Link>
          ))}
          {history.items.length === 0 ? (
            <p className="p-6 t-sm" style={{ color: "var(--ink-3)" }}>
              No commits are recorded for this path.
            </p>
          ) : null}
        </div>
      </main>
    </div>
  );
}
