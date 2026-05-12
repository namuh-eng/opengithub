"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { type FormEvent, useEffect, useState, useTransition } from "react";
import { LabelPicker } from "@/components/LabelPicker";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  DiscussionAuthorSummary,
  DiscussionCategoryChoice,
  DiscussionCommentView,
  DiscussionEventView,
  DiscussionPollView,
  DiscussionReactionContent,
  DiscussionReactionSummary,
  DiscussionReplyView,
  DiscussionSubscriptionState,
  DiscussionTransferTargetsView,
  DiscussionVoteResponse,
  IssueListLabel,
  RepositoryDiscussionDetailView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryDiscussionDetailHref,
  repositoryDiscussionsHref,
} from "@/lib/navigation";

type RepositoryDiscussionDetailPageProps = {
  repository: RepositoryOverview;
  detail: RepositoryDiscussionDetailView;
};

const reactionOptions: Array<{
  value: DiscussionReactionContent;
  label: string;
}> = [
  { value: "+1", label: "+1" },
  { value: "-1", label: "-1" },
  { value: "laugh", label: "Laugh" },
  { value: "hooray", label: "Hooray" },
  { value: "confused", label: "Confused" },
  { value: "heart", label: "Heart" },
  { value: "rocket", label: "Rocket" },
  { value: "eyes", label: "Eyes" },
];

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) return "recently";
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  return `${Math.floor(months / 12)}y ago`;
}

function pollCompatibleCategories(
  detail: RepositoryDiscussionDetailView,
  categories: DiscussionCategoryChoice[],
) {
  const isPollDiscussion = Boolean(detail.poll);
  return categories.filter((category) => category.isPoll === isPollDiscussion);
}

function pollCategoryConstraintText(detail: RepositoryDiscussionDetailView) {
  return detail.poll
    ? "Poll discussions must stay in poll categories."
    : "Normal discussions cannot move into poll categories.";
}

function Avatar({
  user,
  size = "sm",
}: {
  user: DiscussionAuthorSummary;
  size?: "sm" | "lg";
}) {
  return (
    <span className={`av ${size}`} title={user.displayName || user.login}>
      {user.login.slice(0, 1).toUpperCase()}
    </span>
  );
}

function categoryAcceptsAnswers(detail: RepositoryDiscussionDetailView) {
  return Boolean(
    detail.sidebar.categoryOptions.find(
      (category) => category.slug === detail.category.slug,
    )?.acceptsAnswers,
  );
}

function statusChip(detail: RepositoryDiscussionDetailView) {
  if (detail.discussion.answered)
    return <span className="chip ok">Answered</span>;
  if (detail.discussion.state === "closed")
    return <span className="chip soft">Closed</span>;
  if (categoryAcceptsAnswers(detail))
    return <span className="chip warn">Unanswered</span>;
  return <span className="chip ok">Open</span>;
}

function ReactionSummary({
  label,
  reactions,
  canReact,
  onToggle,
}: {
  label: string;
  reactions: DiscussionReactionSummary[];
  canReact: boolean;
  onToggle?: (content: DiscussionReactionContent, reacted: boolean) => void;
}) {
  const byContent = new Map(
    reactions.map((reaction) => [reaction.content, reaction]),
  );
  const visible = reactionOptions
    .map((option) => ({
      ...option,
      reaction: byContent.get(option.value),
    }))
    .filter(({ reaction }) => canReact || (reaction?.count ?? 0) > 0);
  if (!visible.length) return null;
  return (
    <fieldset
      aria-label={label}
      className="mt-4 flex flex-wrap gap-2 border-0 p-0"
    >
      {visible.map(({ label, reaction, value }) => {
        const active = Boolean(reaction?.viewerReacted);
        const count = reaction?.count ?? 0;
        return canReact && onToggle ? (
          <button
            aria-pressed={active}
            className={active ? "chip active" : "chip soft"}
            key={value}
            onClick={() => onToggle(value, !active)}
            type="button"
          >
            {label} <span className="t-num">{count}</span>
          </button>
        ) : (
          <span className={active ? "chip active" : "chip soft"} key={value}>
            {label} <span className="t-num">{count}</span>
          </span>
        );
      })}
    </fieldset>
  );
}

