"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  PullRequestDiffFile,
  PullRequestDiffLine,
  PullRequestDiffReviewComment,
  PullRequestDiffReviewView,
  RepositoryOverview,
} from "@/lib/api";

type PullRequestFilesChangedPageProps = {
  diffReview: PullRequestDiffReviewView;
  repository: RepositoryOverview;
  viewerAuthenticated: boolean;
};

function lineSign(kind: PullRequestDiffLine["kind"]) {
  if (kind === "added") {
    return "+";
  }
  if (kind === "removed") {
    return "-";
  }
  return " ";
}

function lineStyles(kind: PullRequestDiffLine["kind"]) {
  if (kind === "added") {
    return {
      background: "var(--code-add)",
      numberBackground: "var(--code-add-strong)",
    };
  }
  if (kind === "removed") {
    return {
      background: "var(--code-del)",
      numberBackground: "var(--code-del-strong)",
    };
  }
  return {
    background: "transparent",
    numberBackground: "var(--surface-2)",
  };
}

function statusMark(status: string) {
  if (status === "added") {
    return "+";
  }
  if (status === "removed") {
    return "-";
  }
  if (status === "renamed") {
    return "R";
  }
  return "M";
}

function encodedParams(
  current: PullRequestDiffReviewView["settings"],
  overrides: Partial<PullRequestDiffReviewView["settings"]>,
) {
  const params = new URLSearchParams();
  const next = { ...current, ...overrides };
  if (next.view && next.view !== "unified") {
    params.set("view", next.view);
  }
  if (next.whitespace && next.whitespace !== "show") {
    params.set("whitespace", next.whitespace);
  }
  if (next.commit) {
    params.set("commit", next.commit);
  }
  if (next.filter) {
    params.set("filter", next.filter);
  }
  if (next.page && next.page > 1) {
    params.set("page", String(next.page));
  }
  if (next.pageSize && next.pageSize !== 50) {
    params.set("pageSize", String(next.pageSize));
  }
  return params.toString();
}

function settingsHref(
  basePath: string,
  settings: PullRequestDiffReviewView["settings"],
  overrides: Partial<PullRequestDiffReviewView["settings"]>,
) {
  const params = encodedParams(settings, { ...overrides, page: 1 });
  return params ? `${basePath}?${params}` : basePath;
}

function visibleLineNumber(value: number | null) {
  return value == null ? "" : value.toLocaleString();
}

function fileViewedLabel(file: PullRequestDiffFile) {
  return file.viewed
    ? `Mark ${file.path} as not viewed`
    : `Mark ${file.path} as viewed`;
}

function ViewedToggle({
  activePath,
  file,
  viewerAuthenticated,
}: {
  activePath: string;
  file: PullRequestDiffFile;
  viewerAuthenticated: boolean;
}) {
  const [viewed, setViewed] = useState(file.viewed);
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);

  async function toggleViewed() {
    if (!viewerAuthenticated) {
      setFeedback("Sign in to persist viewed files.");
      return;
    }
    const nextViewed = !viewed;
    setViewed(nextViewed);
    setSaving(true);
    setFeedback(null);
    try {
      const response = await fetch(`${activePath}/viewed`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          fileId: file.id,
          versionKey: file.versionKey,
          viewed: nextViewed,
        }),
      });
      if (!response.ok) {
        setViewed(!nextViewed);
        setFeedback("Viewed state could not be saved.");
        return;
      }
      setFeedback(
        nextViewed ? "File marked as viewed." : "File marked as not viewed.",
      );
    } catch {
      setViewed(!nextViewed);
      setFeedback("Viewed state could not be saved.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col items-end gap-1">
      <button
        aria-pressed={viewed}
        className="btn sm"
        disabled={saving}
        onClick={toggleViewed}
        type="button"
      >
        {viewed ? "Viewed" : "Viewed?"}
      </button>
      <span className="sr-only">{fileViewedLabel({ ...file, viewed })}</span>
      {feedback ? (
        <span className="t-xs max-w-44 text-right">{feedback}</span>
      ) : null}
    </div>
  );
}

