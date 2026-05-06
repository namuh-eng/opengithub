"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type {
  ProjectWorkflowDefinition,
  ProjectWorkflowSettings,
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
  const openWorkflow =
    settings.workflows.find((workflow) => workflow.id === openWorkflowId) ??
    null;
  const canManage = settings.viewerPermissions.canManageWorkflows;
  const fieldsById = new Map(
    settings.eligibleFields.map((field) => [field.id, field]),
  );

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
          <div className="t-label">Project workflows</div>
          <h1 className="t-h1" style={{ marginTop: 6 }}>
            {settings.project.title}
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
                    ? fieldSettingsHref(scope, owner, settings.project.number)
                    : workflowSettingsHref(
                        scope,
                        owner,
                        settings.project.number,
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
                    settings.workflows.filter((workflow) => workflow.enabled)
                      .length
                  }{" "}
                  enabled · {settings.automationActor}
                </div>
              </div>
              <span className="chip soft">
                {settings.repositoryTargets.length} repositories
              </span>
            </div>

            {settings.workflows.length > 0 ? (
              settings.workflows.map((workflow) => (
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
                          settings.project.number,
                          workflow.id,
                        )}
                        onClick={() => setOpenWorkflowId(workflow.id)}
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
                      onClick={() => setOpenWorkflowId(workflow.id)}
                      type="button"
                    >
                      Edit
                    </button>
                    <button
                      className={workflow.enabled ? "btn sm" : "btn sm primary"}
                      disabled={!canManage}
                      onClick={() => setOpenWorkflowId(workflow.id)}
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
                  Rule editing persists in the next implementation phase. This
                  panel is route-backed now so every card has a concrete edit
                  target.
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
                  <label className="t-label" htmlFor="workflow-field">
                    Target field
                  </label>
                  <select
                    className="input"
                    disabled
                    id="workflow-field"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    value={
                      typeof openWorkflow.configuration.statusFieldId ===
                      "string"
                        ? openWorkflow.configuration.statusFieldId
                        : ""
                    }
                  >
                    <option value="">No field selected</option>
                    {settings.eligibleFields.map((field) => (
                      <option key={field.id} value={field.id}>
                        {field.name}
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
                  <div style={{ marginTop: 8 }}>
                    {openWorkflow.repositoryTargetIds.length > 0 ? (
                      openWorkflow.repositoryTargetIds.map((targetId) => {
                        const target = settings.repositoryTargets.find(
                          (repository) => repository.id === targetId,
                        );
                        return (
                          <span
                            className="chip soft"
                            key={targetId}
                            style={{ margin: "0 6px 6px 0" }}
                          >
                            {target?.fullName ?? targetId}
                          </span>
                        );
                      })
                    ) : (
                      <span className="t-xs">All visible project items.</span>
                    )}
                  </div>
                </div>
                <div
                  className="row"
                  style={{ gap: 8, marginTop: 20, flexWrap: "wrap" }}
                >
                  <button className="btn primary" disabled type="button">
                    Save workflow
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
                {!canManage ? (
                  <p className="t-xs" style={{ marginTop: 10 }}>
                    You can inspect this workflow, but project write access is
                    required to change it.
                  </p>
                ) : (
                  <p className="t-xs" style={{ marginTop: 10 }}>
                    Save is disabled until workflow mutations are added.
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
                  Runs are attributed to {settings.automationActor} and keep
                  repository permissions intact.
                </p>
                <div style={{ marginTop: 14 }}>
                  {settings.recentLogs.length > 0 ? (
                    settings.recentLogs.slice(0, 6).map((log) => (
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
