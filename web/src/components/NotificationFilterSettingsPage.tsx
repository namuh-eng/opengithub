"use client";

import { useMemo, useState } from "react";
import type {
  NotificationCustomFilter,
  NotificationFilterSettings,
} from "@/lib/api";

type NotificationFilterSettingsPageProps = {
  initialSettings: NotificationFilterSettings;
};

type Draft = {
  id?: string;
  name: string;
  queryString: string;
};

const EMPTY_DRAFT: Draft = { name: "", queryString: "" };

export function NotificationFilterSettingsPage({
  initialSettings,
}: NotificationFilterSettingsPageProps) {
  const [settings, setSettings] = useState(initialSettings);
  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT);
  const [deleteTarget, setDeleteTarget] =
    useState<NotificationCustomFilter | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const validation = useMemo(
    () => validateDraft(draft, settings.customFilters, settings.limit),
    [draft, settings.customFilters, settings.limit],
  );
  const canSave = !validation && !saving;
  const isEditing = Boolean(draft.id);

  function updateDraft(patch: Partial<Draft>) {
    setDraft((current) => ({ ...current, ...patch }));
    setError(null);
    setToast(null);
  }

  function editFilter(filter: NotificationCustomFilter) {
    setDraft({
      id: filter.id,
      name: filter.name,
      queryString: filter.queryString,
    });
    setError(null);
    setToast(null);
  }

  function resetDraft() {
    setDraft(EMPTY_DRAFT);
    setError(null);
  }

  async function saveFilter() {
    if (!canSave) return;
    setSaving(true);
    try {
      const response = await fetch("/settings/notifications/actions", {
        method: isEditing ? "PATCH" : "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(draft),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Filter could not be saved.");
      }
      setSettings(body as NotificationFilterSettings);
      setDraft(EMPTY_DRAFT);
      setToast(isEditing ? "Filter updated." : "Filter created.");
    } catch (saveError) {
      setError(
        saveError instanceof Error
          ? saveError.message
          : "Filter could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }

  async function deleteFilter() {
    if (!deleteTarget) return;
    setSaving(true);
    try {
      const response = await fetch("/settings/notifications/actions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: deleteTarget.id }),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Filter could not be deleted.");
      }
      setSettings(body as NotificationFilterSettings);
      setDeleteTarget(null);
      if (draft.id === deleteTarget.id) {
        setDraft(EMPTY_DRAFT);
      }
      setToast("Filter deleted.");
    } catch (deleteError) {
      setError(
        deleteError instanceof Error
          ? deleteError.message
          : "Filter could not be deleted.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="grid gap-6">
      {toast ? (
        <div className="chip ok w-fit" role="status">
          {toast}
        </div>
      ) : null}

      <section className="card p-5" aria-labelledby="filters-heading">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <p className="t-label">Inbox rules</p>
            <h3 className="t-h2 mt-2" id="filters-heading">
              Filters
            </h3>
            <p
              className="t-body mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Build saved inbox queries from supported qualifiers. Custom
              filters appear in the notification rail immediately after saving.
            </p>
          </div>
          <span className="chip soft w-fit">
            {settings.customFilters.length}/{settings.limit} custom
          </span>
        </div>

        <div className="mt-5 overflow-x-auto">
          <table className="w-full min-w-[620px] border-collapse text-left">
            <thead>
              <tr className="t-label" style={{ color: "var(--ink-3)" }}>
                <th className="pb-3 font-normal">Name</th>
                <th className="pb-3 font-normal">Query</th>
                <th className="pb-3 text-right font-normal">Actions</th>
              </tr>
            </thead>
            <tbody>
              {settings.defaultFilters.map((filter) => (
                <tr
                  key={filter.id}
                  style={{ borderTop: "1px solid var(--line)" }}
                >
                  <td className="py-3 pr-4">
                    <div className="font-medium">{filter.name}</div>
                    <span className="chip soft mt-1">Default</span>
                  </td>
                  <td className="py-3 pr-4">
                    <code className="t-mono-sm">{filter.queryString}</code>
                  </td>
                  <td className="py-3 text-right">
                    <a className="btn sm" href={filter.href}>
                      Open
                    </a>
                  </td>
                </tr>
              ))}
              {settings.customFilters.map((filter) => (
                <tr
                  key={filter.id}
                  style={{ borderTop: "1px solid var(--line)" }}
                >
                  <td className="py-3 pr-4">
                    <div className="font-medium">{filter.name}</div>
                    <span className="t-xs">Position {filter.position}</span>
                  </td>
                  <td className="py-3 pr-4">
                    <code className="t-mono-sm">{filter.queryString}</code>
                  </td>
                  <td className="py-3">
                    <div className="flex justify-end gap-2">
                      <a className="btn sm" href={filter.href}>
                        Open
                      </a>
                      <button
                        className="btn sm"
                        onClick={() => editFilter(filter)}
                        type="button"
                      >
                        Edit
                      </button>
                      <button
                        className="btn sm"
                        onClick={() => setDeleteTarget(filter)}
                        type="button"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="card p-5" aria-labelledby="custom-filter-heading">
        <p className="t-label">
          {isEditing ? "Edit custom row" : "Custom row"}
        </p>
        <h3 className="t-h2 mt-2" id="custom-filter-heading">
          {isEditing ? "Update filter" : "Create filter"}
        </h3>
        <div className="mt-4 grid gap-4 md:grid-cols-[220px_minmax(0,1fr)]">
          <label className="grid gap-2">
            <span className="t-sm font-medium">Name</span>
            <input
              className="input"
              onChange={(event) => updateDraft({ name: event.target.value })}
              placeholder="My review queue"
              value={draft.name}
            />
          </label>
          <label className="grid gap-2">
            <span className="t-sm font-medium">Query</span>
            <input
              className="input t-mono-sm"
              onChange={(event) =>
                updateDraft({ queryString: event.target.value })
              }
              placeholder="repo:mona/octo-app reason:review_requested"
              value={draft.queryString}
            />
          </label>
        </div>
        <div className="mt-3 flex flex-wrap gap-2">
          {settings.allowedQualifiers.map((qualifier) => (
            <span className="chip soft" key={qualifier}>
              {qualifier}:
            </span>
          ))}
        </div>
        <p className="t-xs mt-3">
          Full-text searches and exclusion queries are rejected for custom
          notification filters.
        </p>
        {validation || error ? (
          <p className="t-sm mt-3" style={{ color: "var(--err)" }}>
            {error ?? validation}
          </p>
        ) : null}
        <div className="mt-5 flex flex-wrap gap-2">
          <button
            className="btn primary"
            disabled={!canSave}
            onClick={saveFilter}
            type="button"
          >
            {saving ? "Saving..." : isEditing ? "Save changes" : "Create"}
          </button>
          {isEditing ? (
            <button className="btn" onClick={resetDraft} type="button">
              Cancel
            </button>
          ) : null}
        </div>
      </section>

      {deleteTarget ? (
        <div
          aria-labelledby="delete-filter-heading"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center bg-[color-mix(in_oklch,var(--ink-1)_42%,transparent)] p-4"
          role="dialog"
        >
          <div className="card w-full max-w-md bg-[var(--surface)] p-5">
            <p className="t-label">Delete filter</p>
            <h3 className="t-h2 mt-2" id="delete-filter-heading">
              Remove {deleteTarget.name}?
            </h3>
            <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
              The filter disappears from the notifications rail. Matching
              notifications are not deleted.
            </p>
            <div className="mt-5 flex flex-wrap gap-2">
              <button
                className="btn primary"
                disabled={saving}
                onClick={deleteFilter}
                type="button"
              >
                Delete
              </button>
              <button
                className="btn"
                disabled={saving}
                onClick={() => setDeleteTarget(null)}
                type="button"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function validateDraft(
  draft: Draft,
  filters: NotificationCustomFilter[],
  limit: number,
) {
  const name = draft.name.trim();
  const query = draft.queryString.trim();
  if (!name || !query) {
    return "Name and query are required.";
  }
  if (!draft.id && filters.length >= limit) {
    return "You can create up to 15 custom notification filters.";
  }
  const duplicate = filters.some(
    (filter) =>
      filter.id !== draft.id &&
      filter.name.toLowerCase() === name.toLowerCase(),
  );
  if (duplicate) {
    return "A custom notification filter with that name already exists.";
  }
  for (const token of query.split(/\s+/)) {
    if (token.toUpperCase() === "NOT" || token.startsWith("-")) {
      return "Custom filters do not support NOT or exclusion searches.";
    }
    const [qualifier, value] = token.split(":", 2);
    if (!value) {
      return "Use repo:, org:, author:, is:, or reason: qualifiers.";
    }
    if (!["repo", "org", "author", "is", "reason"].includes(qualifier)) {
      return "Use repo:, org:, author:, is:, or reason: qualifiers.";
    }
  }
  return null;
}
