"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  MilestoneSort,
  RepositoryMilestoneMutation,
  RepositoryMilestoneSummary,
  RepositoryMilestonesView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryIssuesHref,
  repositoryMilestoneHref,
  repositoryMilestonesHref,
} from "@/lib/navigation";

type RepositoryMilestonesPageProps = {
  repository: RepositoryOverview;
  milestones: RepositoryMilestonesView;
  query: {
    state?: string | null;
    sort?: string | null;
    q?: string | null;
  };
};

type EditingState =
  | { mode: "create"; milestone: null }
  | { mode: "edit"; milestone: RepositoryMilestoneSummary }
  | null;

const SORT_OPTIONS: { sort: MilestoneSort; label: string; hint: string }[] = [
  { sort: "updated-desc", label: "Recently updated", hint: "Updated" },
  { sort: "due-desc", label: "Furthest due date", hint: "Due" },
  { sort: "due-asc", label: "Closest due date", hint: "Due" },
  { sort: "complete-asc", label: "Least complete", hint: "%" },
  { sort: "complete-desc", label: "Most complete", hint: "%" },
  { sort: "alpha-asc", label: "Alphabetical", hint: "A-Z" },
  { sort: "alpha-desc", label: "Reverse alphabetical", hint: "Z-A" },
  { sort: "issues-desc", label: "Most issues", hint: "#" },
  { sort: "issues-asc", label: "Fewest issues", hint: "#" },
];

function errorMessage(error: unknown, fallback: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return (
    envelope?.error.message ??
    (error instanceof Error ? error.message : fallback)
  );
}

function formatDate(value: string | null) {
  if (!value) return "No due date";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(value));
}

function formatRelativeDate(value: string) {
  return `Updated ${formatDate(value)}`;
}

function MilestoneForm({
  milestone,
  mode,
  onCancel,
  onSaved,
  owner,
  repo,
}: {
  milestone: RepositoryMilestoneSummary | null;
  mode: "create" | "edit";
  onCancel: () => void;
  onSaved: () => void;
  owner: string;
  repo: string;
}) {
  const [title, setTitle] = useState(milestone?.title ?? "");
  const [description, setDescription] = useState(milestone?.description ?? "");
  const [dueOn, setDueOn] = useState(milestone?.dueOn?.slice(0, 10) ?? "");
  const [error, setError] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();

  function submit() {
    const request: RepositoryMilestoneMutation = {
      title: title.trim(),
      description: description.trim() || null,
      dueOn: dueOn || null,
    };
    setError(null);
    startTransition(async () => {
      try {
        const endpoint = milestone
          ? `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions/${encodeURIComponent(milestone.id)}`
          : `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions`;
        const response = await fetch(endpoint, {
          method: milestone ? "PATCH" : "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(request),
        });
        const body = await response.json().catch(() => null);
        if (!response.ok) {
          const envelope = body as ApiErrorEnvelope | null;
          throw new Error(
            envelope?.error.message ?? "Milestone could not be saved.",
            { cause: envelope },
          );
        }
        onSaved();
      } catch (saveError) {
        setError(errorMessage(saveError, "Milestone could not be saved."));
      }
    });
  }

  return (
    <section
      aria-label={
        mode === "create" ? "New milestone form" : `Edit ${milestone?.title}`
      }
      className="card p-4"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="t-h3">
            {mode === "create" ? "New milestone" : `Edit ${milestone?.title}`}
          </h2>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            Set a due date and description for repository planning.
          </p>
        </div>
        <span className="chip accent">
          {dueOn ? formatDate(dueOn) : "Open"}
        </span>
      </div>
      <div className="mt-4 grid gap-3 md:grid-cols-[1fr_180px]">
        <label className="grid gap-2 t-sm">
          <span className="t-label">Title</span>
          <input
            aria-label="Milestone title"
            className="input"
            onChange={(event) => setTitle(event.target.value)}
            value={title}
          />
        </label>
        <label className="grid gap-2 t-sm">
          <span className="t-label">Due date</span>
          <input
            aria-label="Milestone due date"
            className="input"
            onChange={(event) => setDueOn(event.target.value)}
            type="date"
            value={dueOn}
          />
        </label>
      </div>
      <label className="mt-3 grid gap-2 t-sm">
        <span className="t-label">Description</span>
        <textarea
          aria-label="Milestone description"
          className="input min-h-28"
          onChange={(event) => setDescription(event.target.value)}
          value={description}
        />
      </label>
      {error ? (
        <p className="chip err mt-3" role="alert">
          {error}
        </p>
      ) : null}
      <div className="mt-4 flex flex-wrap gap-2">
        <button
          aria-disabled={isPending}
          className="btn primary"
          disabled={isPending}
          onClick={submit}
          type="button"
        >
          {isPending ? "Saving..." : "Save milestone"}
        </button>
        <button className="btn" onClick={onCancel} type="button">
          Cancel
        </button>
      </div>
    </section>
  );
}

