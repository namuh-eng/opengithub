"use client";

import { useId, useState, useTransition } from "react";
import type { ProfileViewerState } from "@/lib/api";

type ProfileControlsProps = {
  login: string;
  viewer: ProfileViewerState;
  initialFollowerCount: number;
};

type Status = { tone: "idle" | "ok" | "err"; message: string };

async function mutateProfileAction<T>(
  login: string,
  action: "follow" | "block" | "report",
  method: "PUT" | "DELETE" | "POST",
  body?: unknown,
): Promise<T> {
  const response = await fetch(
    `/${encodeURIComponent(login)}/actions/${action}`,
    {
      method,
      headers: body ? { "content-type": "application/json" } : undefined,
      body: body ? JSON.stringify(body) : undefined,
    },
  );
  const payload = (await response.json().catch(() => null)) as
    | { error?: { message?: string } }
    | T
    | null;
  if (!response.ok) {
    throw new Error(
      (payload as { error?: { message?: string } } | null)?.error?.message ??
        "Profile action failed",
    );
  }
  return payload as T;
}

export function ProfileControls({
  login,
  viewer,
  initialFollowerCount,
}: ProfileControlsProps) {
  const reportDialogId = useId();
  const [following, setFollowing] = useState(viewer.following);
  const [blocked, setBlocked] = useState(viewer.blocked);
  const [followerCount, setFollowerCount] = useState(initialFollowerCount);
  const [reportOpen, setReportOpen] = useState(false);
  const [reason, setReason] = useState("spam");
  const [details, setDetails] = useState("");
  const [status, setStatus] = useState<Status>({ tone: "idle", message: "" });
  const [isPending, startTransition] = useTransition();

  const signedOutMessage = "Sign in to use profile relationship controls.";

  function toggleFollow() {
    if (!viewer.authenticated) {
      setStatus({ tone: "err", message: signedOutMessage });
      return;
    }
    const next = !following;
    const previous = following;
    const previousCount = followerCount;
    setFollowing(next);
    setFollowerCount(Math.max(0, followerCount + (next ? 1 : -1)));
    setStatus({ tone: "idle", message: "" });
    startTransition(async () => {
      try {
        const result = await mutateProfileAction<{
          following: boolean;
          followerCount: number;
        }>(login, "follow", next ? "PUT" : "DELETE");
        setFollowing(result.following);
        setFollowerCount(result.followerCount);
        setStatus({ tone: "ok", message: next ? "Following" : "Unfollowed" });
      } catch (error) {
        setFollowing(previous);
        setFollowerCount(previousCount);
        setStatus({
          tone: "err",
          message: error instanceof Error ? error.message : "Follow failed",
        });
      }
    });
  }

  function toggleBlock() {
    if (!viewer.authenticated) {
      setStatus({ tone: "err", message: signedOutMessage });
      return;
    }
    const next = !blocked;
    startTransition(async () => {
      try {
        const result = await mutateProfileAction<{ blocked: boolean }>(
          login,
          "block",
          next ? "PUT" : "DELETE",
        );
        setBlocked(result.blocked);
        if (result.blocked) {
          setFollowing(false);
        }
        setStatus({
          tone: "ok",
          message: result.blocked ? "User blocked" : "User unblocked",
        });
      } catch (error) {
        setStatus({
          tone: "err",
          message: error instanceof Error ? error.message : "Block failed",
        });
      }
    });
  }

  function submitReport() {
    if (!viewer.authenticated) {
      setStatus({ tone: "err", message: signedOutMessage });
      return;
    }
    startTransition(async () => {
      try {
        await mutateProfileAction<{ id: string; status: string }>(
          login,
          "report",
          "POST",
          {
            reason,
            details,
          },
        );
        setDetails("");
        setReportOpen(false);
        setStatus({ tone: "ok", message: "Report received for review" });
      } catch (error) {
        setStatus({
          tone: "err",
          message: error instanceof Error ? error.message : "Report failed",
        });
      }
    });
  }

  return (
    <div className="grid gap-3">
      <button
        className={`btn w-full ${following ? "ghost" : "primary"}`}
        disabled={isPending || !viewer.canFollow}
        onClick={toggleFollow}
        type="button"
      >
        {following ? "Following" : "Follow"}
      </button>
      <p className="t-sm t-num" style={{ color: "var(--ink-3)" }}>
        {followerCount.toLocaleString()} followers
      </p>
      <div className="grid grid-cols-2 gap-2">
        <button
          className="btn sm ghost"
          disabled={isPending || !viewer.canBlock}
          onClick={toggleBlock}
          type="button"
        >
          {blocked ? "Unblock" : "Block"}
        </button>
        <button
          aria-controls={reportDialogId}
          aria-expanded={reportOpen}
          className="btn sm ghost"
          disabled={isPending || !viewer.canReport}
          onClick={() => {
            if (!viewer.authenticated) {
              setStatus({ tone: "err", message: signedOutMessage });
              return;
            }
            setReportOpen((value) => !value);
          }}
          type="button"
        >
          Report
        </button>
      </div>
      {status.message ? (
        <p
          className={`chip ${status.tone === "err" ? "err" : "ok"}`}
          role="status"
        >
          {status.message}
        </p>
      ) : null}
      {reportOpen ? (
        <div className="card grid gap-3 p-3" id={reportDialogId} role="dialog">
          <label className="grid gap-1">
            <span className="t-label">Reason</span>
            <select
              className="input"
              onChange={(event) => setReason(event.target.value)}
              value={reason}
            >
              <option value="spam">Spam or automation abuse</option>
              <option value="harassment">Harassment or abusive behavior</option>
              <option value="impersonation">Impersonation</option>
              <option value="other">Other policy concern</option>
            </select>
          </label>
          <label className="grid gap-1">
            <span className="t-label">Details</span>
            <textarea
              className="input min-h-24"
              onChange={(event) => setDetails(event.target.value)}
              placeholder="Add concise context for reviewers"
              value={details}
            />
          </label>
          <button
            className="btn sm primary"
            disabled={isPending}
            onClick={submitReport}
            type="button"
          >
            Submit report
          </button>
        </div>
      ) : null}
    </div>
  );
}
