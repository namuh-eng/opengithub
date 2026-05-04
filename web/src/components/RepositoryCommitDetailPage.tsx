import Link from "next/link";
import { CopyButton } from "@/components/CopyButton";
import type {
  RepositoryCommitDetailFile,
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

function DiffLine({ line }: { line: RepositoryCommitDetailLine }) {
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
      <code className="min-w-0 whitespace-pre px-2 py-1.5">{line.content}</code>
    </div>
  );
}

function CommitDiffFile({ file }: { file: RepositoryCommitDetailFile }) {
  return (
    <article className="card overflow-hidden" id={file.anchor}>
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
                <DiffLine key={`${hunk.id}-${line.position}`} line={line} />
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

export function RepositoryCommitDetailPage({
  detail,
}: RepositoryCommitDetailPageProps) {
  const repository = detail.repository;
  const commit = detail.commit;
  const author = commit.authorLogin ?? "Unknown author";
  const statusText = statusLabel(detail.status);

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
            <div className="grid gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
              <aside className="card h-fit p-2" aria-label="Changed file tree">
                <div
                  className="border-b px-2 py-2 t-label"
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  Files
                </div>
                <nav
                  aria-label="Changed file tree"
                  className="mt-2 max-h-[520px] space-y-1 overflow-y-auto"
                >
                  {detail.fileTree.map((node) => (
                    <a
                      className="flex items-center gap-2 rounded-[var(--radius)] px-2 py-1.5 t-sm hover:bg-[var(--hover)]"
                      href={node.href}
                      key={node.path}
                      style={{ paddingLeft: 8 + node.depth * 12 }}
                    >
                      <span
                        className="t-mono-sm"
                        style={{ color: "var(--ink-4)" }}
                      >
                        {fileStatusMark(node.status)}
                      </span>
                      <span className="min-w-0 flex-1 truncate t-mono-sm">
                        {node.name}
                      </span>
                      <span
                        className="t-xs t-num"
                        style={{ color: "var(--ink-4)" }}
                      >
                        +{node.additions}/-{node.deletions}
                      </span>
                    </a>
                  ))}
                </nav>
              </aside>
              <div className="min-w-0 space-y-4">
                {detail.files.length ? (
                  detail.files.map((file) => (
                    <CommitDiffFile file={file} key={file.path} />
                  ))
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