function SortMenu({
  owner,
  repo,
  sort,
  state,
  q,
}: {
  owner: string;
  repo: string;
  sort: string;
  state: string;
  q: string;
}) {
  const [open, setOpen] = useState(false);
  return (
    <div className="relative">
      <button
        aria-expanded={open}
        aria-haspopup="menu"
        className="btn"
        onClick={() => setOpen((value) => !value)}
        type="button"
      >
        Sort
      </button>
      {open ? (
        <div
          className="card absolute right-0 z-10 mt-2 w-72 p-2 shadow-md"
          role="menu"
        >
          {SORT_OPTIONS.map((option) => (
            <Link
              aria-checked={sort === option.sort}
              className="flex items-center gap-3 rounded-md px-3 py-2 t-sm hover:bg-[var(--surface-2)]"
              href={repositoryMilestonesHref(owner, repo, {
                state,
                sort: option.sort,
                q,
              })}
              key={option.sort}
              onClick={() => setOpen(false)}
              role="menuitemradio"
            >
              <span aria-hidden>{sort === option.sort ? "●" : "○"}</span>
              <span className="flex-1">{option.label}</span>
              <span className="kbd">{option.hint}</span>
            </Link>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function RepositoryMilestonesPage({
  milestones,
  query,
  repository,
}: RepositoryMilestonesPageProps) {
  const router = useRouter();
  const [editing, setEditing] = useState<EditingState>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const owner = repository.owner_login;
  const repo = repository.name;
  const state = milestones.filters.state || query.state || "open";
  const sort = milestones.filters.sort || query.sort || "updated-desc";
  const search = milestones.filters.q || query.q || "";
  const canWrite =
    milestones.viewer.canEditMilestones && !milestones.repository.isArchived;

  function refreshAfterMutation() {
    setEditing(null);
    router.refresh();
  }

  async function deleteMilestone(milestone: RepositoryMilestoneSummary) {
    if (!window.confirm(`Delete milestone ${milestone.title}?`)) {
      return;
    }
    setDeleteError(null);
    setDeletingId(milestone.id);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions/${encodeURIComponent(milestone.id)}`,
        { method: "DELETE" },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = body as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Milestone could not be deleted.",
          { cause: envelope },
        );
      }
      router.refresh();
    } catch (error) {
      setDeleteError(errorMessage(error, "Milestone could not be deleted."));
    } finally {
      setDeletingId(null);
    }
  }

  async function updateMilestoneState(
    milestone: RepositoryMilestoneSummary,
    action: "close" | "reopen",
  ) {
    setDeleteError(null);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/milestones/actions/${encodeURIComponent(milestone.id)}`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ action }),
        },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = body as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Milestone state could not be updated.",
          { cause: envelope },
        );
      }
      router.refresh();
    } catch (error) {
      setDeleteError(
        errorMessage(error, "Milestone state could not be updated."),
      );
    }
  }

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/issues`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <main className="space-y-5">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository planning
            </p>
            <h1 className="t-h1 mt-1">Milestones</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {milestones.items.length} of {milestones.total} milestones in this
              view.
            </p>
          </div>
          {canWrite ? (
            <button
              className="btn primary"
              onClick={() => setEditing({ mode: "create", milestone: null })}
              type="button"
            >
              New milestone
            </button>
          ) : null}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3">
          <nav aria-label="Milestone state tabs" className="tabs">
            {[
              ["open", "Open", milestones.openCount],
              ["closed", "Closed", milestones.closedCount],
            ].map(([tabState, label, count]) => (
              <Link
                className={`tab ${state === tabState ? "active" : ""}`}
                href={repositoryMilestonesHref(owner, repo, {
                  state: String(tabState),
                  sort,
                  q: search,
                })}
                key={tabState}
              >
                {label} <span className="t-num">{count}</span>
              </Link>
            ))}
          </nav>
          <SortMenu
            owner={owner}
            repo={repo}
            sort={sort}
            state={state}
            q={search}
          />
        </div>

        <form
          action={repositoryMilestonesHref(owner, repo)}
          className="card p-3"
          method="get"
        >
          <div className="flex flex-wrap gap-3">
            <input name="state" type="hidden" value={state} />
            <input name="sort" type="hidden" value={sort} />
            <label className="min-w-64 flex-1 t-sm">
              <span className="sr-only">Search milestones</span>
              <input
                aria-label="Search milestones"
                className="input w-full"
                defaultValue={search}
                name="q"
                placeholder="Search milestones"
                type="search"
              />
            </label>
            <button className="btn" type="submit">
              Search
            </button>
            {search ? (
              <Link
                className="btn"
                href={repositoryMilestonesHref(owner, repo, { state, sort })}
              >
                Clear
              </Link>
            ) : null}
          </div>
        </form>

        {editing ? (
          <MilestoneForm
            milestone={editing.milestone}
            mode={editing.mode}
            onCancel={() => setEditing(null)}
            onSaved={refreshAfterMutation}
            owner={owner}
            repo={repo}
          />
        ) : null}

        {deleteError ? (
          <p className="chip err" role="alert">
            {deleteError}
          </p>
        ) : null}

        <section
          aria-label="Repository milestones"
          className="card overflow-hidden"
        >
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3"
            style={{ borderColor: "var(--line)" }}
          >
            <span className="t-label">{milestones.total} milestones</span>
            <span className="t-xs">
              Sorted by{" "}
              {SORT_OPTIONS.find((option) => option.sort === sort)?.label ??
                "Recently updated"}
            </span>
          </div>
          {milestones.items.length ? (
            milestones.items.map((milestone) => {
              const progress = milestone.progress.percentComplete;
              const detailHref =
                milestone.href ||
                repositoryMilestoneHref(owner, repo, milestone.id);
              const openHref =
                milestone.openIssuesHref ||
                repositoryIssuesHref(owner, repo, {
                  q: `milestone:"${milestone.title}" state:open`,
                  state: "open",
                  milestone: milestone.title,
                });
              const closedHref =
                milestone.closedIssuesHref ||
                repositoryIssuesHref(owner, repo, {
                  q: `milestone:"${milestone.title}" state:closed`,
                  state: "closed",
                  milestone: milestone.title,
                });
              return (
                <article className="list-row px-5 py-4" key={milestone.id}>
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-start gap-3">
                      <Link
                        className="t-h3 min-w-0 break-words hover:underline"
                        href={detailHref}
                      >
                        {milestone.title}
                      </Link>
                      <span
                        className={
                          milestone.state === "closed" ? "chip ok" : "chip soft"
                        }
                      >
                        {milestone.state}
                      </span>
                    </div>
                    <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                      {milestone.description || "No description"}
                    </p>
                    <div className="mt-3 grid gap-2">
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
                        className="flex flex-wrap gap-x-4 gap-y-1 t-sm"
                        style={{ color: "var(--ink-3)" }}
                      >
                        <span className="t-num">{progress}% complete</span>
                        <span>{formatDate(milestone.dueOn)}</span>
                        <span>{formatRelativeDate(milestone.updatedAt)}</span>
                      </div>
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center justify-end gap-3 t-sm">
                    <Link className="hover:underline" href={openHref}>
                      <span className="t-num">
                        {milestone.progress.openCount}
                      </span>{" "}
                      open issues
                    </Link>
                    <Link className="hover:underline" href={closedHref}>
                      <span className="t-num">
                        {milestone.progress.closedCount}
                      </span>{" "}
                      closed issues
                    </Link>
                    {canWrite ? (
                      <span className="flex gap-2">
                        <button
                          className="btn sm"
                          onClick={() =>
                            setEditing({ mode: "edit", milestone })
                          }
                          type="button"
                        >
                          Edit
                        </button>
                        <button
                          aria-disabled={deletingId === milestone.id}
                          className="btn sm"
                          disabled={deletingId === milestone.id}
                          onClick={() => deleteMilestone(milestone)}
                          type="button"
                        >
                          Delete
                        </button>
                        {milestone.state === "open" ? (
                          <button
                            className="btn sm"
                            onClick={() =>
                              void updateMilestoneState(milestone, "close")
                            }
                            type="button"
                          >
                            Close
                          </button>
                        ) : (
                          <button
                            className="btn sm"
                            onClick={() =>
                              void updateMilestoneState(milestone, "reopen")
                            }
                            type="button"
                          >
                            Reopen
                          </button>
                        )}
                      </span>
                    ) : null}
                  </div>
                </article>
              );
            })
          ) : (
            <div className="p-8 text-center">
              <h2 className="t-h3">
                {state === "closed"
                  ? "No closed milestones"
                  : "No open milestones"}
              </h2>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                {canWrite
                  ? "Create a milestone to group issues and pull requests around a delivery target."
                  : "Milestones will appear here once maintainers create them."}
              </p>
              {canWrite ? (
                <button
                  className="btn primary mt-4"
                  onClick={() =>
                    setEditing({ mode: "create", milestone: null })
                  }
                  type="button"
                >
                  New milestone
                </button>
              ) : null}
            </div>
          )}
        </section>
      </main>
    </RepositoryShell>
  );
}
