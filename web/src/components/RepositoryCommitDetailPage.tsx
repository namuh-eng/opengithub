"use client";

import Link from "next/link";
import type React from "react";
import { useMemo, useState } from "react";
import { CopyButton } from "@/components/CopyButton";
import type {
  RepositoryCommitDetailFile,
  RepositoryCommitDetailFileTreeNode,
  RepositoryCommitDetailLine,
  RepositoryCommitDetailView,
  RepositoryCommitStatusSummary,
  RepositoryCommitVerificationSummary,
} from "@/lib/api";

type RepositoryCommitDetailPageProps = {
  detail: RepositoryCommitDetailView;
};

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const diffMs = Date.now() - timestamp;
  const absMs = Math.abs(diffMs);
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 1000 * 60 * 60 * 24 * 365],
    ["month", 1000 * 60 * 60 * 24 * 30],
    ["day", 1000 * 60 * 60 * 24],
    ["hour", 1000 * 60 * 60],
    ["minute", 1000 * 60],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, unitMs] of units) {
    if (absMs >= unitMs) {
      return formatter.format(Math.round(-diffMs / unitMs), unit);
    }
  }
  return "just now";
}

function initials(login: string | null) {
  const fallback = login?.trim() || "unknown";
  return fallback
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function statusLabel(status: RepositoryCommitStatusSummary) {
  if (status.totalCount === 0) {
    return "No checks";
  }
  if (status.status === "running") {
    return `${status.completedCount}/${status.totalCount} checks running`;
  }
  if (status.conclusion === "success") {
    return `${status.totalCount} checks passed`;
  }
  if (status.failedCount > 0 || status.conclusion === "failure") {
    return `${status.failedCount || 1} checks failed`;
  }
  return `${status.completedCount}/${status.totalCount} checks complete`;
}

function statusChipClass(status: RepositoryCommitStatusSummary) {
  if (status.totalCount === 0) {
    return "chip soft";
  }
  if (status.conclusion === "success") {
    return "chip ok";
  }
  if (status.failedCount > 0 || status.conclusion === "failure") {
    return "chip err";
  }
  if (status.status === "running") {
    return "chip accent";
  }
  return "chip warn";
}

function verificationLabel(verification: RepositoryCommitVerificationSummary) {
  if (verification.verified) {
    return "Verified";
  }
  if (verification.signatureState === "vigilant_unverified") {
    return "Partially verified";
  }
  return "Unverified";
}

function verificationClass(verification: RepositoryCommitVerificationSummary) {
  if (verification.verified) {
    return "chip ok";
  }
  if (verification.signatureState === "vigilant_unverified") {
    return "chip warn";
  }
  return "chip soft";
}

function fileStatusMark(status: string) {
  if (status === "added") return "A";
  if (status === "removed") return "D";
  if (status === "renamed") return "R";
  return "M";
}

function linePrefix(line: RepositoryCommitDetailLine) {
  if (line.kind === "added") return "+";
  if (line.kind === "removed") return "-";
  return " ";
}

function lineBackground(line: RepositoryCommitDetailLine) {
  if (line.kind === "added")
    return "color-mix(in oklab, var(--ok) 10%, transparent)";
  if (line.kind === "removed")
    return "color-mix(in oklab, var(--err) 10%, transparent)";
  return "transparent";
}

function lineAccent(line: RepositoryCommitDetailLine) {
  if (line.kind === "added") return "var(--ok)";
  if (line.kind === "removed") return "var(--err)";
  return "var(--ink-4)";
}

function formatByteSize(byteSize: number) {
  if (byteSize < 1024) return `${byteSize} bytes`;
  const kib = byteSize / 1024;
  if (kib < 1024) return `${kib.toFixed(1)} KB`;
  return `${(kib / 1024).toFixed(1)} MB`;
}

function normalizeQuery(value: string) {
  return value.trim().toLocaleLowerCase();
}

function textMatches(value: string, query: string) {
  return value.toLocaleLowerCase().includes(query);
}

function countLineMatches(line: RepositoryCommitDetailLine, query: string) {
  if (!query) return 0;
  const haystack = line.content.toLocaleLowerCase();
  let count = 0;
  let index = haystack.indexOf(query);
  while (index >= 0) {
    count += 1;
    index = haystack.indexOf(query, index + query.length);
  }
  return count;
}

function searchMatchCount(
  files: RepositoryCommitDetailFile[],
  searchQuery: string,
) {
  const normalized = normalizeQuery(searchQuery);
  if (!normalized) return 0;
  return files.reduce(
    (fileTotal, file) =>
      fileTotal +
      file.hunks.reduce(
        (hunkTotal, hunk) =>
          hunkTotal +
          hunk.lines.reduce(
            (lineTotal, line) => lineTotal + countLineMatches(line, normalized),
            0,
          ),
        0,
      ),
    0,
  );
}

function HighlightedCode({
  content,
  searchQuery,
}: {
  content: string;
  searchQuery: string;
}) {
  const normalizedQuery = normalizeQuery(searchQuery);
  if (!normalizedQuery) {
    return <>{content}</>;
  }

  const lowerContent = content.toLocaleLowerCase();
  const parts: React.ReactNode[] = [];
  let cursor = 0;
  let matchIndex = lowerContent.indexOf(normalizedQuery);
  while (matchIndex >= 0) {
    if (matchIndex > cursor) {
      parts.push(content.slice(cursor, matchIndex));
    }
    const match = content.slice(
      matchIndex,
      matchIndex + normalizedQuery.length,
    );
    parts.push(
      <mark
        className="rounded-[var(--radius)] px-0.5"
        key={`${matchIndex}-${match}`}
        style={{
          background: "var(--accent-soft)",
          color: "var(--ink-1)",
        }}
      >
        {match}
      </mark>,
    );
    cursor = matchIndex + normalizedQuery.length;
    matchIndex = lowerContent.indexOf(normalizedQuery, cursor);
  }
  if (cursor < content.length) {
    parts.push(content.slice(cursor));
  }
  return <>{parts}</>;
}

function DiffLine({
  line,
  searchQuery,
}: {
  line: RepositoryCommitDetailLine;
  searchQuery: string;
}) {
  return (
    <div
      className="grid min-w-[760px] grid-cols-[64px_64px_32px_minmax(0,1fr)] border-b t-mono-sm"
      style={{
        background: lineBackground(line),
        borderColor: "var(--line-soft)",
      }}
    >
      <span
        className="select-none px-3 py-1.5 text-right"
        style={{ color: "var(--ink-4)" }}
      >
        {line.oldLine ?? ""}
      </span>
      <span
        className="select-none border-l px-3 py-1.5 text-right"
        style={{ borderColor: "var(--line-soft)", color: "var(--ink-4)" }}
      >
        {line.newLine ?? ""}
      </span>
      <span
        className="select-none border-l px-2 py-1.5"
        style={{ borderColor: "var(--line-soft)", color: lineAccent(line) }}
      >
        {linePrefix(line)}
      </span>
      <code className="min-w-0 whitespace-pre px-2 py-1.5">
        <HighlightedCode content={line.content} searchQuery={searchQuery} />
      </code>
    </div>
  );
}

function RepositoryCommitDiffFile({
  file,
  isActive,
  searchQuery,
}: {
  file: RepositoryCommitDetailFile;
  isActive: boolean;
  searchQuery: string;
}) {
  return (
    <article
      aria-label={`Diff for ${file.path}${isActive ? " selected" : ""}`}
      className="card scroll-mt-24 overflow-hidden outline-none"
      id={file.anchor}
      tabIndex={-1}
    >
      <div
        className="flex flex-wrap items-center gap-3 border-b px-4 py-3"
        style={{ background: "var(--surface-2)", borderColor: "var(--line)" }}
      >
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {fileStatusMark(file.status)}
        </span>
        <h3 className="t-mono-sm min-w-0 flex-1 break-all">{file.path}</h3>
        {file.language ? (
          <span className="chip soft">{file.language}</span>
        ) : null}
        <span className="t-xs t-num">
          <span style={{ color: "var(--ok)" }}>+{file.additions}</span>{" "}
          <span style={{ color: "var(--err)" }}>-{file.deletions}</span>
        </span>
        <Link className="btn ghost sm" href={file.rawHref}>
          Raw
        </Link>
        <Link className="btn sm" href={file.viewHref}>
          View file
        </Link>
      </div>
      {file.isBinary || file.isLarge ? (
        <div className="px-4 py-5">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            {file.isBinary
              ? "Binary file diff is not rendered inline."
              : `Large file diff is bounded inline (${formatByteSize(file.byteSize)}).`}
          </p>
        </div>
      ) : file.hunks.length ? (
        <div className="overflow-x-auto">
          {file.hunks.map((hunk) => (
            <div key={hunk.id}>
              <div
                className="border-b px-4 py-2 t-mono-sm"
                style={{
                  background: "var(--surface-3)",
                  borderColor: "var(--line-soft)",
                  color: "var(--ink-3)",
                }}
              >
                {hunk.header}
              </div>
              {hunk.lines.map((line) => (
                <DiffLine
                  key={`${hunk.id}-${line.position}`}
                  line={line}
                  searchQuery={searchQuery}
                />
              ))}
            </div>
          ))}
        </div>
      ) : (
        <div className="px-4 py-5">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            This file has summary metadata, but no expanded text rows are
            available.
          </p>
        </div>
      )}
    </article>
  );
}

function RepositoryCommitFileTree({
  activeFilePath,
  fileTree,
  onSelectFile,
}: {
  activeFilePath: string | null;
  fileTree: RepositoryCommitDetailFileTreeNode[];
  onSelectFile: (node: RepositoryCommitDetailFileTreeNode) => void;
}) {
  return (
    <nav
      aria-label="Changed file tree"
      className="mt-2 max-h-[520px] space-y-1 overflow-y-auto"
    >
      {fileTree.map((node) => {
        const active = node.path === activeFilePath;
        return (
          <button
            aria-pressed={active}
            className="flex w-full items-center gap-2 rounded-[var(--radius)] px-2 py-1.5 text-left t-sm hover:bg-[var(--hover)]"
            key={node.path}
            onClick={() => onSelectFile(node)}
            style={{
              background: active ? "var(--accent-soft)" : "transparent",
              paddingLeft: 8 + node.depth * 12,
            }}
            type="button"
          >
            <span className="sr-only">
              {active ? "Selected file " : "Focus file "}
            </span>
            <span className="t-mono-sm" style={{ color: "var(--ink-4)" }}>
              {fileStatusMark(node.status)}
            </span>
            <span className="min-w-0 flex-1 truncate t-mono-sm">
              {node.name}
            </span>
            <span className="t-xs t-num" style={{ color: "var(--ink-4)" }}>
              +{node.additions}/-{node.deletions}
            </span>
          </button>
        );
      })}
    </nav>
  );
}

function RepositoryCommitDiffSearch({
  fileFilter,
  matchCount,
  onClear,
  onFileFilterChange,
  onSearchQueryChange,
  searchQuery,
  visibleFileCount,
}: {
  fileFilter: string;
  matchCount: number;
  onClear: () => void;
  onFileFilterChange: (value: string) => void;
  onSearchQueryChange: (value: string) => void;
  searchQuery: string;
  visibleFileCount: number;
}) {
  return (
    <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] md:items-end">
      <label className="block">
        <span className="t-label">Filter files</span>
        <input
          aria-label="Filter files"
          className="input mt-2 h-9 w-full px-3 t-sm"
          onChange={(event) => onFileFilterChange(event.target.value)}
          placeholder="Path or filename"
          value={fileFilter}
        />
      </label>
      <label className="block">
        <span className="t-label">Search within code</span>
        <input
          aria-label="Search within code"
          className="input mt-2 h-9 w-full px-3 t-sm"
          onChange={(event) => onSearchQueryChange(event.target.value)}
          placeholder="Function, selector, or text"
          value={searchQuery}
        />
      </label>
      <button className="btn sm" onClick={onClear} type="button">
        Clear filters
      </button>
      <p className="t-xs md:col-span-3" role="status">
        {visibleFileCount} visible {visibleFileCount === 1 ? "file" : "files"}
        {searchQuery.trim()
          ? ` · ${matchCount} ${matchCount === 1 ? "match" : "matches"}`
          : ""}
      </p>
    </div>
  );
}

