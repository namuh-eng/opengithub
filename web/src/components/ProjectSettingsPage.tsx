"use client";

import Link from "next/link";
import type { ProjectSettings } from "@/lib/api";
import {
  organizationProjectAccessSettingsHref,
  organizationProjectDangerSettingsHref,
  organizationProjectFieldSettingsHref,
  organizationProjectSettingsHref,
  organizationProjectTemplateSettingsHref,
  organizationProjectWorkflowSettingsHref,
  organizationProjectWorkspaceHref,
  userProjectAccessSettingsHref,
  userProjectDangerSettingsHref,
  userProjectFieldSettingsHref,
  userProjectSettingsHref,
  userProjectTemplateSettingsHref,
  userProjectWorkflowSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectSettingsPageProps = {
  settings: ProjectSettings;
  scope: "user" | "organization";
  owner: string;
};

type SettingsNavKey =
  | "general"
  | "access"
  | "fields"
  | "workflows"
  | "templates"
  | "danger";

const STATUS_OPTIONS = ["on_track", "at_risk", "off_track", "complete"];

function settingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  key: SettingsNavKey,
) {
  if (scope === "organization") {
    if (key === "access") {
      return organizationProjectAccessSettingsHref(owner, projectNumber);
    }
    if (key === "fields") {
      return organizationProjectFieldSettingsHref(owner, projectNumber);
    }
    if (key === "workflows") {
      return organizationProjectWorkflowSettingsHref(owner, projectNumber);
    }
    if (key === "templates") {
      return organizationProjectTemplateSettingsHref(owner, projectNumber);
    }
    if (key === "danger") {
      return organizationProjectDangerSettingsHref(owner, projectNumber);
    }
    return organizationProjectSettingsHref(owner, projectNumber);
  }
  if (key === "access")
    return userProjectAccessSettingsHref(owner, projectNumber);
  if (key === "fields")
    return userProjectFieldSettingsHref(owner, projectNumber);
  if (key === "workflows") {
    return userProjectWorkflowSettingsHref(owner, projectNumber);
  }
  if (key === "templates") {
    return userProjectTemplateSettingsHref(owner, projectNumber);
  }
  if (key === "danger")
    return userProjectDangerSettingsHref(owner, projectNumber);
  return userProjectSettingsHref(owner, projectNumber);
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
  if (!value) return "Not set";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function statusLabel(value: string) {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1)}`)
    .join(" ");
}

function statusChipClass(value: string) {
  if (value === "on_track" || value === "complete") return "chip ok";
  if (value === "at_risk") return "chip warn";
  if (value === "off_track") return "chip err";
  return "chip soft";
}

function disabledReason(canEdit: boolean, fallback: string) {
  return canEdit ? null : fallback;
}

export function ProjectSettingsPage({
  settings,
  scope,
  owner,
}: ProjectSettingsPageProps) {
  const canEditGeneral =
    settings.viewerPermissions.canEditGeneral &&
    settings.dangerState.state !== "deleted";
  const canChangeVisibility =
    canEditGeneral &&
    settings.viewerPermissions.canChangeVisibility &&
    settings.policy.visibilityChangesAllowed;
  const canPublishStatus =
    settings.viewerPermissions.canPublishStatus &&
    settings.dangerState.state === "open";
  const latestStatus = settings.statusUpdates[0] ?? null;
  const visibilityReason =
    settings.policy.visibilityLockedReason ??
    "This organization policy prevents visibility changes.";
  const projectNumber = settings.project.number;
  const navItems: Array<{
    key: SettingsNavKey;
    label: string;
    disabled: boolean;
  }> = [
    { key: "general", label: "General", disabled: false },
    { key: "access", label: "Access", disabled: true },
    { key: "fields", label: "Fields", disabled: false },
    { key: "workflows", label: "Workflows", disabled: false },
    { key: "templates", label: "Templates", disabled: true },
    { key: "danger", label: "Danger Zone", disabled: true },
  ];

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
          href={workspaceHref(scope, owner, projectNumber)}
        >
          Back to project
        </Link>
        <span className="chip active">Settings</span>
        <span className="t-xs t-mono-sm">#{projectNumber}</span>
        <span className="t-xs">
          {settings.viewerPermissions.viewerRole ?? "viewer"}
        </span>
      </div>

      <header style={{ marginBottom: 24 }}>
        <div className="t-label">Project settings</div>
        <h1 className="t-h1" style={{ marginTop: 6 }}>
          {settings.general.title}
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)", marginTop: 8 }}>
          Manage the public metadata, repository defaults, and status signal for
          this project. Destructive and access changes are kept in their own
          settings sections.
        </p>
      </header>

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
          {navItems.map((item) =>
            item.disabled ? (
              <button
                className="btn ghost"
                disabled
                key={item.key}
                style={{
                  justifyContent: "flex-start",
                  marginTop: 2,
                  width: "100%",
                }}
                type="button"
              >
                {item.label}
              </button>
            ) : (
              <Link
                className={
                  item.key === "general" ? "btn ghost active" : "btn ghost"
                }
                href={settingsHref(scope, owner, projectNumber, item.key)}
                key={item.key}
                style={{
                  justifyContent: "flex-start",
                  marginTop: 2,
                  width: "100%",
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
          <div style={{ display: "grid", gap: 18 }}>
            {settings.unavailableReason ? (
              <div className="card" style={{ padding: 16 }}>
                <div className="t-label">Unavailable</div>
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 6 }}
                >
                  {settings.unavailableReason}
                </p>
              </div>
            ) : null}

            <form
              action={settingsHref(scope, owner, projectNumber, "general")}
              className="card"
              method="post"
              style={{ padding: 18 }}
            >
              <div className="row" style={{ gap: 10, flexWrap: "wrap" }}>
                <div style={{ flex: 1, minWidth: 220 }}>
                  <div className="t-label">General</div>
                  <h2 className="t-h2" style={{ marginTop: 6 }}>
                    Project metadata
                  </h2>
                </div>
                <span className="chip soft">{settings.general.visibility}</span>
              </div>

              <div style={{ marginTop: 18 }}>
                <label className="t-label" htmlFor="project-title">
                  Title
                </label>
                <input
                  className="input"
                  defaultValue={settings.general.title}
                  disabled={!canEditGeneral}
                  id="project-title"
                  name="title"
                  style={{ display: "block", marginTop: 8, width: "100%" }}
                />
              </div>

              <div style={{ marginTop: 16 }}>
                <label className="t-label" htmlFor="project-description">
                  Short description
                </label>
                <input
                  className="input"
                  defaultValue={settings.general.description ?? ""}
                  disabled={!canEditGeneral}
                  id="project-description"
                  name="description"
                  placeholder="How this project is used"
                  style={{ display: "block", marginTop: 8, width: "100%" }}
                />
              </div>

              <div style={{ marginTop: 16 }}>
                <label className="t-label" htmlFor="project-readme">
                  README Markdown
                </label>
                <textarea
                  className="input"
                  defaultValue={settings.general.readme ?? ""}
                  disabled={!canEditGeneral}
                  id="project-readme"
                  name="readme"
                  rows={8}
                  style={{
                    display: "block",
                    marginTop: 8,
                    resize: "vertical",
                    width: "100%",
                  }}
                />
                <p className="t-xs" style={{ marginTop: 6 }}>
                  {settings.general.readmeRevisionCount} saved revisions ·
                  updated {formatDate(settings.general.updatedAt)}
                </p>
              </div>

              <div
                style={{
                  display: "grid",
                  gap: 16,
                  gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
                  marginTop: 16,
                }}
              >
                <div>
                  <label className="t-label" htmlFor="project-visibility">
                    Visibility
                  </label>
                  <select
                    className="input"
                    defaultValue={settings.general.visibility}
                    disabled={!canChangeVisibility}
                    id="project-visibility"
                    name="visibility"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                  >
                    <option value="private">Private</option>
                    <option value="public">Public</option>
                  </select>
                  {canChangeVisibility ? null : (
                    <p className="t-xs" style={{ marginTop: 6 }}>
                      {visibilityReason}
                    </p>
                  )}
                </div>
                <div>
                  <label className="t-label" htmlFor="default-repository">
                    Default repository
                  </label>
                  <select
                    className="input"
                    defaultValue={settings.general.defaultRepositoryId ?? ""}
                    disabled={
                      !canEditGeneral || settings.repositories.length === 0
                    }
                    id="default-repository"
                    name="defaultRepositoryId"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                  >
                    <option value="">No default repository</option>
                    {settings.repositories.map((repository) => (
                      <option
                        key={repository.repositoryId}
                        value={repository.repositoryId}
                      >
                        {repository.fullName}
                      </option>
                    ))}
                  </select>
                  <p className="t-xs" style={{ marginTop: 6 }}>
                    New linked issues route through this repository when one is
                    selected.
                  </p>
                </div>
              </div>

              {disabledReason(
                canEditGeneral,
                "You can inspect settings, but this project role cannot change metadata.",
              ) ? (
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 16 }}
                >
                  {disabledReason(
                    canEditGeneral,
                    "You can inspect settings, but this project role cannot change metadata.",
                  )}
                </p>
              ) : null}

              <div className="row" style={{ gap: 10, marginTop: 18 }}>
                <button
                  className="btn primary"
                  disabled={!canEditGeneral}
                  type="submit"
                >
                  Save changes
                </button>
                <Link
                  className="btn"
                  href={workspaceHref(scope, owner, projectNumber)}
                >
                  View project
                </Link>
              </div>
            </form>

            <form
              action={`${settingsHref(scope, owner, projectNumber, "general")}#status`}
              className="card"
              id="status"
              method="post"
              style={{ padding: 18 }}
            >
              <div className="row" style={{ gap: 10, flexWrap: "wrap" }}>
                <div style={{ flex: 1, minWidth: 220 }}>
                  <div className="t-label">Status update</div>
                  <h2 className="t-h2" style={{ marginTop: 6 }}>
                    Publish project health
                  </h2>
                </div>
                {latestStatus ? (
                  <span className={statusChipClass(latestStatus.status)}>
                    {latestStatus.label}
                  </span>
                ) : (
                  <span className="chip soft">No update</span>
                )}
              </div>

              <div
                style={{
                  display: "grid",
                  gap: 16,
                  gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
                  marginTop: 18,
                }}
              >
                <div>
                  <label className="t-label" htmlFor="status-state">
                    State
                  </label>
                  <select
                    className="input"
                    disabled={!canPublishStatus}
                    id="status-state"
                    name="status"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    defaultValue={latestStatus?.status ?? "on_track"}
                  >
                    {STATUS_OPTIONS.map((status) => (
                      <option key={status} value={status}>
                        {statusLabel(status)}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="t-label" htmlFor="status-start-date">
                    Start date
                  </label>
                  <input
                    className="input"
                    defaultValue={latestStatus?.startDate ?? ""}
                    disabled={!canPublishStatus}
                    id="status-start-date"
                    name="startDate"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    type="date"
                  />
                </div>
                <div>
                  <label className="t-label" htmlFor="status-target-date">
                    Target date
                  </label>
                  <input
                    className="input"
                    defaultValue={latestStatus?.targetDate ?? ""}
                    disabled={!canPublishStatus}
                    id="status-target-date"
                    name="targetDate"
                    style={{ display: "block", marginTop: 8, width: "100%" }}
                    type="date"
                  />
                </div>
              </div>

              <div style={{ marginTop: 16 }}>
                <label className="t-label" htmlFor="status-message">
                  Message
                </label>
                <textarea
                  className="input"
                  defaultValue={latestStatus?.body ?? ""}
                  disabled={!canPublishStatus}
                  id="status-message"
                  name="body"
                  placeholder="Summarize progress, risks, or next checkpoints."
                  rows={5}
                  style={{
                    display: "block",
                    marginTop: 8,
                    resize: "vertical",
                    width: "100%",
                  }}
                />
              </div>

              <div className="row" style={{ gap: 10, marginTop: 18 }}>
                <button
                  className="btn primary"
                  disabled={!canPublishStatus}
                  type="submit"
                >
                  Publish update
                </button>
                {canPublishStatus ? null : (
                  <span className="t-xs">
                    Status updates require project write access and an open
                    project.
                  </span>
                )}
              </div>
            </form>
          </div>

          <aside style={{ display: "grid", gap: 18 }}>
            <section className="card" style={{ padding: 18 }}>
              <div className="t-label">Policy</div>
              <div
                className="row"
                style={{ gap: 8, marginTop: 12, flexWrap: "wrap" }}
              >
                <span
                  className={
                    settings.policy.projectsEnabled ? "chip ok" : "chip err"
                  }
                >
                  Projects{" "}
                  {settings.policy.projectsEnabled ? "enabled" : "disabled"}
                </span>
                <span className="chip soft">
                  Base {settings.policy.basePermission ?? "none"}
                </span>
              </div>
              <p
                className="t-sm"
                style={{ color: "var(--ink-3)", marginTop: 12 }}
              >
                {settings.policy.ownerKind === "organization"
                  ? "Organization policy shapes visibility, repository links, and base project access."
                  : "User-owned projects use direct project permissions."}
              </p>
              {settings.policy.visibilityLockedReason ? (
                <p className="t-xs" style={{ marginTop: 10 }}>
                  {settings.policy.visibilityLockedReason}
                </p>
              ) : null}
            </section>

            <section className="card" style={{ padding: 18 }}>
              <div className="t-label">Repositories</div>
              <div style={{ display: "grid", gap: 10, marginTop: 12 }}>
                {settings.repositories.length > 0 ? (
                  settings.repositories.map((repository) => (
                    <Link
                      className="list-row"
                      href={repository.href}
                      key={repository.id}
                      style={{ padding: "10px 0", textDecoration: "none" }}
                    >
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div
                          className="t-sm"
                          style={{ color: "var(--ink-1)", fontWeight: 600 }}
                        >
                          {repository.fullName}
                        </div>
                        <div className="t-xs">
                          {repository.visibility} ·{" "}
                          {repository.viewerPermission ?? "no role"}
                        </div>
                      </div>
                      {repository.isDefault ? (
                        <span className="chip active">Default</span>
                      ) : null}
                    </Link>
                  ))
                ) : (
                  <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                    No visible repositories are linked to this project.
                  </p>
                )}
              </div>
            </section>

            <section className="card" style={{ padding: 18 }}>
              <div className="t-label">Latest status</div>
              {latestStatus ? (
                <div style={{ marginTop: 12 }}>
                  <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
                    <span className={statusChipClass(latestStatus.status)}>
                      {latestStatus.label}
                    </span>
                    <span className="t-xs">
                      {formatDate(latestStatus.createdAt)}
                    </span>
                  </div>
                  <p
                    className="t-sm"
                    style={{ color: "var(--ink-3)", marginTop: 10 }}
                  >
                    {latestStatus.body ??
                      "No message was published with this update."}
                  </p>
                  <div className="t-xs" style={{ marginTop: 10 }}>
                    {latestStatus.author?.login ?? "System"} ·{" "}
                    {formatDate(latestStatus.startDate)} to{" "}
                    {formatDate(latestStatus.targetDate)}
                  </div>
                </div>
              ) : (
                <p
                  className="t-sm"
                  style={{ color: "var(--ink-3)", marginTop: 10 }}
                >
                  Publish an update when this project needs an explicit health
                  signal.
                </p>
              )}
            </section>

            <section className="card" style={{ padding: 18 }}>
              <div className="t-label">Template</div>
              <div
                className="row"
                style={{ gap: 8, marginTop: 12, flexWrap: "wrap" }}
              >
                <span
                  className={
                    settings.template.isTemplate ? "chip ok" : "chip soft"
                  }
                >
                  {settings.template.isTemplate ? "Template" : "Project"}
                </span>
                {settings.template.isPublic ? (
                  <span className="chip soft">Public copy source</span>
                ) : null}
              </div>
              <p
                className="t-sm"
                style={{ color: "var(--ink-3)", marginTop: 10 }}
              >
                Template controls move to the Templates section in the next
                project settings slice.
              </p>
            </section>
          </aside>
        </section>
      </div>
    </main>
  );
}