function PollVotingCard({
  poll,
  detailHref,
  onPollUpdate,
  onMessage,
}: {
  poll: DiscussionPollView;
  detailHref: string;
  onPollUpdate: (poll: DiscussionPollView) => void;
  onMessage: (message: string | null) => void;
}) {
  const initialSelection = poll.viewerVoteOptionIds ?? [];
  const [selectedOptionIds, setSelectedOptionIds] =
    useState<string[]>(initialSelection);
  const [isPending, startTransition] = useTransition();
  const hasVote = initialSelection.length > 0;
  const canVote = Boolean(poll.viewerCanVote);
  const resultsVisible = Boolean(poll.resultsVisible);
  const canSubmit =
    canVote &&
    selectedOptionIds.length > 0 &&
    (poll.allowsMultiple || selectedOptionIds.length === 1);
  const actionLabel = hasVote ? "Update vote" : "Vote";
  const prompt =
    poll.unavailableReasons?.[0] ?? "Sign in to vote in this poll.";

  useEffect(() => {
    setSelectedOptionIds(poll.viewerVoteOptionIds ?? []);
  }, [poll.viewerVoteOptionIds]);

  function toggleOption(optionId: string, checked: boolean) {
    if (!poll.allowsMultiple) {
      setSelectedOptionIds([optionId]);
      return;
    }
    setSelectedOptionIds((current) =>
      checked
        ? Array.from(new Set([...current, optionId]))
        : current.filter((id) => id !== optionId),
    );
  }

  function submitVote(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    onMessage(null);
    startTransition(() => {
      void fetch(`${detailHref}/poll/vote`, {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ optionIds: selectedOptionIds }),
      })
        .then(async (response) => {
          const payload = await response.json().catch(() => null);
          if (!response.ok) {
            onMessage(
              payload?.error?.message ??
                "Discussion poll vote could not be updated.",
            );
            return;
          }
          onPollUpdate(payload.poll);
          onMessage(
            payload.changed ? "Poll vote updated." : "Poll vote saved.",
          );
        })
        .catch(() => {
          onMessage("Discussion poll vote could not be updated.");
        });
    });
  }

  return (
    <section className="card mt-5 p-4" aria-labelledby="discussion-poll-title">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="t-label">Poll</p>
          <h3 className="t-h3 mt-1 break-words" id="discussion-poll-title">
            {poll.question}
          </h3>
        </div>
        <span className="chip soft">
          {poll.allowsMultiple ? "Multiple choice" : "Single choice"}
        </span>
      </div>
      <form className="mt-4 grid gap-3" onSubmit={submitVote}>
        <fieldset
          aria-describedby="discussion-poll-help"
          className="m-0 grid gap-2 border-0 p-0"
        >
          {poll.options.map((option) => {
            const selected = selectedOptionIds.includes(option.id);
            const percentage = Math.max(
              0,
              Math.min(100, option.percentage ?? 0),
            );
            return (
              <label
                className="card block cursor-pointer p-3"
                htmlFor={`discussion-poll-option-${option.id}`}
                key={option.id}
              >
                <span className="flex min-w-0 items-start gap-3">
                  <input
                    checked={selected}
                    disabled={!canVote}
                    id={`discussion-poll-option-${option.id}`}
                    name="discussion-poll-option"
                    onChange={(event) =>
                      toggleOption(option.id, event.currentTarget.checked)
                    }
                    type={poll.allowsMultiple ? "checkbox" : "radio"}
                  />
                  <span className="min-w-0 flex-1">
                    <span className="t-sm block break-words">
                      {option.label}
                    </span>
                    {resultsVisible ? (
                      <span className="mt-2 grid gap-1">
                        <span
                          aria-label={`${option.label} has ${formatNumber(
                            option.votesCount ?? 0,
                          )} votes and ${percentage}%`}
                          aria-valuemax={100}
                          aria-valuemin={0}
                          aria-valuenow={percentage}
                          className="block h-2 overflow-hidden"
                          role="progressbar"
                          style={{
                            background: "var(--surface-2)",
                            borderRadius: "var(--radius-pill)",
                          }}
                        >
                          <span
                            className="block h-full"
                            style={{
                              background: selected
                                ? "var(--accent)"
                                : "var(--line-strong)",
                              width: `${percentage}%`,
                            }}
                          />
                        </span>
                        <span
                          className="t-xs flex flex-wrap justify-between gap-2"
                          style={{ color: "var(--ink-3)" }}
                        >
                          <span>{percentage}%</span>
                          <span>
                            {formatNumber(option.votesCount ?? 0)} votes
                          </span>
                        </span>
                      </span>
                    ) : null}
                  </span>
                </span>
              </label>
            );
          })}
        </fieldset>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="t-xs" id="discussion-poll-help">
            {resultsVisible
              ? `${formatNumber(poll.totalVotes ?? 0)} total votes`
              : prompt}
          </p>
          {canVote ? (
            <button
              className="btn primary sm"
              disabled={!canSubmit || isPending}
              type="submit"
            >
              {isPending ? "Saving..." : actionLabel}
            </button>
          ) : (
            <Link className="btn sm" href="/login">
              Sign in to vote
            </Link>
          )}
        </div>
      </form>
    </section>
  );
}

function CommentCard({
  comment,
  isAnswer,
  canComment,
  canReact,
  onReply,
  onReaction,
  onAnswerToggle,
}: {
  comment: DiscussionCommentView | DiscussionReplyView;
  isAnswer?: boolean;
  canComment: boolean;
  canReact: boolean;
  onReply?: (commentId: string, body: string) => Promise<void>;
  onReaction?: (
    commentId: string,
    content: DiscussionReactionContent,
    reacted: boolean,
  ) => void;
  onAnswerToggle?: (commentId: string, marked: boolean) => void;
}) {
  const labelId = `discussion-comment-${comment.id}`;
  const [replyBody, setReplyBody] = useState("");
  const [replyOpen, setReplyOpen] = useState(false);
  const [isPending, startTransition] = useTransition();
  const hasReplies = "replies" in comment;

  function submitReply(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!onReply || !replyBody.trim()) return;
    startTransition(() => {
      void onReply(comment.id, replyBody).then(() => {
        setReplyBody("");
        setReplyOpen(false);
      });
    });
  }

  return (
    <article className="flex min-w-0 gap-4" id={comment.id}>
      <Avatar size="lg" user={comment.author} />
      <div
        className="card min-w-0 flex-1 overflow-hidden"
        style={isAnswer ? { borderColor: "var(--ok)" } : undefined}
      >
        <div
          className="flex flex-wrap items-center gap-2 border-b px-4 py-3"
          style={{
            background: isAnswer ? "var(--ok-soft)" : "var(--surface-2)",
            borderColor: "var(--line)",
          }}
        >
          <h2 className="t-sm font-semibold" id={labelId}>
            {comment.author.login}
          </h2>
          <span className="t-xs">
            commented {relativeTime(comment.createdAt)}
          </span>
          {comment.edited ? <span className="chip soft">edited</span> : null}
          {isAnswer ? <span className="chip ok ml-auto">answer</span> : null}
          {onAnswerToggle && hasReplies ? (
            <button
              className={isAnswer ? "btn sm" : "btn accent sm"}
              onClick={() => onAnswerToggle(comment.id, !isAnswer)}
              type="button"
            >
              {isAnswer ? "Unmark answer" : "Mark as answer"}
            </button>
          ) : null}
          <a
            className={isAnswer ? "chip ok" : "chip soft ml-auto"}
            href={comment.href}
          >
            Permalink
          </a>
        </div>
        <div className="p-5">
          {comment.deleted ? (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              {comment.deletedReason ?? "This comment was deleted."}
            </p>
          ) : (
            <MarkdownBody html={comment.body.html} labelledBy={labelId} />
          )}
          <ReactionSummary
            canReact={canReact}
            label={`Reactions for ${comment.author.login} comment`}
            onToggle={
              onReaction
                ? (content, reacted) => onReaction(comment.id, content, reacted)
                : undefined
            }
            reactions={comment.reactions}
          />
          {hasReplies && canComment ? (
            <div className="mt-4">
              {replyOpen ? (
                <form className="grid gap-3" onSubmit={submitReply}>
                  <label className="t-label" htmlFor={`reply-${comment.id}`}>
                    Reply
                  </label>
                  <textarea
                    className="input min-h-24 w-full"
                    id={`reply-${comment.id}`}
                    onChange={(event) => setReplyBody(event.target.value)}
                    placeholder="Write a thoughtful reply"
                    value={replyBody}
                  />
                  <div className="flex flex-wrap gap-2">
                    <button
                      className="btn accent sm"
                      disabled={isPending || !replyBody.trim()}
                      type="submit"
                    >
                      Reply
                    </button>
                    <button
                      className="btn sm"
                      onClick={() => setReplyOpen(false)}
                      type="button"
                    >
                      Cancel
                    </button>
                  </div>
                </form>
              ) : (
                <button
                  className="btn sm"
                  onClick={() => setReplyOpen(true)}
                  type="button"
                >
                  Reply
                </button>
              )}
            </div>
          ) : null}
        </div>
      </div>
    </article>
  );
}

