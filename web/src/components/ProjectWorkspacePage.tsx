"use client";

import Link from "next/link";
import { type FormEvent, Fragment, useMemo, useState } from "react";
import type {
  ProjectWorkspace,
  ProjectWorkspaceBoardColumn,
  ProjectWorkspaceField,
  ProjectWorkspaceFieldValue,
  ProjectWorkspaceItem,
} from "@/lib/api";
import {
  organizationProjectWorkspaceHref,
  projectItemHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectWorkspacePageProps = {
  workspace: ProjectWorkspace;
  scope: "user" | "organization";
  owner: string;
  viewNumber: number;
};

type EditingCell = {
  itemId: string;
  fieldId: string;
};

const SORT_OPTIONS = [
  { value: "manual", label: "Manual order" },
  { value: "title_asc", label: "Title A-Z" },
  { value: "title_desc", label: "Title Z-A" },
  { value: "updated_desc", label: "Recently updated" },
  { value: "updated_asc", label: "Least recently updated" },
];

function workspaceHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  viewNumber: number | string,
  query: Parameters<typeof userProjectWorkspaceHref>[3] = {},
) {
  return scope === "organization"
    ? organizationProjectWorkspaceHref(owner, projectNumber, viewNumber, query)
    : userProjectWorkspaceHref(owner, projectNumber, viewNumber, query);
}

function fieldValue(
  item: ProjectWorkspaceItem,
  field: ProjectWorkspaceField,
): ProjectWorkspaceFieldValue | null {
  return item.fieldValues.find((value) => value.fieldId === field.id) ?? null;
}

function itemIcon(item: ProjectWorkspaceItem) {
  if (item.itemType === "pull_request") return "PR";
  if (item.itemType === "issue") return "#";
  return "D";
}

function itemTypeLabel(item: ProjectWorkspaceItem) {
  if (item.itemType === "pull_request") return "Pull request";
  if (item.itemType === "issue") return "Issue";
  return "Draft";
}

function groupItems(workspace: ProjectWorkspace) {
  if (!workspace.filters.group) {
    return [
      {
        label: "All items",
        count: workspace.items.length,
        items: workspace.items,
      },
    ];
  }
  const groupField = workspace.fields.find(
    (field) =>
      field.name === workspace.filters.group ||
      field.id === workspace.filters.group,
  );
  return workspace.groups.map((group) => ({
    label: group.label,
    count: group.count,
    items: workspace.items.filter((item) => {
      if (!groupField) return true;
      return (
        (fieldValue(item, groupField)?.displayValue || "No value") ===
        group.label
      );
    }),
  }));
}

function boardColumnValue(column: ProjectWorkspaceBoardColumn) {
  return column.key === "no-value" ? "" : column.key;
}

function boardColumnItems(
  items: ProjectWorkspaceItem[],
  field: ProjectWorkspaceField | undefined,
  column: ProjectWorkspaceBoardColumn,
) {
  if (!field) return [];
  return items.filter((item) => {
    const value = fieldValue(item, field);
    const displayValue = value?.displayValue || "No value";
    return displayValue === column.label || displayValue === column.key;
  });
}

function boardSwimlaneGroups(
  items: ProjectWorkspaceItem[],
  field: ProjectWorkspaceField | undefined,
) {
  if (!field) {
    return [{ key: "all", label: "All cards", items }];
  }
  const groups = new Map<string, ProjectWorkspaceItem[]>();
  for (const item of items) {
    const label = fieldValue(item, field)?.displayValue || "No value";
    groups.set(label, [...(groups.get(label) ?? []), item]);
  }
  return Array.from(groups, ([label, groupItems]) => ({
    key: label,
    label,
    items: groupItems,
  }));
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(new Date(value));
}

function inputTypeForField(field: ProjectWorkspaceField) {
  if (field.fieldType === "date") return "date";
  if (field.fieldType === "number") return "number";
  return "text";
}

function editableFieldValue(value: ProjectWorkspaceFieldValue | null) {
  if (Array.isArray(value?.value)) return value.value.join(", ");
  if (typeof value?.value === "string" || typeof value?.value === "number") {
    return String(value.value);
  }
  return value?.displayValue ?? "";
}

function requestValueForField(field: ProjectWorkspaceField, raw: string) {
  if (field.fieldType === "number") return raw.trim() ? Number(raw) : 0;
  if (field.fieldType === "labels" || field.fieldType === "assignees") {
    return raw
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
  }
  return raw;
}

