"use client";

import Link from "next/link";
import { type FormEvent, Fragment, useMemo, useState } from "react";
import type {
  ProjectWorkspace,
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

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(new Date(value));
}

export function ProjectWorkspacePage({
  workspace,
  scope,
  owner,
  viewNumber,
}: ProjectWorkspacePageProps) {
  const [query, setQuery] = useState(workspace.filters.query ?? "");
  const visibleFields = workspace.fields.filter((field) => !field.hidden);
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
              disabled
              title="Field visibility and saved view editing arrive in Phase 3."
              type="button"
            >
              View configuration
            </button>
          </div>

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
                    <th className="t-label min-w-[120px] px-3 py-3 text-left">
                      Updated
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
                                    <span className="chip soft" key={label.id}>
                                      {label.name}
                                    </span>
                                  ))}
                                  {item.assignees.map((assignee) => (
                                    <span
                                      className="av sm"
                                      key={assignee.id}
                                      title={assignee.login}
                                    >
                                      {assignee.login.slice(0, 1).toUpperCase()}
                                    </span>
                                  ))}
                                </div>
                              </div>
                            </div>
                          </td>
                          {visibleFields.map((field) => {
                            const value = fieldValue(item, field);
                            return (
                              <td className="t-sm px-3 py-3" key={field.id}>
                                {value ? (
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
                                ) : (
                                  <span style={{ color: "var(--ink-4)" }}>
                                    No value
                                  </span>
                                )}
                              </td>
                            );
                          })}
                          <td className="t-xs px-3 py-3">
                            {formatDate(item.updatedAt)}
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
              <button
                className="btn sm"
                disabled={!workspace.viewerPermissions.canAddItems}
                title={
                  workspace.viewerPermissions.canAddItems
                    ? "Add row is implemented in Phase 5."
                    : "You need write access to add project items."
                }
                type="button"
              >
                Add item
              </button>
              <span className="t-xs">
                Paste issue or pull request URLs, create drafts, and reorder
                rows in the add-row phase.
              </span>
            </div>
          </div>
        </section>
      </div>
    </main>
  );
}
