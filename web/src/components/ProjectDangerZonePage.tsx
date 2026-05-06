"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { type FormEvent, useState } from "react";
import type { ProjectDeleteResponse, ProjectSettings } from "@/lib/api";
import {
  organizationProjectAccessSettingsHref,
  organizationProjectFieldSettingsHref,
  organizationProjectSettingsHref,
  organizationProjectTemplateSettingsHref,
  organizationProjectWorkflowSettingsHref,
  organizationProjectWorkspaceHref,
  userProjectAccessSettingsHref,
  userProjectFieldSettingsHref,
  userProjectSettingsHref,
  userProjectTemplateSettingsHref,
  userProjectWorkflowSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectDangerZonePageProps = {
  settings: ProjectSettings;
  scope: "user" | "organization";
  owner: string;
};

function settingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  key: "general" | "access" | "fields" | "workflows" | "templates",
) {
  if (scope === "organization") {
    if (key === "access")
      return organizationProjectAccessSettingsHref(owner, projectNumber);
    if (key === "fields")
      return organizationProjectFieldSettingsHref(owner, projectNumber);
    if (key === "workflows")
      return organizationProjectWorkflowSettingsHref(owner, projectNumber);
    if (key === "templates")
      return organizationProjectTemplateSettingsHref(owner, projectNumber);
    return organizationProjectSettingsHref(owner, projectNumber);
  }
  if (key === "access")
    return userProjectAccessSettingsHref(owner, projectNumber);
  if (key === "fields")
    return userProjectFieldSettingsHref(owner, projectNumber);
  if (key === "workflows")
    return userProjectWorkflowSettingsHref(owner, projectNumber);
  if (key === "templates")
    return userProjectTemplateSettingsHref(owner, projectNumber);
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

export function ProjectDangerZonePage({
  settings,
  scope,
  owner,
}: ProjectDangerZonePageProps) {
  const router = useRouter();
  const [currentSettings, setCurrentSettings] = useState(settings);
  const [deleteText, setDeleteText] = useState("");
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{
    kind: "success" | "error";
    message: string;
  } | null>(null);
  settings = currentSettings;
  const projectNumber = settings.project.number;
  const isClosed = settings.dangerState.state === "closed";
  const canClose = settings.viewerPermissions.canClose;
  const canReopen = settings.viewerPermissions.canReopen;
  const canDelete = settings.viewerPermissions.canDelete;

  async function mutateLifecycle(action: "close" | "reopen", success: string) {
    setPendingAction(action);
    setFeedback(null);
    try {
      const response = await fetch(
        `/api/projects/${encodeURIComponent(settings.project.id)}/${action}`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            expectedUpdatedAt: settings.general.updatedAt,
          }),
        },
      );
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          payload?.error?.message ?? "Project lifecycle could not be changed.",
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
            : "Project lifecycle could not be changed.",
      });
    } finally {
      setPendingAction(null);
    }
  }

  async function handleDelete(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPendingAction("delete");
    setFeedback(null);
    try {
      const response = await fetch(
        `/api/projects/${encodeURIComponent(settings.project.id)}`,
        {
          method: "DELETE",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            confirmation: deleteText,
            expectedUpdatedAt: settings.general.updatedAt,
          }),
        },
      );
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          payload?.error?.message ?? "Project could not be deleted.",
        );
      }
      const deletion = payload as ProjectDeleteResponse;
      setFeedback({ kind: "success", message: "Project deleted." });
      router.push(deletion.destinationHref);
    } catch (error) {
      setFeedback({
        kind: "error",
        message:
          error instanceof Error
            ? error.message
            : "Project could not be deleted.",
      });
    } finally {
      setPendingAction(null);
    }
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
          className="btn sm"
          href={workspaceHref(scope, owner, projectNumber)}
        >
          Back to project
        </Link>
        <span className={isClosed ? "chip warn" : "chip ok"}>
          {isClosed ? "Closed" : "Open"}
        </span>
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
          Close, reopen, or delete this project with explicit confirmation and
          audit history.
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
            href={settingsHref(scope, owner, projectNumber, "general")}
          >
            General
          </Link>
          <Link
            className="list-row"
            href={settingsHref(scope, owner, projectNumber, "access")}
          >
            Access
          </Link>
          <Link
            className="list-row"
            href={settingsHref(scope, owner, projectNumber, "fields")}
          >
            Fields
          </Link>
          <Link
            className="list-row"
            href={settingsHref(scope, owner, projectNumber, "workflows")}
          >
            Workflows
          </Link>
          <Link
            className="list-row"
            href={settingsHref(scope, owner, projectNumber, "templates")}
          >
            Templates
          </Link>
          <span
            className="list-row"
            aria-current="page"
            style={{ color: "var(--accent)" }}
          >
            Danger Zone
          </span>
        </nav>

        <section className="grid gap-4">
          <article className="card p-5">
            <div className="row" style={{ alignItems: "flex-start", gap: 16 }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <h2 className="t-h3">Project lifecycle</h2>
                <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  Closed projects stay readable, but settings, fields, access,
                  items, and workflow edits are disabled until reopened.
                </p>
                <p className="t-xs mt-3">
                  Closed {formatDate(settings.dangerState.closedAt)}
                  {settings.dangerState.closedBy
                    ? ` by ${settings.dangerState.closedBy.login}`
                    : ""}
                </p>
              </div>
              {isClosed ? (
                <button
                  className="btn primary"
                  disabled={!canReopen || pendingAction === "reopen"}
                  onClick={() =>
                    void mutateLifecycle("reopen", "Project reopened.")
                  }
                  type="button"
                >
                  Reopen project
                </button>
              ) : (
                <button
                  className="btn"
                  disabled={!canClose || pendingAction === "close"}
                  onClick={() =>
                    void mutateLifecycle("close", "Project closed.")
                  }
                  type="button"
                >
                  Close project
                </button>
              )}
            </div>
            {!canClose && !canReopen ? (
              <p className="t-xs mt-3" style={{ color: "var(--ink-3)" }}>
                Only project admins can change lifecycle state.
              </p>
            ) : null}
          </article>

          <form className="card p-5" onSubmit={handleDelete}>
            <div className="row" style={{ alignItems: "flex-start", gap: 16 }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <h2 className="t-h3">Delete project</h2>
                <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                  Delete hides this project from project lists and direct reads.
                  Repository issues, pull requests, Actions logs, and audit rows
                  remain intact.
                </p>
                <label className="t-label mt-4 block" htmlFor="delete-confirm">
                  Type {settings.dangerState.deleteConfirmation}
                </label>
                <input
                  className="input mt-2"
                  disabled={!canDelete || pendingAction === "delete"}
                  id="delete-confirm"
                  onChange={(event) => setDeleteText(event.target.value)}
                  value={deleteText}
                />
              </div>
              <button
                className="btn"
                disabled={
                  !canDelete ||
                  pendingAction === "delete" ||
                  deleteText.trim() !== settings.dangerState.deleteConfirmation
                }
                type="submit"
              >
                Delete project
              </button>
            </div>
            {!canDelete ? (
              <p className="t-xs mt-3" style={{ color: "var(--ink-3)" }}>
                Only project admins can delete this project.
              </p>
            ) : null}
          </form>
        </section>
      </div>
    </main>
  );
}
