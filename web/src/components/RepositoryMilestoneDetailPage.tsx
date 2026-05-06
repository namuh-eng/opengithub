"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  IssueState,
  MilestoneIssueItem,
  RepositoryMilestoneDetail,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryMilestoneHref,
  repositoryMilestonesHref,
} from "@/lib/navigation";

type RepositoryMilestoneDetailPageProps = {
  repository: RepositoryOverview;
  milestone: RepositoryMilestoneDetail;
  query: { state?: string | null };
};

function formatDate(value: string | null) {
  if (!value) return "No due date";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(value));
}

function relativeDate(value: string) {
  return `Updated ${formatDate(value)}`;
}

function itemKind(item: MilestoneIssueItem) {
  return item.isPullRequest ? "Pull request" : "Issue";
}

function stateChipClass(state: IssueState) {
  return state === "closed" ? "chip ok" : "chip soft";
}

function errorMessage(error: unknown, fallback: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return (
    envelope?.error.message ??
    (error instanceof Error ? error.message : fallback)
  );
}

export function RepositoryMilestoneDetailPage({
  milestone,
  query,
  repository,
}: RepositoryMilestoneDetailPageProps) {
  const router = useRouter();
  const owner = repository.owner_login;
  const repo = repository.name;
  const state = query.state === "closed" ? "closed" : "open";
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [orderedItems, setOrderedItems] = useState(milestone.items);
  const [message, setMessage] = useState<string | null>(null);
  const [isMutating, setIsMutating] = useState(false);
  const [isSavingOrder, setIsSavingOrder] = useState(false);
  const canWrite =
    milestone.viewer.canEditMilestones && !milestone.repository.isArchived;
  const visibleItems = useMemo(
    () => orderedItems.filter((item) => item.state === state),
    [orderedItems, state],
  );
  const progress = milestone.progress.percentComplete;
  const detailHref = repositoryMilestoneHref(owner, repo, milestone.id);
  const newIssueHref = `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/new?milestone=${encodeURIComponent(milestone.id)}`;

  async function mutate(action: "close" | "reopen" | "delete") {
    if (
      action === "delete" &&
      !window.confirm(`Delete milestone ${milestone.title}?`)
    ) {
      return;
    }
    setMessage(null);
    setIsMutating(true);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions/${encodeURIComponent(milestone.id)}`,
        action === "delete"
          ? { method: "DELETE" }
          : {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify({ action }),
            },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = body as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Milestone could not be updated.",
          { cause: envelope },
        );
      }
      if (action === "delete") {
        router.push(repositoryMilestonesHref(owner, repo));
        return;
      }
      router.refresh();
    } catch (error) {
      setMessage(errorMessage(error, "Milestone could not be updated."));
    } finally {
      setIsMutating(false);
    }
  }

  function toggleSelected(itemId: string) {
    setSelectedIds((current) =>
      current.includes(itemId)
        ? current.filter((id) => id !== itemId)
        : [...current, itemId],
    );
  }

  async function moveItem(itemId: string, direction: -1 | 1) {
    const openItems = orderedItems.filter((item) => item.state === "open");
    const currentIndex = openItems.findIndex((item) => item.id === itemId);
    const nextIndex = currentIndex + direction;
    if (
      !milestone.order.canReorder ||
      state !== "open" ||
      currentIndex < 0 ||
      nextIndex < 0 ||
      nextIndex >= openItems.length
    ) {
      return;
    }
    const reorderedOpen = [...openItems];
    const [moved] = reorderedOpen.splice(currentIndex, 1);
    reorderedOpen.splice(nextIndex, 0, moved);
    const nextItems = [
      ...reorderedOpen,
      ...orderedItems.filter((item) => item.state !== "open"),
    ];
    setOrderedItems(nextItems);
    setMessage(null);
    setIsSavingOrder(true);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions/${encodeURIComponent(milestone.id)}`,
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            itemIds: reorderedOpen.map((item) => item.id),
            expectedVersion: milestone.order.version,
          }),
        },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = body as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Milestone order could not be saved.",
          { cause: envelope },
        );
      }
      router.refresh();
    } catch (error) {
      setOrderedItems(milestone.items);
      setMessage(errorMessage(error, "Milestone order could not be saved."));
    } finally {
      setIsSavingOrder(false);
    }
  }

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/issues`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <main className="space-y-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <Link
              className="btn sm"
              href={repositoryMilestonesHref(owner, repo)}
            >
              Back to Milestones
            </Link>
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <span className={stateChipClass(milestone.state)}>
                {milestone.state}
              </span>
              <span className="chip soft">{formatDate(milestone.dueOn)}</span>
              <span className="t-xs">{relativeDate(milestone.updatedAt)}</span>
            </div>
            <h1 className="t-h1 mt-3 break-words">{milestone.title}</h1>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <Link className="btn primary" href={newIssueHref}>
              New issue
            </Link>
            {canWrite ? (
              <>
                {milestone.state === "open" ? (
                  <button
                    aria-disabled={isMutating}
                    className="btn"
                    disabled={isMutating}
                    onClick={() => void mutate("close")}
                    type="button"
                  >
                    Close
                  </button>
                ) : (
                  <button
                    aria-disabled={isMutating}
                    className="btn"
                    disabled={isMutating}
                    onClick={() => void mutate("reopen")}
                    type="button"
                  >
                    Reopen
                  </button>
                )}
                <button
                  aria-disabled={isMutating}
                  className="btn"
                  disabled={isMutating}
                  onClick={() => void mutate("delete")}
                  type="button"
                >
                  Delete
                </button>
              </>
            ) : null}
          </div>
        </div>

        {message ? (
          <p className="chip err" role="alert">
            {message}
          </p>
        ) : null}

        <section className="grid gap-4 lg:grid-cols-[1fr_320px]">
          <div className="card overflow-hidden">
            <div
              className="border-b p-5"
              style={{ borderColor: "var(--line)" }}
            >
              <div
                aria-label={`${progress}% complete`}
                className="h-2 overflow-hidden rounded-full"
                role="img"
                style={{ background: "var(--surface-3)" }}
              >
                <div
                  className="h-full rounded-full"
                  style={{
                    background: "var(--accent)",
                    width: `${Math.max(0, Math.min(progress, 100))}%`,
                  }}
                />
              </div>
              <div
                className="mt-3 flex flex-wrap gap-x-4 gap-y-1 t-sm"
                style={{ color: "var(--ink-3)" }}
              >
                <span className="t-num">{progress}% complete</span>
                <span>
                  <span className="t-num">{milestone.progress.openCount}</span>{" "}
                  open
                </span>
                <span>
                  <span className="t-num">
                    {milestone.progress.closedCount}
                  </span>{" "}
                  closed
                </span>
              </div>
            </div>
            <div className="p-5">
              {milestone.descriptionHtml ? (
                <MarkdownBody html={milestone.descriptionHtml} />
              ) : (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No description.
                </p>
              )}
            </div>
          </div>

          <aside className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Selection
            </p>
            <p className="t-h2 mt-2">
              <span className="t-num">{selectedIds.length}</span> selected
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Select rows to stage milestone metadata changes. Bulk actions are
              available to repository writers.
            </p>
            <p className="chip soft mt-4">
              {canWrite
                ? "Use each selected row link to edit metadata on its detail page."
                : "Read-only view"}
            </p>
          </aside>
        </section>

        <section className="card overflow-hidden">
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3"
            style={{ borderColor: "var(--line)" }}
          >
            <nav aria-label="Milestone item state tabs" className="tabs">
              {(["open", "closed"] as const).map((tab) => (
                <Link
                  className={`tab ${state === tab ? "active" : ""}`}
                  href={`${detailHref}?state=${tab}`}
                  key={tab}
                >
                  {tab === "open" ? "Open" : "Closed"}{" "}
                  <span className="t-num">
                    {tab === "open"
                      ? milestone.progress.openCount
                      : milestone.progress.closedCount}
                  </span>
                </Link>
              ))}
            </nav>
            <span className="t-xs">
              <span className="t-num">{visibleItems.length}</span> items
            </span>
          </div>
          {state === "open" && !milestone.order.canReorder ? (
            <p
              className="border-b px-5 py-3 t-sm"
              style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
            >
              {milestone.order.reason}
            </p>
          ) : null}

          {visibleItems.length ? (
            visibleItems.map((item, index) => (
              <article className="list-row px-5 py-4" key={item.id}>
                <label className="flex min-w-0 flex-1 items-start gap-3">
                  <input
                    aria-label={`Select ${itemKind(item)} ${item.number}`}
                    checked={selectedIds.includes(item.id)}
                    className="mt-1"
                    onChange={() => toggleSelected(item.id)}
                    type="checkbox"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="flex flex-wrap items-center gap-2">
                      <Link
                        className="t-h3 break-words hover:underline"
                        href={item.href}
                      >
                        {item.title}
                      </Link>
                      <span className={stateChipClass(item.state)}>
                        {itemKind(item)}
                      </span>
                    </span>
                    <span
                      className="mt-2 flex flex-wrap gap-x-3 gap-y-1 t-xs"
                      style={{ color: "var(--ink-3)" }}
                    >
                      <span className="t-num">#{item.number}</span>
                      <span>{item.commentCount} comments</span>
                      <span>{relativeDate(item.updatedAt)}</span>
                    </span>
                    <span className="mt-2 flex flex-wrap gap-2">
                      {item.labelNames.map((label) => (
                        <span className="chip soft" key={label}>
                          {label}
                        </span>
                      ))}
                      {item.assigneeLogins.map((login) => (
                        <span className="chip soft" key={login}>
                          @{login}
                        </span>
                      ))}
                    </span>
                  </span>
                </label>
                {state === "open" && milestone.order.canReorder ? (
                  <div className="ml-3 flex shrink-0 gap-2">
                    <button
                      aria-disabled={isSavingOrder || index === 0}
                      aria-label={`Move ${item.title} up`}
                      className="btn sm"
                      disabled={isSavingOrder || index === 0}
                      onClick={() => void moveItem(item.id, -1)}
                      type="button"
                    >
                      Up
                    </button>
                    <button
                      aria-disabled={
                        isSavingOrder || index === visibleItems.length - 1
                      }
                      aria-label={`Move ${item.title} down`}
                      className="btn sm"
                      disabled={
                        isSavingOrder || index === visibleItems.length - 1
                      }
                      onClick={() => void moveItem(item.id, 1)}
                      type="button"
                    >
                      Down
                    </button>
                  </div>
                ) : null}
              </article>
            ))
          ) : (
            <div className="p-8 text-center">
              <h2 className="t-h3">No {state} items</h2>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                {state === "open"
                  ? "Open issues and pull requests assigned to this milestone will appear here."
                  : "Closed milestone work will appear here after items are completed."}
              </p>
              <Link className="btn primary mt-4" href={newIssueHref}>
                New issue
              </Link>
            </div>
          )}
        </section>
      </main>
    </RepositoryShell>
  );
}
