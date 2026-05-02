"use client";

import { type FormEvent, type ReactNode, useMemo, useState } from "react";
import type {
  ApiErrorEnvelope,
  RepositorySettings,
  RepositorySettingsFeatureFlags,
  RepositorySettingsMergeMethods,
  RepositoryVisibility,
} from "@/lib/api";

type RepositorySettingsOverviewProps = {
  initialSettings: RepositorySettings;
};

type ModalAction = "archive" | "transfer" | "delete" | null;

const VISIBILITY_COPY: Record<RepositoryVisibility, string> = {
  public: "Anyone can see this repository.",
  private: "Only people with access can see this repository.",
  internal: "Organization members can see this repository.",
};

function apiMessage(error: unknown) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return envelope?.error.message ?? "Settings could not be saved.";
}

function ToggleRow({
  checked,
  description,
  label,
  name,
  onChange,
}: {
  checked: boolean;
  description: string;
  label: string;
  name: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex items-start gap-3 py-3">
      <input
        checked={checked}
        className="mt-1 h-4 w-4 accent-[var(--accent)]"
        name={name}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
      <span className="min-w-0">
        <span
          className="block t-sm font-semibold"
          style={{ color: "var(--ink-1)" }}
        >
          {label}
        </span>
        <span
          className="block t-xs leading-5"
          style={{ color: "var(--ink-3)" }}
        >
          {description}
        </span>
      </span>
    </label>
  );
}

function SettingsCard({
  children,
  description,
  title,
}: {
  children: ReactNode;
  description?: string;
  title: string;
}) {
  return (
    <section className="card overflow-hidden">
      <div
        className="border-b px-5 py-4"
        style={{ borderColor: "var(--line)" }}
      >
        <h2 className="t-h3">{title}</h2>
        {description ? (
          <p className="mt-1 t-xs leading-5" style={{ color: "var(--ink-3)" }}>
            {description}
          </p>
        ) : null}
      </div>
      <div className="px-5 py-4">{children}</div>
    </section>
  );
}

