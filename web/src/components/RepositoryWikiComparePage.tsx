"use client";

import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryOverview,
  RepositoryWikiCompareFetchResult,
  RepositoryWikiCompareRevisionSummary,
  RepositoryWikiCompareView,
  RepositoryWikiDiffLine,
} from "@/lib/api";

type RepositoryWikiComparePageProps = {
  repository: RepositoryOverview;
  compareResult: RepositoryWikiCompareFetchResult;
};

function revisionLabel(revision: RepositoryWikiCompareRevisionSummary) {
  return revision.shortOid ?? revision.id.slice(0, 8);
}

function authorLabel(revision: RepositoryWikiCompareRevisionSummary) {
  return revision.author?.displayName ?? revision.author?.login ?? "Unknown";
}

function lineClass(line: RepositoryWikiDiffLine) {
  if (line.kind === "addition") return "wiki-diff-line wiki-diff-line-add";
  if (line.kind === "deletion") return "wiki-diff-line wiki-diff-line-del";
  return "wiki-diff-line";
}

function linePrefix(line: RepositoryWikiDiffLine) {
  if (line.kind === "addition") return "+";
  if (line.kind === "deletion") return "-";
  return " ";
}

function CompareReader({ compare }: { compare: RepositoryWikiCompareView }) {
  return (
    <div className="grid gap-5">
      <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository wiki
          </p>
          <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
            Compare revisions
          </h1>
          <p className="t-sm mt-3 max-w-3xl" style={{ color: "var(--ink-3)" }}>
            Changes to{" "}
            <Link className="hover:underline" href={compare.links.pageHref}>
              {compare.page.title}
            </Link>{" "}
            from{" "}
            <Link
              className="t-mono-sm hover:underline"
              href={compare.base.href}
            >
              {revisionLabel(compare.base)}
            </Link>{" "}
            to{" "}
            <Link
              className="t-mono-sm hover:underline"
              href={compare.head.href}
            >
              {revisionLabel(compare.head)}
            </Link>
            .
          </p>
        </div>
        <div className="flex flex-wrap gap-2 md:justify-end">
          <Link className="btn sm" href={compare.links.historyHref}>
            History
          </Link>
          <Link className="btn sm" href={compare.links.pageHref}>
            Page
          </Link>
        </div>
      </section>

      <section className="card p-4" aria-label="Compare summary">
        <div className="grid gap-3 md:grid-cols-[1fr_auto_1fr] md:items-center">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Base
            </p>
            <Link
              className="t-h3 mt-1 block hover:underline"
              href={compare.base.href}
            >
              {compare.base.message}
            </Link>
            <p className="t-xs mt-1">
              {revisionLabel(compare.base)} by {authorLabel(compare.base)}
            </p>
          </div>
          <span className="chip soft justify-self-start md:justify-self-center">
            {compare.stats.additions} additions / {compare.stats.deletions}{" "}
            deletions
          </span>
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Head
            </p>
            <Link
              className="t-h3 mt-1 block hover:underline"
              href={compare.head.href}
            >
              {compare.head.message}
            </Link>
            <p className="t-xs mt-1">
              {revisionLabel(compare.head)} by {authorLabel(compare.head)}
            </p>
          </div>
        </div>
        {compare.stats.truncated ? (
          <p className="chip warn mt-4">
            Large wiki diff truncated for inline viewing.
          </p>
        ) : null}
      </section>

      <section className="grid gap-4" aria-label="Wiki diff">
        {compare.files.map((file) => (
          <article className="card overflow-hidden" key={file.path}>
            <header
              className="flex flex-wrap items-center justify-between gap-2 px-4 py-3"
              style={{ borderBottom: "1px solid var(--line)" }}
            >
              <div>
                <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
                  {file.path}
                </h2>
                <p className="t-xs">
                  {file.oldPath} {"->"} {file.newPath}
                </p>
              </div>
              <span className="chip soft">
                +{file.additions} / -{file.deletions}
              </span>
            </header>
            {file.hunks.map((hunk) => (
              <div className="overflow-x-auto" key={hunk.header}>
                <div className="t-mono-sm wiki-diff-hunk px-4 py-2">
                  {hunk.header}
                </div>
                <table className="w-full border-collapse t-mono-sm">
                  <tbody>
                    {hunk.lines.map((line) => (
                      <tr
                        className={lineClass(line)}
                        key={`${line.kind}-${line.oldNumber ?? "x"}-${line.newNumber ?? "x"}-${line.content}`}
                      >
                        <td className="wiki-diff-num">
                          {line.oldNumber ?? ""}
                        </td>
                        <td className="wiki-diff-num">
                          {line.newNumber ?? ""}
                        </td>
                        <td className="wiki-diff-prefix">{linePrefix(line)}</td>
                        <td className="wiki-diff-code">
                          <code>{line.content || " "}</code>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ))}
          </article>
        ))}
      </section>
    </div>
  );
}

function WikiCompareUnavailable({ message }: { message: string }) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Repository wiki
      </p>
      <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
        Compare unavailable
      </h1>
      <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
        {message}
      </p>
    </section>
  );
}

export function RepositoryWikiComparePage({
  repository,
  compareResult,
}: RepositoryWikiComparePageProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/wiki`}
      repository={repository}
    >
      {compareResult.ok ? (
        <CompareReader compare={compareResult.compare} />
      ) : (
        <WikiCompareUnavailable message={compareResult.message} />
      )}
    </RepositoryShell>
  );
}