export function ProjectWorkspacePage({
  workspace,
  scope,
  owner,
  viewNumber,
}: ProjectWorkspacePageProps) {
  const [query, setQuery] = useState(workspace.filters.query ?? "");
  const [configOpen, setConfigOpen] = useState(false);
  const [configQuery, setConfigQuery] = useState(workspace.filters.query ?? "");
  const [configSort, setConfigSort] = useState(workspace.filters.sort);
  const [configGroup, setConfigGroup] = useState(workspace.filters.group ?? "");
  const [configSlice, setConfigSlice] = useState(workspace.filters.slice ?? "");
  const [hiddenFieldIds, setHiddenFieldIds] = useState(
    workspace.fields.filter((field) => field.hidden).map((field) => field.id),
  );
  const [saving, setSaving] = useState(false);
  const [layoutSaving, setLayoutSaving] = useState<string | null>(null);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [editingCell, setEditingCell] = useState<EditingCell | null>(null);
  const [editValue, setEditValue] = useState("");
  const [fieldSaving, setFieldSaving] = useState(false);
  const [fieldMessage, setFieldMessage] = useState<string | null>(null);
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [addMode, setAddMode] = useState<"url" | "draft" | "bulk">("url");
  const [addUrl, setAddUrl] = useState("");
  const [bulkUrls, setBulkUrls] = useState("");
  const [draftTitle, setDraftTitle] = useState("");
  const [draftBody, setDraftBody] = useState("");
  const [itemSaving, setItemSaving] = useState(false);
  const [itemMessage, setItemMessage] = useState<string | null>(null);
  const [itemError, setItemError] = useState<string | null>(null);
  const [emptyColumnsVisible, setEmptyColumnsVisible] = useState(
    workspace.boardConfig?.emptyColumnsVisible ?? true,
  );
  const visibleFields = workspace.fields.filter((field) => !field.hidden);
  const boardColumnField = workspace.boardConfig?.columnField
    ? workspace.fields.find(
        (field) => field.id === workspace.boardConfig?.columnField?.id,
      )
    : undefined;
  const boardSwimlaneField = workspace.boardConfig?.swimlaneField
    ? workspace.fields.find(
        (field) => field.id === workspace.boardConfig?.swimlaneField?.id,
      )
    : undefined;
  const boardColumns = (workspace.boardConfig?.columns ?? []).filter(
    (column) => column.visible && (emptyColumnsVisible || column.count > 0),
  );
  const boardGroups = boardSwimlaneGroups(workspace.items, boardSwimlaneField);
  const groupedItems = useMemo(() => groupItems(workspace), [workspace]);
  const baseQuery = {
    q: workspace.filters.query,
    sort: workspace.filters.sort,
    group: workspace.filters.group,
    slice: workspace.filters.slice,
  };
  const currentHref = workspaceHref(
    scope,
    owner,
    workspace.project.number,
    viewNumber,
    baseQuery,
  );
  const activeLayoutChoice = workspace.layoutChoices?.find(
    (choice) => choice.active,
  );

  function submitFilter(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const href = workspaceHref(
      scope,
      owner,
      workspace.project.number,
      viewNumber,
      {
        ...baseQuery,
        q: query,
        page: 1,
      },
    );
    window.location.assign(href);
  }

  async function saveViewState(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setSaveError(null);
    setSaveMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/views/${encodeURIComponent(workspace.selectedView.id)}/state`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          query: configQuery.trim() || null,
          sort: configSort,
          group: configGroup || null,
          slice: configSlice || null,
          hiddenFieldIds,
          expectedUpdatedAt: workspace.selectedView.updatedAt,
        }),
      },
    ).catch(() => null);
    setSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setSaveError(
        body?.error?.message ?? "Project view state could not be saved.",
      );
      return;
    }
    setSaveMessage("View saved");
    window.location.assign(
      workspaceHref(scope, owner, workspace.project.number, viewNumber, {}),
    );
  }

  function revertViewState() {
    setQuery("");
    setConfigQuery("");
    setConfigSort("manual");
    setConfigGroup("");
    setConfigSlice("");
    setHiddenFieldIds(
      Array.isArray(workspace.selectedView.configuration.hiddenFieldIds)
        ? workspace.selectedView.configuration.hiddenFieldIds.filter(
            (value): value is string => typeof value === "string",
          )
        : [],
    );
    window.location.assign(
      workspaceHref(scope, owner, workspace.project.number, viewNumber, {}),
    );
  }

  async function saveProjectLayout(layout: "table" | "board" | "roadmap") {
    const choice = workspace.layoutChoices?.find(
      (entry) => entry.layout === layout,
    );
    if (!choice?.enabled || layoutSaving) return;
    setLayoutSaving(layout);
    setSaveError(null);
    setSaveMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/views/${encodeURIComponent(workspace.selectedView.id)}/layout`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          layout,
          columnFieldId:
            layout === "board"
              ? (workspace.boardConfig?.columnField?.id ??
                workspace.boardConfig?.eligibleColumnFields[0]?.id ??
                null)
              : null,
          swimlaneFieldId:
            layout === "board"
              ? (workspace.boardConfig?.swimlaneField?.id ?? null)
              : null,
          startFieldId:
            layout === "roadmap"
              ? (workspace.roadmapConfig?.startDateField?.id ??
                workspace.roadmapConfig?.eligibleDateFields[0]?.id ??
                null)
              : null,
          targetFieldId:
            layout === "roadmap"
              ? (workspace.roadmapConfig?.targetDateField?.id ??
                workspace.roadmapConfig?.eligibleDateFields[0]?.id ??
                null)
              : null,
          expectedUpdatedAt: workspace.selectedView.updatedAt,
        }),
      },
    ).catch(() => null);
    setLayoutSaving(null);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setSaveError(
        body?.error?.message ?? "Project view layout could not be saved.",
      );
      return;
    }
    setSaveMessage(`${choice.label} layout saved`);
    window.location.assign(currentHref);
  }

  function openFieldEditor(
    item: ProjectWorkspaceItem,
    field: ProjectWorkspaceField,
    value: ProjectWorkspaceFieldValue | null,
  ) {
    setEditingCell({ itemId: item.id, fieldId: field.id });
    setEditValue(editableFieldValue(value));
    setFieldMessage(null);
    setFieldError(null);
  }

  async function saveFieldValue(
    event: FormEvent<HTMLFormElement>,
    item: ProjectWorkspaceItem,
    field: ProjectWorkspaceField,
  ) {
    event.preventDefault();
    setFieldSaving(true);
    setFieldError(null);
    setFieldMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/${encodeURIComponent(item.id)}/fields/${encodeURIComponent(field.id)}`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          value: requestValueForField(field, editValue),
          expectedUpdatedAt: item.updatedAt,
        }),
      },
    ).catch(() => null);
    setFieldSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setFieldError(
        body?.error?.message ?? "Project field could not be saved.",
      );
      return;
    }
    setFieldMessage(`${field.name} saved`);
    setEditingCell(null);
    window.location.assign(currentHref);
  }

  async function submitAddItem(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const url = addUrl.trim();
    const isPull = /\/pull\//.test(url);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          itemType: isPull ? "pull_request" : "issue",
          url,
          positionAfterItemId: workspace.items.at(-1)?.id ?? null,
        }),
      },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(body?.error?.message ?? "Project item could not be added.");
      return;
    }
    setItemMessage("Item added");
    window.location.assign(currentHref);
  }

  async function submitDraftItem(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          itemType: "draft_issue",
          title: draftTitle.trim(),
          body: draftBody.trim() || null,
          positionAfterItemId: workspace.items.at(-1)?.id ?? null,
        }),
      },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(body?.error?.message ?? "Draft issue could not be created.");
      return;
    }
    setItemMessage("Draft issue created");
    window.location.assign(currentHref);
  }

  async function submitBulkItems(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const items = bulkUrls
      .split(/\s+/)
      .map((url) => url.trim())
      .filter(Boolean)
      .map((url) => ({
        itemType: /\/pull\//.test(url) ? "pull_request" : "issue",
        url,
      }));
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/bulk`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ items }),
      },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(body?.error?.message ?? "Project items could not be added.");
      return;
    }
    setItemMessage("Items added");
    window.location.assign(currentHref);
  }

  async function moveItem(
    item: ProjectWorkspaceItem,
    direction: "up" | "down",
  ) {
    const currentIndex = workspace.items.findIndex(
      (entry) => entry.id === item.id,
    );
    const targetIndex =
      direction === "up" ? currentIndex - 1 : currentIndex + 1;
    const target = workspace.items[targetIndex];
    if (currentIndex < 0 || !target) return;
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/${encodeURIComponent(item.id)}/position`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          beforeItemId: direction === "up" ? target.id : null,
          afterItemId: direction === "down" ? target.id : null,
          expectedUpdatedAt: item.updatedAt,
        }),
      },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(
        body?.error?.message ?? "Project item position could not be saved.",
      );
      return;
    }
    setItemMessage("Row order saved");
    window.location.assign(currentHref);
  }

  async function moveItemToBoardColumn(
    item: ProjectWorkspaceItem,
    column: ProjectWorkspaceBoardColumn,
  ) {
    if (!boardColumnField) return;
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/${encodeURIComponent(item.id)}/position`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          beforeItemId: null,
          afterItemId: null,
          groupFieldId: column.fieldId,
          groupValue: boardColumnValue(column),
          expectedUpdatedAt: item.updatedAt,
        }),
      },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(
        body?.error?.message ?? "Project board card could not be moved.",
      );
      return;
    }
    setItemMessage(`Moved to ${column.label}`);
    window.location.assign(currentHref);
  }

  async function removeItem(item: ProjectWorkspaceItem) {
    setItemSaving(true);
    setItemError(null);
    setItemMessage(null);
    const response = await fetch(
      `/api/projects/${encodeURIComponent(workspace.project.id)}/items/${encodeURIComponent(item.id)}`,
      { method: "DELETE" },
    ).catch(() => null);
    setItemSaving(false);
    if (!response?.ok) {
      const body = await response?.json().catch(() => null);
      setItemError(
        body?.error?.message ?? "Project item could not be removed.",
      );
      return;
    }
    setItemMessage("Item removed");
    window.location.assign(currentHref);
  }

  return (
    <main className="mx-auto w-full max-w-[1240px] px-5 py-6 md:px-8">
      <div className="mb-5 flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="t-label mb-2 flex flex-wrap items-center gap-2">
            <Link
              className="no-underline"
              href={scope === "organization" ? `/orgs/${owner}` : `/${owner}`}
            >
              {owner}
            </Link>
            <span>/</span>
            <Link
              className="no-underline"
              href={
                scope === "organization"
                  ? `/orgs/${owner}/projects`
                  : `/${owner}?tab=projects`
              }
            >
              Projects
            </Link>
            <span>/</span>
            <span className="t-mono-sm">#{workspace.project.number}</span>
          </div>
          <h1 className="t-h1">{workspace.project.title}</h1>
          {workspace.project.description ? (
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {workspace.project.description}
            </p>
          ) : null}
        </div>
        <div className="flex flex-wrap gap-2">
          <Link className="btn sm primary" href={currentHref}>
            View
          </Link>
          <button
            className="btn sm"
            disabled
            title="Insights charts are scheduled after the table workspace slice."
            type="button"
          >
            Insights
          </button>
          <button
            className="btn sm"
            disabled
            title="Project settings are outside this feature phase."
            type="button"
          >
            Settings
          </button>
        </div>
      </div>

      <nav className="tabs mb-4" aria-label="Saved project views">
        {workspace.views.map((view) => (
          <Link
            aria-current={
              view.id === workspace.selectedView.id ? "page" : undefined
            }
            className={`tab ${view.id === workspace.selectedView.id ? "active" : ""}`}
            href={workspaceHref(
              scope,
              owner,
              workspace.project.number,
              view.number,
              baseQuery,
            )}
            key={view.id}
          >
            {view.name}
          </Link>
        ))}
        <button
          className="tab"
          disabled
          title="New view persistence is implemented in the next phase."
          type="button"
        >
          + View
        </button>
      </nav>

      <div className="grid gap-4 lg:grid-cols-[180px_minmax(0,1fr)]">
        <aside className="card h-fit p-3">
          <div className="t-label mb-3">Slices</div>
          <Link
            className={`chip mb-2 no-underline ${workspace.filters.slice ? "soft" : "active"}`}
            href={workspaceHref(
              scope,
              owner,
              workspace.project.number,
              viewNumber,
              {
                ...baseQuery,
                slice: null,
                page: 1,
              },
            )}
          >
            All items <span className="t-num">{workspace.total}</span>
          </Link>
          <div className="flex flex-col gap-2">
            {workspace.slices.map((slice) => (
              <Link
                className={`chip no-underline ${workspace.filters.slice === slice.key ? "active" : "soft"}`}
                href={workspaceHref(
                  scope,
                  owner,
                  workspace.project.number,
                  viewNumber,
                  {
                    ...baseQuery,
                    slice: slice.key,
                    page: 1,
                  },
                )}
                key={slice.key}
              >
                {slice.label} <span className="t-num">{slice.count}</span>
              </Link>
            ))}
          </div>
        </aside>

        <section className="min-w-0">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <form
              className="input min-w-[260px] flex-1"
              onSubmit={submitFilter}
            >
              <input
                aria-label="Filter project items"
                name="q"
                onChange={(event) => setQuery(event.target.value)}
                placeholder="is:open assignee:@me label:backend"
                value={query}
              />
              <button className="btn sm ghost" type="submit">
                Filter
              </button>
            </form>
            <label className="input h-9 max-w-[210px]">
              <span className="sr-only">Sort project items</span>
              <select
                aria-label="Sort project items"
                defaultValue={workspace.filters.sort}
                onChange={(event) => {
                  window.location.assign(
                    workspaceHref(
                      scope,
                      owner,
                      workspace.project.number,
                      viewNumber,
                      {
                        ...baseQuery,
                        sort: event.target.value,
                        page: 1,
                      },
                    ),
                  );
                }}
              >
                {SORT_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="btn sm"
              onClick={() => setConfigOpen((open) => !open)}
              title={
                workspace.viewerPermissions.canChangeLayout
                  ? "Open saved view layout and configuration controls"
                  : "You can inspect layout choices, but write access is required to save changes."
              }
              type="button"
            >
              View menu
            </button>
          </div>

          {configOpen ? (
            <section aria-label="View menu" className="card mb-3 p-4">
              <div className="mb-3 flex flex-wrap items-start justify-between gap-3">
                <div>
                  <h2 className="t-h3">View menu</h2>
                  <p className="t-xs mt-1">
                    Save layout, filters, sorting, grouping, slicing, and
                    visible fields for this project view.
                  </p>
                </div>
                <span className="chip active">
                  {activeLayoutChoice?.label ?? workspace.selectedView.layout}
                </span>
              </div>
              <div className="mb-4 grid gap-2 md:grid-cols-3">
                {(workspace.layoutChoices ?? []).map((choice) => (
                  <button
                    className={`chip justify-between ${choice.active ? "active" : "soft"}`}
                    disabled={
                      !choice.enabled ||
                      !workspace.viewerPermissions.canChangeLayout ||
                      layoutSaving != null
                    }
                    key={choice.layout}
                    onClick={() =>
                      saveProjectLayout(
                        choice.layout as "table" | "board" | "roadmap",
                      )
                    }
                    title={
                      choice.unavailableReason ??
                      `Switch to ${choice.label} layout`
                    }
                    type="button"
                  >
                    <span>{choice.label}</span>
                    <span className="kbd">{choice.keyboardHint}</span>
                  </button>
                ))}
              </div>
              <div className="mb-4 grid gap-2 md:grid-cols-2 xl:grid-cols-3">
                {[
                  [
                    "Fields",
                    `${visibleFields.length} visible`,
                    "Manage table columns and card metadata.",
                  ],
                  [
                    "Column by",
                    workspace.boardConfig?.columnField?.name ?? "Not set",
                    workspace.boardConfig?.unavailableReason ??
                      "Board columns use a status or single-select field.",
                  ],
                  [
                    "Swimlanes",
                    workspace.boardConfig?.swimlaneField?.name ?? "None",
                    "Group board cards across horizontal lanes.",
                  ],
                  [
                    "Sort by",
                    SORT_OPTIONS.find(
                      (option) => option.value === workspace.filters.sort,
                    )?.label ?? workspace.filters.sort,
                    "Preserves the URL-backed sort state.",
                  ],
                  [
                    "Field sum",
                    "Scheduled",
                    "Numeric summaries are implemented with board rendering.",
                  ],
                  [
                    "Slice by",
                    workspace.filters.slice ?? "All items",
                    "Use slices to keep focused worksets visible.",
                  ],
                ].map(([label, value, description]) => (
                  <div
                    className="p-3"
                    key={label}
                    style={{
                      border: "1px solid var(--line-soft)",
                      borderRadius: "var(--radius)",
                    }}
                  >
                    <div className="t-label mb-1">{label}</div>
                    <div className="t-sm">{value}</div>
                    <div className="t-xs mt-1">{description}</div>
                  </div>
                ))}
              </div>
              <form aria-label="View configuration" onSubmit={saveViewState}>
                <div className="mb-3 flex flex-wrap items-start justify-between gap-3">
                  <h3 className="t-h3">Table state</h3>
                  {workspace.unsavedView.active ? (
                    <span className="chip warn">
                      Unsaved: {workspace.unsavedView.reasons.join(", ")}
                    </span>
                  ) : null}
                </div>
                <div className="grid gap-3 md:grid-cols-2">
                  <label className="t-sm">
                    Filter query
                    <input
                      className="input mt-1 w-full"
                      onChange={(event) => setConfigQuery(event.target.value)}
                      placeholder="is:open label:frontend"
                      value={configQuery}
                    />
                  </label>
                  <label className="t-sm">
                    Sort
                    <select
                      className="input mt-1 w-full"
                      onChange={(event) => setConfigSort(event.target.value)}
                      value={configSort}
                    >
                      {SORT_OPTIONS.map((option) => (
                        <option key={option.value} value={option.value}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="t-sm">
                    Group by
                    <select
                      className="input mt-1 w-full"
                      onChange={(event) => setConfigGroup(event.target.value)}
                      value={configGroup}
                    >
                      <option value="">No grouping</option>
                      {workspace.fields.map((field) => (
                        <option key={field.id} value={field.name}>
                          {field.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="t-sm">
                    Slice by
                    <select
                      className="input mt-1 w-full"
                      onChange={(event) => setConfigSlice(event.target.value)}
                      value={configSlice}
                    >
                      <option value="">All items</option>
                      {workspace.fields.map((field) => (
                        <option key={field.id} value={field.name}>
                          {field.name}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>
                <fieldset className="mt-4">
                  <legend className="t-label mb-2">Visible fields</legend>
                  <div className="flex flex-wrap gap-2">
                    {workspace.fields.map((field) => {
                      const checked = !hiddenFieldIds.includes(field.id);
                      return (
                        <label
                          className="chip soft cursor-pointer"
                          key={field.id}
                        >
                          <input
                            checked={checked}
                            className="mr-2"
                            onChange={(event) => {
                              setHiddenFieldIds((current) =>
                                event.target.checked
                                  ? current.filter((id) => id !== field.id)
                                  : [...current, field.id],
                              );
                            }}
                            type="checkbox"
                          />
                          {field.name}
                        </label>
                      );
                    })}
                  </div>
                </fieldset>
                {saveError ? (
                  <p className="chip err mt-3">{saveError}</p>
                ) : null}
                {saveMessage ? (
                  <p className="chip ok mt-3">{saveMessage}</p>
                ) : null}
                <div className="mt-4 flex flex-wrap gap-2">
                  <button
                    className="btn sm primary"
                    disabled={saving}
                    type="submit"
                  >
                    {saving ? "Saving..." : "Save view"}
                  </button>
                  <button
                    className="btn sm"
                    onClick={revertViewState}
                    type="button"
                  >
                    Revert
                  </button>
                </div>
              </form>
            </section>
          ) : null}

          <div className="mb-3 flex flex-wrap items-center gap-2">
            <span className="chip soft">
              <span className="t-num">{workspace.total}</span> matching items
            </span>
            {workspace.filters.tokens.map((token) => (
              <Link
                className="chip soft no-underline"
                href={workspaceHref(
                  scope,
                  owner,
                  workspace.project.number,
                  viewNumber,
                  {
                    ...baseQuery,
                    q: workspace.filters.tokens
                      .filter((item) => item !== token)
                      .join(" "),
                    page: 1,
                  },
                )}
                key={token}
              >
                {token} x
              </Link>
            ))}
            {workspace.unsavedView.active ? (
              <span className="chip warn">Unsaved view</span>
            ) : null}
          </div>

          {workspace.selectedView.layout === "board" ? (
            <div className="card overflow-hidden">
              <div
                className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                <div>
                  <h2 className="t-h3">Board</h2>
                  <p className="t-xs mt-1">
                    Cards are grouped by{" "}
                    {workspace.boardConfig?.columnField?.name ?? "a field"}
                    {workspace.boardConfig?.swimlaneField
                      ? ` with swimlanes by ${workspace.boardConfig.swimlaneField.name}`
                      : ""}
                    .
                  </p>
                </div>
                <button
                  className="btn sm"
                  onClick={() => setEmptyColumnsVisible((visible) => !visible)}
                  type="button"
                >
                  {emptyColumnsVisible
                    ? "Hide empty columns"
                    : "Show empty columns"}
                </button>
              </div>
              {workspace.boardConfig?.unavailableReason || !boardColumnField ? (
                <div className="p-4">
                  <span className="chip warn">
                    {workspace.boardConfig?.unavailableReason ??
                      "Board layout needs a column field."}
                  </span>
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <div className="min-w-[980px] p-4">
                    {boardGroups.map((swimlane) => (
                      <section className="mb-5 last:mb-0" key={swimlane.key}>
                        <div className="mb-3 flex items-center gap-2">
                          <span className="t-label">{swimlane.label}</span>
                          <span className="t-xs t-num">
                            {swimlane.items.length}
                          </span>
                        </div>
                        <div className="grid auto-cols-[minmax(250px,1fr)] grid-flow-col gap-3">
                          {boardColumns.map((column) => {
                            const cards = boardColumnItems(
                              swimlane.items,
                              boardColumnField,
                              column,
                            );
                            return (
                              <section
                                aria-label={`${column.label} board column`}
                                className="min-h-[220px] rounded-[var(--radius)]"
                                key={`${swimlane.key}-${column.key}`}
                                style={{
                                  background: "var(--surface-2)",
                                  border: "1px solid var(--line-soft)",
                                }}
                              >
                                <div
                                  className="flex items-start justify-between gap-2 px-3 py-3"
                                  style={{
                                    borderBottom: "1px solid var(--line-soft)",
                                  }}
                                >
                                  <div className="min-w-0">
                                    <div className="t-sm font-medium">
                                      {column.label}
                                    </div>
                                    <div className="t-xs t-num mt-1">
                                      {cards.length} cards
                                      {column.itemLimit != null
                                        ? ` / limit ${column.itemLimit}`
                                        : ""}
                                    </div>
                                  </div>
                                  {column.overLimit ? (
                                    <span className="chip warn">
                                      Over limit
                                    </span>
                                  ) : null}
                                </div>
                                <div className="grid gap-2 p-2">
                                  {cards.length === 0 ? (
                                    <div className="t-xs p-3">
                                      No cards in this column.
                                    </div>
                                  ) : null}
                                  {cards.map((item) => (
                                    <article className="card p-3" key={item.id}>
                                      <div className="mb-2 flex items-start gap-2">
                                        <span className="chip soft t-mono-sm">
                                          {itemIcon(item)}
                                        </span>
                                        <Link
                                          className="min-w-0 flex-1 font-medium no-underline"
                                          href={projectItemHref(
                                            item,
                                            currentHref,
                                          )}
                                        >
                                          {item.title}
                                        </Link>
                                      </div>
                                      <div className="t-xs mb-3 flex flex-wrap gap-2">
                                        <span>{itemTypeLabel(item)}</span>
                                        {item.repository ? (
                                          <span>
                                            {item.repository.fullName}
                                          </span>
                                        ) : null}
                                        {item.number ? (
                                          <span className="t-mono-sm">
                                            #{item.number}
                                          </span>
                                        ) : null}
                                        {item.labels.map((label) => (
                                          <span
                                            className="chip soft"
                                            key={label.id}
                                          >
                                            {label.name}
                                          </span>
                                        ))}
                                        {item.assignees.map((assignee) => (
                                          <span
                                            className="av sm"
                                            key={assignee.id}
                                            title={assignee.login}
                                          >
                                            {assignee.login
                                              .slice(0, 1)
                                              .toUpperCase()}
                                          </span>
                                        ))}
                                      </div>
                                      <div className="mb-3 flex flex-wrap gap-2">
                                        {visibleFields
                                          .filter(
                                            (field) =>
                                              field.id !==
                                                boardColumnField.id &&
                                              field.id !==
                                                boardSwimlaneField?.id,
                                          )
                                          .slice(0, 3)
                                          .map((field) => {
                                            const value = fieldValue(
                                              item,
                                              field,
                                            );
                                            return value ? (
                                              <span
                                                className="chip soft"
                                                key={field.id}
                                              >
                                                {field.name}:{" "}
                                                {value.displayValue}
                                              </span>
                                            ) : null;
                                          })}
                                      </div>
                                      <label className="t-xs">
                                        Move to column
                                        <select
                                          aria-label={`Move ${item.title} to column`}
                                          className="input mt-1 w-full"
                                          disabled={
                                            itemSaving ||
                                            !workspace.viewerPermissions.canEdit
                                          }
                                          onChange={(event) => {
                                            const target = boardColumns.find(
                                              (entry) =>
                                                entry.key ===
                                                event.target.value,
                                            );
                                            if (target) {
                                              void moveItemToBoardColumn(
                                                item,
                                                target,
                                              );
                                            }
                                          }}
                                          value={
                                            fieldValue(item, boardColumnField)
                                              ?.displayValue ?? "no-value"
                                          }
                                        >
                                          {boardColumns.map((choice) => (
                                            <option
                                              key={choice.key}
                                              value={choice.key}
                                            >
                                              {choice.label}
                                            </option>
                                          ))}
                                        </select>
                                      </label>
                                    </article>
                                  ))}
                                  <button
                                    className="btn sm ghost"
                                    disabled={
                                      !workspace.viewerPermissions.canAddItems
                                    }
                                    onClick={() => setAddOpen(true)}
                                    type="button"
                                  >
                                    Add item
                                  </button>
                                </div>
                              </section>
                            );
                          })}
                        </div>
                      </section>
                    ))}
                  </div>
                </div>
              )}
              <div
                className="flex flex-wrap items-center gap-3 border-t px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                {itemError ? (
                  <span className="chip err">{itemError}</span>
                ) : null}
                {itemMessage ? (
                  <span className="chip ok">{itemMessage}</span>
                ) : null}
                <button
                  className="btn sm"
                  disabled={!workspace.viewerPermissions.canAddItems}
                  onClick={() => setAddOpen((open) => !open)}
                  type="button"
                >
                  Add item
                </button>
                <span className="t-xs">
                  Board moves use the same project item field rules as inline
                  table edits.
                </span>
              </div>
              {addOpen ? (
                <AddProjectItemPanel
                  addMode={addMode}
                  addUrl={addUrl}
                  bulkUrls={bulkUrls}
                  draftBody={draftBody}
                  draftTitle={draftTitle}
                  itemSaving={itemSaving}
                  onAddModeChange={setAddMode}
                  onAddUrlChange={setAddUrl}
                  onBulkUrlsChange={setBulkUrls}
                  onDraftBodyChange={setDraftBody}
                  onDraftTitleChange={setDraftTitle}
                  onSubmitAddItem={submitAddItem}
                  onSubmitBulkItems={submitBulkItems}
                  onSubmitDraftItem={submitDraftItem}
                />
              ) : null}
            </div>
          ) : (
            <div className="card overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full min-w-[920px] border-collapse">
                  <thead>
                    <tr style={{ borderBottom: "1px solid var(--line)" }}>
                      <th className="t-label w-14 px-4 py-3 text-left">#</th>
                      <th className="t-label min-w-[300px] px-3 py-3 text-left">
                        Item
                      </th>
                      {visibleFields.map((field) => (
                        <th
                          className="t-label min-w-[150px] px-3 py-3 text-left"
                          key={field.id}
                        >
                          {field.name}
                        </th>
                      ))}
                      <th className="t-label min-w-[190px] px-3 py-3 text-left">
                        Controls
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {groupedItems.map((group) => (
                      <Fragment key={group.label}>
                        <tr style={{ background: "var(--surface-2)" }}>
                          <td
                            className="px-4 py-2"
                            colSpan={visibleFields.length + 3}
                          >
                            <span className="t-label">{group.label}</span>{" "}
                            <span className="t-xs t-num">{group.count}</span>
                          </td>
                        </tr>
                        {group.items.map((item, index) => (
                          <tr className="list-row" key={item.id}>
                            <td className="t-mono-sm px-4 py-3">{index + 1}</td>
                            <td className="px-3 py-3">
                              <div className="flex min-w-0 items-start gap-3">
                                <span className="chip soft t-mono-sm">
                                  {itemIcon(item)}
                                </span>
                                <div className="min-w-0">
                                  <Link
                                    className="font-medium no-underline"
                                    href={projectItemHref(item, currentHref)}
                                  >
                                    {item.title}
                                  </Link>
                                  <div className="t-xs mt-1 flex flex-wrap gap-2">
                                    <span>{itemTypeLabel(item)}</span>
                                    {item.repository ? (
                                      <span>{item.repository.fullName}</span>
                                    ) : null}
                                    {item.number ? (
                                      <span className="t-mono-sm">
                                        #{item.number}
                                      </span>
                                    ) : null}
                                    {item.labels.map((label) => (
                                      <span
                                        className="chip soft"
                                        key={label.id}
                                      >
                                        {label.name}
                                      </span>
                                    ))}
                                    {item.assignees.map((assignee) => (
                                      <span
                                        className="av sm"
                                        key={assignee.id}
                                        title={assignee.login}
                                      >
                                        {assignee.login
                                          .slice(0, 1)
                                          .toUpperCase()}
                                      </span>
                                    ))}
                                  </div>
                                </div>
                              </div>
                            </td>
                            {visibleFields.map((field) => {
                              const value = fieldValue(item, field);
                              const isEditing =
                                editingCell?.itemId === item.id &&
                                editingCell.fieldId === field.id;
                              return (
                                <td className="t-sm px-3 py-3" key={field.id}>
                                  {isEditing ? (
                                    <form
                                      aria-label={`Edit ${field.name} for ${item.title}`}
                                      className="flex min-w-[180px] flex-wrap gap-2"
                                      onSubmit={(event) =>
                                        saveFieldValue(event, item, field)
                                      }
                                    >
                                      <input
                                        aria-label={`${field.name} value`}
                                        className="input min-w-[130px] flex-1"
                                        onChange={(event) =>
                                          setEditValue(event.target.value)
                                        }
                                        type={inputTypeForField(field)}
                                        value={editValue}
                                      />
                                      <button
                                        className="btn sm primary"
                                        disabled={fieldSaving}
                                        type="submit"
                                      >
                                        {fieldSaving ? "Saving..." : "Save"}
                                      </button>
                                      <button
                                        className="btn sm"
                                        onClick={() => setEditingCell(null)}
                                        type="button"
                                      >
                                        Cancel
                                      </button>
                                    </form>
                                  ) : value ? (
                                    <div className="flex flex-wrap items-center gap-2">
                                      <Link
                                        className="chip soft no-underline"
                                        href={workspaceHref(
                                          scope,
                                          owner,
                                          workspace.project.number,
                                          viewNumber,
                                          {
                                            ...baseQuery,
                                            q: `${workspace.filters.query ?? ""} ${field.name}:${value.displayValue}`.trim(),
                                            page: 1,
                                          },
                                        )}
                                      >
                                        {value.displayValue}
                                      </Link>
                                      <button
                                        className="btn sm ghost"
                                        disabled={
                                          !workspace.viewerPermissions
                                            .canEdit || !field.editable
                                        }
                                        onClick={() =>
                                          openFieldEditor(item, field, value)
                                        }
                                        title={
                                          field.editable
                                            ? `Edit ${field.name}`
                                            : `${field.name} cannot be edited inline.`
                                        }
                                        type="button"
                                      >
                                        Edit
                                      </button>
                                    </div>
                                  ) : (
                                    <button
                                      className="btn sm ghost"
                                      disabled={
                                        !workspace.viewerPermissions.canEdit ||
                                        !field.editable
                                      }
                                      onClick={() =>
                                        openFieldEditor(item, field, value)
                                      }
                                      title={
                                        field.editable
                                          ? `Set ${field.name}`
                                          : `${field.name} cannot be edited inline.`
                                      }
                                      type="button"
                                    >
                                      No value
                                    </button>
                                  )}
                                </td>
                              );
                            })}
                            <td className="t-xs px-3 py-3">
                              <div className="flex flex-wrap items-center gap-2">
                                <span>{formatDate(item.updatedAt)}</span>
                                <button
                                  className="btn sm ghost"
                                  disabled={
                                    itemSaving ||
                                    !workspace.viewerPermissions.canEdit ||
                                    workspace.items[0]?.id === item.id
                                  }
                                  onClick={() => moveItem(item, "up")}
                                  title="Move row up"
                                  type="button"
                                >
                                  Up
                                </button>
                                <button
                                  className="btn sm ghost"
                                  disabled={
                                    itemSaving ||
                                    !workspace.viewerPermissions.canEdit ||
                                    workspace.items.at(-1)?.id === item.id
                                  }
                                  onClick={() => moveItem(item, "down")}
                                  title="Move row down"
                                  type="button"
                                >
                                  Down
                                </button>
                                <button
                                  className="btn sm ghost"
                                  disabled={
                                    itemSaving ||
                                    !workspace.viewerPermissions.canEdit
                                  }
                                  onClick={() => removeItem(item)}
                                  title="Remove item from project"
                                  type="button"
                                >
                                  Remove
                                </button>
                              </div>
                            </td>
                          </tr>
                        ))}
                      </Fragment>
                    ))}
                  </tbody>
                </table>
              </div>
              <div
                className="flex flex-wrap items-center gap-3 border-t px-4 py-3"
                style={{ borderColor: "var(--line)" }}
              >
                {fieldError ? (
                  <span className="chip err">{fieldError}</span>
                ) : null}
                {fieldMessage ? (
                  <span className="chip ok">{fieldMessage}</span>
                ) : null}
                {itemError ? (
                  <span className="chip err">{itemError}</span>
                ) : null}
                {itemMessage ? (
                  <span className="chip ok">{itemMessage}</span>
                ) : null}
                <button
                  className="btn sm"
                  disabled={!workspace.viewerPermissions.canAddItems}
                  onClick={() => setAddOpen((open) => !open)}
                  title={
                    workspace.viewerPermissions.canAddItems
                      ? "Add issue, pull request, or draft item"
                      : "You need write access to add project items."
                  }
                  type="button"
                >
                  Add item
                </button>
                <span className="t-xs">
                  Paste issue or pull request URLs, create drafts, or bulk add
                  rows.
                </span>
              </div>
              {addOpen ? (
                <AddProjectItemPanel
                  addMode={addMode}
                  addUrl={addUrl}
                  bulkUrls={bulkUrls}
                  draftBody={draftBody}
                  draftTitle={draftTitle}
                  itemSaving={itemSaving}
                  onAddModeChange={setAddMode}
                  onAddUrlChange={setAddUrl}
                  onBulkUrlsChange={setBulkUrls}
                  onDraftBodyChange={setDraftBody}
                  onDraftTitleChange={setDraftTitle}
                  onSubmitAddItem={submitAddItem}
                  onSubmitBulkItems={submitBulkItems}
                  onSubmitDraftItem={submitDraftItem}
                />
              ) : null}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

type AddProjectItemPanelProps = {
  addMode: "url" | "draft" | "bulk";
  addUrl: string;
  bulkUrls: string;
  draftBody: string;
  draftTitle: string;
  itemSaving: boolean;
  onAddModeChange: (mode: "url" | "draft" | "bulk") => void;
  onAddUrlChange: (value: string) => void;
  onBulkUrlsChange: (value: string) => void;
  onDraftBodyChange: (value: string) => void;
  onDraftTitleChange: (value: string) => void;
  onSubmitAddItem: (event: FormEvent<HTMLFormElement>) => void;
  onSubmitBulkItems: (event: FormEvent<HTMLFormElement>) => void;
  onSubmitDraftItem: (event: FormEvent<HTMLFormElement>) => void;
};

function AddProjectItemPanel({
  addMode,
  addUrl,
  bulkUrls,
  draftBody,
  draftTitle,
  itemSaving,
  onAddModeChange,
  onAddUrlChange,
  onBulkUrlsChange,
  onDraftBodyChange,
  onDraftTitleChange,
  onSubmitAddItem,
  onSubmitBulkItems,
  onSubmitDraftItem,
}: AddProjectItemPanelProps) {
  return (
    <section
      aria-label="Add project item"
      className="border-t p-4"
      style={{ borderColor: "var(--line)" }}
    >
      <div className="mb-3 flex flex-wrap gap-2">
        {(["url", "draft", "bulk"] as const).map((mode) => (
          <button
            className={`chip ${addMode === mode ? "active" : "soft"}`}
            key={mode}
            onClick={() => onAddModeChange(mode)}
            type="button"
          >
            {mode === "url"
              ? "Paste URL"
              : mode === "draft"
                ? "Draft issue"
                : "Bulk add"}
          </button>
        ))}
      </div>
      {addMode === "url" ? (
        <form
          aria-label="Add linked issue or pull request"
          className="flex flex-wrap gap-2"
          onSubmit={onSubmitAddItem}
        >
          <input
            aria-label="Issue or pull request URL"
            className="input min-w-[280px] flex-1"
            onChange={(event) => onAddUrlChange(event.target.value)}
            placeholder="/namuh/opengithub/issues/42"
            value={addUrl}
          />
          <button
            className="btn sm primary"
            disabled={itemSaving || !addUrl.trim()}
            type="submit"
          >
            {itemSaving ? "Adding..." : "Add linked item"}
          </button>
        </form>
      ) : null}
      {addMode === "draft" ? (
        <form
          aria-label="Create draft project item"
          className="grid gap-2"
          onSubmit={onSubmitDraftItem}
        >
          <input
            aria-label="Draft title"
            className="input"
            onChange={(event) => onDraftTitleChange(event.target.value)}
            placeholder="Draft issue title"
            value={draftTitle}
          />
          <textarea
            aria-label="Draft body"
            className="input min-h-[84px]"
            onChange={(event) => onDraftBodyChange(event.target.value)}
            placeholder="Optional notes"
            value={draftBody}
          />
          <button
            className="btn sm primary w-fit"
            disabled={itemSaving || !draftTitle.trim()}
            type="submit"
          >
            {itemSaving ? "Creating..." : "Create draft"}
          </button>
        </form>
      ) : null}
      {addMode === "bulk" ? (
        <form
          aria-label="Bulk add project items"
          className="grid gap-2"
          onSubmit={onSubmitBulkItems}
        >
          <textarea
            aria-label="Bulk issue and pull request URLs"
            className="input min-h-[110px]"
            onChange={(event) => onBulkUrlsChange(event.target.value)}
            placeholder="/namuh/opengithub/issues/42 /namuh/opengithub/pull/43"
            value={bulkUrls}
          />
          <button
            className="btn sm primary w-fit"
            disabled={itemSaving || !bulkUrls.trim()}
            type="submit"
          >
            {itemSaving ? "Adding..." : "Bulk add"}
          </button>
        </form>
      ) : null}
    </section>
  );
}