function EventRow({ event }: { event: DiscussionEventView }) {
  const actor = event.actor?.login ?? "opengithub";
  const label = event.eventType.replaceAll("_", " ");
  return (
    <div className="flex gap-4 pl-14">
      <span
        aria-hidden="true"
        className="mt-1 flex size-8 items-center justify-center rounded-full border"
        style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
      >
        ·
      </span>
      <p className="t-sm py-2">
        <strong>{actor}</strong> {label} ·{" "}
        <span style={{ color: "var(--ink-3)" }}>
          {relativeTime(event.createdAt)}
        </span>
      </p>
    </div>
  );
}

function eventLabel(event: DiscussionEventView) {
  const payload =
    typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : null;
  const reason =
    payload && typeof payload.reason === "string" ? ` (${payload.reason})` : "";
  return `${event.eventType.replaceAll("_", " ")}${reason}`;
}

function SortLinks({
  detail,
  owner,
  repo,
}: {
  detail: RepositoryDiscussionDetailView;
  owner: string;
  repo: string;
}) {
  return (
    <nav aria-label="Timeline sort" className="tabs">
      {["oldest", "newest", "top"].map((sort) => {
        const href = `${repositoryDiscussionDetailHref(owner, repo, detail.discussion.number)}?sort=${sort}`;
        return (
          <Link
            aria-current={detail.sort === sort ? "page" : undefined}
            className={detail.sort === sort ? "tab active" : "tab"}
            href={href}
            key={sort}
          >
            {sort[0].toUpperCase()}
            {sort.slice(1)}
          </Link>
        );
      })}
    </nav>
  );
}

function ReplyComposer({
  detail,
  onComment,
}: {
  detail: RepositoryDiscussionDetailView;
  onComment: (body: string) => Promise<void>;
}) {
  const canComment = detail.viewer.authenticated && detail.viewer.canComment;
  const [body, setBody] = useState("");
  const [mode, setMode] = useState<"write" | "preview">("write");
  const [isPending, startTransition] = useTransition();

  function submitComment(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canComment || !body.trim()) return;
    startTransition(() => {
      void onComment(body).then(() => setBody(""));
    });
  }

  return (
    <form
      className="card p-4"
      aria-label="Reply composer"
      onSubmit={submitComment}
    >
      <div className="tabs">
        <button
          className={mode === "write" ? "tab active" : "tab"}
          onClick={() => setMode("write")}
          type="button"
        >
          Write
        </button>
        <button
          className={mode === "preview" ? "tab active" : "tab"}
          onClick={() => setMode("preview")}
          type="button"
        >
          Preview
        </button>
      </div>
      <label className="t-label mt-4 block" htmlFor="discussion-reply">
        Reply
      </label>
      {mode === "preview" ? (
        <div className="card mt-2 min-h-32 p-4">
          <p className="t-sm whitespace-pre-wrap">
            {body.trim() || "Nothing to preview yet."}
          </p>
        </div>
      ) : (
        <textarea
          className="input mt-2 min-h-32 w-full"
          disabled={!canComment || isPending}
          id="discussion-reply"
          onChange={(event) => setBody(event.target.value)}
          placeholder={
            canComment
              ? "Write a comment"
              : "Sign in with repository access to join this discussion."
          }
          value={body}
        />
      )}
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button className="btn sm" disabled type="button">
          Saved replies
        </button>
        <button className="btn sm" disabled type="button">
          Attach files
        </button>
        <button
          className="btn accent sm"
          disabled={!canComment || isPending || !body.trim()}
          type="submit"
        >
          Comment
        </button>
      </div>
      <p className="t-xs mt-3" style={{ color: "var(--ink-3)" }}>
        Attachment upload storage is not yet connected; comment text is saved
        now.
      </p>
    </form>
  );
}

