"use client";

import Link from "next/link";
import { type FormEvent, useState } from "react";
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

type ProjectTemplateSettingsPageProps = {
  settings: ProjectSettings;
  scope: "user" | "organization";
  owner: string;
};

type NavKey =
  | "general"
  | "access"
  | "fields"
  | "workflows"
  | "templates"
  | "danger";

function navHref(
  scope: "user" | "organization",
  owner: string,
  number: number,
  key: NavKey,
) {
  if (scope === "organization") {
    if (key === "access")
      return organizationProjectAccessSettingsHref(owner, number);
    if (key === "fields")
      return organizationProjectFieldSettingsHref(owner, number);
    if (key === "workflows")
      return organizationProjectWorkflowSettingsHref(owner, number);
    if (key === "templates")
      return organizationProjectTemplateSettingsHref(owner, number);
    if (key === "danger")
      return organizationProjectDangerSettingsHref(owner, number);
    return organizationProjectSettingsHref(owner, number);
  }
  if (key === "access") return userProjectAccessSettingsHref(owner, number);
  if (key === "fields") return userProjectFieldSettingsHref(owner, number);
  if (key === "workflows")
    return userProjectWorkflowSettingsHref(owner, number);
  if (key === "templates")
    return userProjectTemplateSettingsHref(owner, number);
  if (key === "danger") return userProjectDangerSettingsHref(owner, number);
  return userProjectSettingsHref(owner, number);
}

function workspaceHref(
  scope: "user" | "organization",
  owner: string,
  number: number,
) {
  return scope === "organization"
    ? organizationProjectWorkspaceHref(owner, number, 1)
    : userProjectWorkspaceHref(owner, number, 1);
}

function formatDate(value: string | null) {
  if (!value) return "Not set";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

export function ProjectTemplateSettingsPage({
  settings,
  scope,
  owner,
}: ProjectTemplateSettingsPageProps) {
  const [currentSettings, setCurrentSettings] = useState(settings);
  const [pending, setPending] = useState(false);
  const [feedback, setFeedback] = useState<{
    kind: "success" | "error";
    message: string;
  } | null>(null);
  settings = currentSettings;
  const projectNumber = settings.project.number;
  const canManage =
    settings.viewerPermissions.canManageTemplate &&
    settings.dangerState.state === "open";

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setPending(true);
    setFeedback(null);
    try {
      const response = await fetch(
        `/api/projects/${encodeURIComponent(settings.project.id)}/template`,
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            isTemplate: form.get("isTemplate") === "on",
            title: form.get("templateTitle"),
            description: form.get("templateDescription"),
            isPublic: form.get("isPublic") === "on",
            expectedUpdatedAt: settings.general.updatedAt,
          }),
        },
      );
      const payload = await response.json().catch(() => null);
      if (!response.ok)
        throw new Error(
          payload?.error?.message ?? "Template settings could not be saved.",
        );
      setCurrentSettings(payload as ProjectSettings);
      setFeedback({ kind: "success", message: "Template settings saved." });
    } catch (error) {
      setFeedback({
        kind: "error",
        message:
          error instanceof Error
            ? error.message
            : "Template settings could not be saved.",
      });
    } finally {
      setPending(false);
    }
  }

  const navItems: Array<{ key: NavKey; label: string }> = [
    { key: "general", label: "General" },
    { key: "access", label: "Access" },
    { key: "fields", label: "Fields" },
    { key: "workflows", label: "Workflows" },
    { key: "templates", label: "Templates" },
    { key: "danger", label: "Danger Zone" },
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
        <span className="chip active">Templates</span>
        <span className="chip soft">
          {settings.viewerPermissions.viewerRole ?? "visitor"}
        </span>
      </div>

      <div className="mb-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Project settings
        </p>
        <h1 className="t-h1 mt-2">{settings.project.title}</h1>
        <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
          Control whether this project can be reused as a copy source, and
          explain what the template includes.
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
          {navItems.map((item) =>
            item.key === "templates" ? (
              <span
                className="list-row"
                aria-current="page"
                key={item.key}
                style={{ color: "var(--accent)" }}
              >
                {item.label}
              </span>
            ) : (
              <Link
                className="list-row"
                href={navHref(scope, owner, projectNumber, item.key)}
                key={item.key}
              >
                {item.label}
              </Link>
            ),
          )}
        </nav>

        <form className="card p-5" onSubmit={handleSubmit}>
          <div
            className="row between"
            style={{ gap: 16, alignItems: "flex-start", flexWrap: "wrap" }}
          >
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Template source
              </p>
              <h2 className="t-h2 mt-2">Copy-source settings</h2>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                Templates keep fields, views, workflows, repository links, and
                README guidance ready for future projects.
              </p>
            </div>
            <span
              className={settings.template.isTemplate ? "chip ok" : "chip soft"}
            >
              {settings.template.isTemplate ? "Template" : "Standard project"}
            </span>
          </div>

          {!canManage ? (
            <p className="t-sm mt-4" style={{ color: "var(--ink-3)" }}>
              Only project admins can manage template settings for open
              projects.
            </p>
          ) : null}

          <label
            className="row t-sm mt-5"
            style={{ gap: 8, justifyContent: "flex-start" }}
          >
            <input
              defaultChecked={settings.template.isTemplate}
              disabled={!canManage}
              name="isTemplate"
              type="checkbox"
            />
            Set this project as a template
          </label>

          <div
            className="grid mt-4"
            style={{
              gap: 16,
              gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
            }}
          >
            <label>
              <span className="t-label">Template title</span>
              <input
                className="input mt-2"
                defaultValue={settings.template.title ?? settings.general.title}
                disabled={!canManage}
                name="templateTitle"
              />
            </label>
            <label>
              <span className="t-label">Copy visibility</span>
              <span
                className="row t-sm mt-2"
                style={{ gap: 8, justifyContent: "flex-start" }}
              >
                <input
                  aria-label="Allow copies from visible users"
                  defaultChecked={settings.template.isPublic}
                  disabled={!canManage}
                  name="isPublic"
                  type="checkbox"
                />
                Allow copies from visible users
              </span>
            </label>
          </div>

          <label className="block mt-4">
            <span className="t-label">Copy-source information</span>
            <textarea
              className="input mt-2"
              defaultValue={settings.template.description ?? ""}
              disabled={!canManage}
              name="templateDescription"
              rows={5}
              style={{ resize: "vertical", width: "100%" }}
            />
          </label>

          <div className="card mt-5 p-4">
            <div className="t-label">Current template record</div>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {settings.template.templateId
                ? `Template ${settings.template.templateId}`
                : "No reusable template has been published yet."}
            </p>
            <p className="t-xs mt-2">
              Created {formatDate(settings.template.createdAt)}
            </p>
          </div>

          <button
            className="btn primary mt-5"
            disabled={!canManage || pending}
            type="submit"
          >
            {pending ? "Saving..." : "Save template"}
          </button>
        </form>
      </div>
    </main>
  );
}
