"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type {
  ProjectFieldSettings,
  ProjectFieldSettingsField,
} from "@/lib/api";
import {
  organizationProjectFieldSettingsHref,
  organizationProjectWorkspaceHref,
  userProjectFieldSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectFieldSettingsPageProps = {
  settings: ProjectFieldSettings;
  scope: "user" | "organization";
  owner: string;
  selectedFieldId?: string | null;
};

const FIELD_TYPE_LABELS: Record<string, string> = {
  title: "Title",
  assignees: "Assignees",
  labels: "Labels",
  repository: "Repository",
  milestone: "Milestone",
  linked_pull_request: "Pull request",
  single_select: "Single-select",
  iteration: "Iteration",
  date: "Date",
  text: "Text",
  number: "Number",
};

const CREATE_FIELD_TYPES = [
  { label: "Single-select", value: "single_select" },
  { label: "Date", value: "date" },
  { label: "Text", value: "text" },
  { label: "Number", value: "number" },
  { label: "Iteration", value: "iteration" },
];

const SETTINGS_NAV = [
  { label: "General", key: "general", disabled: true },
  { label: "Fields", key: "fields", disabled: false },
  { label: "Views", key: "views", disabled: true },
  { label: "Workflows", key: "workflows", disabled: true },
  { label: "Access", key: "access", disabled: true },
];

function fieldSettingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  fieldId?: string | null,
) {
  return scope === "organization"
    ? organizationProjectFieldSettingsHref(owner, projectNumber, fieldId)
    : userProjectFieldSettingsHref(owner, projectNumber, fieldId);
}

function workspaceHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
) {
  return scope === "organization"
    ? organizationProjectWorkspaceHref(owner, projectNumber, 1)
    : userProjectWorkspaceHref(owner, projectNumber, 1);
}

function fieldTypeLabel(field: ProjectFieldSettingsField) {
  return (
    FIELD_TYPE_LABELS[field.fieldType] ?? field.fieldType.replaceAll("_", " ")
  );
}

function fieldKind(field: ProjectFieldSettingsField) {
  if (field.builtIn) return "Built-in";
  if (field.fieldType === "single_select") return "Options";
  if (field.fieldType === "iteration") return "Schedule";
  return "Custom";
}

