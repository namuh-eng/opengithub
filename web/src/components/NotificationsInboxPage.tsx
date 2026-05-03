"use client";

import { useState } from "react";
import type {
  ApiErrorEnvelope,
  NotificationInboxRow,
  NotificationInboxView,
  NotificationTriageAction,
  NotificationTriageResponse,
} from "@/lib/api";

type NotificationsInboxPageProps = {
  view: NotificationInboxView | ApiErrorEnvelope;
};

function isError(
  view: NotificationInboxView | ApiErrorEnvelope,
): view is ApiErrorEnvelope {
  return "error" in view;
}

function applyOptimisticTriage(
  view: NotificationInboxView,
  notificationId: string,
  action: NotificationTriageAction,
): NotificationInboxView {
  const unreadDelta = action === "read" ? -1 : action === "unread" ? 1 : 0;
  const savedDelta = action === "save" ? 1 : action === "unsave" ? -1 : 0;
  const removeFromCurrentFolder =
    (action === "done" && view.query.folder === "inbox") ||
    (action === "inbox" && view.query.folder === "done");

  return mapNotificationRows(
    view,
    notificationId,
    (row) => ({
      ...row,
      unread:
        action === "read" ? false : action === "unread" ? true : row.unread,
      saved: action === "save" ? true : action === "unsave" ? false : row.saved,
      done: action === "done" ? true : action === "inbox" ? false : row.done,
    }),
    {
      unreadDelta,
      savedDelta,
      removeFromCurrentFolder,
    },
  );
}

function applyConfirmedTriage(
  view: NotificationInboxView,
  response: NotificationTriageResponse,
): NotificationInboxView {
  return mapNotificationRows(
    view,
    response.id,
    (row) => ({
      ...row,
      unread: response.unread,
      saved: response.saved,
      done: response.done,
    }),
    {
      unreadCount: response.unreadCount,
      folderCounts: response.folderCounts,
    },
  );
}

function mapNotificationRows(
  view: NotificationInboxView,
  notificationId: string,
  update: (row: NotificationInboxRow) => NotificationInboxRow,
  counts: {
    unreadDelta?: number;
    savedDelta?: number;
    unreadCount?: number;
    folderCounts?: NotificationTriageResponse["folderCounts"];
    removeFromCurrentFolder?: boolean;
  } = {},
): NotificationInboxView {
  const groups = view.groups
    .map((group) => {
      const rows = group.rows
        .map((row) => (row.id === notificationId ? update(row) : row))
        .filter(
          (row) =>
            !(counts.removeFromCurrentFolder && row.id === notificationId),
        );
      return {
        ...group,
        count: rows.length,
        rows,
      };
    })
    .filter((group) => group.rows.length > 0);
  const folderCounts = counts.folderCounts;
  const nextTotal = counts.removeFromCurrentFolder
    ? Math.max(0, view.total - 1)
    : view.total;
  return {
    ...view,
    total: nextTotal,
    unreadCount:
      counts.unreadCount ??
      Math.max(0, view.unreadCount + (counts.unreadDelta ?? 0)),
    folders: view.folders.map((folder) => {
      if (folderCounts) {
        const count = folderCounts[folder.id as keyof typeof folderCounts];
        return typeof count === "number" ? { ...folder, count } : folder;
      }
      if (folder.id === "saved") {
        return {
          ...folder,
          count: Math.max(0, folder.count + (counts.savedDelta ?? 0)),
        };
      }
      return folder;
    }),
    groups,
  };
}

async function patchNotificationTriage(
  notificationId: string,
  action: NotificationTriageAction,
): Promise<NotificationTriageResponse> {
  const response = await fetch(
    `/notifications/${encodeURIComponent(notificationId)}/triage`,
    {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ action }),
    },
  );
  if (!response.ok) {
    throw new Error("Notification action failed");
  }
  return (await response.json()) as NotificationTriageResponse;
}

