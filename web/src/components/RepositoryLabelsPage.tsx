"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState, useTransition } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  RepositoryLabelMutationRequest,
  RepositoryLabelSummary,
  RepositoryLabelsView,
  RepositoryOverview,
} from "@/lib/api";
import {
  repositoryIssuesHref,
  repositoryLabelsHref,
  repositoryPullRequestsHref,
} from "@/lib/navigation";

type RepositoryLabelsPageProps = {
  repository: RepositoryOverview;
  labels: RepositoryLabelsView;
  query: {
    q?: string | null;
    sort?: string | null;
    direction?: string | null;
  };
};

type EditingState =
  | { mode: "create"; label: null }
  | { mode: "edit"; label: RepositoryLabelSummary }
  | null;

const RANDOM_COLORS = [
  "b85c38",
  "7f6a42",
  "3f7a5b",
  "5f6f9f",
  "9b5a74",
  "a16d2d",
];

function colorValue(value: string) {
  const normalized = value.replace(/^#/, "").trim();
  return /^[0-9a-fA-F]{6}$/.test(normalized) ? `#${normalized}` : "#b85c38";
}

function labelTextColor(hex: string) {
  const value = hex.replace(/^#/, "");
  const r = Number.parseInt(value.slice(0, 2), 16);
  const g = Number.parseInt(value.slice(2, 4), 16);
  const b = Number.parseInt(value.slice(4, 6), 16);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.62 ? "var(--ink-1)" : "var(--bg)";
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

function RepositoryLabelForm({
  label,
  mode,
  onCancel,
  onSaved,
  owner,
  repo,
}: {
  label: RepositoryLabelSummary | null;
  mode: "create" | "edit";
  onCancel: () => void;
  onSaved: (label: RepositoryLabelSummary, mode: "create" | "edit") => void;
  owner: string;
  repo: string;
}) {
  const [name, setName] = useState(label?.name ?? "");
  const [description, setDescription] = useState(label?.description ?? "");
  const [color, setColor] = useState(label?.color ?? "b85c38");
  const [error, setError] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();
  const previewColor = colorValue(color);

  function submit() {
    const request: RepositoryLabelMutationRequest = {
      name: name.trim(),
      description: description.trim() || null,
      color: color.replace(/^#/, "").trim(),
    };
    setError(null);
    startTransition(async () => {
      try {
        const endpoint = label
          ? `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels/actions/${encodeURIComponent(label.id)}`
          : `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels/actions`;
        const response = await fetch(endpoint, {
          method: label ? "PATCH" : "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(request),
        });
        const body = await response.json().catch(() => null);
        if (!response.ok) {
          const envelope = body as ApiErrorEnvelope | null;
          throw new Error(
            envelope?.error.message ?? "Label could not be saved.",
            {
              cause: envelope,
            },
          );
        }
        const result = body as { label?: RepositoryLabelSummary } | null;
        if (!result?.label) {
          throw new Error("Label response was missing label details.");
        }
        onSaved(result.label, mode);
      } catch (saveError) {
        setError(errorMessage(saveError, "Label could not be saved."));
      }
    });
  }

  return (
    <section
      aria-label={mode === "create" ? "New label form" : `Edit ${label?.name}`}
      className="card p-4"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="t-h3">
            {mode === "create" ? "New label" : `Edit ${label?.name}`}
          </h2>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            Labels organize issues, pull requests, and discussions with one
            shared repository vocabulary.
          </p>
        </div>
        <span
          className="chip"
          style={{
            background: previewColor,
            color: labelTextColor(previewColor),
            borderColor: "transparent",
          }}
        >
          {name.trim() || "Preview"}
        </span>
      </div>

      <div className="mt-4 grid gap-3 md:grid-cols-[1fr_1fr_180px]">
        <label className="grid gap-2 t-sm">
          <span className="t-label">Name</span>
          <input
            aria-label="Label name"
            className="input"
            onChange={(event) => setName(event.target.value)}
            value={name}
          />
        </label>
        <label className="grid gap-2 t-sm">
          <span className="t-label">Description</span>
          <input
            aria-label="Label description"
            className="input"
            onChange={(event) => setDescription(event.target.value)}
            value={description}
          />
        </label>
        <label className="grid gap-2 t-sm">
          <span className="t-label">Color</span>
          <span className="flex gap-2">
            <input
              aria-label="Label color"
              className="input"
              onChange={(event) => setColor(event.target.value)}
              value={color}
            />
            <button
              className="btn sm"
              onClick={() =>
                setColor(
                  RANDOM_COLORS[
                    Math.floor(Math.random() * RANDOM_COLORS.length)
                  ],
                )
              }
              type="button"
            >
              Random
            </button>
          </span>
        </label>
      </div>

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
          {isPending ? "Saving..." : "Save label"}
        </button>
        <button className="btn" onClick={onCancel} type="button">
          Cancel
        </button>
      </div>
    </section>
  );
}

function SortMenu({
  direction,
  owner,
  query,
  repo,
  sort,
}: {
  direction: string;
  owner: string;
  query: string | null | undefined;
  repo: string;
  sort: string;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const options = [
    { sort: "name", direction: "asc", label: "Name", hint: "A-Z" },
    {
      sort: "total_issue_count",
      direction: "desc",
      label: "Total issue count",
      hint: "#",
    },
    { sort: "name", direction: "desc", label: "Descending", hint: "Z-A" },
    {
      sort: "total_issue_count",
      direction: "asc",
      label: "Ascending",
      hint: "0-9",
    },
  ];

  useEffect(() => {
    if (!open) {
      return;
    }

    function closeOnOutsidePointer(event: PointerEvent) {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("pointerdown", closeOnOutsidePointer);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsidePointer);
    };
  }, [open]);

  return (
    <div className="relative" ref={containerRef}>
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
          className="card absolute right-0 z-10 mt-2 w-64 p-2 shadow-md"
          role="menu"
        >
          {options.map((option) => {
            const active =
              sort === option.sort && direction === option.direction;
            return (
              <Link
                aria-checked={active}
                className="flex items-center gap-3 rounded-md px-3 py-2 t-sm hover:bg-[var(--surface-2)]"
                href={repositoryLabelsHref(owner, repo, {
                  q: query,
                  sort: option.sort,
                  direction: option.direction,
                })}
                key={`${option.sort}:${option.direction}`}
                onClick={() => setOpen(false)}
                role="menuitemradio"
              >
                <span aria-hidden>{active ? "●" : "○"}</span>
                <span className="flex-1">{option.label}</span>
                <span className="kbd">{option.hint}</span>
              </Link>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}

export function RepositoryLabelsPage({
  labels,
  query,
  repository,
}: RepositoryLabelsPageProps) {
  const router = useRouter();
  const [editing, setEditing] = useState<EditingState>(null);
  const [localLabels, setLocalLabels] = useState(labels.items);
  const [localTotal, setLocalTotal] = useState(labels.total);
  const [status, setStatus] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const owner = repository.owner_login;
  const repo = repository.name;
  const canWrite = labels.viewer.canWrite && !labels.repository.isArchived;
  const currentQuery = query.q ?? labels.filters.query ?? "";
  const sortedBy = labels.filters.sort || query.sort || "name";
  const direction = labels.filters.direction || query.direction || "asc";

  useEffect(() => {
    setLocalLabels(labels.items);
    setLocalTotal(labels.total);
  }, [labels.items, labels.total]);

  function sortLabels(items: RepositoryLabelSummary[]) {
    return [...items].sort((left, right) => {
      const ordering =
        sortedBy === "total_issue_count"
          ? left.counts.totalIssueCount - right.counts.totalIssueCount ||
            left.name.localeCompare(right.name, undefined, {
              sensitivity: "base",
            })
          : left.name.localeCompare(right.name, undefined, {
              sensitivity: "base",
            });
      return direction === "desc" ? -ordering : ordering;
    });
  }

  function refreshAfterMutation(
    savedLabel: RepositoryLabelSummary,
    mode: "create" | "edit",
  ) {
    setEditing(null);
    setDeleteError(null);
    setStatus(mode === "create" ? "Label created." : "Label updated.");
    setLocalLabels((current) => {
      const withoutSaved = current.filter((item) => item.id !== savedLabel.id);
      return sortLabels([...withoutSaved, savedLabel]);
    });
    if (mode === "create") {
      setLocalTotal((value) => value + 1);
    }
    router.refresh();
  }

  async function deleteLabel(label: RepositoryLabelSummary) {
    if (!window.confirm(`Delete label ${label.name}?`)) {
      return;
    }
    setDeleteError(null);
    setStatus(null);
    setDeletingId(label.id);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/labels/actions/${encodeURIComponent(label.id)}`,
        { method: "DELETE" },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        const envelope = body as ApiErrorEnvelope | null;
        throw new Error(
          envelope?.error.message ?? "Label could not be deleted.",
          { cause: envelope },
        );
      }
      setLocalLabels((current) =>
        current.filter((item) => item.id !== label.id),
      );
      setLocalTotal((value) => Math.max(0, value - 1));
      setStatus("Label deleted.");
      router.refresh();
    } catch (error) {
      setDeleteError(errorMessage(error, "Label could not be deleted."));
    } finally {
      setDeletingId(null);
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
              Repository metadata
            </p>
            <h1 className="t-h1 mt-1">Labels</h1>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {localLabels.length} of {localTotal} labels match this view.
            </p>
          </div>
          {canWrite ? (
            <button
              className="btn primary"
              onClick={() => setEditing({ mode: "create", label: null })}
              type="button"
            >
              New label
            </button>
          ) : null}
        </div>

        <form
          action={repositoryLabelsHref(owner, repo)}
          className="flex flex-wrap gap-3"
        >
          <input name="sort" type="hidden" value={sortedBy} />
          <input name="direction" type="hidden" value={direction} />
          <label className="min-w-[260px] flex-1">
            <span className="sr-only">Search all labels</span>
            <input
              className="input w-full"
              defaultValue={currentQuery}
              name="q"
              placeholder="Search all labels"
              type="search"
            />
          </label>
          <button className="btn" type="submit">
            Search
          </button>
          <SortMenu
            direction={direction}
            owner={owner}
            query={currentQuery}
            repo={repo}
            sort={sortedBy}
          />
        </form>

        {editing ? (
          <RepositoryLabelForm
            label={editing.label}
            mode={editing.mode}
            onCancel={() => setEditing(null)}
            onSaved={refreshAfterMutation}
            owner={owner}
            repo={repo}
          />
        ) : null}

        {status ? (
          <p className="chip ok" role="status">
            {status}
          </p>
        ) : null}

        {deleteError ? (
          <p className="chip err" role="alert">
            {deleteError}
          </p>
        ) : null}

        <section
          aria-label="Repository labels"
          className="card overflow-hidden"
        >
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3"
            style={{ borderColor: "var(--line)" }}
          >
            <span className="t-label">{localTotal} labels</span>
            <span className="t-xs">
              Sorted by{" "}
              {sortedBy === "total_issue_count" ? "issue count" : "name"}{" "}
              {direction === "desc" ? "descending" : "ascending"}
            </span>
          </div>

          {localLabels.length ? (
            localLabels.map((label) => {
              const chipColor = colorValue(label.color);
              const issuesHref =
                label.issuesHref ||
                repositoryIssuesHref(owner, repo, {
                  q: `is:issue state:open label:"${label.name}"`,
                  state: "open",
                  labels: [label.name],
                });
              const pullsHref =
                label.pullRequestsHref ||
                repositoryPullRequestsHref(owner, repo, {
                  q: `is:pr state:open label:"${label.name}"`,
                  state: "open",
                  labels: [label.name],
                });
              return (
                <article className="list-row px-5 py-4" key={label.id}>
                  <Link
                    className="flex min-w-0 flex-1 items-start gap-3"
                    href={issuesHref}
                  >
                    <span
                      aria-hidden
                      className="mt-1 h-4 w-4 shrink-0 rounded-full border"
                      style={{
                        background: chipColor,
                        borderColor: "var(--line-strong)",
                      }}
                    />
                    <span className="min-w-0">
                      <span
                        className="chip inline-flex max-w-full whitespace-normal break-words"
                        style={{
                          background: chipColor,
                          color: labelTextColor(chipColor),
                          borderColor: "transparent",
                        }}
                      >
                        {label.name}
                      </span>
                      <span
                        className="t-sm mt-2 block"
                        style={{ color: "var(--ink-3)" }}
                      >
                        {label.description || "No description"}
                      </span>
                    </span>
                  </Link>
                  <div className="flex flex-wrap items-center justify-end gap-3 t-sm">
                    <Link className="hover:underline" href={issuesHref}>
                      <span className="t-num">{label.counts.openIssues}</span>{" "}
                      open issues
                    </Link>
                    <Link className="hover:underline" href={pullsHref}>
                      <span className="t-num">
                        {label.counts.openPullRequests}
                      </span>{" "}
                      open pull requests
                    </Link>
                    {canWrite ? (
                      <span className="flex gap-2">
                        <button
                          className="btn sm"
                          onClick={() => setEditing({ mode: "edit", label })}
                          type="button"
                        >
                          Edit
                        </button>
                        <button
                          aria-disabled={deletingId === label.id}
                          className="btn sm"
                          disabled={deletingId === label.id}
                          onClick={() => deleteLabel(label)}
                          type="button"
                        >
                          Delete
                        </button>
                      </span>
                    ) : null}
                  </div>
                </article>
              );
            })
          ) : (
            <div className="p-8 text-center">
              <h2 className="t-h3">No labels match this search</h2>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                Try a different label name or description.
              </p>
              {canWrite ? (
                <button
                  className="btn primary mt-4"
                  onClick={() => setEditing({ mode: "create", label: null })}
                  type="button"
                >
                  New label
                </button>
              ) : null}
            </div>
          )}
        </section>
      </main>
    </RepositoryShell>
  );
}