function fieldSummary(field: ProjectFieldSettingsField) {
  if (field.fieldType === "single_select") {
    return `${field.options.length} options`;
  }
  if (field.fieldType === "iteration") {
    return `${field.iterations.length} iterations, ${field.breaks.length} breaks`;
  }
  return `${field.usageCount} item values`;
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function optionSwatchColor(color: string) {
  if (color === "rust" || color === "red" || color === "orange") {
    return "var(--accent)";
  }
  if (color === "green") return "var(--ok)";
  if (color === "yellow") return "var(--warn)";
  if (color === "blue" || color === "purple") return "var(--info)";
  return "var(--ink-4)";
}

export function ProjectFieldSettingsPage({
  settings,
  scope,
  owner,
  selectedFieldId,
}: ProjectFieldSettingsPageProps) {
  const [fields, setFields] = useState(settings.fields);
  const [newFieldOpen, setNewFieldOpen] = useState(false);
  const [newFieldName, setNewFieldName] = useState("");
  const [newFieldType, setNewFieldType] = useState("single_select");
  const [fieldName, setFieldName] = useState("");
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const selectedField = useMemo(
    () =>
      fields.find((field) => field.id === selectedFieldId) ??
      fields.find((field) => !field.builtIn) ??
      fields[0] ??
      null,
    [fields, selectedFieldId],
  );
  const canManageAny =
    settings.viewerPermissions.canCreateFields ||
    settings.viewerPermissions.canRenameFields ||
    settings.viewerPermissions.canDeleteFields;
  const baseFieldsHref = fieldSettingsHref(
    scope,
    owner,
    settings.project.number,
  );

  const activeFieldName =
    fieldName || (selectedField ? selectedField.name : "");
  const canRenameSelected = Boolean(
    selectedField?.editable && settings.viewerPermissions.canRenameFields,
  );
  const canDeleteSelected = Boolean(
    selectedField?.deletable && settings.viewerPermissions.canDeleteFields,
  );

  async function submitFieldMutation(
    action: string,
    path: string,
    init: RequestInit,
  ) {
    setPendingAction(action);
    setNotice(null);
    setError(null);
    try {
      const response = await fetch(path, {
        ...init,
        headers: {
          "content-type": "application/json",
          ...(init.headers ?? {}),
        },
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        const message =
          payload?.error?.message ?? "Project field change could not be saved.";
        throw new Error(message);
      }
      if (Array.isArray(payload?.fields)) {
        setFields(payload.fields);
      }
      return payload as ProjectFieldSettings;
    } catch (mutationError) {
      setError(
        mutationError instanceof Error
          ? mutationError.message
          : "Project field change could not be saved.",
      );
      return null;
    } finally {
      setPendingAction(null);
    }
  }

  async function createField() {
    const payload = await submitFieldMutation(
      "create",
      `/api/projects/${encodeURIComponent(settings.project.id)}/fields`,
      {
        method: "POST",
        body: JSON.stringify({
          name: newFieldName,
          fieldType: newFieldType,
        }),
      },
    );
    if (!payload) return;
    const created = payload.fields.find(
      (field) =>
        field.name.trim().toLowerCase() === newFieldName.trim().toLowerCase(),
    );
    setNewFieldOpen(false);
    setNewFieldName("");
    setNewFieldType("single_select");
    setNotice("Field created.");
    if (created) {
      window.history.replaceState(
        null,
        "",
        fieldSettingsHref(scope, owner, settings.project.number, created.id),
      );
    }
  }

  async function renameField() {
    if (!selectedField) return;
    const payload = await submitFieldMutation(
      "rename",
      `/api/projects/${encodeURIComponent(settings.project.id)}/fields/${encodeURIComponent(selectedField.id)}`,
      {
        method: "PATCH",
        body: JSON.stringify({
          name: activeFieldName,
          expectedUpdatedAt: selectedField.updatedAt,
        }),
      },
    );
    if (!payload) return;
    setFieldName("");
    setNotice("Field renamed.");
  }

  async function deleteField() {
    if (!selectedField) return;
    const payload = await submitFieldMutation(
      "delete",
      `/api/projects/${encodeURIComponent(settings.project.id)}/fields/${encodeURIComponent(selectedField.id)}`,
      {
        method: "DELETE",
        body: JSON.stringify({ expectedUpdatedAt: selectedField.updatedAt }),
      },
    );
    if (!payload) return;
    setDeleteConfirmOpen(false);
    setFieldName("");
    setNotice("Field deleted. Existing item values were removed.");
    window.history.replaceState(
      null,
      "",
      fieldSettingsHref(scope, owner, settings.project.number),
    );
  }

  return (
    <main
      style={{ maxWidth: 1240, margin: "0 auto", padding: "24px 32px 48px" }}
    >
      <div
        className="row"
        style={{ gap: 10, marginBottom: 18, flexWrap: "wrap" }}
      >
        <Link
          className="chip soft"
          href={workspaceHref(scope, owner, settings.project.number)}
        >
          Back to project
        </Link>
        <span className="chip active">Settings</span>
        <span className="t-xs t-mono-sm">#{settings.project.number}</span>
      </div>

      <div
        className="row"
        style={{ alignItems: "flex-start", gap: 18, marginBottom: 24 }}
      >
        <div style={{ flex: 1, minWidth: 0 }}>
          <div className="t-label">Project fields</div>
          <h1 className="t-h1" style={{ marginTop: 6 }}>
            {settings.project.title}
          </h1>
          <p
            className="t-sm"
            style={{ color: "var(--ink-3)", maxWidth: 720, marginTop: 8 }}
          >
            Manage the fields that shape table, board, and roadmap views.
          </p>
        </div>
        <button
          className="btn primary"
          disabled={
            !settings.viewerPermissions.canCreateFields ||
            pendingAction !== null ||
            settings.limits.remainingFields <= 0
          }
          onClick={() => setNewFieldOpen(true)}
          type="button"
        >
          New field
        </button>
      </div>

      {settings.unavailableReason ? (
        <div
          className="card"
          role="alert"
          style={{ padding: 16, marginBottom: 18 }}
        >
          <div className="t-label">Unavailable</div>
          <p className="t-sm" style={{ marginTop: 6, color: "var(--ink-3)" }}>
            {settings.unavailableReason}
          </p>
        </div>
      ) : null}

      {notice ? (
        <div className="chip ok" role="status" style={{ marginBottom: 14 }}>
          {notice}
        </div>
      ) : null}
      {error ? (
        <div className="chip err" role="alert" style={{ marginBottom: 14 }}>
          {error}
        </div>
      ) : null}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "220px minmax(0, 1fr)",
          gap: 20,
          alignItems: "start",
        }}
      >
        <aside className="card" style={{ padding: 8 }}>
          <div className="t-label" style={{ padding: "8px 10px" }}>
            Settings
          </div>
          {SETTINGS_NAV.map((item) =>
            item.disabled ? (
              <button
                className="btn ghost"
                disabled
                key={item.key}
                style={{
                  justifyContent: "flex-start",
                  width: "100%",
                  marginTop: 2,
                }}
                type="button"
              >
                {item.label}
              </button>
            ) : (
              <Link
                className="btn ghost active"
                href={baseFieldsHref}
                key={item.key}
                style={{
                  justifyContent: "flex-start",
                  width: "100%",
                  marginTop: 2,
                }}
              >
                {item.label}
              </Link>
            ),
          )}
        </aside>

        <section
          style={{
            display: "grid",
            gridTemplateColumns: "minmax(280px, 420px) minmax(0, 1fr)",
            gap: 20,
          }}
        >
          <div className="card" style={{ overflow: "hidden" }}>
            <div
              className="row"
              style={{
                padding: 16,
                borderBottom: "1px solid var(--line)",
                gap: 10,
              }}
            >
              <div style={{ flex: 1 }}>
                <div className="t-label">Fields</div>
                <div
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 4 }}
                >
                  {settings.limits.usedFields} of {settings.limits.maxFields}{" "}
                  used · {settings.limits.remainingFields} remaining
                </div>
              </div>
              <span
                className={
                  settings.limits.remainingFields > 0
                    ? "chip soft"
                    : "chip warn"
                }
              >
                {settings.limits.remainingFields > 0
                  ? "Room available"
                  : "Limit reached"}
              </span>
            </div>

            {fields.map((field) => {
              const active = selectedField?.id === field.id;
              return (
                <Link
                  aria-current={active ? "page" : undefined}
                  className="list-row"
                  href={fieldSettingsHref(
                    scope,
                    owner,
                    settings.project.number,
                    field.id,
                  )}
                  key={field.id}
                  style={{
                    padding: "14px 16px",
                    gap: 12,
                    background: active ? "var(--surface-2)" : "transparent",
                    textDecoration: "none",
                  }}
                >
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
                      <span
                        className="t-sm"
                        style={{ fontWeight: 600, color: "var(--ink-1)" }}
                      >
                        {field.name}
                      </span>
                      <span
                        className={field.builtIn ? "chip soft" : "chip accent"}
                      >
                        {fieldKind(field)}
                      </span>
                    </div>
                    <div className="t-xs" style={{ marginTop: 5 }}>
                      {fieldTypeLabel(field)} · {fieldSummary(field)}
                    </div>
                  </div>
                  <span className="t-mono-sm" style={{ color: "var(--ink-4)" }}>
                    {field.position}
                  </span>
                </Link>
              );
            })}
          </div>

          <div className="card" style={{ padding: 20, minWidth: 0 }}>
            {selectedField ? (
              <>
                <div
                  className="row"
                  style={{ gap: 12, alignItems: "flex-start" }}
                >
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div className="t-label">
                      {fieldKind(selectedField)} field
                    </div>
                    <h2 className="t-h2" style={{ marginTop: 6 }}>
                      {selectedField.name}
                    </h2>
                    <div
                      className="row"
                      style={{ gap: 8, marginTop: 10, flexWrap: "wrap" }}
                    >
                      <span className="chip soft">
                        {fieldTypeLabel(selectedField)}
                      </span>
                      <span className="chip soft">
                        {selectedField.usageCount} values
                      </span>
                      <span className="chip soft">
                        Updated {formatDate(selectedField.updatedAt)}
                      </span>
                    </div>
                  </div>
                  <span className="t-mono-sm" style={{ color: "var(--ink-4)" }}>
                    v{selectedField.cacheVersion}
                  </span>
                </div>

                <div style={{ marginTop: 22 }}>
                  <label className="t-label" htmlFor="project-field-name">
                    Name
                  </label>
                  <input
                    className="input"
                    disabled={!canRenameSelected || pendingAction !== null}
                    id="project-field-name"
                    onChange={(event) => setFieldName(event.target.value)}
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={activeFieldName}
                  />
                </div>

                <div style={{ marginTop: 18 }}>
                  <label className="t-label" htmlFor="project-field-type">
                    Type
                  </label>
                  <select
                    className="input"
                    disabled
                    id="project-field-type"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={selectedField.fieldType}
                  >
                    <option value={selectedField.fieldType}>
                      {fieldTypeLabel(selectedField)}
                    </option>
                  </select>
                  <p className="t-xs" style={{ marginTop: 8 }}>
                    Field type changes are not supported after creation.
                  </p>
                </div>

                {selectedField.fieldType === "single_select" ? (
                  <div style={{ marginTop: 24 }}>
                    <div
                      className="row"
                      style={{ justifyContent: "space-between", gap: 12 }}
                    >
                      <div>
                        <div className="t-label">Options</div>
                        <p className="t-xs" style={{ marginTop: 4 }}>
                          {selectedField.options.length} of{" "}
                          {settings.limits.maxOptionsPerField} options.
                        </p>
                      </div>
                      <button className="btn sm" disabled type="button">
                        Add option
                      </button>
                    </div>
                    <div style={{ marginTop: 10 }}>
                      {selectedField.options.length > 0 ? (
                        selectedField.options.map((option) => (
                          <div
                            className="list-row"
                            key={option.id}
                            style={{ padding: "10px 0", gap: 10 }}
                          >
                            <span
                              aria-hidden="true"
                              style={{
                                width: 12,
                                height: 12,
                                borderRadius: "var(--radius-pill)",
                                background: optionSwatchColor(option.color),
                                border: "1px solid var(--line-strong)",
                                flex: "0 0 auto",
                              }}
                            />
                            <span className="t-sm" style={{ flex: 1 }}>
                              {option.name}
                            </span>
                            <span className="t-xs">
                              {option.description ?? "No description"}
                            </span>
                          </div>
                        ))
                      ) : (
                        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                          No options have been added yet.
                        </p>
                      )}
                    </div>
                  </div>
                ) : null}

                {selectedField.fieldType === "iteration" ? (
                  <div style={{ marginTop: 24 }}>
                    <div
                      className="row"
                      style={{ justifyContent: "space-between", gap: 12 }}
                    >
                      <div>
                        <div className="t-label">Iterations</div>
                        <p className="t-xs" style={{ marginTop: 4 }}>
                          Cycles and breaks are managed on this field.
                        </p>
                      </div>
                      <button className="btn sm" disabled type="button">
                        Add iteration
                      </button>
                    </div>
                    <div style={{ marginTop: 12, display: "grid", gap: 10 }}>
                      {selectedField.iterations.map((iteration) => (
                        <div
                          className="card"
                          key={iteration.id}
                          style={{ padding: 12 }}
                        >
                          <div
                            className="row"
                            style={{ justifyContent: "space-between", gap: 12 }}
                          >
                            <span className="t-sm" style={{ fontWeight: 600 }}>
                              {iteration.name}
                            </span>
                            <span className="chip soft">
                              {iteration.durationDays} days
                            </span>
                          </div>
                          <div className="t-xs" style={{ marginTop: 6 }}>
                            Starts {formatDate(iteration.startDate)}
                          </div>
                        </div>
                      ))}
                      {selectedField.breaks.map((fieldBreak) => (
                        <div
                          className="card"
                          key={fieldBreak.id}
                          style={{
                            padding: 12,
                            background: "var(--surface-2)",
                          }}
                        >
                          <div
                            className="row"
                            style={{ justifyContent: "space-between", gap: 12 }}
                          >
                            <span className="t-sm" style={{ fontWeight: 600 }}>
                              {fieldBreak.name}
                            </span>
                            <span className="chip warn">
                              {fieldBreak.durationDays} day break
                            </span>
                          </div>
                          <div className="t-xs" style={{ marginTop: 6 }}>
                            Starts {formatDate(fieldBreak.startDate)}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : null}

                <div
                  className="row"
                  style={{ gap: 10, marginTop: 26, flexWrap: "wrap" }}
                >
                  <button
                    className="btn primary"
                    disabled={
                      !canRenameSelected ||
                      pendingAction !== null ||
                      activeFieldName.trim() === selectedField.name
                    }
                    onClick={renameField}
                    type="button"
                  >
                    Save changes
                  </button>
                  <button
                    className="btn"
                    disabled={
                      !canRenameSelected ||
                      pendingAction !== null ||
                      activeFieldName.trim() === selectedField.name
                    }
                    onClick={renameField}
                    type="button"
                  >
                    Rename
                  </button>
                  <button
                    className="btn"
                    disabled={!canDeleteSelected || pendingAction !== null}
                    onClick={() => setDeleteConfirmOpen(true)}
                    type="button"
                  >
                    Delete
                  </button>
                </div>

                {canManageAny ? (
                  <p className="t-xs" style={{ marginTop: 14 }}>
                    Field changes are saved to the project schema and refresh
                    table, board, and roadmap views.
                  </p>
                ) : (
                  <p className="t-xs" style={{ marginTop: 14 }}>
                    You can inspect fields, but this project role cannot change
                    them.
                  </p>
                )}
              </>
            ) : (
              <div>
                <div className="t-label">No fields</div>
                <h2 className="t-h2" style={{ marginTop: 6 }}>
                  This project has no fields yet.
                </h2>
              </div>
            )}
          </div>
        </section>
      </div>

      {newFieldOpen ? (
        <div
          aria-modal="true"
          className="card"
          role="dialog"
          style={{
            position: "fixed",
            inset: "auto 24px 24px auto",
            width: "min(420px, calc(100vw - 48px))",
            padding: 18,
            boxShadow: "var(--shadow-lg)",
            background: "var(--surface)",
            zIndex: 20,
          }}
        >
          <div
            className="row"
            style={{ justifyContent: "space-between", gap: 12 }}
          >
            <h2 className="t-h3">New field</h2>
            <button
              className="btn sm"
              onClick={() => setNewFieldOpen(false)}
              type="button"
            >
              Close
            </button>
          </div>
          <label
            className="t-label"
            htmlFor="new-project-field-name"
            style={{ display: "block", marginTop: 16 }}
          >
            Name
          </label>
          <input
            className="input"
            id="new-project-field-name"
            onChange={(event) => setNewFieldName(event.target.value)}
            placeholder="Priority"
            style={{ display: "block", marginTop: 8, width: "100%" }}
            value={newFieldName}
          />
          <label
            className="t-label"
            htmlFor="new-project-field-type"
            style={{ display: "block", marginTop: 14 }}
          >
            Type
          </label>
          <select
            className="input"
            id="new-project-field-type"
            onChange={(event) => setNewFieldType(event.target.value)}
            style={{ display: "block", marginTop: 8, width: "100%" }}
            value={newFieldType}
          >
            {CREATE_FIELD_TYPES.map((fieldType) => (
              <option key={fieldType.value} value={fieldType.value}>
                {fieldType.label}
              </option>
            ))}
          </select>
          <p className="t-xs" style={{ marginTop: 12 }}>
            Field type is fixed after creation. Options and iteration cycles are
            configured after the field exists.
          </p>
          <button
            className="btn primary"
            disabled={
              pendingAction !== null ||
              !newFieldName.trim() ||
              settings.limits.remainingFields <= 0
            }
            onClick={createField}
            style={{ marginTop: 14 }}
            type="button"
          >
            Create field
          </button>
        </div>
      ) : null}

      {deleteConfirmOpen && selectedField ? (
        <div
          aria-modal="true"
          className="card"
          role="dialog"
          style={{
            position: "fixed",
            inset: "auto 24px 24px auto",
            width: "min(420px, calc(100vw - 48px))",
            padding: 18,
            boxShadow: "var(--shadow-lg)",
            background: "var(--surface)",
            zIndex: 21,
          }}
        >
          <div className="t-label">Delete field</div>
          <h2 className="t-h3" style={{ marginTop: 6 }}>
            Delete {selectedField.name}?
          </h2>
          <p className="t-sm" style={{ color: "var(--ink-3)", marginTop: 10 }}>
            This removes {selectedField.usageCount} saved project values from
            items. Linked issues and pull requests are not deleted.
          </p>
          <div className="row" style={{ gap: 10, marginTop: 16 }}>
            <button
              className="btn"
              onClick={() => setDeleteConfirmOpen(false)}
              type="button"
            >
              Cancel
            </button>
            <button
              className="btn primary"
              disabled={pendingAction !== null}
              onClick={deleteField}
              type="button"
            >
              Delete field
            </button>
          </div>
        </div>
      ) : null}
    </main>
  );
}
