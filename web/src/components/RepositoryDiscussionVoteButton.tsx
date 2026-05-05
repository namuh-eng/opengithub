"use client";

import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";

type RepositoryDiscussionVoteButtonProps = {
  owner: string;
  repo: string;
  discussionNumber: number;
  initialVotesCount: number;
  initialViewerVoted: boolean;
  canVote: boolean;
  authenticated: boolean;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

export function RepositoryDiscussionVoteButton({
  owner,
  repo,
  discussionNumber,
  initialVotesCount,
  initialViewerVoted,
  canVote,
  authenticated,
}: RepositoryDiscussionVoteButtonProps) {
  const router = useRouter();
  const [votesCount, setVotesCount] = useState(initialVotesCount);
  const [viewerVoted, setViewerVoted] = useState(initialViewerVoted);
  const [error, setError] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();

  if (!authenticated) {
    return (
      <a
        aria-label={`Sign in to upvote discussion ${discussionNumber}`}
        className="chip soft"
        href={`/login?next=/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions`}
        title="Sign in to upvote this discussion"
      >
        ▲ <span className="t-num">{formatNumber(initialVotesCount)}</span>
        <span className="sr-only">Sign in to upvote discussion</span>
      </a>
    );
  }

  async function toggleVote() {
    if (!canVote || isPending) return;
    const nextVoted = !viewerVoted;
    const nextCount = Math.max(0, votesCount + (nextVoted ? 1 : -1));
    setViewerVoted(nextVoted);
    setVotesCount(nextCount);
    setError(null);

    startTransition(() => {
      void (async () => {
        const response = await fetch(
          `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/${encodeURIComponent(String(discussionNumber))}/vote`,
          {
            method: nextVoted ? "PUT" : "DELETE",
          },
        );
        const payload = await response.json().catch(() => null);
        if (!response.ok) {
          setViewerVoted(!nextVoted);
          setVotesCount(votesCount);
          setError(
            payload?.error?.message ?? "Discussion vote could not be updated.",
          );
          return;
        }
        setViewerVoted(Boolean(payload.viewerVoted));
        setVotesCount(Number(payload.votesCount) || 0);
        router.refresh();
      })();
    });
  }

  const label = viewerVoted
    ? `Remove upvote from discussion ${discussionNumber}`
    : `Upvote discussion ${discussionNumber}`;

  return (
    <span className="inline-flex flex-col items-start gap-1">
      <button
        aria-label={label}
        aria-pressed={viewerVoted}
        className={viewerVoted ? "chip accent" : "chip soft"}
        disabled={!canVote || isPending}
        onClick={toggleVote}
        title={
          canVote
            ? label
            : "Archived or disabled repositories do not accept discussion votes"
        }
        type="button"
      >
        ▲ <span className="t-num">{formatNumber(votesCount)}</span>
        <span className="sr-only">{label}</span>
      </button>
      {error ? (
        <span
          className="t-xs max-w-24"
          role="status"
          style={{ color: "var(--err)" }}
        >
          {error}
        </span>
      ) : null}
    </span>
  );
}