function notificationActionLabel(
  action: NotificationTriageAction,
  response: NotificationTriageResponse,
) {
  if (action === "done") {
    return "Notification moved to Done.";
  }
  if (action === "inbox") {
    return "Notification moved to Inbox.";
  }
  if (action === "save" || (action === "unsave" && response.saved)) {
    return "Notification saved.";
  }
  if (action === "unsave") {
    return "Notification unsaved.";
  }
  if (response.unread) {
    return "Notification marked unread.";
  }
  return "Notification marked read.";
}

function withQuery(overrides: Record<string, string | null | undefined>) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(overrides)) {
    if (value?.trim()) {
      params.set(key, value.trim());
    }
  }
  const suffix = params.toString();
  return suffix ? `/notifications?${suffix}` : "/notifications";
}

export function NotificationsInboxPage({
  view: initialView,
}: NotificationsInboxPageProps) {
  const [currentView, setCurrentView] = useState(initialView);
  const [pendingId, setPendingId] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const visibleView = currentView;

  async function runAction(
    row: NotificationInboxRow,
    action: NotificationTriageAction,
  ) {
    if (isError(visibleView)) {
      return;
    }
    const previous = visibleView;
    setPendingId(`${row.id}:${action}`);
    setToast(null);
    setCurrentView(applyOptimisticTriage(previous, row.id, action));

    try {
      const response = await patchNotificationTriage(row.id, action);
      setCurrentView((latest) =>
        isError(latest) ? latest : applyConfirmedTriage(latest, response),
      );
      setToast(notificationActionLabel(action, response));
    } catch {
      setCurrentView(previous);
      setToast("Notification action failed. Your inbox was restored.");
    } finally {
      setPendingId(null);
    }
  }

  if (isError(visibleView)) {
    return (
      <section className="card p-8" aria-labelledby="notifications-error-title">
        <p className="t-label" style={{ color: "var(--err)" }}>
          Notifications unavailable
        </p>
        <h1 className="t-h2 mt-2" id="notifications-error-title">
          {visibleView.error.message}
        </h1>
        <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
          Try refreshing the inbox after the API connection recovers.
        </p>
      </section>
    );
  }

  const view: NotificationInboxView = visibleView;
  const { query } = view;
  const allHref = withQuery({
    folder: query.folder === "inbox" ? null : query.folder,
    q: query.q,
    sort: query.sort === "newest" ? null : query.sort,
    group: query.group === "date" ? null : query.group,
    repo: query.repo ?? null,
  });
  const unreadHref = withQuery({
    folder: query.folder === "inbox" ? null : query.folder,
    tab: "unread",
    q: query.q,
    sort: query.sort === "newest" ? null : query.sort,
    group: query.group === "date" ? null : query.group,
    repo: query.repo ?? null,
  });

  return (
    <section aria-labelledby="notifications-title">
      <div className="mb-6 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Inbox
          </p>
          <h1 className="t-h1 mt-1" id="notifications-title">
            {view.total} {query.tab === "unread" ? "unread" : "notifications"}
          </h1>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Triage repository activity, mentions, review requests, and delivery
            states.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span className="chip accent">{view.unreadCount} unread</span>
          <a className="btn sm" href="/settings/notifications">
            Manage notifications
          </a>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
        <aside className="space-y-5" aria-label="Notification navigation">
          <FacetSection title="Folders" facets={view.folders} />
          <FacetSection title="Default filters" facets={view.filters} />
          <div className="card p-4">
            <div className="mb-3 flex items-center justify-between gap-3">
              <h2 className="t-label">Repositories</h2>
              <a
                className="t-xs"
                href="/settings/notifications"
                style={{ color: "var(--ink-3)" }}
              >
                Manage
              </a>
            </div>
            {view.repositories.length ? (
              <nav
                className="space-y-1"
                aria-label="Repository notification buckets"
              >
                {view.repositories.map((repo) => (
                  <a
                    className={`flex items-center justify-between gap-3 rounded-[var(--radius)] px-2 py-2 t-sm ${
                      repo.active ? "chip active" : ""
                    }`}
                    href={repo.href}
                    key={repo.id}
                  >
                    <span className="truncate t-mono-sm">{repo.label}</span>
                    <span className="t-num" style={{ color: "var(--ink-3)" }}>
                      {repo.count}
                    </span>
                  </a>
                ))}
              </nav>
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Repository buckets appear once notifications are delivered.
              </p>
            )}
          </div>
        </aside>

        <div className="min-w-0 space-y-4">
          <div className="card p-4">
            <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
              <nav className="tabs" aria-label="Notification read state">
                <a
                  className={`tab ${query.tab === "all" ? "active" : ""}`}
                  href={allHref}
                >
                  All
                </a>
                <a
                  className={`tab ${query.tab === "unread" ? "active" : ""}`}
                  href={unreadHref}
                >
                  Unread
                </a>
              </nav>
              <form
                action="/notifications"
                className="flex min-w-0 flex-1 flex-wrap items-center gap-2"
              >
                <input
                  name="folder"
                  type="hidden"
                  value={query.folder === "inbox" ? "" : query.folder}
                />
                <input
                  name="tab"
                  type="hidden"
                  value={query.tab === "all" ? "" : query.tab}
                />
                <input
                  name="sort"
                  type="hidden"
                  value={query.sort === "newest" ? "" : query.sort}
                />
                <input
                  name="group"
                  type="hidden"
                  value={query.group === "date" ? "" : query.group}
                />
                <input name="repo" type="hidden" value={query.repo ?? ""} />
                <label className="sr-only" htmlFor="notifications-query">
                  Search notifications
                </label>
                <input
                  className="input min-w-[260px] flex-1"
                  type="search"
                  defaultValue={query.q}
                  id="notifications-query"
                  name="q"
                  placeholder="is:unread repo:mona/octo-app reason:mention"
                />
                <button className="btn sm primary" type="submit">
                  Search
                </button>
              </form>
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <ChoiceGroup label="Sort by" options={view.sortOptions} />
              <ChoiceGroup label="Group by" options={view.groupOptions} />
              <span className="chip soft">
                Cleanup: unread + saved stay visible until done
              </span>
            </div>
          </div>

          {toast ? (
            <div className="chip soft" role="status" aria-live="polite">
              {toast}
            </div>
          ) : null}

          {view.groups.length ? (
            view.groups.map((group) => (
              <section
                aria-labelledby={`notification-group-${group.id}`}
                key={group.id}
              >
                <div className="mb-2 flex items-center gap-2 px-1">
                  <h2 className="t-label" id={`notification-group-${group.id}`}>
                    {group.label}
                  </h2>
                  <span className="t-num t-xs">{group.count}</span>
                </div>
                <div className="card overflow-hidden">
                  {group.rows.map((row) => (
                    <article className="list-row items-start" key={row.id}>
                      <span
                        role="img"
                        aria-label={row.unread ? "Unread" : "Read"}
                        className={
                          row.unread
                            ? "dot live mt-2"
                            : "mt-2 inline-block h-[6px] w-[6px]"
                        }
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          {row.repositoryHref ? (
                            <a
                              className="t-mono-sm"
                              href={row.repositoryHref}
                              style={{ color: "var(--ink-3)" }}
                            >
                              {row.repositoryName}
                            </a>
                          ) : (
                            <span
                              className="t-mono-sm"
                              style={{ color: "var(--ink-3)" }}
                            >
                              {row.repositoryName}
                            </span>
                          )}
                          <span className="chip soft">{row.reasonLabel}</span>
                          <span className="t-xs">{row.relativeTime}</span>
                        </div>
                        <a
                          className="mt-1 block t-body"
                          href={row.openHref}
                          style={{
                            color: "var(--ink-1)",
                            fontWeight: row.unread ? 600 : 400,
                          }}
                        >
                          {row.title}
                          {row.subjectNumber ? (
                            <span
                              className="t-mono-sm"
                              style={{ color: "var(--ink-4)" }}
                            >
                              {" #"}
                              {row.subjectNumber}
                            </span>
                          ) : null}
                        </a>
                      </div>
                      <div className="flex flex-wrap justify-end gap-1 text-right">
                        <button
                          aria-label={
                            row.unread
                              ? `Mark ${row.title} as read`
                              : `Mark ${row.title} as unread`
                          }
                          className="btn sm ghost"
                          disabled={
                            pendingId ===
                            `${row.id}:${row.unread ? "read" : "unread"}`
                          }
                          onClick={() =>
                            runAction(row, row.unread ? "read" : "unread")
                          }
                          title={row.unread ? "Mark read" : "Mark unread"}
                          type="button"
                        >
                          {row.unread ? "Read" : "Unread"}
                        </button>
                        <button
                          aria-label={
                            row.saved
                              ? `Unsave ${row.title}`
                              : `Save ${row.title}`
                          }
                          className={
                            row.saved ? "btn sm primary" : "btn sm ghost"
                          }
                          disabled={
                            pendingId ===
                            `${row.id}:${row.saved ? "unsave" : "save"}`
                          }
                          onClick={() =>
                            runAction(row, row.saved ? "unsave" : "save")
                          }
                          title={row.saved ? "Unsave" : "Save"}
                          type="button"
                        >
                          {row.saved ? "Saved" : "Save"}
                        </button>
                        <button
                          aria-label={
                            row.done
                              ? `Move ${row.title} to inbox`
                              : `Move ${row.title} to Done`
                          }
                          className={
                            row.done ? "btn sm primary" : "btn sm ghost"
                          }
                          disabled={
                            pendingId ===
                            `${row.id}:${row.done ? "inbox" : "done"}`
                          }
                          onClick={() =>
                            runAction(row, row.done ? "inbox" : "done")
                          }
                          title={row.done ? "Move to inbox" : "Done"}
                          type="button"
                        >
                          {row.done ? "Move to inbox" : "Done"}
                        </button>
                        <span
                          className={row.subscribed ? "chip info" : "chip soft"}
                        >
                          {row.subscribed ? "Subscribed" : "Unsubscribed"}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              </section>
            ))
          ) : (
            <div className="card p-10 text-center">
              <p className="t-display text-[28px]">{view.emptyTitle}</p>
              <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
                {view.emptyMessage}
              </p>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}

function FacetSection({
  title,
  facets,
}: {
  title: string;
  facets: NotificationInboxView["folders"];
}) {
  return (
    <div className="card p-4">
      <h2 className="t-label mb-3">{title}</h2>
      <nav className="space-y-1" aria-label={title}>
        {facets.map((facet) => (
          <a
            className={`flex items-center justify-between gap-3 rounded-[var(--radius)] px-2 py-2 t-sm ${
              facet.active ? "chip active" : ""
            }`}
            href={facet.href}
            key={facet.id}
          >
            <span>{facet.label}</span>
            <span className="t-num" style={{ color: "var(--ink-3)" }}>
              {facet.count}
            </span>
          </a>
        ))}
      </nav>
      {title === "Default filters" ? (
        <a className="btn sm mt-3 w-full" href="/settings/notifications">
          Add new filter
        </a>
      ) : null}
    </div>
  );
}

function ChoiceGroup({
  label,
  options,
}: {
  label: string;
  options: NotificationInboxView["sortOptions"];
}) {
  return (
    <fieldset className="flex flex-wrap items-center gap-1">
      <legend className="t-xs mr-1">{label}</legend>
      {options.map((option) => (
        <a
          className={`chip ${option.active ? "active" : "soft"}`}
          href={option.href}
          key={option.id}
        >
          {option.label}
        </a>
      ))}
    </fieldset>
  );
}