function commentMatchesLine(
  comment: PullRequestDiffReviewComment,
  line: PullRequestDiffLine,
) {
  return (
    (line.newLine !== null && comment.newLine === line.newLine) ||
    (line.oldLine !== null && comment.oldLine === line.oldLine) ||
    comment.position === line.position
  );
}

function commentSideForLine(line: PullRequestDiffLine) {
  return line.kind === "removed" ? "left" : "right";
}

function ReviewCommentThread({
  activePath,
  comment,
  onDelete,
  onUpdate,
}: {
  activePath: string;
  comment: PullRequestDiffReviewComment;
  onDelete: (commentId: string) => void;
  onUpdate: (comment: PullRequestDiffReviewComment) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [body, setBody] = useState(comment.body);
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);

  async function updateDraft() {
    if (!body.trim()) {
      setFeedback("Write a comment before saving.");
      return;
    }
    setSaving(true);
    setFeedback(null);
    try {
      const response = await fetch(
        `${activePath}/review-comments/drafts/${comment.id}`,
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ body }),
        },
      );
      if (!response.ok) {
        setFeedback("Pending comment could not be updated.");
        return;
      }
      const updated = (await response.json()) as PullRequestDiffReviewComment;
      onUpdate(updated);
      setEditing(false);
      setFeedback("Pending comment updated.");
    } catch {
      setFeedback("Pending comment could not be updated.");
    } finally {
      setSaving(false);
    }
  }

  async function deleteDraft() {
    setSaving(true);
    setFeedback(null);
    try {
      const response = await fetch(
        `${activePath}/review-comments/drafts/${comment.id}`,
        { method: "DELETE" },
      );
      if (!response.ok) {
        setFeedback("Pending comment could not be deleted.");
        return;
      }
      onDelete(comment.id);
    } catch {
      setFeedback("Pending comment could not be deleted.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      className="border-t px-4 py-3"
      style={{ borderColor: "var(--line-soft)" }}
    >
      <div className="flex gap-3">
        <div className="av sm">{comment.author.login[0]?.toUpperCase()}</div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <p className="t-sm">
              <strong>{comment.author.login}</strong>{" "}
              <span className="muted">
                {comment.state === "pending"
                  ? "left a pending review comment"
                  : "commented"}
              </span>
            </p>
            {comment.state === "pending" ? (
              <span className="chip warn">pending</span>
            ) : null}
          </div>
          {editing ? (
            <div className="mt-2">
              <textarea
                aria-label="Edit pending review comment"
                className="input min-h-24 w-full p-3"
                onChange={(event) => setBody(event.currentTarget.value)}
                value={body}
              />
              <div className="mt-2 flex flex-wrap items-center gap-2">
                <button
                  className="btn primary sm"
                  disabled={saving}
                  onClick={updateDraft}
                  type="button"
                >
                  Save
                </button>
                <button
                  className="btn ghost sm"
                  disabled={saving}
                  onClick={() => {
                    setBody(comment.body);
                    setEditing(false);
                  }}
                  type="button"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div className="t-sm mt-1">
              <MarkdownBody html={comment.bodyHtml} />
            </div>
          )}
          {comment.state === "pending" && !editing ? (
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <button
                className="btn ghost sm"
                disabled={saving}
                onClick={() => setEditing(true)}
                type="button"
              >
                Edit
              </button>
              <button
                className="btn ghost sm"
                disabled={saving}
                onClick={deleteDraft}
                type="button"
              >
                Delete
              </button>
            </div>
          ) : null}
          {feedback ? <p className="t-xs mt-2">{feedback}</p> : null}
        </div>
      </div>
    </div>
  );
}

function InlineCommentComposer({
  activePath,
  file,
  line,
  onCancel,
  onSaved,
}: {
  activePath: string;
  file: PullRequestDiffFile;
  line: PullRequestDiffLine;
  onCancel: () => void;
  onSaved: (comment: PullRequestDiffReviewComment) => void;
}) {
  const [body, setBody] = useState("");
  const [tab, setTab] = useState<"write" | "preview">("write");
  const [previewHtml, setPreviewHtml] = useState("");
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);

  async function preview() {
    setTab("preview");
    if (!body.trim()) {
      setPreviewHtml("");
      return;
    }
    const response = await fetch("/markdown/preview", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ markdown: body, enableTaskToggles: false }),
    });
    if (response.ok) {
      const rendered = (await response.json()) as { html: string };
      setPreviewHtml(rendered.html);
    }
  }

  async function saveDraft() {
    if (!body.trim()) {
      setFeedback("Write a comment before saving.");
      return;
    }
    setSaving(true);
    setFeedback(null);
    try {
      const response = await fetch(`${activePath}/review-comments/drafts`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          fileId: file.id,
          body,
          side: commentSideForLine(line),
          oldLine: line.oldLine,
          newLine: line.newLine,
          position: line.position,
        }),
      });
      if (!response.ok) {
        setFeedback("Pending comment could not be saved.");
        return;
      }
      const comment = (await response.json()) as PullRequestDiffReviewComment;
      onSaved(comment);
      setBody("");
      onCancel();
    } catch {
      setFeedback("Pending comment could not be saved.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      className="border-t px-4 py-3"
      style={{ borderColor: "var(--line-soft)" }}
    >
      <div className="tabs mb-3">
        <button
          aria-selected={tab === "write"}
          className={`tab${tab === "write" ? " active" : ""}`}
          onClick={() => setTab("write")}
          role="tab"
          type="button"
        >
          Write
        </button>
        <button
          aria-selected={tab === "preview"}
          className={`tab${tab === "preview" ? " active" : ""}`}
          onClick={preview}
          role="tab"
          type="button"
        >
          Preview
        </button>
      </div>
      {tab === "write" ? (
        <textarea
          aria-label={`Pending review comment for ${file.path} line ${line.newLine ?? line.oldLine}`}
          className="input min-h-28 w-full p-3"
          onChange={(event) => setBody(event.currentTarget.value)}
          placeholder="Leave a pending review comment"
          value={body}
        />
      ) : previewHtml ? (
        <div className="card p-3">
          <MarkdownBody html={previewHtml} />
        </div>
      ) : (
        <p className="t-sm muted">Nothing to preview.</p>
      )}
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button
          className="btn primary sm"
          disabled={saving}
          onClick={saveDraft}
          type="button"
        >
          Save pending comment
        </button>
        <button
          className="btn ghost sm"
          disabled={saving}
          onClick={onCancel}
          type="button"
        >
          Cancel
        </button>
        {feedback ? <span className="t-xs">{feedback}</span> : null}
      </div>
    </div>
  );
}