function RepositoryDiscussionModerationPanel({
  detail,
  onCategory,
  onLock,
  onPin,
  onState,
}: {
  detail: RepositoryDiscussionDetailView;
  onCategory: (categorySlug: string) => void;
  onLock: (locked: boolean, allowReactions?: boolean) => void;
  onPin: (
    method: "PUT" | "PATCH" | "DELETE",
    request?: Record<string, unknown>,
  ) => void;
  onState: (state: "open" | "closed", reason?: string) => void;
}) {
  const [pinOpen, setPinOpen] = useState(false);
  const [pinTarget, setPinTarget] = useState<"global" | "category">("global");
  const [pinTitle, setPinTitle] = useState(
    detail.moderation.globalPin?.customTitle ?? "",
  );
  const [pinBody, setPinBody] = useState(
    detail.moderation.globalPin?.customBody ?? "",
  );
  const [lockOpen, setLockOpen] = useState(false);
  const [allowReactions, setAllowReactions] = useState(
    detail.moderation.lockAllowsReactions,
  );
  const pinned = detail.moderation.globalPin ?? detail.moderation.categoryPin;
  const canCategoryPin = !detail.poll;
  const compatibleCategories = pollCompatibleCategories(
    detail,
    detail.sidebar.categoryOptions,
  );
  const hiddenCategoryCount =
    detail.sidebar.categoryOptions.length - compatibleCategories.length;

  return (
    <section aria-labelledby="discussion-moderation-title" className="card p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h2 className="t-label" id="discussion-moderation-title">
            Moderator controls
          </h2>
          <p className="t-xs mt-2" style={{ color: "var(--ink-3)" }}>
            Changes are recorded in the discussion timeline and audit log.
          </p>
        </div>
        {detail.discussion.locked ? (
          <span className="chip warn">Locked</span>
        ) : (
          <span className="chip soft">Unlocked</span>
        )}
      </div>

      <div className="mt-4 grid gap-2">
        <button
          className="btn sm"
          onClick={() => setPinOpen(true)}
          type="button"
        >
          {pinned ? "Edit pinned discussion" : "Pin discussion"}
        </button>
        {pinned ? (
          <button
            className="btn ghost sm"
            onClick={() => onPin("DELETE")}
            type="button"
          >
            Unpin
          </button>
        ) : null}
        <button
          className="btn sm"
          onClick={() => setLockOpen(true)}
          type="button"
        >
          {detail.discussion.locked
            ? "Unlock conversation"
            : "Lock conversation"}
        </button>
        {detail.discussion.state === "closed" ? (
          <button
            className="btn accent sm"
            onClick={() => onState("open")}
            type="button"
          >
            Reopen discussion
          </button>
        ) : (
          <div className="grid gap-2">
            <span className="t-label">Close discussion</span>
            <div className="flex flex-wrap gap-2">
              {["resolved", "duplicate", "outdated", "off-topic"].map(
                (reason) => (
                  <button
                    className="btn sm"
                    key={reason}
                    onClick={() => onState("closed", reason)}
                    type="button"
                  >
                    {reason}
                  </button>
                ),
              )}
            </div>
          </div>
        )}
        <label className="grid gap-2 t-sm">
          <span className="t-label">Category</span>
          <select
            aria-label="Moderation category"
            className="input"
            onChange={(event) => onCategory(event.target.value)}
            value={detail.category.slug}
          >
            {compatibleCategories.map((category) => (
              <option key={category.id} value={category.slug}>
                {category.emoji} {category.name}
              </option>
            ))}
          </select>
          {hiddenCategoryCount > 0 ? (
            <span className="chip warn w-fit">
              {pollCategoryConstraintText(detail)}
            </span>
          ) : null}
        </label>
      </div>

      {pinOpen ? (
        <div
          aria-labelledby="discussion-pin-dialog-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklab, var(--ink-1) 28%, transparent)",
          }}
        >
          <form
            className="card grid w-full max-w-lg gap-4 p-5"
            onSubmit={(event) => {
              event.preventDefault();
              onPin(pinned ? "PATCH" : "PUT", {
                target: pinTarget,
                categorySlug:
                  pinTarget === "category" ? detail.category.slug : undefined,
                title: pinTitle,
                body: pinBody,
              });
              setPinOpen(false);
            }}
          >
            <div>
              <p className="t-label">Pinned discussion</p>
              <h3 className="t-h2 mt-1" id="discussion-pin-dialog-title">
                {pinned ? "Edit pinned discussion" : "Pin discussion"}
              </h3>
            </div>
            <fieldset className="grid gap-2 border-0 p-0">
              <legend className="t-label">Target</legend>
              <label className="flex items-center gap-2 t-sm">
                <input
                  checked={pinTarget === "global"}
                  name="pin-target"
                  onChange={() => setPinTarget("global")}
                  type="radio"
                />
                Repository discussions
              </label>
              <label className="flex items-center gap-2 t-sm">
                <input
                  checked={pinTarget === "category"}
                  disabled={!canCategoryPin}
                  name="pin-target"
                  onChange={() => setPinTarget("category")}
                  type="radio"
                />
                Current category
              </label>
              {!canCategoryPin ? (
                <p className="t-xs" style={{ color: "var(--ink-3)" }}>
                  Poll discussions cannot be category-pinned in this phase.
                </p>
              ) : null}
            </fieldset>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Custom title</span>
              <input
                className="input"
                maxLength={120}
                onChange={(event) => setPinTitle(event.target.value)}
                placeholder={detail.discussion.title}
                value={pinTitle}
              />
            </label>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Pinned note</span>
              <textarea
                className="input min-h-24"
                maxLength={500}
                onChange={(event) => setPinBody(event.target.value)}
                placeholder="Explain why this discussion is pinned."
                value={pinBody}
              />
            </label>
            <div className="card p-3">
              <p className="t-label">Preview</p>
              <p className="t-sm mt-2 break-words">
                {pinTitle.trim() || detail.discussion.title}
              </p>
              <p
                className="t-xs mt-1 break-words"
                style={{ color: "var(--ink-3)" }}
              >
                {pinBody.trim() || "No pinned note."}
              </p>
            </div>
            <div className="flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setPinOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button className="btn accent sm" type="submit">
                {pinned ? "Save pinned copy" : "Pin discussion"}
              </button>
            </div>
          </form>
        </div>
      ) : null}

      {lockOpen ? (
        <div
          aria-labelledby="discussion-lock-dialog-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklab, var(--ink-1) 28%, transparent)",
          }}
        >
          <form
            className="card grid w-full max-w-md gap-4 p-5"
            onSubmit={(event) => {
              event.preventDefault();
              onLock(!detail.discussion.locked, allowReactions);
              setLockOpen(false);
            }}
          >
            <div>
              <p className="t-label">Conversation lock</p>
              <h3 className="t-h2 mt-1" id="discussion-lock-dialog-title">
                {detail.discussion.locked
                  ? "Unlock conversation"
                  : "Lock conversation"}
              </h3>
            </div>
            {!detail.discussion.locked ? (
              <label className="flex items-center gap-2 t-sm">
                <input
                  checked={allowReactions}
                  onChange={(event) => setAllowReactions(event.target.checked)}
                  type="checkbox"
                />
                Allow reactions while locked
              </label>
            ) : null}
            <div className="flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setLockOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button className="btn accent sm" type="submit">
                {detail.discussion.locked ? "Unlock" : "Lock"}
              </button>
            </div>
          </form>
        </div>
      ) : null}
    </section>
  );
}

