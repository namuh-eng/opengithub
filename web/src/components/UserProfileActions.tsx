"use client";

import { type FormEvent, useEffect, useMemo, useState } from "react";
import type { ApiErrorEnvelope, ProfileViewerState } from "@/lib/api";

type UserProfileActionsProps = {
  followerCount: number | null;
  followingCount: number | null;
  isPrivate: boolean;
  login: string;
  viewerState: ProfileViewerState;
};

function countLabel(count: number, singular: string, plural = `${singular}s`) {
  return `${count.toLocaleString()} ${count === 1 ? singular : plural}`;
}

async function readError(response: Response, fallback: string) {
  const envelope = (await response
    .json()
    .catch(() => null)) as ApiErrorEnvelope | null;
  return envelope?.error.message ?? fallback;
}

function loginUrl() {
  const next =
    typeof window === "undefined"
      ? "/dashboard"
      : `${window.location.pathname}${window.location.search}`;
  return `/login?next=${encodeURIComponent(next)}`;
}

export function UserProfileActions({
  followerCount,
  followingCount,
  isPrivate,
  login,
  viewerState,
}: UserProfileActionsProps) {
  const [state, setState] = useState(viewerState);
  const [followers, setFollowers] = useState(followerCount);
  const [menuOpen, setMenuOpen] = useState(false);
  const [loginGateOpen, setLoginGateOpen] = useState(false);
  const [blockDialogOpen, setBlockDialogOpen] = useState(false);
  const [reportDialogOpen, setReportDialogOpen] = useState(false);
  const [pending, setPending] = useState<"follow" | "block" | "report" | null>(
    null,
  );
  const [feedback, setFeedback] = useState<string | null>(null);
  const [reportReason, setReportReason] = useState("spam");
  const [reportDetails, setReportDetails] = useState("");

  useEffect(() => {
    setState(viewerState);
    setFollowers(followerCount);
  }, [followerCount, viewerState]);

  const showActions = !isPrivate && !state.isSelf;
  const followLabel = state.isFollowing ? "Following" : "Follow";
  const relationshipLabel = useMemo(() => {
    if (followers === null) {
      return null;
    }
    if (followingCount === null) {
      return countLabel(followers, "follower");
    }
    return `${countLabel(followers, "follower")} · ${countLabel(
      followingCount,
      "following",
      "following",
    )}`;
  }, [followers, followingCount]);

  async function toggleFollow() {
    if (!state.authenticated) {
      setLoginGateOpen(true);
      return;
    }
    const nextFollowing = !state.isFollowing;
    const previousState = state;
    const previousFollowers = followers;
    setFeedback(null);
    setPending("follow");
    setState({ ...state, isFollowing: nextFollowing });
    setFollowers((count) =>
      count === null ? count : Math.max(0, count + (nextFollowing ? 1 : -1)),
    );

    const response = await fetch(
      `/${encodeURIComponent(login)}/actions/follow`,
      {
        method: nextFollowing ? "PUT" : "DELETE",
      },
    );
    if (!response.ok) {
      setState(previousState);
      setFollowers(previousFollowers);
      setFeedback(await readError(response, "Follow update failed."));
      setPending(null);
      return;
    }
    const result = await response.json();
    setState(result.viewerState);
    setFollowers(result.followerCount);
    setFeedback(
      nextFollowing ? "Now following this profile." : "Profile unfollowed.",
    );
    setPending(null);
  }

  async function blockProfile() {
    if (!state.authenticated) {
      setLoginGateOpen(true);
      return;
    }
    setPending("block");
    setFeedback(null);
    const response = await fetch(
      `/${encodeURIComponent(login)}/actions/block`,
      {
        body: JSON.stringify({ reason: "User requested block from profile" }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      },
    );
    if (!response.ok) {
      setFeedback(await readError(response, "Profile block failed."));
      setPending(null);
      return;
    }
    const result = await response.json();
    setState(result.viewerState);
    setFollowers(result.followerCount);
    setBlockDialogOpen(false);
    setMenuOpen(false);
    setFeedback("Profile blocked.");
    setPending(null);
  }

  async function reportProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!state.authenticated) {
      setLoginGateOpen(true);
      return;
    }
    setPending("report");
    setFeedback(null);
    const response = await fetch(
      `/${encodeURIComponent(login)}/actions/report`,
      {
        body: JSON.stringify({
          details: reportDetails,
          reason: reportReason,
        }),
        headers: { "content-type": "application/json" },
        method: "POST",
      },
    );
    if (!response.ok) {
      setFeedback(await readError(response, "Profile report failed."));
      setPending(null);
      return;
    }
    setReportDialogOpen(false);
    setMenuOpen(false);
    setFeedback("Report submitted for review.");
    setPending(null);
  }

  return (
    <div className="mt-4 grid gap-3">
      {relationshipLabel ? (
        <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {relationshipLabel}
        </p>
      ) : null}

      {isPrivate ? null : state.isSelf ? (
        <span className="chip soft w-fit">Your profile</span>
      ) : showActions ? (
        <div className="flex flex-wrap items-center gap-2">
          <button
            className={`btn sm ${state.isFollowing ? "" : "primary"}`}
            disabled={pending === "follow" || state.isBlocking}
            onClick={toggleFollow}
            type="button"
          >
            {pending === "follow" ? "Saving..." : followLabel}
          </button>
          <div className="relative">
            <button
              aria-expanded={menuOpen}
              aria-haspopup="menu"
              className="btn sm"
              onClick={() => setMenuOpen((open) => !open)}
              type="button"
            >
              More
            </button>
            {menuOpen ? (
              <div
                className="card absolute left-0 top-[calc(100%+6px)] z-20 grid min-w-44 gap-1 p-2"
                role="menu"
              >
                <button
                  className="btn ghost sm justify-start"
                  onClick={() => {
                    if (!state.authenticated) {
                      setLoginGateOpen(true);
                      return;
                    }
                    setReportDialogOpen(true);
                  }}
                  role="menuitem"
                  type="button"
                >
                  Report profile
                </button>
                <button
                  className="btn ghost sm justify-start"
                  disabled={state.isBlocking}
                  onClick={() => {
                    if (!state.authenticated) {
                      setLoginGateOpen(true);
                      return;
                    }
                    setBlockDialogOpen(true);
                  }}
                  role="menuitem"
                  type="button"
                >
                  {state.isBlocking ? "Blocked" : "Block profile"}
                </button>
              </div>
            ) : null}
          </div>
        </div>
      ) : null}

      {feedback ? (
        <p className="t-sm" role="status" style={{ color: "var(--ink-2)" }}>
          {feedback}
        </p>
      ) : null}

      {loginGateOpen ? (
        <div
          aria-labelledby="profile-login-required"
          aria-modal="true"
          className="card fixed left-1/2 top-1/2 z-50 grid w-[min(92vw,360px)] -translate-x-1/2 -translate-y-1/2 gap-4 p-5 shadow-lg"
          role="dialog"
        >
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Sign in required
            </p>
            <h2 className="t-h3 mt-1" id="profile-login-required">
              Continue to interact with @{login}
            </h2>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <button
              className="btn sm"
              onClick={() => setLoginGateOpen(false)}
              type="button"
            >
              Cancel
            </button>
            <a className="btn primary sm" href={loginUrl()}>
              Sign in
            </a>
          </div>
        </div>
      ) : null}

      {blockDialogOpen ? (
        <div
          aria-labelledby="profile-block-title"
          aria-modal="true"
          className="card fixed left-1/2 top-1/2 z-50 grid w-[min(92vw,380px)] -translate-x-1/2 -translate-y-1/2 gap-4 p-5 shadow-lg"
          role="dialog"
        >
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Block profile
            </p>
            <h2 className="t-h3 mt-1" id="profile-block-title">
              Block @{login}?
            </h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              Blocking removes follow relationships and records a profile safety
              event.
            </p>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <button
              className="btn sm"
              onClick={() => setBlockDialogOpen(false)}
              type="button"
            >
              Cancel
            </button>
            <button
              className="btn accent sm"
              disabled={pending === "block"}
              onClick={blockProfile}
              type="button"
            >
              {pending === "block" ? "Blocking..." : "Block"}
            </button>
          </div>
        </div>
      ) : null}

      {reportDialogOpen ? (
        <form
          aria-labelledby="profile-report-title"
          aria-modal="true"
          className="card fixed left-1/2 top-1/2 z-50 grid w-[min(92vw,420px)] -translate-x-1/2 -translate-y-1/2 gap-4 p-5 shadow-lg"
          onSubmit={reportProfile}
          role="dialog"
        >
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Report profile
            </p>
            <h2 className="t-h3 mt-1" id="profile-report-title">
              Tell us what is wrong with @{login}
            </h2>
          </div>
          <label className="grid gap-2">
            <span className="t-label">Reason</span>
            <select
              className="input"
              onChange={(event) => setReportReason(event.target.value)}
              value={reportReason}
            >
              <option value="spam">Spam or misleading content</option>
              <option value="abuse">Harassment or abuse</option>
              <option value="impersonation">Impersonation</option>
            </select>
          </label>
          <label className="grid gap-2">
            <span className="t-label">Details</span>
            <textarea
              className="input min-h-24"
              onChange={(event) => setReportDetails(event.target.value)}
              placeholder="Add context for the review team"
              value={reportDetails}
            />
          </label>
          <div className="flex flex-wrap justify-end gap-2">
            <button
              className="btn sm"
              onClick={() => setReportDialogOpen(false)}
              type="button"
            >
              Cancel
            </button>
            <button
              className="btn primary sm"
              disabled={pending === "report"}
              type="submit"
            >
              {pending === "report" ? "Submitting..." : "Submit report"}
            </button>
          </div>
        </form>
      ) : null}
    </div>
  );
}
