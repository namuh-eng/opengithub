import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  PullRequestCompareView,
  PullRequestDetailView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryCompareRangeHref } from "@/lib/navigation";

type PullRequestFilesChangedPageProps = {
  compare: PullRequestCompareView | null;
  pullRequest: PullRequestDetailView;
  repository: RepositoryOverview;
};

function changeLabel(additions: number, deletions: number) {
  return `${additions.toLocaleString()} additions and ${deletions.toLocaleString()} deletions`;
}

export function PullRequestFilesChangedPage({
  compare,
  pullRequest,
  repository,
}: PullRequestFilesChangedPageProps) {
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const activePath = `${basePath}/pull/${pullRequest.number}/files`;
  const conversationHref = `${basePath}/pull/${pullRequest.number}`;
  const compareHref = repositoryCompareRangeHref(
    repository.owner_login,
    repository.name,
    pullRequest.baseRef,
    pullRequest.headRef,
  );
  const files = compare?.files ?? [];

  return (
    <RepositoryShell
      activePath={`${basePath}/pulls`}
      frameClassName="max-w-6xl"
      repository={repository}
    >
      <main className="min-w-0">
        <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <p className="t-label mb-2">Pull request #{pullRequest.number}</p>
            <h1 className="t-h1 break-words">
              Files changed{" "}
              <span className="t-num" style={{ color: "var(--ink-4)" }}>
                {pullRequest.stats.files}
              </span>
            </h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Comparing <span className="t-mono-sm">{pullRequest.baseRef}</span>{" "}
              with <span className="t-mono-sm">{pullRequest.headRef}</span> for{" "}
              <Link className="hover:underline" href={conversationHref}>
                {pullRequest.title}
              </Link>
              .
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn" href={conversationHref}>
              Conversation
            </Link>
            <Link className="btn primary" href={compareHref}>
              Open compare
            </Link>
          </div>
        </div>

        <nav aria-label="Pull request sections" className="tabs mb-6">
          <Link className="tab" href={conversationHref}>
            Conversation
            <span className="badge t-num">{pullRequest.stats.comments}</span>
          </Link>
          <Link className="tab" href={`${conversationHref}/commits`}>
            Commits
            <span className="badge t-num">{pullRequest.stats.commits}</span>
          </Link>
          <Link className="tab" href={`${conversationHref}/checks`}>
            Checks
            {pullRequest.checks.totalCount ? (
              <span className="badge t-num">
                {pullRequest.checks.totalCount}
              </span>
            ) : null}
          </Link>
          <Link aria-current="page" className="tab active" href={activePath}>
            Files changed
            <span className="badge t-num">{pullRequest.stats.files}</span>
          </Link>
        </nav>

        <section className="card overflow-hidden">
          <div
            className="flex flex-wrap items-center gap-3 border-b px-5 py-4"
            style={{ borderColor: "var(--line)" }}
          >
            <h2 className="t-h3 flex-1">Changed files summary</h2>
            <span className="chip soft">
              <span className="t-num">{pullRequest.stats.files}</span> files
            </span>
            <span className="chip soft">
              <span className="t-num">{pullRequest.stats.additions}</span>{" "}
              additions
            </span>
            <span className="chip soft">
              <span className="t-num">{pullRequest.stats.deletions}</span>{" "}
              deletions
            </span>
          </div>

          {files.length ? (
            <div>
              {files.map((file) => (
                <div className="list-row items-start px-5 py-4" key={file.path}>
                  <div className="min-w-0 flex-1">
                    <p className="t-mono-sm break-all">{file.path}</p>
                    <p className="t-xs mt-1">
                      {file.status.replaceAll("_", " ")} ·{" "}
                      {changeLabel(file.additions, file.deletions)}
                    </p>
                  </div>
                  <div className="flex shrink-0 gap-2">
                    <span className="chip soft">
                      +<span className="t-num">{file.additions}</span>
                    </span>
                    <span className="chip soft">
                      -<span className="t-num">{file.deletions}</span>
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="px-5 py-8">
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                This pull request has a stored change summary, but no expanded
                per-file comparison is available for these refs yet.
              </p>
              <Link className="btn mt-4" href={compareHref}>
                Review compare context
              </Link>
            </div>
          )}
        </section>
      </main>
    </RepositoryShell>
  );
}
