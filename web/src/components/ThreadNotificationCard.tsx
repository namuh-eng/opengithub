"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import type { ThreadSubscriptionEvent } from "@/lib/api";

export type ThreadSubscriptionState = {
  subscribed: boolean;
  reason: string;
  customEvents: ThreadSubscriptionEvent[];
  canCustomize: boolean;
};

type ThreadNotificationCardProps = {
  activePath: string;
  disabled: boolean;
  events?: ThreadSubscriptionEvent[];
  isMutating: boolean;
  subscription: ThreadSubscriptionState;
  viewerAuthenticated: boolean;
  onSave: (
    subscribed: boolean,
    customEvents: ThreadSubscriptionEvent[],
  ) => Promise<void>;
};

const THREAD_EVENTS: Array<{
  value: ThreadSubscriptionEvent;
  label: string;
  description: string;
}> = [
  {
    value: "closed",
    label: "Closed",
    description: "Notify when this thread is closed.",
  },
  {
    value: "reopened",
    label: "Reopened",
    description: "Notify when work resumes.",
  },
  {
    value: "merged",
    label: "Merged",
    description: "Notify when a pull request merges.",
  },
];

function reasonLabel(reason: string) {
  return reason.replaceAll("_", " ");
}

export function ThreadNotificationCard({
  activePath,
  disabled,
  events = ["closed", "reopened", "merged"],
  isMutating,
  subscription,
  viewerAuthenticated,
  onSave,
}: ThreadNotificationCardProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [draftSubscribed, setDraftSubscribed] = useState(
    subscription.subscribed,
  );
  const [draftEvents, setDraftEvents] = useState(subscription.customEvents);

  useEffect(() => {
    if (!dialogOpen) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setDialogOpen(false);
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [dialogOpen]);

  function openDialog() {
    setDraftSubscribed(subscription.subscribed);
    setDraftEvents(subscription.customEvents);
    setDialogOpen(true);
  }

  function toggleDraftEvent(event: ThreadSubscriptionEvent) {
    setDraftSubscribed(true);
    setDraftEvents((current) =>
      current.includes(event)
        ? current.filter((item) => item !== event)
        : [...current, event],
    );
  }

  async function saveDialog() {
    await onSave(draftSubscribed, draftEvents);
    setDialogOpen(false);
  }

  return (
    <section aria-labelledby="thread-notifications-title">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="t-h3" id="thread-notifications-title">
            Notifications
          </h3>
          <p className="t-xs mt-1">
            {subscription.subscribed
              ? `Subscribed: ${reasonLabel(subscription.reason)}`
              : "Not subscribed"}
          </p>
          {subscription.customEvents.length ? (
            <p className="t-xs mt-1">
              Custom events:{" "}
              {subscription.customEvents.map(reasonLabel).join(", ")}
            </p>
          ) : null}
        </div>
      </div>
      {viewerAuthenticated ? (
        <div className="mt-3 flex flex-wrap gap-2">
          <button
            className="btn sm"
            disabled={isMutating || disabled}
            onClick={() =>
              void onSave(!subscription.subscribed, subscription.customEvents)
            }
            type="button"
          >
            {subscription.subscribed ? "Unsubscribe" : "Subscribe"}
          </button>
          <button
            aria-haspopup="dialog"
            className="btn ghost sm"
            disabled={isMutating || disabled || !subscription.canCustomize}
            onClick={openDialog}
            type="button"
          >
            Customize
          </button>
        </div>
      ) : (
        <Link
          className="btn sm mt-3"
          href={`/login?next=${encodeURIComponent(activePath)}`}
        >
          Sign in to subscribe
        </Link>
      )}

      {dialogOpen ? (
        <div
          className="fixed inset-0 z-50 grid place-items-center bg-[color-mix(in_oklch,var(--ink-1)_42%,transparent)] p-4"
          role="dialog"
          aria-modal="true"
          aria-labelledby="thread-notifications-dialog-title"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              setDialogOpen(false);
            }
          }}
        >
          <div className="card w-full max-w-[420px] bg-[var(--surface)] p-5 shadow-lg">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="t-label">Thread notifications</p>
                <h4
                  className="t-h2 mt-1"
                  id="thread-notifications-dialog-title"
                >
                  Customize updates
                </h4>
              </div>
              <button
                aria-label="Close notification customization"
                className="btn ghost sm"
                onClick={() => setDialogOpen(false)}
                type="button"
              >
                Esc
              </button>
            </div>
            <label className="card mt-4 flex items-start gap-3 p-3">
              <input
                checked={draftSubscribed}
                className="mt-1"
                onChange={(event) => setDraftSubscribed(event.target.checked)}
                type="checkbox"
              />
              <span>
                <span className="t-sm block font-medium">
                  Subscribe to this thread
                </span>
                <span className="t-xs">
                  Thread settings override repository watch preferences.
                </span>
              </span>
            </label>
            <div className="mt-4 space-y-2">
              {THREAD_EVENTS.filter((event) =>
                events.includes(event.value),
              ).map((event) => (
                <label
                  className="card flex items-start gap-3 p-3"
                  key={event.value}
                >
                  <input
                    checked={draftEvents.includes(event.value)}
                    className="mt-1"
                    onChange={() => toggleDraftEvent(event.value)}
                    type="checkbox"
                  />
                  <span>
                    <span className="t-sm block font-medium">
                      {event.label}
                    </span>
                    <span className="t-xs">{event.description}</span>
                  </span>
                </label>
              ))}
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setDialogOpen(false)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn primary sm"
                disabled={isMutating || disabled}
                onClick={() => void saveDialog()}
                type="button"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