function DiffLine({
  activePath,
  comments,
  file,
  line,
  onDeleteComment,
  onSaveComment,
  onUpdateComment,
  viewerAuthenticated,
}: {
  activePath: string;
  comments: PullRequestDiffReviewComment[];
  file: PullRequestDiffFile;
  line: PullRequestDiffLine;
  onDeleteComment: (commentId: string) => void;
  onSaveComment: (comment: PullRequestDiffReviewComment) => void;
  onUpdateComment: (comment: PullRequestDiffReviewComment) => void;
  viewerAuthenticated: boolean;
}) {
  const [composerOpen, setComposerOpen] = useState(false);
  const styles = lineStyles(line.kind);
  return (
    <div>
      <div
        className="group grid min-w-[760px] grid-cols-[56px_56px_28px_minmax(0,1fr)_44px] font-mono text-[12.5px] leading-[22px]"
        style={{ background: styles.background, fontFamily: "var(--mono)" }}
      >
        <span
          className="select-none pr-2 text-right"
          style={{ background: styles.numberBackground, color: "var(--ink-4)" }}
        >
          {visibleLineNumber(line.oldLine)}
        </span>
        <span
          className="select-none border-r pr-2 text-right"
          style={{
            background: styles.numberBackground,
            borderColor: "var(--line-soft)",
            color: "var(--ink-4)",
          }}
        >
          {visibleLineNumber(line.newLine)}
        </span>
        <span
          className="select-none text-center"
          style={{ color: "var(--ink-4)" }}
        >
          {lineSign(line.kind)}
        </span>
        <code className="min-w-0 overflow-x-auto whitespace-pre pr-4">
          {line.content || " "}
        </code>
        {viewerAuthenticated ? (
          <button
            aria-label={`Add comment at diff position ${line.position}`}
            className="opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
            onClick={() => setComposerOpen(true)}
            type="button"
          >
            +
          </button>
        ) : (
          <Link
            aria-label={`Sign in to comment at diff position ${line.position}`}
            className="text-center"
            href={`/login?next=${encodeURIComponent(file.href)}`}
          >
            +
          </Link>
        )}
      </div>
      {comments.map((comment) => (
        <ReviewCommentThread
          activePath={activePath}
          comment={comment}
          key={comment.id}
          onDelete={onDeleteComment}
          onUpdate={onUpdateComment}
        />
      ))}
      {composerOpen ? (
        <InlineCommentComposer
          activePath={activePath}
          file={file}
          line={line}
          onCancel={() => setComposerOpen(false)}
          onSaved={onSaveComment}
        />
      ) : null}
    </div>
  );
}

