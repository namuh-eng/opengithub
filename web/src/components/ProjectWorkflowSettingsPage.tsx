"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import type {
  ProjectWorkflowDefinition,
  ProjectWorkflowSettings,
  ProjectWorkflowUpdateRequest,
} from "@/lib/api";
import {
  organizationProjectFieldSettingsHref,
  organizationProjectWorkflowSettingsHref,
  organizationProjectWorkspaceHref,
  userProjectFieldSettingsHref,
  userProjectWorkflowSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectWorkflowSettingsPageProps = {
  settings: ProjectWorkflowSettings;
  scope: "user" | "organization";
  owner: string;
  selectedWorkflowId?: string | null;
};

const SETTINGS_NAV = [
  { label: "General", key: "general", disabled: true },
  { label: "Access", key: "access", disabled: true },
  { label: "Fields", key: "fields", disabled: false },
  { label: "Workflows", key: "workflows", disabled: false },
  { label: "Templates", key: "templates", disabled: true },
  { label: "Danger Zone", key: "danger", disabled: true },
] as const;

function workflowSettingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  workflowId?: string | null,
) {
  return scope === "organization"
    ? organizationProjectWorkflowSettingsHref(owner, projectNumber, workflowId)
    : userProjectWorkflowSettingsHref(owner, projectNumber, workflowId);
}

function fieldSettingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
) {
  return scope === "organization"
    ? organizationProjectFieldSettingsHref(owner, projectNumber)
    : userProjectFieldSettingsHref(owner, projectNumber);
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

function formatDate(value: string | null) {
  if (!value) return "Never run";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}

function triggerLabel(value: string) {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => part[0]?.toUpperCase() + part.slice(1))
    .join(" ");
}

function statusChip(workflow: ProjectWorkflowDefinition) {
  if (!workflow.enabled) return "chip soft";
  if (workflow.lastRunStatus === "failed") return "chip err";
  if (workflow.lastRunStatus === "skipped") return "chip warn";
  return "chip ok";
}