function RepositoryDiscussionManagementPanel({
  detail,
  targets,
  onLoadTargets,
  onTransfer,
  onDelete,
}: {
  detail: RepositoryDiscussionDetailView;
  targets: DiscussionTransferTargetsView | null;
  onLoadTargets: () => void;
  onTransfer: (repositoryId: string, categorySlug: string) => void;
  onDelete: (confirmation: string, reason?: string) => void;
}) {
  const [transferOpen, setTransferOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [repositoryId, setRepositoryId] = useState("");
  const [categorySlug, setCategorySlug] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [reason, setReason] = useState("");
  const selectedTarget = targets?.targets.find(
    (target) => target.repositoryId === repositoryId,
  );
  const selectedTargetCategories = selectedTarget
    ? pollCompatibleCategories(detail, selectedTarget.categoryOptions)
    : [];
  const hiddenTargetCategoryCount = selectedTarget
    ? selectedTarget.categoryOptions.length - selectedTargetCategories.length
    : 0;
  const requiredConfirmation = `delete discussion ${detail.discussion.number}`;

  return (
    <section aria-labelledby="discussion-management-title" className="card p-4">
      <h2 className="t-label" id="discussion-management-title">
        Management
      </h2>
      <p className="t-xs mt-2" style={{ color: "var(--ink-3)" }}>
        Transfers and deletions are restricted to write members and leave audit
        evidence.
      </p>
      <div className="mt-4 grid gap-2">
        <button
          className="btn sm"
          onClick={() => {
            setTransferOpen(true);
            onLoadTargets();
          }}
          type="button"
        >
          Transfer discussion
        </button>
        <button
          className="btn ghost sm"
          onClick={() => setDeleteOpen(true)}
          type="button"
        >
          Delete discussion
        </button>
      </div>

      {transferOpen ? (
        <div
          aria-labelledby="discussion-transfer-dialog-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklab, var(--ink-1) 28%, transparent)",
          }}
        >
          <form
            className="card grid w-full max-w-lg gap-4 p-5"
            onSubmit={(event) => {
              event.preventDefault();
              onTransfer(repositoryId, categorySlug);
              setTransferOpen(false);
            }}
          >
            <div>
              <p className="t-label">Transfer</p>
              <h3 className="t-h2 mt-1" id="discussion-transfer-dialog-title">
                Move this discussion
              </h3>
            </div>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Repository</span>
              <select
                aria-label="Transfer destination repository"
                className="input"
                onChange={(event) => {
                  const next = event.target.value;
                  setRepositoryId(next);
                  const target = targets?.targets.find(
                    (candidate) => candidate.repositoryId === next,
                  );
                  setCategorySlug(
                    target
                      ? (pollCompatibleCategories(
                          detail,
                          target.categoryOptions,
                        )[0]?.slug ?? "")
                      : "",
                  );
                }}
                value={repositoryId}
              >
                <option value="">Choose a repository</option>
                {(targets?.targets ?? []).map((target) => (
                  <option key={target.repositoryId} value={target.repositoryId}>
                    {target.owner}/{target.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Destination category</span>
              <select
                aria-label="Transfer destination category"
                className="input"
                disabled={!selectedTarget}
                onChange={(event) => setCategorySlug(event.target.value)}
                value={categorySlug}
              >
                <option value="">Choose a category</option>
                {selectedTargetCategories.map((category) => (
                  <option key={category.id} value={category.slug}>
                    {category.emoji} {category.name}
                  </option>
                ))}
              </select>
              {hiddenTargetCategoryCount > 0 ? (
                <span className="chip warn w-fit">
                  {pollCategoryConstraintText(detail)}
                </span>
              ) : null}
            </label>
            <div className="flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setTransferOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn accent sm"
                disabled={!repositoryId || !categorySlug}
                type="submit"
              >
                Transfer
              </button>
            </div>
          </form>
        </div>
      ) : null}

      {deleteOpen ? (
        <div
          aria-labelledby="discussion-delete-dialog-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklab, var(--ink-1) 28%, transparent)",
          }}
        >
          <form
            className="card grid w-full max-w-lg gap-4 p-5"
            onSubmit={(event) => {
              event.preventDefault();
              onDelete(confirmation, reason);
              setDeleteOpen(false);
            }}
          >
            <div>
              <p className="t-label">Delete</p>
              <h3 className="t-h2 mt-1" id="discussion-delete-dialog-title">
                Delete this discussion
              </h3>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                Comments and body content are hidden from future reads. A
                tombstone keeps hashes and audit metadata only.
              </p>
            </div>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Reason</span>
              <textarea
                className="input min-h-20"
                maxLength={280}
                onChange={(event) => setReason(event.target.value)}
                value={reason}
              />
            </label>
            <label className="grid gap-2 t-sm">
              <span className="t-label">Type {requiredConfirmation}</span>
              <input
                className="input"
                onChange={(event) => setConfirmation(event.target.value)}
                value={confirmation}
              />
            </label>
            <div className="flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setDeleteOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn accent sm"
                disabled={confirmation !== requiredConfirmation}
                type="submit"
              >
                Delete discussion
              </button>
            </div>
          </form>
        </div>
      ) : null}
    </section>
  );
}

function Sidebar({
  detail,
  owner,
  repo,
  subscription,
  transferTargets,
  onSubscription,
  onMetadata,
  onModerationCategory,
  onModerationLock,
  onModerationPin,
  onModerationState,
  onLoadTransferTargets,
  onTransfer,
  onDelete,
}: {
  detail: RepositoryDiscussionDetailView;
  owner: string;
  repo: string;
  subscription: DiscussionSubscriptionState;
  transferTargets: DiscussionTransferTargetsView | null;
  onSubscription: (subscribed: boolean) => void;
  onMetadata: (request: { categorySlug?: string; labelIds?: string[] }) => void;
  onModerationCategory: (categorySlug: string) => void;
  onModerationLock: (locked: boolean, allowReactions?: boolean) => void;
  onModerationPin: (
    method: "PUT" | "PATCH" | "DELETE",
    request?: Record<string, unknown>,
  ) => void;
  onModerationState: (state: "open" | "closed", reason?: string) => void;
  onLoadTransferTargets: () => void;
  onTransfer: (repositoryId: string, categorySlug: string) => void;
  onDelete: (confirmation: string, reason?: string) => void;
}) {
  const canModerate = detail.viewer.authenticated && detail.viewer.canModerate;
  const [labelPickerOpen, setLabelPickerOpen] = useState(false);
  const compatibleCategories = pollCompatibleCategories(
    detail,
    detail.sidebar.categoryOptions,
  );
  const hiddenCategoryCount =
    detail.sidebar.categoryOptions.length - compatibleCategories.length;
  return (
    <aside className="space-y-5">
      {canModerate ? (
        <>
          <RepositoryDiscussionModerationPanel
            detail={detail}
            onCategory={onModerationCategory}
            onLock={onModerationLock}
            onPin={onModerationPin}
            onState={onModerationState}
          />
          <RepositoryDiscussionManagementPanel
            detail={detail}
            onDelete={onDelete}
            onLoadTargets={onLoadTransferTargets}
            onTransfer={onTransfer}
            targets={transferTargets}
          />
        </>
      ) : (
        <section className="card p-4">
          <h2 className="t-label">Moderator controls</h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Triage, write, and admin members can moderate this discussion.
          </p>
        </section>
      )}
      <section className="card p-4">
        <h2 className="t-label">Notifications</h2>
        <p className="t-sm mt-2">
          {subscription.subscribed ? "Subscribed" : "Not subscribed"}
        </p>
        <button
          className="btn sm mt-3"
          disabled={!subscription.canChange}
          onClick={() => onSubscription(!subscription.subscribed)}
          type="button"
        >
          {subscription.subscribed ? "Unsubscribe" : "Subscribe"}
        </button>
      </section>
      <section className="card p-4">
        <h2 className="t-label">Category</h2>
        <Link className="chip soft mt-3" href={detail.category.href}>
          <span aria-hidden="true">{detail.category.emoji}</span>
          {detail.category.name}
        </Link>
        {canModerate ? (
          <label className="mt-3 grid gap-2 t-sm">
            <span className="t-label">Change category</span>
            <select
              aria-label="Change discussion category"
              className="input"
              onChange={(event) =>
                onMetadata({ categorySlug: event.target.value })
              }
              value={detail.category.slug}
            >
              {compatibleCategories.map((category) => (
                <option key={category.id} value={category.slug}>
                  {category.emoji} {category.name}
                </option>
              ))}
            </select>
            {hiddenCategoryCount > 0 ? (
              <span className="chip warn w-fit">
                {pollCategoryConstraintText(detail)}
              </span>
            ) : null}
          </label>
        ) : null}
      </section>
      <section className="card p-4">
        <h2 className="t-label">Labels</h2>
        <div className="mt-3 flex items-center justify-between gap-2">
          <span className="t-xs">Classify this discussion.</span>
          {canModerate ? (
            <button
              aria-expanded={labelPickerOpen}
              className="btn sm"
              onClick={() => setLabelPickerOpen((open) => !open)}
              type="button"
            >
              Edit
            </button>
          ) : null}
        </div>
        {labelPickerOpen ? (
          <LabelPicker
            labels={detail.sidebar.labelOptions}
            onCancel={() => setLabelPickerOpen(false)}
            onSave={(labels: IssueListLabel[]) => {
              setLabelPickerOpen(false);
              onMetadata({ labelIds: labels.map((label) => label.id) });
            }}
            selectedLabels={detail.labels}
            title="Discussion label picker"
          />
        ) : null}
        <div className="mt-3 flex flex-wrap gap-2">
          {detail.labels.length ? (
            detail.labels.map((label) => (
              <Link
                className="chip soft"
                href={repositoryDiscussionsHref(owner, repo, {
                  label: label.name,
                  page: 1,
                })}
                key={label.id}
                title={label.description ?? label.name}
              >
                <span
                  aria-hidden="true"
                  className="inline-block h-2 w-2 rounded-full"
                  style={{ background: label.color }}
                />
                {label.name}
              </Link>
            ))
          ) : (
            <span className="t-sm" style={{ color: "var(--ink-3)" }}>
              None yet
            </span>
          )}
        </div>
      </section>
      <section className="card p-4">
        <h2 className="t-label">Participants</h2>
        <div className="mt-3 flex flex-wrap gap-2">
          {detail.sidebar.participants.map((participant) => (
            <Avatar key={participant.id} user={participant} />
          ))}
        </div>
      </section>
      <section className="card p-4">
        <h2 className="t-label">Events</h2>
        <ul className="mt-3 space-y-2">
          {detail.sidebar.events.slice(0, 5).map((event) => (
            <li className="t-xs" key={event.id}>
              {event.actor?.login ?? "opengithub"} · {eventLabel(event)}
            </li>
          ))}
        </ul>
      </section>
    </aside>
  );
}

export function RepositoryDiscussionDetailPage({
  repository,
  detail,
}: RepositoryDiscussionDetailPageProps) {
  const [currentDetail, setCurrentDetail] = useState(detail);
  const [subscription, setSubscription] = useState(detail.subscription);
  const [reactions, setReactions] = useState(detail.reactions);
  const [transferTargets, setTransferTargets] =
    useState<DiscussionTransferTargetsView | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const router = useRouter();
  const owner = repository.owner_login;
  const repo = repository.name;
  const bodyLabelId = "discussion-body";
  const detailHref = repositoryDiscussionDetailHref(
    owner,
    repo,
    detail.discussion.number,
  );
  const canReact =
    currentDetail.viewer.authenticated && currentDetail.viewer.canReact;
  const canComment =
    currentDetail.viewer.authenticated && currentDetail.viewer.canComment;
  const canVote =
    currentDetail.viewer.authenticated && currentDetail.viewer.canRead;

  async function mutateDetail(path: string, body: string) {
    setMessage(null);
    const response = await fetch(path, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ body }),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(payload?.error?.message ?? "Discussion could not be updated.");
      return;
    }
    setCurrentDetail(payload);
    setSubscription(payload.subscription);
    setReactions(payload.reactions);
    setMessage("Discussion updated.");
  }

  async function toggleReaction(
    content: DiscussionReactionContent,
    reacted: boolean,
    commentId?: string,
  ) {
    setMessage(null);
    const target = commentId
      ? `${detailHref}/comments/${encodeURIComponent(commentId)}/reactions`
      : `${detailHref}/reactions`;
    const response = await fetch(target, {
      method: reacted ? "PUT" : "DELETE",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ content }),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(payload?.error?.message ?? "Reaction could not be updated.");
      return;
    }
    if (commentId) {
      setCurrentDetail((previous) => ({
        ...previous,
        timeline: previous.timeline.map((item) =>
          item.kind === "comment"
            ? {
                ...updateCommentReactions(item, commentId, payload),
                kind: "comment" as const,
              }
            : item,
        ),
      }));
    } else {
      setReactions(payload);
    }
  }

  async function toggleVote(voted: boolean) {
    if (!canVote) return;
    setMessage(null);
    const response = await fetch(`${detailHref}/vote`, {
      method: voted ? "PUT" : "DELETE",
    });
    const payload = (await response.json().catch(() => null)) as
      | DiscussionVoteResponse
      | { error?: { message?: string } }
      | null;
    if (!response.ok) {
      setMessage(
        payload && "error" in payload
          ? (payload.error?.message ?? "Discussion vote could not be updated.")
          : "Discussion vote could not be updated.",
      );
      return;
    }
    if (!payload || !("votesCount" in payload)) return;
    setCurrentDetail((previous) => ({
      ...previous,
      viewer: { ...previous.viewer, viewerVoted: payload.viewerVoted },
      discussion: {
        ...previous.discussion,
        votesCount: payload.votesCount,
      },
    }));
    setMessage(payload.viewerVoted ? "Discussion upvoted." : "Upvote removed.");
  }

  async function toggleSubscription(subscribed: boolean) {
    setMessage(null);
    const response = await fetch(`${detailHref}/subscription`, {
      method: subscribed ? "PUT" : "DELETE",
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(
        payload?.error?.message ??
          "Notification subscription could not be updated.",
      );
      return;
    }
    setSubscription(payload);
    setMessage(payload.subscribed ? "Subscribed." : "Unsubscribed.");
  }

  async function mutateModeration(
    path: string,
    method: "PUT" | "DELETE" | "PATCH",
    body: Record<string, unknown>,
    success: string,
  ) {
    setMessage(null);
    const response = await fetch(path, {
      method,
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(payload?.error?.message ?? "Discussion could not be updated.");
      return;
    }
    setCurrentDetail(payload);
    setSubscription(payload.subscription);
    setReactions(payload.reactions);
    setMessage(success);
  }

  function toggleAnswer(commentId: string, marked: boolean) {
    void mutateModeration(
      `${detailHref}/answer`,
      marked ? "PUT" : "DELETE",
      { commentId },
      marked ? "Answer marked." : "Answer unmarked.",
    );
  }

  function updateState(state: "open" | "closed", reason?: string) {
    void mutateModeration(
      `${detailHref}/state`,
      "PUT",
      { state, reason },
      state === "open" ? "Discussion reopened." : "Discussion closed.",
    );
  }

  function updatePin(
    method: "PUT" | "PATCH" | "DELETE",
    request: Record<string, unknown> = {},
  ) {
    void mutateModeration(
      `${detailHref}/pin`,
      method,
      request,
      method === "DELETE"
        ? "Discussion unpinned."
        : method === "PATCH"
          ? "Pinned discussion updated."
          : "Discussion pinned.",
    );
  }

  function updateLock(locked: boolean, allowReactions = true) {
    void mutateModeration(
      `${detailHref}/lock`,
      locked ? "PUT" : "DELETE",
      { allowReactions },
      locked ? "Discussion locked." : "Discussion unlocked.",
    );
  }

  function updateCategory(categorySlug: string) {
    void mutateModeration(
      `${detailHref}/category`,
      "PATCH",
      { categorySlug },
      "Discussion category changed.",
    );
  }

  function updateMetadata(request: {
    categorySlug?: string;
    labelIds?: string[];
  }) {
    void mutateModeration(
      `${detailHref}/metadata`,
      "PATCH",
      request,
      "Discussion metadata updated.",
    );
  }

  async function loadTransferTargets() {
    if (transferTargets) return;
    setMessage(null);
    const response = await fetch(`${detailHref}/transfer-targets`);
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(
        payload?.error?.message ??
          "Discussion transfer targets could not be loaded.",
      );
      return;
    }
    setTransferTargets(payload);
  }

  async function transferDiscussion(
    repositoryId: string,
    categorySlug: string,
  ) {
    setMessage(null);
    const response = await fetch(`${detailHref}/transfer`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ repositoryId, categorySlug }),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(
        payload?.error?.message ?? "Discussion could not be transferred.",
      );
      return;
    }
    setMessage("Discussion transferred.");
    router.push(payload.destinationHref);
  }

  async function deleteDiscussion(confirmation: string, reason?: string) {
    setMessage(null);
    const response = await fetch(`${detailHref}/delete`, {
      method: "DELETE",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ confirmation, reason }),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMessage(payload?.error?.message ?? "Discussion could not be deleted.");
      return;
    }
    setMessage("Discussion deleted.");
    router.push(payload.discussionsHref);
  }

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/discussions`}
      frameClassName="grid grid-cols-[minmax(0,1fr)_300px] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0 space-y-5">
        <nav aria-label="Discussion breadcrumbs" className="t-sm">
          <Link href={repositoryDiscussionsHref(owner, repo)}>Discussions</Link>
          <span style={{ color: "var(--ink-4)" }}> / </span>
          <Link href={detail.category.href}>{detail.category.name}</Link>
        </nav>
        <section className="space-y-3">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0">
              <h1 className="t-h1 break-words">
                {detail.discussion.title}{" "}
                <span style={{ color: "var(--ink-4)", fontWeight: 400 }}>
                  #{detail.discussion.number}
                </span>
              </h1>
              <div className="mt-3 flex flex-wrap items-center gap-2">
                {statusChip(currentDetail)}
                <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                  <Avatar user={detail.author} /> {detail.author.login} opened{" "}
                  {relativeTime(detail.discussion.createdAt)}
                </span>
                <span className="chip soft">
                  {formatNumber(currentDetail.discussion.commentsCount)}{" "}
                  comments
                </span>
                <button
                  aria-pressed={currentDetail.viewer.viewerVoted}
                  className={
                    currentDetail.viewer.viewerVoted
                      ? "chip active"
                      : "chip soft"
                  }
                  disabled={!canVote}
                  onClick={() => toggleVote(!currentDetail.viewer.viewerVoted)}
                  type="button"
                >
                  {currentDetail.viewer.viewerVoted
                    ? "Remove upvote"
                    : "Upvote"}{" "}
                  <span className="t-num">
                    {formatNumber(currentDetail.discussion.votesCount)}
                  </span>
                </button>
              </div>
            </div>
            <a className="btn" href={detail.discussion.href}>
              Permalink
            </a>
          </div>
          {currentDetail.answer ? (
            <div className="card p-4" style={{ borderColor: "var(--ok)" }}>
              <p className="t-label" style={{ color: "var(--ok)" }}>
                Answered
              </p>
              <p className="t-sm mt-2">
                Marked by{" "}
                {currentDetail.answer.markedBy?.login ?? "a maintainer"} ·{" "}
                <a href={currentDetail.answer.href}>View full answer</a>
              </p>
            </div>
          ) : null}
        </section>

        <article className="flex min-w-0 gap-4">
          <Avatar size="lg" user={detail.author} />
          <div className="card min-w-0 flex-1 overflow-hidden">
            <div
              className="flex flex-wrap items-center gap-2 border-b px-4 py-3"
              style={{
                background: "var(--surface-2)",
                borderColor: "var(--line)",
              }}
            >
              <h2 className="t-sm font-semibold" id={bodyLabelId}>
                {detail.author.login}
              </h2>
              <span className="t-xs">started this discussion</span>
              <span className="chip soft ml-auto">author</span>
            </div>
            <div className="p-5">
              <MarkdownBody html={detail.body.html} labelledBy={bodyLabelId} />
              {detail.formAnswers.length ? (
                <dl className="mt-5 grid gap-3">
                  {detail.formAnswers.map((answer) => (
                    <div className="card p-3" key={answer.fieldId}>
                      <dt className="t-label">{answer.fieldLabel}</dt>
                      <dd className="t-sm mt-1 break-words">{answer.value}</dd>
                    </div>
                  ))}
                </dl>
              ) : null}
              {detail.poll ? (
                <PollVotingCard
                  detailHref={detailHref}
                  onMessage={setMessage}
                  onPollUpdate={(poll) =>
                    setCurrentDetail((previous) => ({ ...previous, poll }))
                  }
                  poll={currentDetail.poll ?? detail.poll}
                />
              ) : null}
              <ReactionSummary
                canReact={canReact}
                label="Discussion reactions"
                onToggle={(content, reacted) =>
                  toggleReaction(content, reacted)
                }
                reactions={reactions}
              />
            </div>
          </div>
        </article>

        <div className="flex flex-wrap items-center justify-between gap-3">
          <SortLinks detail={detail} owner={owner} repo={repo} />
          <p className="t-xs" style={{ color: "var(--ink-3)" }}>
            {formatNumber(currentDetail.totalComments)} timeline comments
          </p>
        </div>

        {message ? (
          <p className="chip soft" role="status">
            {message}
          </p>
        ) : null}

        <section aria-label="Discussion timeline" className="space-y-5">
          {currentDetail.timeline.map((item) =>
            item.kind === "event" ? (
              <EventRow event={item} key={item.id} />
            ) : (
              <div className="space-y-4" key={item.id}>
                <CommentCard
                  canComment={canComment}
                  canReact={canReact}
                  comment={item}
                  isAnswer={item.answer}
                  onAnswerToggle={
                    currentDetail.viewer.canMarkAnswer
                      ? toggleAnswer
                      : undefined
                  }
                  onReaction={(commentId, content, reacted) =>
                    toggleReaction(content, reacted, commentId)
                  }
                  onReply={(commentId, body) =>
                    mutateDetail(
                      `${detailHref}/comments/${encodeURIComponent(commentId)}/replies`,
                      body,
                    )
                  }
                />
                {item.replies.length ? (
                  <div className="space-y-4 pl-8 sm:pl-16">
                    {item.replies.map((reply) => (
                      <CommentCard
                        canComment={false}
                        canReact={canReact}
                        comment={reply}
                        key={reply.id}
                        onReaction={(commentId, content, reacted) =>
                          toggleReaction(content, reacted, commentId)
                        }
                      />
                    ))}
                  </div>
                ) : null}
              </div>
            ),
          )}
        </section>

        <ReplyComposer
          detail={currentDetail}
          onComment={(body) => mutateDetail(`${detailHref}/comments`, body)}
        />
      </main>
      <Sidebar
        detail={currentDetail}
        owner={owner}
        repo={repo}
        onMetadata={updateMetadata}
        onDelete={deleteDiscussion}
        onLoadTransferTargets={loadTransferTargets}
        onModerationCategory={updateCategory}
        onModerationLock={updateLock}
        onModerationPin={updatePin}
        onModerationState={updateState}
        onSubscription={toggleSubscription}
        onTransfer={transferDiscussion}
        subscription={subscription}
        transferTargets={transferTargets}
      />
    </RepositoryShell>
  );
}

function updateCommentReactions(
  comment: DiscussionCommentView,
  commentId: string,
  reactions: DiscussionReactionSummary[],
): DiscussionCommentView {
  if (comment.id === commentId) return { ...comment, reactions };
  return {
    ...comment,
    replies: comment.replies.map((reply) =>
      reply.id === commentId ? { ...reply, reactions } : reply,
    ),
  };
}
