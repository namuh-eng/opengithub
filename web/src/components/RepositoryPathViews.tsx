import Link from "next/link";
import type { ReactNode } from "react";
import { RepositoryBlobViewer } from "@/components/RepositoryBlobViewer";
import { RepositoryCommitHistoryPage } from "@/components/RepositoryCommitHistoryPage";
import { RepositoryTreeBrowser } from "@/components/RepositoryTreeBrowser";
import type {
  RepositoryBlameView,
  RepositoryBlobView,
  RepositoryPathBreadcrumb,
  RepositoryPathOverview,
} from "@/lib/api";

export { RepositoryCommitHistoryPage as RepositoryCommitHistoryView };

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