export function ProjectWorkflowSettingsPage({
  settings,
  scope,
  owner,
  selectedWorkflowId,
}: ProjectWorkflowSettingsPageProps) {
  const [currentSettings, setCurrentSettings] = useState(settings);
  const initialWorkflow = useMemo(
    () =>
      settings.workflows.find(
        (workflow) => workflow.id === selectedWorkflowId,
      ) ?? null,
    [settings.workflows, selectedWorkflowId],
  );
  const [openWorkflowId, setOpenWorkflowId] = useState<string | null>(
    initialWorkflow?.id ?? null,
  );
  const [formState, setFormState] = useState<ProjectWorkflowUpdateRequest>({});
  const [saveState, setSaveState] = useState<{
    status: "idle" | "saving" | "saved" | "error";
    message: string | null;
  }>({ status: "idle", message: null });
  useEffect(() => setCurrentSettings(settings), [settings]);
  const openWorkflow =
    currentSettings.workflows.find(
      (workflow) => workflow.id === openWorkflowId,
    ) ?? null;
  const canManage = currentSettings.viewerPermissions.canManageWorkflows;
  const fieldsById = new Map(
    currentSettings.eligibleFields.map((field) => [field.id, field]),
  );
  const selectedFieldId =
    formState.statusFieldId ??
    (typeof openWorkflow?.configuration.target === "object" &&
    openWorkflow.configuration.target &&
    "fieldId" in openWorkflow.configuration.target &&
    typeof openWorkflow.configuration.target.fieldId === "string"
      ? openWorkflow.configuration.target.fieldId
      : "");
  const selectedField = currentSettings.eligibleFields.find(
    (field) => field.id === selectedFieldId,
  );
  const selectedOptionId =
    formState.statusOptionId ??
    (typeof openWorkflow?.configuration.target === "object" &&
    openWorkflow.configuration.target &&
    "optionId" in openWorkflow.configuration.target &&
    typeof openWorkflow.configuration.target.optionId === "string"
      ? openWorkflow.configuration.target.optionId
      : "");
  const selectedRepositoryIds =
    formState.repositoryTargetIds ?? openWorkflow?.repositoryTargetIds ?? [];
  const condition =
    formState.condition ??
    (typeof openWorkflow?.configuration.condition === "string"
      ? openWorkflow.configuration.condition
      : "");
  const archiveAfterDays =
    formState.archiveAfterDays ??
    (typeof openWorkflow?.configuration.archiveAfterDays === "number"
      ? openWorkflow.configuration.archiveAfterDays
      : null);
  const closeOnStatus =
    formState.closeOnStatus ??
    (typeof openWorkflow?.configuration.closeOnStatus === "boolean"
      ? openWorkflow.configuration.closeOnStatus
      : false);

  function openEditor(workflowId: string) {
    setOpenWorkflowId(workflowId);
    setFormState({});
    setSaveState({ status: "idle", message: null });
  }

  async function saveWorkflow(enabled?: boolean) {
    if (!openWorkflow || !canManage || saveState.status === "saving") return;
    setSaveState({ status: "saving", message: null });
    const request: ProjectWorkflowUpdateRequest = {
      ...formState,
      enabled,
      condition,
      repositoryTargetIds: selectedRepositoryIds,
      archiveAfterDays,
      closeOnStatus,
      expectedUpdatedAt: openWorkflow.updatedAt,
    };
    if (selectedFieldId && selectedOptionId) {
      request.statusFieldId = selectedFieldId;
      request.statusOptionId = selectedOptionId;
    }
    const response = await fetch(
      `/api/projects/${encodeURIComponent(currentSettings.project.id)}/workflows/${encodeURIComponent(openWorkflow.id)}`,
      {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(request),
      },
    );
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setSaveState({
        status: "error",
        message:
          payload?.error?.message ?? "Project workflow could not be saved.",
      });
      return;
    }
    setCurrentSettings(payload as ProjectWorkflowSettings);
    setFormState({});
    setSaveState({ status: "saved", message: "Workflow configuration saved." });
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
          href={workspaceHref(scope, owner, currentSettings.project.number)}
        >
          Back to project
        </Link>
        <span className="chip active">Settings</span>
        <span className="t-xs t-mono-sm">
          #{currentSettings.project.number}
        </span>
      </div>

      <div
        className="row"
        style={{ alignItems: "flex-start", gap: 18, marginBottom: 24 }}
      >
        <div style={{ flex: 1, minWidth: 0 }}>
          <div className="t-label">Project workflows</div>
          <h1 className="t-h1" style={{ marginTop: 6 }}>
            {currentSettings.project.title}
          </h1>
          <p
            className="t-sm"
            style={{ color: "var(--ink-3)", maxWidth: 760, marginTop: 8 }}
          >
            Built-in automation keeps status, archive state, and linked issue or
            pull request transitions in sync.
          </p>
        </div>
        <span className={canManage ? "chip accent" : "chip soft"}>
          {canManage ? "Editable" : "Read-only"}
        </span>
      </div>

      {currentSettings.unavailableReason ? (
        <div
          className="card"
          role="alert"
          style={{ padding: 16, marginBottom: 18 }}
        >
          <div className="t-label">Unavailable</div>
          <p className="t-sm" style={{ marginTop: 6, color: "var(--ink-3)" }}>
            {currentSettings.unavailableReason}
          </p>
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
                className={
                  item.key === "workflows" ? "btn ghost active" : "btn ghost"
                }
                href={
                  item.key === "fields"
                    ? fieldSettingsHref(
                        scope,
                        owner,
                        currentSettings.project.number,
                      )
                    : workflowSettingsHref(
                        scope,
                        owner,
                        currentSettings.project.number,
                      )
                }
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
            gridTemplateColumns: "minmax(0, 1fr) minmax(280px, 360px)",
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
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="t-label">Default workflows</div>
                <div
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 4 }}
                >
                  {
                    currentSettings.workflows.filter(
                      (workflow) => workflow.enabled,
                    ).length
                  }{" "}
                  enabled · {currentSettings.automationActor}
                </div>
              </div>
              <span className="chip soft">
                {currentSettings.repositoryTargets.length} repositories
              </span>
            </div>

            {currentSettings.workflows.length > 0 ? (
              currentSettings.workflows.map((workflow) => (
                <article
                  className="list-row"
                  key={workflow.id}
                  style={{
                    alignItems: "flex-start",
                    padding: "18px 16px",
                    gap: 14,
                  }}
                >
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
                      <Link
                        className="t-sm"
                        href={workflowSettingsHref(
                          scope,
                          owner,
                          currentSettings.project.number,
                          workflow.id,
                        )}
                        onClick={() => openEditor(workflow.id)}
                        style={{
                          color: "var(--ink-1)",
                          fontWeight: 600,
                          textDecoration: "none",
                        }}
                      >
                        {workflow.name}
                      </Link>
                      <span className={statusChip(workflow)}>
                        {workflow.enabled ? "Enabled" : "Disabled"}
                      </span>
                      <span className="chip soft">
                        {triggerLabel(workflow.triggerEvent)}
                      </span>
                    </div>
                    <p
                      className="t-sm"
                      style={{ color: "var(--ink-3)", marginTop: 7 }}
                    >
                      {workflow.description}
                    </p>
                    <div
                      className="row"
                      style={{
                        gap: 8,
                        marginTop: 10,
                        flexWrap: "wrap",
                        color: "var(--ink-3)",
                      }}
                    >
                      <span className="t-xs">
                        Last run: {formatDate(workflow.lastRunAt)}
                      </span>
                      {workflow.lastRunMessage ? (
                        <span className="t-xs">{workflow.lastRunMessage}</span>
                      ) : null}
                      <span className="t-xs t-mono-sm">
                        {workflow.actorLabel}
                      </span>
                    </div>
                  </div>
                  <div
                    className="row"
                    style={{
                      gap: 8,
                      flexWrap: "wrap",
                      justifyContent: "flex-end",
                    }}
                  >
                    <button
                      className="btn sm"
                      onClick={() => openEditor(workflow.id)}
                      type="button"
                    >
                      Edit
                    </button>
                    <button
                      className={workflow.enabled ? "btn sm" : "btn sm primary"}
                      disabled={!canManage}
                      onClick={() => openEditor(workflow.id)}
                      type="button"
                    >
                      {workflow.enabled ? "Review" : "Turn on"}
                    </button>
                  </div>
                </article>
              ))
            ) : (
              <div style={{ padding: 20 }}>
                <div className="t-label">No workflows</div>
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 6 }}
                >
                  Default project automation will appear here once the project
                  has fields that support workflow rules.
                </p>
              </div>
            )}
          </div>

          <aside className="card" style={{ padding: 18 }}>
            {openWorkflow ? (
              <>
                <div className="t-label">Workflow editor</div>
                <h2 className="t-h2" style={{ marginTop: 6 }}>
                  {openWorkflow.name}
                </h2>
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 8 }}
                >
                  Configure the condition, target status, repositories, and
                  lifecycle behavior that this built-in workflow should use.
                </p>
                <div style={{ marginTop: 18 }}>
                  <label className="t-label" htmlFor="workflow-trigger">
                    Event
                  </label>
                  <input
                    className="input"
                    disabled
                    id="workflow-trigger"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={triggerLabel(openWorkflow.triggerEvent)}
                  />
                </div>
                <div style={{ marginTop: 16 }}>
                  <label className="t-label" htmlFor="workflow-condition">
                    Condition
                  </label>
                  <input
                    className="input"
                    disabled={!canManage}
                    id="workflow-condition"
                    onChange={(event) =>
                      setFormState((current) => ({
                        ...current,
                        condition: event.target.value,
                      }))
                    }
                    placeholder="state:closed label:ready"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={condition}
                  />
                </div>
                <div style={{ marginTop: 16 }}>
                  <label className="t-label" htmlFor="workflow-field">
                    Target field
                  </label>
                  <select
                    className="input"
                    disabled={!canManage}
                    id="workflow-field"
                    onChange={(event) =>
                      setFormState((current) => ({
                        ...current,
                        statusFieldId: event.target.value || null,
                        statusOptionId: null,
                      }))
                    }
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={selectedFieldId}
                  >
                    <option value="">No field selected</option>
                    {currentSettings.eligibleFields
                      .filter((field) => field.supportsStatusTarget)
                      .map((field) => (
                        <option key={field.id} value={field.id}>
                          {field.name}
                        </option>
                      ))}
                  </select>
                </div>
                <div style={{ marginTop: 16 }}>
                  <label className="t-label" htmlFor="workflow-option">
                    Target value
                  </label>
                  <select
                    className="input"
                    disabled={!canManage || !selectedField}
                    id="workflow-option"
                    onChange={(event) =>
                      setFormState((current) => ({
                        ...current,
                        statusOptionId: event.target.value || null,
                      }))
                    }
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={selectedOptionId}
                  >
                    <option value="">No value selected</option>
                    {selectedField?.options.map((option) => (
                      <option key={option.id} value={option.id}>
                        {option.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div style={{ marginTop: 16 }}>
                  <div className="t-label">Rules</div>
                  <div style={{ marginTop: 8 }}>
                    {openWorkflow.rules.length > 0 ? (
                      openWorkflow.rules.map((rule) => (
                        <div
                          className="chip soft"
                          key={rule.id}
                          style={{ margin: "0 6px 6px 0" }}
                        >
                          {rule.ruleType.replaceAll("_", " ")}
                        </div>
                      ))
                    ) : (
                      <span className="t-xs">
                        Uses the default event condition.
                      </span>
                    )}
                  </div>
                </div>
                <div style={{ marginTop: 16 }}>
                  <div className="t-label">Repository targets</div>
                  <div style={{ display: "grid", gap: 8, marginTop: 8 }}>
                    {currentSettings.repositoryTargets.length > 0 ? (
                      currentSettings.repositoryTargets.map((target) => (
                        <label className="chip soft" key={target.id}>
                          <input
                            checked={selectedRepositoryIds.includes(target.id)}
                            disabled={!canManage}
                            onChange={(event) =>
                              setFormState((current) => {
                                const previous =
                                  current.repositoryTargetIds ??
                                  openWorkflow.repositoryTargetIds;
                                return {
                                  ...current,
                                  repositoryTargetIds: event.target.checked
                                    ? [...previous, target.id]
                                    : previous.filter((id) => id !== target.id),
                                };
                              })
                            }
                            style={{ marginRight: 6 }}
                            type="checkbox"
                          />
                          {target.fullName}
                        </label>
                      ))
                    ) : (
                      <span className="t-xs">All visible project items.</span>
                    )}
                  </div>
                </div>
                <div style={{ marginTop: 16 }}>
                  <label className="t-label" htmlFor="workflow-archive-days">
                    Archive criteria
                  </label>
                  <input
                    className="input"
                    disabled={!canManage}
                    id="workflow-archive-days"
                    min={1}
                    max={365}
                    onChange={(event) =>
                      setFormState((current) => ({
                        ...current,
                        archiveAfterDays: event.target.value
                          ? Number(event.target.value)
                          : null,
                      }))
                    }
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    type="number"
                    value={archiveAfterDays ?? ""}
                  />
                </div>
                <label className="chip soft" style={{ marginTop: 14 }}>
                  <input
                    checked={closeOnStatus}
                    disabled={!canManage}
                    onChange={(event) =>
                      setFormState((current) => ({
                        ...current,
                        closeOnStatus: event.target.checked,
                      }))
                    }
                    style={{ marginRight: 6 }}
                    type="checkbox"
                  />
                  Close linked issue or pull request when status matches
                </label>
                <div
                  className="row"
                  style={{ gap: 8, marginTop: 20, flexWrap: "wrap" }}
                >
                  <button
                    className="btn primary"
                    disabled={!canManage || saveState.status === "saving"}
                    onClick={() => saveWorkflow(openWorkflow.enabled)}
                    type="button"
                  >
                    {saveState.status === "saving" ? "Saving" : "Save workflow"}
                  </button>
                  <button
                    className="btn accent"
                    disabled={!canManage || saveState.status === "saving"}
                    onClick={() => saveWorkflow(!openWorkflow.enabled)}
                    type="button"
                  >
                    {openWorkflow.enabled ? "Turn off" : "Save and turn on"}
                  </button>
                  <button
                    className="btn"
                    disabled={!canManage}
                    onClick={() => setOpenWorkflowId(null)}
                    type="button"
                  >
                    Close
                  </button>
                </div>
                <p
                  aria-atomic="true"
                  aria-live="polite"
                  className="t-xs"
                  role="status"
                  style={{
                    color: "var(--ink-3)",
                    marginTop: 10,
                    minHeight: 16,
                  }}
                >
                  {saveState.status === "saved" ? saveState.message : ""}
                </p>
                {saveState.status === "error" && saveState.message ? (
                  <p
                    className="t-xs"
                    role="alert"
                    style={{
                      color: "var(--err)",
                      marginTop: 10,
                    }}
                  >
                    {saveState.message}
                  </p>
                ) : null}
                {!canManage ? (
                  <p className="t-xs" style={{ marginTop: 10 }}>
                    You can inspect this workflow, but project write access is
                    required to change it.
                  </p>
                ) : (
                  <p className="t-xs" style={{ marginTop: 10 }}>
                    Saved workflows write an activity log and audit event before
                    later phases execute item automation.
                  </p>
                )}
              </>
            ) : (
              <>
                <div className="t-label">Activity</div>
                <h2 className="t-h2" style={{ marginTop: 6 }}>
                  Recent automation
                </h2>
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 8 }}
                >
                  Runs are attributed to {currentSettings.automationActor} and
                  keep repository permissions intact.
                </p>
                <div style={{ marginTop: 14 }}>
                  {currentSettings.recentLogs.length > 0 ? (
                    currentSettings.recentLogs.slice(0, 6).map((log) => (
                      <div
                        className="list-row"
                        key={log.id}
                        style={{ padding: "12px 0", alignItems: "flex-start" }}
                      >
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div className="row" style={{ gap: 8 }}>
                            <span
                              className={
                                log.status === "failed"
                                  ? "chip err"
                                  : log.status === "skipped"
                                    ? "chip warn"
                                    : "chip ok"
                              }
                            >
                              {log.status}
                            </span>
                            <span className="t-xs">
                              {triggerLabel(log.eventType)}
                            </span>
                          </div>
                          <p
                            className="t-xs"
                            style={{ marginTop: 6, color: "var(--ink-3)" }}
                          >
                            {log.message ?? "Workflow completed."}
                          </p>
                        </div>
                        <span
                          className="t-mono-sm"
                          style={{ color: "var(--ink-4)" }}
                        >
                          {formatDate(log.createdAt)}
                        </span>
                      </div>
                    ))
                  ) : (
                    <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                      No workflow executions have been recorded yet.
                    </p>
                  )}
                </div>
              </>
            )}
          </aside>
        </section>
      </div>

      <div className="sr-only">
        {Array.from(fieldsById.values())
          .map((field) => field.name)
          .join(", ")}
      </div>
    </main>
  );
}