export function RepositorySettingsOverview({
  initialSettings,
}: RepositorySettingsOverviewProps) {
  const [settings, setSettings] = useState(initialSettings);
  const [name, setName] = useState(initialSettings.name);
  const [description, setDescription] = useState(
    initialSettings.description ?? "",
  );
  const [visibility, setVisibility] = useState<RepositoryVisibility>(
    initialSettings.visibility,
  );
  const [defaultBranch, setDefaultBranch] = useState(
    initialSettings.defaultBranch,
  );
  const [isTemplate, setIsTemplate] = useState(initialSettings.isTemplate);
  const [features, setFeatures] = useState<RepositorySettingsFeatureFlags>(
    initialSettings.features,
  );
  const [mergeMethods, setMergeMethods] =
    useState<RepositorySettingsMergeMethods>(initialSettings.mergeMethods);
  const [allowForking, setAllowForking] = useState(
    initialSettings.allowForking,
  );
  const [webCommitSignoffRequired, setWebCommitSignoffRequired] = useState(
    initialSettings.webCommitSignoffRequired,
  );
  const [saving, setSaving] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [modalAction, setModalAction] = useState<ModalAction>(null);
  const [confirmation, setConfirmation] = useState("");

  const fullName = `${settings.ownerLogin}/${settings.name}`;
  const mergeValid =
    mergeMethods.mergeCommit || mergeMethods.squash || mergeMethods.rebase;
  const changedSummary = useMemo(
    () =>
      [
        name !== settings.name ? "name" : null,
        description !== (settings.description ?? "") ? "description" : null,
        visibility !== settings.visibility ? "visibility" : null,
        defaultBranch !== settings.defaultBranch ? "default branch" : null,
      ].filter(Boolean),
    [defaultBranch, description, name, settings, visibility],
  );

  async function saveSettings(
    section: string,
    payload: Record<string, unknown>,
  ) {
    setSaving(section);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(
        `/${settings.ownerLogin}/${settings.name}/settings`,
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        },
      );
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Settings could not be saved.",
          {
            cause: body,
          },
        );
      }
      const nextSettings = body as RepositorySettings;
      setSettings(nextSettings);
      setName(nextSettings.name);
      setDescription(nextSettings.description ?? "");
      setVisibility(nextSettings.visibility);
      setDefaultBranch(nextSettings.defaultBranch);
      setIsTemplate(nextSettings.isTemplate);
      setFeatures(nextSettings.features);
      setMergeMethods(nextSettings.mergeMethods);
      setAllowForking(nextSettings.allowForking);
      setWebCommitSignoffRequired(nextSettings.webCommitSignoffRequired);
      setNotice(
        `Saved ${section}. Audit event #${nextSettings.auditEventCount} recorded.`,
      );
    } catch (saveError) {
      setError(apiMessage(saveError));
    } finally {
      setSaving(null);
    }
  }

  function submitGeneral(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void saveSettings("general settings", {
      name,
      description: description.trim() ? description : null,
      visibility,
      defaultBranch,
      isTemplate,
    });
  }

  function updateMerge(next: RepositorySettingsMergeMethods) {
    setMergeMethods(next);
    if (!next.mergeCommit && !next.squash && !next.rebase) {
      setError("At least one pull request merge method must stay enabled.");
      return;
    }
    void saveSettings("merge methods", { mergeMethods: next });
  }

  function modalTitle() {
    if (modalAction === "archive") return "Archive repository";
    if (modalAction === "transfer") return "Transfer repository";
    return "Delete repository";
  }

  return (
    <div className="grid gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Settings overview
          </p>
          <p className="mt-1 t-sm" style={{ color: "var(--ink-2)" }}>
            Admin-only controls for {fullName}. Last saved{" "}
            {new Date(settings.updatedAt).toLocaleString()}.
          </p>
        </div>
        <span className="chip soft">{settings.viewerPermission}</span>
      </div>

      {notice ? (
        <div className="chip ok justify-start px-3 py-2">{notice}</div>
      ) : null}
      {error ? (
        <div className="chip err justify-start px-3 py-2">{error}</div>
      ) : null}

      <form className="grid gap-5" onSubmit={submitGeneral}>
        <SettingsCard
          description="Rename, describe, and classify this repository. Name conflicts return a structured conflict error."
          title="Repository name and visibility"
        >
          <div className="grid gap-4 md:grid-cols-[1fr_220px]">
            <label className="grid gap-2">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Repository name
              </span>
              <input
                className="input"
                maxLength={100}
                onChange={(event) => setName(event.target.value)}
                required
                value={name}
              />
            </label>
            <label className="grid gap-2">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Visibility
              </span>
              <select
                className="input"
                onChange={(event) =>
                  setVisibility(event.target.value as RepositoryVisibility)
                }
                value={visibility}
              >
                <option value="public">Public</option>
                <option value="private">Private</option>
                <option value="internal">Internal</option>
              </select>
            </label>
          </div>
          <p className="mt-2 t-xs" style={{ color: "var(--ink-3)" }}>
            {VISIBILITY_COPY[visibility]}
          </p>
          <label className="mt-4 grid gap-2">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              Description / social preview
            </span>
            <textarea
              className="input min-h-24"
              maxLength={350}
              onChange={(event) => setDescription(event.target.value)}
              value={description}
            />
          </label>
          <div className="mt-4 grid gap-3 md:grid-cols-[1fr_220px]">
            <label className="grid gap-2">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Default branch
              </span>
              <input
                className="input"
                onChange={(event) => setDefaultBranch(event.target.value)}
                required
                value={defaultBranch}
              />
            </label>
            <ToggleRow
              checked={isTemplate}
              description="Allow new repositories to start from this repository."
              label="Template repository"
              name="isTemplate"
              onChange={setIsTemplate}
            />
          </div>
          <div className="mt-4 flex flex-wrap items-center gap-3">
            <button
              className="btn sm"
              disabled={saving === "general settings"}
              style={{ background: "var(--ok)", color: "var(--bg)" }}
              type="submit"
            >
              {saving === "general settings" ? "Saving…" : "Save changes"}
            </button>
            {changedSummary.length > 0 ? (
              <span className="t-xs" style={{ color: "var(--ink-3)" }}>
                Unsaved: {changedSummary.join(", ")}
              </span>
            ) : null}
          </div>
        </SettingsCard>
      </form>

      <SettingsCard
        description="Feature flags update only after the Rust API confirms the write."
        title="Features"
      >
        <div className="divide-y" style={{ borderColor: "var(--line)" }}>
          <ToggleRow
            checked={features.issues}
            description="Show issue tracking tabs and issue creation links."
            label="Issues"
            name="issues"
            onChange={(checked) =>
              void saveSettings("feature toggles", {
                features: { ...features, issues: checked },
              })
            }
          />
          <ToggleRow
            checked={features.projects}
            description="Show project planning links for this repository."
            label="Projects"
            name="projects"
            onChange={(checked) =>
              void saveSettings("feature toggles", {
                features: { ...features, projects: checked },
              })
            }
          />
          <ToggleRow
            checked={features.wiki}
            description="Show repository wiki navigation."
            label="Wiki"
            name="wiki"
            onChange={(checked) =>
              void saveSettings("feature toggles", {
                features: { ...features, wiki: checked },
              })
            }
          />
        </div>
      </SettingsCard>

      <SettingsCard
        description="Pull request settings require at least one enabled merge method."
        title="Pull request merge methods"
      >
        {!mergeValid ? (
          <p className="mb-3 t-sm" style={{ color: "var(--err)" }}>
            At least one pull request merge method must stay enabled.
          </p>
        ) : null}
        <ToggleRow
          checked={mergeMethods.mergeCommit}
          description="Create a merge commit when a pull request is merged."
          label="Allow merge commits"
          name="mergeCommit"
          onChange={(checked) =>
            updateMerge({ ...mergeMethods, mergeCommit: checked })
          }
        />
        <ToggleRow
          checked={mergeMethods.squash}
          description="Combine commits into one commit on the base branch."
          label="Allow squash merging"
          name="squash"
          onChange={(checked) =>
            updateMerge({ ...mergeMethods, squash: checked })
          }
        />
        <ToggleRow
          checked={mergeMethods.rebase}
          description="Rebase pull request commits onto the base branch."
          label="Allow rebase merging"
          name="rebase"
          onChange={(checked) =>
            updateMerge({ ...mergeMethods, rebase: checked })
          }
        />
        <ToggleRow
          checked={mergeMethods.autoMerge}
          description="Backend support is not available yet, so auto-merge remains read-only."
          label="Allow auto-merge"
          name="autoMerge"
          onChange={() =>
            setError(
              "Auto-merge settings are unavailable until backend support exists.",
            )
          }
        />
      </SettingsCard>

      <SettingsCard title="Forking and web commits">
        <ToggleRow
          checked={allowForking}
          description="Permit users with access to fork this repository."
          label="Allow forking"
          name="allowForking"
          onChange={(checked) =>
            void saveSettings("forking", { allowForking: checked })
          }
        />
        <ToggleRow
          checked={webCommitSignoffRequired}
          description="Require signoff for commits created through the web editor."
          label="Require web commit signoff"
          name="webCommitSignoffRequired"
          onChange={(checked) =>
            void saveSettings("web commit signoff", {
              webCommitSignoffRequired: checked,
            })
          }
        />
      </SettingsCard>

      <div id="danger-zone">
        <SettingsCard
          description="Destructive flows collect typed confirmation now, but final mutation is disabled until backend support exists."
          title="Danger Zone"
        >
          <div className="grid gap-3">
            {(["archive", "transfer", "delete"] as const).map((action) => (
              <div
                className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3"
                key={action}
                style={{ borderColor: "var(--line)" }}
              >
                <div>
                  <p className="t-sm font-semibold capitalize">
                    {action} repository
                  </p>
                  <p className="t-xs" style={{ color: "var(--ink-3)" }}>
                    {action === "archive"
                      ? "Make the repository read-only."
                      : action === "transfer"
                        ? "Move ownership to another user or organization."
                        : "Permanently remove repository content and settings."}
                  </p>
                </div>
                <button
                  className="btn sm"
                  onClick={() => {
                    setConfirmation("");
                    setModalAction(action);
                  }}
                  style={{ borderColor: "var(--err)", color: "var(--err)" }}
                  type="button"
                >
                  Open confirmation
                </button>
              </div>
            ))}
          </div>
        </SettingsCard>
      </div>

      {modalAction ? (
        <div
          aria-labelledby="danger-dialog-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklch, var(--ink-1) 30%, transparent)",
          }}
        >
          <div className="card max-w-lg bg-[var(--surface)] p-5 shadow-lg">
            <h2 className="t-h3" id="danger-dialog-title">
              {modalTitle()}
            </h2>
            <p className="mt-2 t-sm" style={{ color: "var(--ink-2)" }}>
              Type <span className="t-mono-sm">{fullName}</span> to confirm.
              Final {modalAction} support is disabled until the backend endpoint
              exists.
            </p>
            <input
              className="input mt-4 w-full"
              onChange={(event) => setConfirmation(event.target.value)}
              placeholder={fullName}
              value={confirmation}
            />
            <div className="mt-4 flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setModalAction(null)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn sm"
                disabled={true}
                style={{ borderColor: "var(--err)", color: "var(--err)" }}
                title="Unavailable until backend support exists"
                type="button"
              >
                {modalTitle()} unavailable
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
