import Link from "next/link";
import { CopyButton } from "@/components/CopyButton";
import type {
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

          <section className="card overflow-hidden" aria-label="Commit diff">
            <div
              className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3"
              style={{
                borderColor: "var(--line-soft)",
                background: "var(--surface-2)",
              }}
            >
              <div>
                <h2 className="t-h3">Changed files</h2>
                <p className="mt-1 t-xs">{detail.diffPlaceholder.nextPhase}</p>
              </div>
              <span className="chip soft">Summary ready</span>
            </div>
            <div className="p-6">
              <p className="t-body" style={{ color: "var(--ink-3)" }}>
                {detail.diffPlaceholder.message}
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                <Link className="btn sm" href={commit.browseHref}>
                  Browse files at this commit
                </Link>
                <Link className="btn ghost sm" href={detail.status.href}>
                  View check summary
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
