"use client";

import Link from "next/link";
import { type FormEvent, useState } from "react";
import type { ProjectSettings } from "@/lib/api";
import {
  organizationProjectDangerSettingsHref,
  organizationProjectFieldSettingsHref,
  organizationProjectSettingsHref,
  organizationProjectTemplateSettingsHref,
  organizationProjectWorkflowSettingsHref,
  organizationProjectWorkspaceHref,
  userProjectDangerSettingsHref,
  userProjectFieldSettingsHref,
  userProjectSettingsHref,
  userProjectTemplateSettingsHref,
  userProjectWorkflowSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectAccessSettingsPageProps = {
  settings: ProjectSettings;
  scope: "user" | "organization";
  owner: string;
};

const roles = ["read", "write", "admin"] as const;

function roleLabel(role: string) {
  return role.charAt(0).toUpperCase() + role.slice(1);
}

function roleChipClass(role: string) {
  if (role === "admin") return "chip accent";
  if (role === "write") return "chip ok";
  return "chip soft";
}

function navHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  key: "general" | "fields" | "workflows" | "templates" | "danger",
) {
  if (scope === "organization") {
    if (key === "fields")
      return organizationProjectFieldSettingsHref(owner, projectNumber);
    if (key === "workflows")
      return organizationProjectWorkflowSettingsHref(owner, projectNumber);
    if (key === "templates")
      return organizationProjectTemplateSettingsHref(owner, projectNumber);
    if (key === "danger")
      return organizationProjectDangerSettingsHref(owner, projectNumber);
    return organizationProjectSettingsHref(owner, projectNumber);
  }
  if (key === "fields")
    return userProjectFieldSettingsHref(owner, projectNumber);
  if (key === "workflows")
    return userProjectWorkflowSettingsHref(owner, projectNumber);
  if (key === "templates")
    return userProjectTemplateSettingsHref(owner, projectNumber);
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

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

export function ProjectAccessSettingsPage({
  settings,
  scope,
  owner,
}: ProjectAccessSettingsPageProps) {
  const [currentSettings, setCurrentSettings] = useState(settings);
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{
    kind: "success" | "error";
    message: string;
  } | null>(null);
  settings = currentSettings;
  const projectNumber = settings.project.number;
  const canManage =
    settings.viewerPermissions.canManageAccess &&
    settings.dangerState.state === "open";

  async function submitJson(
    action: string,
    path: string,
    method: "POST" | "PATCH" | "DELETE",
    body: Record<string, unknown>,
    success: string,
  ) {
    setPendingAction(action);
    setFeedback(null);
    try {
      const response = await fetch(path, {
        method,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          payload?.error?.message ?? "Project access could not be changed.",
        );
      }
      setCurrentSettings(payload as ProjectSettings);
      setFeedback({ kind: "success", message: success });
    } catch (error) {
      setFeedback({
        kind: "error",
        message:
          error instanceof Error
            ? error.message
            : "Project access could not be changed.",
      });
    } finally {
      setPendingAction(null);
    }
  }

  function handleAddGrant(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const target = String(form.get("target") ?? "");
    const role = String(form.get("role") ?? "read");
    if (!target.includes(":")) {
      setFeedback({ kind: "error", message: "Choose a collaborator or team." });
      return;
    }
    const [targetType, targetId] = target.split(":");
    void submitJson(
      "add-grant",
      `/api/projects/${encodeURIComponent(settings.project.id)}/access-grants`,
      "POST",
      {
        targetType,
        targetId,
        role,
        expectedUpdatedAt: settings.general.updatedAt,
      },
      "Project access granted.",
    );
  }

  function handleRoleChange(grantId: string, role: string) {
    void submitJson(
      `role-${grantId}`,
      `/api/projects/${encodeURIComponent(settings.project.id)}/access-grants/${encodeURIComponent(grantId)}`,
      "PATCH",
      { role, expectedUpdatedAt: settings.general.updatedAt },
      "Project access role changed.",
    );
  }

  function handleRemove(grantId: string) {
    void submitJson(
      `remove-${grantId}`,
      `/api/projects/${encodeURIComponent(settings.project.id)}/access-grants/${encodeURIComponent(grantId)}`,
      "DELETE",
      { expectedUpdatedAt: settings.general.updatedAt },
      "Project access removed.",
    );
  }

  const targetOptions = [
    ...settings.eligibleUsers.map((user) => ({
      value: `user:${user.id}`,
      label: `User: ${user.login}`,
    })),
    ...settings.eligibleTeams.map((team) => ({
      value: `team:${team.id}`,
      label: `Team: ${team.name}`,
    })),
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
          className="btn sm"
          href={workspaceHref(scope, owner, projectNumber)}
        >
          Back to project
        </Link>
        <span className="chip soft">
          {settings.viewerPermissions.viewerRole ?? "visitor"}
        </span>
        {settings.policy.basePermission ? (
          <span className="chip info">
            Base {settings.policy.basePermission}
          </span>
        ) : null}
      </div>

      <div className="mb-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Project settings
        </p>
        <h1 className="t-h1 mt-2">{settings.project.title}</h1>
        <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
          Manage direct collaborator and team access without changing repository
          permissions.
        </p>
      </div>

      {feedback ? (
        <div
          className={`chip ${feedback.kind === "success" ? "ok" : "err"} mb-4`}
          role="status"
        >
          {feedback.message}
        </div>
      ) : null}

      <div
        className="grid"
        style={{ gridTemplateColumns: "220px minmax(0, 1fr)", gap: 28 }}
      >
        <nav className="card p-3" aria-label="Project settings">
          <Link
            className="list-row"
            href={navHref(scope, owner, projectNumber, "general")}
          >
            General
          </Link>
          <span
            className="list-row"
            aria-current="page"
            style={{ color: "var(--accent)" }}
          >
            Access
          </span>
          <Link
            className="list-row"
            href={navHref(scope, owner, projectNumber, "fields")}
          >
            Fields
          </Link>
          <Link
            className="list-row"
            href={navHref(scope, owner, projectNumber, "workflows")}
          >
            Workflows
          </Link>
          <Link
            className="list-row"
            href={navHref(scope, owner, projectNumber, "templates")}
          >
            Templates
          </Link>
          <Link
            className="list-row"
            href={navHref(scope, owner, projectNumber, "danger")}
          >
            Danger Zone
          </Link>
        </nav>

        <div className="grid gap-5">
          <section className="card p-5">
            <div
              className="row between"
              style={{ gap: 16, alignItems: "flex-start" }}
            >
              <div>
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Grant access
                </p>
                <h2 className="t-h3 mt-2">Add collaborators or teams</h2>
                <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  Organization base permission is inherited; direct grants only
                  raise access for this project.
                </p>
              </div>
              {!canManage ? <span className="chip warn">Read-only</span> : null}
            </div>
            <form className="mt-4 grid gap-3" onSubmit={handleAddGrant}>
              <div
                className="grid"
                style={{
                  gridTemplateColumns: "minmax(0, 1fr) 160px auto",
                  gap: 10,
                }}
              >
                <label>
                  <span className="t-label">Collaborator or team</span>
                  <select
                    className="input mt-2"
                    disabled={!canManage}
                    name="target"
                  >
                    <option value="">Choose a target</option>
                    {targetOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span className="t-label">Role</span>
                  <select
                    className="input mt-2"
                    disabled={!canManage}
                    name="role"
                    defaultValue="read"
                  >
                    {roles.map((role) => (
                      <option key={role} value={role}>
                        {roleLabel(role)}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  className="btn primary self-end"
                  disabled={!canManage || pendingAction === "add-grant"}
                  type="submit"
                >
                  Add access
                </button>
              </div>
            </form>
          </section>

          <section className="card p-5">
            <h2 className="t-h3">Collaborators</h2>
            <div className="mt-4">
              {settings.accessGrants.length === 0 ? (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No direct collaborators have been added.
                </p>
              ) : (
                settings.accessGrants.map((grant) => (
                  <div className="list-row" key={grant.id}>
                    <div
                      className="row between"
                      style={{ gap: 16, flexWrap: "wrap" }}
                    >
                      <div>
                        <div className="row" style={{ gap: 10 }}>
                          <span className="av sm">
                            {grant.user.login.slice(0, 2).toUpperCase()}
                          </span>
                          <div>
                            <div className="t-body">{grant.user.login}</div>
                            <div className="t-xs">
                              {grant.source} grant, updated{" "}
                              {formatDate(grant.updatedAt)}
                            </div>
                          </div>
                        </div>
                      </div>
                      <div className="row" style={{ gap: 10 }}>
                        {grant.inherited ? (
                          <span className="chip soft">Inherited</span>
                        ) : null}
                        <span className={roleChipClass(grant.role)}>
                          {roleLabel(grant.role)}
                        </span>
                        <select
                          aria-label={`Role for ${grant.user.login}`}
                          className="input"
                          disabled={!canManage || grant.inherited}
                          onChange={(event) =>
                            handleRoleChange(
                              grant.id,
                              event.currentTarget.value,
                            )
                          }
                          value={grant.role}
                        >
                          {roles.map((role) => (
                            <option key={role} value={role}>
                              {roleLabel(role)}
                            </option>
                          ))}
                        </select>
                        <button
                          className="btn sm"
                          disabled={
                            !canManage ||
                            grant.inherited ||
                            pendingAction === `remove-${grant.id}`
                          }
                          onClick={() => handleRemove(grant.id)}
                          type="button"
                        >
                          Remove
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </section>

          <section className="card p-5">
            <h2 className="t-h3">Teams</h2>
            <div className="mt-4">
              {settings.teamGrants.length === 0 ? (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No teams have been granted project-specific access.
                </p>
              ) : (
                settings.teamGrants.map((grant) => (
                  <div className="list-row" key={grant.id}>
                    <div
                      className="row between"
                      style={{ gap: 16, flexWrap: "wrap" }}
                    >
                      <div>
                        <Link className="t-body" href={grant.team.href}>
                          {grant.team.name}
                        </Link>
                        <div className="t-xs">
                          {grant.memberCount} members, updated{" "}
                          {formatDate(grant.updatedAt)}
                        </div>
                      </div>
                      <div className="row" style={{ gap: 10 }}>
                        <span className={roleChipClass(grant.role)}>
                          {roleLabel(grant.role)}
                        </span>
                        <select
                          aria-label={`Role for ${grant.team.name}`}
                          className="input"
                          disabled={!canManage}
                          onChange={(event) =>
                            handleRoleChange(
                              grant.id,
                              event.currentTarget.value,
                            )
                          }
                          value={grant.role}
                        >
                          {roles.map((role) => (
                            <option key={role} value={role}>
                              {roleLabel(role)}
                            </option>
                          ))}
                        </select>
                        <button
                          className="btn sm"
                          disabled={
                            !canManage || pendingAction === `remove-${grant.id}`
                          }
                          onClick={() => handleRemove(grant.id)}
                          type="button"
                        >
                          Remove
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}
