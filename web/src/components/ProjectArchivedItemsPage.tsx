"use client";

import Link from "next/link";
import { useState } from "react";
import type { ProjectArchivedItem, ProjectWorkspace } from "@/lib/api";
import { projectItemHref } from "@/lib/navigation";

type ProjectArchivedItemsPageProps = {
  workspace: ProjectWorkspace;
  scope: "user" | "organization";
  owner: string;
  viewNumber: number;
  initialItems: ProjectArchivedItem[];
  total: number;
};

export function ProjectArchivedItemsPage({
  workspace,
  scope,
  owner,
  viewNumber,
  initialItems,
  total,
}: ProjectArchivedItemsPageProps) {
  const [items, setItems] = useState(initialItems);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [savingId, setSavingId] = useState<string | null>(null);

  async function restoreItem(item: ProjectArchivedItem) {
    setSavingId(item.item.id);
    setMessage(null);
    setError(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/${encodeURIComponent(item.item.id)}/restore`,
      { method: "PATCH" },
    ).catch(() => null);
    setSavingId(null);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setError(body?.error?.message ?? "Project item could not be restored.");
      return;
    }
    setItems((current) =>
      current.filter((archived) => archived.item.id !== item.item.id),
    );
    setMessage("Item restored");
  }

  const workspaceHref = workspace.selectedView.href;
  return (
    <main className="mx-auto w-full max-w-[980px] px-5 py-6 md:px-8">
      <div className="mb-5 flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="t-label mb-2">Project archive</div>
          <h1 className="t-h2">{workspace.project.title}</h1>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Archived items are hidden from active table, board, and roadmap
            views until restored.
          </p>
        </div>
        <Link className="btn sm" href={workspaceHref}>
          Back to project
        </Link>
      </div>

      <div className="mb-4 flex flex-wrap items-center gap-2">
        <span className="chip soft">
          <span className="t-num">{items.length}</span> shown
        </span>
        <span className="chip soft">
          <span className="t-num">{total}</span> total
        </span>
        {message ? <span className="chip ok">{message}</span> : null}
        {error ? <span className="chip err">{error}</span> : null}
      </div>

      <section className="card overflow-hidden">
        {items.length === 0 ? (
          <div className="p-6">
            <div className="t-label mb-2">No archived items</div>
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              Restored items return to the active project workspace.
            </p>
          </div>
        ) : (
          items.map((archived) => (
            <article className="list-row p-4" key={archived.item.id}>
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="chip soft">{archived.item.itemType}</span>
                  <Link
                    className="t-sm font-medium no-underline"
                    href={projectItemHref(
                      scope,
                      owner,
                      workspace.project.number,
                      archived.item.id,
                      { view: viewNumber },
                    )}
                  >
                    {archived.item.title}
                  </Link>
                </div>
                <p className="t-xs mt-1">
                  Archived {formatArchiveDate(archived.archivedAt)}
                  {archived.archivedBy
                    ? ` by ${archived.archivedBy.login}`
                    : ""}
                  {archived.source
                    ? ` from ${archived.source.repository.fullName}`
                    : ""}
                </p>
              </div>
              <button
                className="btn sm"
                disabled={
                  savingId === archived.item.id ||
                  !archived.viewerPermissions.canRestore
                }
                onClick={() => restoreItem(archived)}
                type="button"
              >
                {savingId === archived.item.id ? "Restoring..." : "Restore"}
              </button>
            </article>
          ))
        )}
      </section>
    </main>
  );
}

function formatArchiveDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(new Date(value));
}
