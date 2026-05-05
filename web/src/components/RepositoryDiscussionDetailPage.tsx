import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  DiscussionAuthorSummary,
  DiscussionCommentView,
  DiscussionEventView,
  DiscussionReactionSummary,
  DiscussionReplyView,
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

function statusChip(detail: RepositoryDiscussionDetailView) {
  if (detail.discussion.answered)
    return <span className="chip ok">Answered</span>;
  if (detail.discussion.state === "closed")
    return <span className="chip soft">Closed</span>;
  return <span className="chip ok">Open</span>;
}

function ReactionSummary({
  label,
  reactions,
}: {
  label: string;
  reactions: DiscussionReactionSummary[];
}) {
  const visible = reactions.filter((reaction) => reaction.count > 0);
  if (!visible.length) return null;
  return (
    <fieldset
      aria-label={label}
      className="mt-4 flex flex-wrap gap-2 border-0 p-0"
    >
      {visible.map((reaction) => (
        <span
          className={reaction.viewerReacted ? "chip active" : "chip soft"}
          key={reaction.content}
        >
          {reaction.content} <span className="t-num">{reaction.count}</span>
        </span>
      ))}
    </fieldset>
  );
}

function CommentCard({
  comment,
  isAnswer,
}: {
  comment: DiscussionCommentView | DiscussionReplyView;
  isAnswer?: boolean;
}) {
  const labelId = `discussion-comment-${comment.id}`;
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
            label={`Reactions for ${comment.author.login} comment`}
            reactions={comment.reactions}
          />
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

function ReplyComposer({ detail }: { detail: RepositoryDiscussionDetailView }) {
  const canComment = detail.viewer.authenticated && detail.viewer.canComment;
  return (
    <section className="card p-4" aria-label="Reply composer">
      <div className="tabs">
        <button className="tab active" type="button">
          Write
        </button>
        <button className="tab" disabled type="button">
          Preview
        </button>
      </div>
      <label className="t-label mt-4 block" htmlFor="discussion-reply">
        Reply
      </label>
      <textarea
        className="input mt-2 min-h-32 w-full"
        disabled
        id="discussion-reply"
        placeholder={
          canComment
            ? "Comment writing lands in the next implementation phase."
            : "Sign in with write access to join this discussion."
        }
      />
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button className="btn sm" disabled type="button">
          Saved replies
        </button>
        <button className="btn sm" disabled type="button">
          Attach files
        </button>
        <button className="btn accent sm" disabled type="button">
          Comment
        </button>
      </div>
      <p className="t-xs mt-3" style={{ color: "var(--ink-3)" }}>
        Comment, attachment, and preview writes are disabled until Phase 3.
      </p>
    </section>
  );
}

function Sidebar({ detail }: { detail: RepositoryDiscussionDetailView }) {
  return (
    <aside className="space-y-5">
      <section className="card p-4">
        <h2 className="t-label">Notifications</h2>
        <p className="t-sm mt-2">
          {detail.subscription.subscribed ? "Subscribed" : "Not subscribed"}
        </p>
        <button
          className="btn sm mt-3"
          disabled={!detail.subscription.canChange}
          type="button"
        >
          {detail.subscription.subscribed ? "Unsubscribe" : "Subscribe"}
        </button>
      </section>
      <section className="card p-4">
        <h2 className="t-label">Category</h2>
        <Link className="chip soft mt-3" href={detail.category.href}>
          <span aria-hidden="true">{detail.category.emoji}</span>
          {detail.category.name}
        </Link>
      </section>
      <section className="card p-4">
        <h2 className="t-label">Labels</h2>
        <div className="mt-3 flex flex-wrap gap-2">
          {detail.labels.length ? (
            detail.labels.map((label) => (
              <span className="chip soft" key={label.id}>
                {label.name}
              </span>
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
              {event.actor?.login ?? "opengithub"} ·{" "}
              {event.eventType.replaceAll("_", " ")}
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
  const owner = repository.owner_login;
  const repo = repository.name;
  const bodyLabelId = "discussion-body";

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
                {statusChip(detail)}
                <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                  <Avatar user={detail.author} /> {detail.author.login} opened{" "}
                  {relativeTime(detail.discussion.createdAt)}
                </span>
                <span className="chip soft">
                  {formatNumber(detail.discussion.commentsCount)} comments
                </span>
                <span className="chip soft">
                  {formatNumber(detail.discussion.votesCount)} votes
                </span>
              </div>
            </div>
            <a className="btn" href={detail.discussion.href}>
              Permalink
            </a>
          </div>
          {detail.answer ? (
            <div className="card p-4" style={{ borderColor: "var(--ok)" }}>
              <p className="t-label" style={{ color: "var(--ok)" }}>
                Answered
              </p>
              <p className="t-sm mt-2">
                Marked by {detail.answer.markedBy?.login ?? "a maintainer"} ·{" "}
                <a href={detail.answer.href}>View full answer</a>
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
                <section className="card mt-5 p-4">
                  <h3 className="t-h3">{detail.poll.question}</h3>
                  <ul className="mt-3 grid gap-2">
                    {detail.poll.options.map((option) => (
                      <li className="chip soft justify-start" key={option.id}>
                        {option.label}
                      </li>
                    ))}
                  </ul>
                </section>
              ) : null}
              <ReactionSummary
                label="Discussion reactions"
                reactions={detail.reactions}
              />
            </div>
          </div>
        </article>

        <div className="flex flex-wrap items-center justify-between gap-3">
          <SortLinks detail={detail} owner={owner} repo={repo} />
          <p className="t-xs" style={{ color: "var(--ink-3)" }}>
            {formatNumber(detail.totalComments)} timeline comments
          </p>
        </div>

        <section aria-label="Discussion timeline" className="space-y-5">
          {detail.timeline.map((item) =>
            item.kind === "event" ? (
              <EventRow event={item} key={item.id} />
            ) : (
              <div className="space-y-4" key={item.id}>
                <CommentCard comment={item} isAnswer={item.answer} />
                {item.replies.length ? (
                  <div className="space-y-4 pl-8 sm:pl-16">
                    {item.replies.map((reply) => (
                      <CommentCard comment={reply} key={reply.id} />
                    ))}
                  </div>
                ) : null}
              </div>
            ),
          )}
        </section>

        <ReplyComposer detail={detail} />
      </main>
      <Sidebar detail={detail} />
    </RepositoryShell>
  );
}