function DiffFile({
  activePath,
  file,
  viewerAuthenticated,
}: {
  activePath: string;
  file: PullRequestDiffFile;
  viewerAuthenticated: boolean;
}) {
  const [comments, setComments] = useState(file.comments);
  const anchor = file.href.split("#")[1] ?? file.id;
  const updateComment = (updated: PullRequestDiffReviewComment) => {
    setComments((current) =>
      current.map((comment) => (comment.id === updated.id ? updated : comment)),
    );
  };
  const deleteComment = (commentId: string) => {
    setComments((current) =>
      current.filter((comment) => comment.id !== commentId),
    );
  };
  return (
    <article className="card mb-4 overflow-hidden" id={anchor}>
      <div
        className="flex flex-wrap items-center gap-3 border-b px-4 py-3"
        style={{ background: "var(--surface-2)", borderColor: "var(--line)" }}
      >
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {statusMark(file.status)}
        </span>
        <h2 className="t-mono-sm min-w-0 flex-1 break-all">{file.path}</h2>
        {file.language ? (
          <span className="chip soft">{file.language}</span>
        ) : null}
        <span className="t-xs t-num">
          <span style={{ color: "var(--ok)" }}>+{file.additions}</span>{" "}
          <span style={{ color: "var(--err)" }}>-{file.deletions}</span>
        </span>
        <ViewedToggle
          activePath={activePath}
          file={file}
          viewerAuthenticated={viewerAuthenticated}
        />
        <Link className="btn ghost sm" href={file.href}>
          Copy link
        </Link>
      </div>
      {file.hunks.length ? (
        <div className="overflow-x-auto">
          {file.hunks.map((hunk) => (
            <div key={hunk.id}>
              <div
                className="t-mono-sm border-b px-4 py-2"
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
                  activePath={activePath}
                  comments={comments.filter((comment) =>
                    commentMatchesLine(comment, line),
                  )}
                  file={file}
                  key={`${hunk.id}-${line.position}`}
                  line={line}
                  onDeleteComment={deleteComment}
                  onSaveComment={(comment) =>
                    setComments((current) => [...current, comment])
                  }
                  onUpdateComment={updateComment}
                  viewerAuthenticated={viewerAuthenticated}
                />
              ))}
            </div>
          ))}
        </div>
      ) : (
        <div className="px-4 py-5">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            This file has summary metadata, but no expanded hunk rows are stored
            yet.
          </p>
        </div>
      )}
      {comments.filter((comment) => comment.position === null).length ? (
        <div
          className="border-t p-4"
          style={{ borderColor: "var(--line-soft)" }}
        >
          <p className="t-label mb-3">File comments</p>
          <div className="space-y-3">
            {comments
              .filter((comment) => comment.position === null)
              .map((comment) => (
                <ReviewCommentThread
                  activePath={activePath}
                  comment={comment}
                  key={comment.id}
                  onDelete={deleteComment}
                  onUpdate={updateComment}
                />
              ))}
          </div>
        </div>
      ) : null}
    </article>
  );
}