export function RepositoryCommitDetailPage({
  detail,
}: RepositoryCommitDetailPageProps) {
  const repository = detail.repository;
  const commit = detail.commit;
  const author = commit.authorLogin ?? "Unknown author";
  const statusText = statusLabel(detail.status);
  const [fileFilter, setFileFilter] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [activeFilePath, setActiveFilePath] = useState<string | null>(
    detail.files[0]?.path ?? null,
  );
  const normalizedFileFilter = normalizeQuery(fileFilter);
  const visibleFiles = useMemo(
    () =>
      normalizedFileFilter
        ? detail.files.filter((file) =>
            textMatches(file.path, normalizedFileFilter),
          )
        : detail.files,
    [detail.files, normalizedFileFilter],
  );
  const visibleFilePaths = useMemo(
    () => new Set(visibleFiles.map((file) => file.path)),
    [visibleFiles],
  );
  const visibleFileTree = useMemo(
    () => detail.fileTree.filter((node) => visibleFilePaths.has(node.path)),
    [detail.fileTree, visibleFilePaths],
  );
  const matchCount = useMemo(
    () => searchMatchCount(visibleFiles, searchQuery),
    [searchQuery, visibleFiles],
  );

  function clearDiffFilters() {
    setFileFilter("");
    setSearchQuery("");
  }

  function selectFile(node: RepositoryCommitDetailFileTreeNode) {
    setActiveFilePath(node.path);
    window.requestAnimationFrame(() => {
      const target = document.getElementById(node.href.replace(/^#/, ""));
      target?.scrollIntoView({ block: "start", behavior: "smooth" });
      target?.focus();
    });
  }

  return (
    <div>
      <header
        className="border-b px-6 py-6"
        style={{ background: "var(--surface-2)", borderColor: "var(--line)" }}
      >
        <div className="mx-auto max-w-7xl">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                {repository.ownerLogin}
              </p>
              <div className="mt-1 flex flex-wrap items-center gap-2">
                <Link
                  className="t-h2 hover:underline"
                  href={repository.href}
                  style={{ color: "var(--ink-1)" }}
                >
                  {repository.name}
                </Link>
                <span className="chip soft capitalize">
                  {repository.visibility}
                </span>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Link className="btn sm" href={repository.commitHistoryHref}>
                Commit history
              </Link>
              <Link className="btn primary sm" href={commit.browseHref}>
                Browse files
              </Link>
            </div>
          </div>
          <div className="mt-6 grid gap-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
            <div className="min-w-0">
              <h1 className="t-h1 break-words">{commit.subject}</h1>
              <div
                className="mt-3 flex flex-wrap items-center gap-2 t-sm"
                style={{ color: "var(--ink-3)" }}
              >
                <span className="av sm" aria-hidden="true">
                  {initials(author)}
                </span>
                <strong style={{ color: "var(--ink-2)", fontWeight: 600 }}>
                  {author}
                </strong>
                <span>committed {formatRelativeTime(commit.committedAt)}</span>
                <span className="chip soft t-mono-sm">{commit.shortOid}</span>
              </div>
            </div>
            <CopyButton
              className="btn sm"
              copiedLabel="Full SHA copied"
              label="Copy full SHA"
              value={commit.oid}
            />
          </div>
        </div>
      </header>

      <main className="mx-auto grid max-w-7xl gap-6 px-6 py-6 lg:grid-cols-[minmax(0,1fr)_320px]">
        <section className="space-y-4">
          <section className="card p-5" aria-label="Commit summary">
            {commit.body ? (
              <p
                className="whitespace-pre-wrap t-body"
                style={{ color: "var(--ink-2)" }}
              >
                {commit.body}
              </p>
            ) : (
              <p className="t-body" style={{ color: "var(--ink-3)" }}>
                This commit has no extended message.
              </p>
            )}
          </section>

          <section className="space-y-4" aria-label="Commit diff">
            <div
              className="card flex flex-wrap items-center justify-between gap-3 px-4 py-3"
              style={{
                background: "var(--surface-2)",
              }}
            >
              <div>
                <h2 className="t-h3">Changed files</h2>
                <p className="mt-1 t-xs">
                  <span className="t-num">{detail.diffSummary.totalFiles}</span>{" "}
                  files changed with{" "}
                  <span className="t-num" style={{ color: "var(--ok)" }}>
                    +{detail.diffSummary.additions}
                  </span>{" "}
                  <span className="t-num" style={{ color: "var(--err)" }}>
                    -{detail.diffSummary.deletions}
                  </span>
                </p>
              </div>
              <span className="chip ok">Diff ready</span>
            </div>
            <section className="card p-4" aria-label="Diff controls">
              <RepositoryCommitDiffSearch
                fileFilter={fileFilter}
                matchCount={matchCount}
                onClear={clearDiffFilters}
                onFileFilterChange={setFileFilter}
                onSearchQueryChange={setSearchQuery}
                searchQuery={searchQuery}
                visibleFileCount={visibleFiles.length}
              />
              {searchQuery.trim() && matchCount === 0 ? (
                <p className="mt-3 t-sm" style={{ color: "var(--ink-3)" }}>
                  No visible diff lines match this search.
                </p>
              ) : null}
            </section>
            <div className="grid gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
              <aside className="card h-fit p-2" aria-label="Changed file tree">
                <div
                  className="border-b px-2 py-2 t-label"
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  Files
                </div>
                {visibleFileTree.length ? (
                  <RepositoryCommitFileTree
                    activeFilePath={activeFilePath}
                    fileTree={visibleFileTree}
                    onSelectFile={selectFile}
                  />
                ) : (
                  <p
                    className="px-2 py-4 t-sm"
                    style={{ color: "var(--ink-3)" }}
                  >
                    No changed files match this filter.
                  </p>
                )}
              </aside>
              <div className="min-w-0 space-y-4">
                {visibleFiles.length ? (
                  visibleFiles.map((file) => (
                    <RepositoryCommitDiffFile
                      file={file}
                      isActive={file.path === activeFilePath}
                      key={file.path}
                      searchQuery={searchQuery}
                    />
                  ))
                ) : normalizedFileFilter ? (
                  <div className="card p-6">
                    <p className="t-body" style={{ color: "var(--ink-3)" }}>
                      No changed files match this filter.
                    </p>
                    <button
                      className="btn mt-4 sm"
                      onClick={clearDiffFilters}
                      type="button"
                    >
                      Clear filters
                    </button>
                  </div>
                ) : (
                  <div className="card p-6">
                    <p className="t-body" style={{ color: "var(--ink-3)" }}>
                      {detail.diffPlaceholder.message}
                    </p>
                  </div>
                )}
                <Link className="btn sm" href={commit.browseHref}>
                  Browse files at this commit
                </Link>
              </div>
            </div>
          </section>
        </section>

        <aside className="space-y-5">
          <section aria-labelledby="commit-status-heading">
            <h2 className="t-label" id="commit-status-heading">
              Status
            </h2>
            <div className="mt-2 flex flex-wrap gap-2">
              <Link
                className={statusChipClass(detail.status)}
                href={detail.status.href}
              >
                {statusText}
              </Link>
              <span
                className={verificationClass(detail.verification)}
                title={detail.verification.signatureSummary ?? undefined}
              >
                {verificationLabel(detail.verification)}
              </span>
            </div>
            {detail.verification.signatureSummary ? (
              <p className="mt-2 t-xs">
                {detail.verification.signatureSummary}
              </p>
            ) : null}
          </section>

          <section aria-labelledby="commit-branches-heading">
            <h2 className="t-label" id="commit-branches-heading">
              Branches and tags
            </h2>
            <div className="mt-2 flex flex-wrap gap-2">
              {detail.branches.length > 0 ? (
                detail.branches.map((branch) => (
                  <Link
                    className="chip soft"
                    href={branch.href}
                    key={branch.qualifiedName}
                  >
                    {branch.name}
                  </Link>
                ))
              ) : (
                <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No branch or tag points directly at this commit.
                </span>
              )}
            </div>
          </section>

          <section aria-labelledby="commit-parents-heading">
            <h2 className="t-label" id="commit-parents-heading">
              Parents
            </h2>
            <div className="mt-2 flex flex-wrap gap-2">
              {detail.parents.length > 0 ? (
                detail.parents.map((parent) => (
                  <Link
                    className="btn sm t-mono-sm"
                    href={parent.href}
                    key={parent.oid}
                  >
                    {parent.shortOid}
                  </Link>
                ))
              ) : (
                <span className="chip soft">Root commit</span>
              )}
            </div>
          </section>

          <section aria-labelledby="commit-prs-heading">
            <h2 className="t-label" id="commit-prs-heading">
              Pull requests
            </h2>
            <div className="mt-2 flex flex-wrap gap-2">
              {detail.pullRequests.length > 0 ? (
                detail.pullRequests.map((pullRequest) => (
                  <Link
                    className="chip soft"
                    href={pullRequest.href}
                    key={pullRequest.number}
                    title={pullRequest.title}
                  >
                    #{pullRequest.number}
                  </Link>
                ))
              ) : (
                <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No linked pull request.
                </span>
              )}
            </div>
          </section>
        </aside>
      </main>
    </div>
  );
}