export function PullRequestFilesChangedPage({
  diffReview,
  repository,
  viewerAuthenticated,
}: PullRequestFilesChangedPageProps) {
  const pullRequest = diffReview.pullRequest;
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const activePath = `${basePath}/pull/${pullRequest.number}/files`;
  const conversationHref = `${basePath}/pull/${pullRequest.number}`;
  const files = diffReview.files;
  const viewedCount = useMemo(
    () => diffReview.fileTree.filter((file) => file.viewed).length,
    [diffReview.fileTree],
  );

  return (
    <RepositoryShell
      activePath={`${basePath}/pulls`}
      frameClassName="max-w-[1320px]"
      repository={repository}
    >
      <main className="min-w-0">
        <div
          className="sticky top-0 z-10 mb-5 border-b py-4 backdrop-blur"
          style={{
            borderColor: "var(--line)",
            background: "color-mix(in oklab, var(--bg) 92%, transparent)",
          }}
        >
          <div className="flex flex-wrap items-center gap-3">
            <Link className="btn ghost sm" href={conversationHref}>
              Back to #{pullRequest.number}
            </Link>
            <div className="min-w-0 flex-1">
              <p className="t-label">Pull request files</p>
              <h1 className="t-h2 truncate">
                Files changed{" "}
                <span className="t-num" style={{ color: "var(--ink-4)" }}>
                  {diffReview.totalFiles}
                </span>
              </h1>
            </div>
            <span className="t-sm row gap-2">
              <span className="t-num" style={{ color: "var(--ok)" }}>
                +{pullRequest.stats.additions}
              </span>
              <span className="t-num" style={{ color: "var(--err)" }}>
                -{pullRequest.stats.deletions}
              </span>
            </span>
            <button
              className="btn primary"
              disabled
              title="Submit review ships in the next phase"
              type="button"
            >
              Review changes
            </button>
          </div>
          <nav aria-label="Pull request sections" className="tabs mt-4">
            <Link
              aria-label="Conversation"
              className="tab"
              href={conversationHref}
            >
              Conversation
              <span className="badge t-num">{pullRequest.stats.comments}</span>
            </Link>
            <Link
              aria-label="Commits"
              className="tab"
              href={`${conversationHref}/commits`}
            >
              Commits
              <span className="badge t-num">{pullRequest.stats.commits}</span>
            </Link>
            <Link
              aria-label="Checks"
              className="tab"
              href={`${conversationHref}/checks`}
            >
              Checks
              {pullRequest.checks.totalCount ? (
                <span className="badge t-num">
                  {pullRequest.checks.totalCount}
                </span>
              ) : null}
            </Link>
            <Link
              aria-current="page"
              aria-label="Files changed"
              className="tab active"
              href={activePath}
            >
              Files changed
              <span className="badge t-num">{pullRequest.stats.files}</span>
            </Link>
          </nav>
        </div>

        <section className="card mb-5 p-4">
          <div className="flex flex-wrap items-end gap-3">
            <form action={activePath} className="min-w-[240px] flex-1">
              <label className="t-label mb-2 block" htmlFor="file-filter">
                File filter
              </label>
              <div className="input">
                <input
                  defaultValue={diffReview.settings.filter ?? ""}
                  id="file-filter"
                  name="filter"
                  placeholder="Filter changed files"
                />
              </div>
              <input
                name="view"
                type="hidden"
                value={diffReview.settings.view}
              />
              <input
                name="whitespace"
                type="hidden"
                value={diffReview.settings.whitespace}
              />
            </form>
            <div className="flex flex-wrap gap-2">
              <Link
                className={`btn sm${diffReview.settings.view === "unified" ? " primary" : ""}`}
                href={settingsHref(activePath, diffReview.settings, {
                  view: "unified",
                })}
              >
                Unified
              </Link>
              <Link
                className={`btn sm${diffReview.settings.view === "split" ? " primary" : ""}`}
                href={settingsHref(activePath, diffReview.settings, {
                  view: "split",
                })}
              >
                Split
              </Link>
              <Link
                className={`btn sm${diffReview.settings.whitespace === "hide" ? " primary" : ""}`}
                href={settingsHref(activePath, diffReview.settings, {
                  whitespace:
                    diffReview.settings.whitespace === "hide" ? "show" : "hide",
                })}
              >
                {diffReview.settings.whitespace === "hide"
                  ? "Show whitespace"
                  : "Hide whitespace"}
              </Link>
            </div>
            <label className="t-label flex flex-col gap-2">
              Commit
              <select
                className="input h-9 min-w-48"
                defaultValue={diffReview.settings.commit ?? ""}
                onChange={(event) => {
                  const href = settingsHref(activePath, diffReview.settings, {
                    commit: event.currentTarget.value || null,
                  });
                  window.location.assign(href);
                }}
              >
                <option value="">All commits</option>
                {diffReview.commits.map((commit) => (
                  <option key={commit.oid} value={commit.oid}>
                    {commit.oid.slice(0, 7)} · {commit.message}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="mt-3 flex flex-wrap gap-2">
            <span className="chip soft">
              <span className="t-num">{viewedCount}</span> viewed
            </span>
            <span className="chip soft">
              <span className="t-num">
                {diffReview.pendingReview.commentCount}
              </span>{" "}
              pending comments
            </span>
            {diffReview.settings.filter ? (
              <Link
                className="btn ghost sm"
                href={settingsHref(activePath, diffReview.settings, {
                  filter: null,
                })}
              >
                Clear filter
              </Link>
            ) : null}
          </div>
        </section>

        <div className="grid grid-cols-[300px_minmax(0,1fr)] gap-5 max-lg:grid-cols-1">
          <aside className="max-lg:order-2">
            <div className="card sticky top-[150px] p-2">
              <p className="t-label px-2 py-2">Files in this PR</p>
              {diffReview.fileTree.map((file) => (
                <Link
                  className="flex w-full items-start gap-2 rounded-md px-2 py-2 text-left hover:no-underline"
                  href={file.href}
                  key={file.id}
                  style={{
                    background: file.viewed
                      ? "var(--surface-2)"
                      : "transparent",
                  }}
                >
                  <span
                    className="t-mono-sm mt-0.5"
                    style={{
                      color:
                        file.status === "added" ? "var(--ok)" : "var(--ink-4)",
                    }}
                  >
                    {statusMark(file.status)}
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="t-mono-sm block break-all">
                      {file.path}
                    </span>
                    <span className="t-xs t-num">
                      <span style={{ color: "var(--ok)" }}>
                        +{file.additions}
                      </span>{" "}
                      <span style={{ color: "var(--err)" }}>
                        -{file.deletions}
                      </span>
                    </span>
                  </span>
                  {file.viewed ? (
                    <span className="chip soft">viewed</span>
                  ) : null}
                </Link>
              ))}
            </div>
          </aside>

          <div className="min-w-0">
            {files.length ? (
              files.map((file) => (
                <DiffFile
                  activePath={activePath}
                  file={file}
                  key={file.id}
                  viewerAuthenticated={viewerAuthenticated}
                />
              ))
            ) : (
              <section className="card p-8">
                <h2 className="t-h3">No changed files match this filter.</h2>
                <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  Clear the file filter or switch commits to review the stored
                  diff.
                </p>
                <Link className="btn mt-4" href={activePath}>
                  Clear file filter
                </Link>
              </section>
            )}
            {diffReview.hasMore ? (
              <div className="mt-4">
                <Link
                  className="btn"
                  href={settingsHref(activePath, diffReview.settings, {
                    page: diffReview.page + 1,
                  })}
                >
                  Next page
                </Link>
              </div>
            ) : null}
          </div>
        </div>
      </main>
    </RepositoryShell>
  );
}
