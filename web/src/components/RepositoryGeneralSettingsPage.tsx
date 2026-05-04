"use client";

import Link from "next/link";
import { type FormEvent, useEffect, useMemo, useState } from "react";
import type {
  ApiErrorEnvelope,
  RepositoryOverview,
  RepositorySettings,
  RepositorySettingsFetchResult,
  RepositorySettingsPatch,
} from "@/lib/api";

type RepositoryGeneralSettingsPageProps = {
  repository: RepositoryOverview;
  settingsResult: RepositorySettingsFetchResult;
};

type SettingsCardProps = {
  children: React.ReactNode;
  kicker: string;
  title: string;
};

type SaveTarget =
  | "profile"
  | "state"
  | "features"
  | "merge"
  | "behavior"
  | "archive";

type FormState = {
  archiveConfirmation: string;
  allowForking: boolean;
  allowMergeCommit: boolean;
  allowRebase: boolean;
  allowSquash: boolean;
  defaultBranch: string;
  defaultMethod: RepositorySettings["merge"]["defaultMethod"];
  description: string;
  issuesEnabled: boolean;
  isArchived: boolean;
  isTemplate: boolean;
  name: string;
  projectsEnabled: boolean;
  visibility: RepositorySettings["visibility"];
  webCommitSignoffRequired: boolean;
  wikiEnabled: boolean;
};

type Feedback = {
  kind: "success" | "error";
  message: string;
  target: SaveTarget;
};

function formStateFromSettings(settings: RepositorySettings): FormState {
  return {
    archiveConfirmation: "",
    allowForking: settings.allowForking,
    allowMergeCommit: settings.merge.allowMergeCommit,
    allowRebase: settings.merge.allowRebase,
    allowSquash: settings.merge.allowSquash,
    defaultBranch: settings.defaultBranch,
    defaultMethod: settings.merge.defaultMethod,
    description: settings.description ?? "",
    issuesEnabled: settings.features.issuesEnabled,
    isArchived: settings.danger.isArchived,
    isTemplate: settings.isTemplate,
    name: settings.name,
    projectsEnabled: settings.features.projectsEnabled,
    visibility: settings.visibility,
    webCommitSignoffRequired: settings.webCommitSignoffRequired,
    wikiEnabled: settings.features.wikiEnabled,
  };
}

function SettingsCard({ children, kicker, title }: SettingsCardProps) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {kicker}
      </p>
      <h2 className="t-h3 mt-2">{title}</h2>
      <div className="mt-4">{children}</div>
    </section>
  );
}

function SettingToggle({
  checked,
  description,
  disabled = false,
  label,
  name,
  onChange,
}: {
  checked: boolean;
  description: string;
  disabled?: boolean;
  label: string;
  name: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label
      className="flex items-start gap-3 py-3"
      style={{ borderTop: "1px solid var(--line-soft)" }}
    >
      <input
        aria-label={label}
        checked={checked}
        className="mt-1"
        disabled={disabled}
        name={name}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
      <span className="min-w-0">
        <span className="block t-sm font-semibold">{label}</span>
        <span className="block t-xs mt-1">{description}</span>
      </span>
    </label>
  );
}

function StateRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div
      className="grid gap-2 py-3 sm:grid-cols-[180px_minmax(0,1fr)]"
      style={{ borderTop: "1px solid var(--line-soft)" }}
    >
      <dt className="t-sm font-semibold" style={{ color: "var(--ink-2)" }}>
        {label}
      </dt>
      <dd className="min-w-0 t-sm" style={{ color: "var(--ink-1)" }}>
        {value}
      </dd>
    </div>
  );
}

function SubmitRow({
  disabled = false,
  label,
  pending,
}: {
  disabled?: boolean;
  label: string;
  pending: boolean;
}) {
  return (
    <button className="btn sm" disabled={disabled || pending} type="submit">
      {pending ? "Saving" : label}
    </button>
  );
}

function FeedbackMessage({
  feedback,
  target,
}: {
  feedback: Feedback | null;
  target: SaveTarget;
}) {
  if (!feedback || feedback.target !== target) {
    return null;
  }
  return (
    <p
      className="t-sm mt-3"
      role={feedback.kind === "error" ? "alert" : "status"}
      style={{
        color: feedback.kind === "error" ? "var(--err)" : "var(--ok)",
      }}
    >
      {feedback.message}
    </p>
  );
}

function mergeMethodLabel(
  method: RepositorySettings["merge"]["defaultMethod"],
) {
  if (method === "merge_commit") {
    return "Merge commit";
  }
  if (method === "rebase") {
    return "Rebase";
  }
  return "Squash";
}

function formattedDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

async function readError(response: Response, fallback: string) {
  const envelope = (await response
    .json()
    .catch(() => null)) as ApiErrorEnvelope | null;
  const field = envelope?.details?.field;
  const suffix = typeof field === "string" ? ` (${field})` : "";
  return `${envelope?.error.message ?? fallback}${suffix}`;
}

function RepositorySettingsUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositorySettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <div className="grid gap-4">
      <section className="card p-6" role="status">
        <span className={`chip ${isForbidden ? "warn" : "err"}`}>
          {isForbidden ? "Admin access required" : "Unavailable"}
        </span>
        <h2 className="t-h2 mt-4">
          {isForbidden
            ? "Repository settings are restricted"
            : "Repository settings could not load"}
        </h2>
        <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
          {isForbidden
            ? "Only repository owners and admins can view or change general settings."
            : result.message}
        </p>
        <div className="mt-5 flex flex-wrap gap-2">
          <Link
            className="btn"
            href={`/${repository.owner_login}/${repository.name}`}
          >
            Repository Code
          </Link>
          <Link className="btn" href="/dashboard">
            Dashboard
          </Link>
        </div>
      </section>
    </div>
  );
}

export function RepositoryGeneralSettingsPage({
  repository,
  settingsResult,
}: RepositoryGeneralSettingsPageProps) {
  if (!settingsResult.ok) {
    return (
      <RepositorySettingsUnavailable
        repository={repository}
        result={settingsResult}
      />
    );
  }

  return (
    <RepositoryGeneralSettingsEditor
      initialSettings={settingsResult.settings}
      repository={repository}
    />
  );
}

function RepositoryGeneralSettingsEditor({
  initialSettings,
  repository,
}: {
  initialSettings: RepositorySettings;
  repository: RepositoryOverview;
}) {
  const [settings, setSettings] = useState(initialSettings);
  const [form, setForm] = useState(() =>
    formStateFromSettings(initialSettings),
  );
  const [pending, setPending] = useState<SaveTarget | null>(null);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const updatePath = `${basePath}/settings/update`;
  const repositoryFullName = `${settings.ownerLogin}/${settings.name}`;
  const policyLock = (field: string) =>
    settings.policyLocks.find((lock) => lock.field === field) ?? null;
  const visibilityLock = policyLock("visibility");
  const forkingLock = policyLock("allowForking");
  const transferLock = policyLock("transferRepository");
  const deleteLock = policyLock("deleteRepository");

  useEffect(() => {
    setForm(formStateFromSettings(settings));
  }, [settings]);

  const enabledMergeMethods = [
    settings.merge.allowSquash ? "Squash" : null,
    settings.merge.allowMergeCommit ? "Merge commit" : null,
    settings.merge.allowRebase ? "Rebase" : null,
  ].filter(Boolean);
  const mergeValid =
    form.allowSquash || form.allowMergeCommit || form.allowRebase;
  const defaultMergeEnabled = useMemo(() => {
    if (form.defaultMethod === "merge_commit") {
      return form.allowMergeCommit;
    }
    if (form.defaultMethod === "rebase") {
      return form.allowRebase;
    }
    return form.allowSquash;
  }, [
    form.allowMergeCommit,
    form.allowRebase,
    form.allowSquash,
    form.defaultMethod,
  ]);

  function updateForm(next: Partial<FormState>) {
    setForm((current) => ({ ...current, ...next }));
  }

  async function save(
    target: SaveTarget,
    patch: RepositorySettingsPatch,
    success: string,
  ) {
    setPending(target);
    setFeedback(null);
    const response = await fetch(updatePath, {
      body: JSON.stringify(patch),
      headers: { "content-type": "application/json" },
      method: "PATCH",
    });
    if (!response.ok) {
      setForm(formStateFromSettings(settings));
      setFeedback({
        kind: "error",
        message: await readError(
          response,
          "Repository settings failed to save.",
        ),
        target,
      });
      setPending(null);
      return;
    }
    const updated = (await response.json()) as RepositorySettings;
    setSettings(updated);
    setFeedback({ kind: "success", message: success, target });
    setPending(null);
  }

  function saveProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void save(
      "profile",
      {
        description: form.description.trim() ? form.description : null,
        name: form.name,
      },
      "Repository profile saved.",
    );
  }

  function saveState(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void save(
      "state",
      {
        defaultBranch: form.defaultBranch,
        isTemplate: form.isTemplate,
        visibility: form.visibility,
      },
      "Repository state saved.",
    );
  }

  function saveFeatures(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void save(
      "features",
      {
        features: {
          issuesEnabled: form.issuesEnabled,
          projectsEnabled: form.projectsEnabled,
          wikiEnabled: form.wikiEnabled,
        },
      },
      "Feature toggles saved.",
    );
  }

  function saveMerge(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!mergeValid) {
      setFeedback({
        kind: "error",
        message: "At least one merge method must remain enabled.",
        target: "merge",
      });
      setForm(formStateFromSettings(settings));
      return;
    }
    if (!defaultMergeEnabled) {
      setFeedback({
        kind: "error",
        message: "The default merge method must stay enabled.",
        target: "merge",
      });
      setForm(formStateFromSettings(settings));
      return;
    }
    void save(
      "merge",
      {
        merge: {
          allowMergeCommit: form.allowMergeCommit,
          allowRebase: form.allowRebase,
          allowSquash: form.allowSquash,
          defaultMethod: form.defaultMethod,
        },
      },
      "Merge methods saved.",
    );
  }

  function saveBehavior(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void save(
      "behavior",
      {
        allowForking: form.allowForking,
        webCommitSignoffRequired: form.webCommitSignoffRequired,
      },
      "Repository behavior saved.",
    );
  }

  function saveArchive(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void save(
      "archive",
      { isArchived: !settings.danger.isArchived },
      settings.danger.isArchived
        ? "Repository unarchived."
        : "Repository archived.",
    );
  }

  return (
    <div className="grid gap-5">
      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository profile
            </p>
            <h2 className="t-h3 mt-2">
              {settings.ownerLogin}/{settings.name}
            </h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Last updated {formattedDate(settings.updatedAt)}
            </p>
          </div>
          <span className="chip ok capitalize">
            {settings.viewerPermission}
          </span>
        </div>
        <form onSubmit={saveProfile}>
          <dl className="mt-4">
            <StateRow
              label="Repository name"
              value={
                <input
                  aria-label="Repository name"
                  className="input w-full max-w-md"
                  name="name"
                  onChange={(event) => updateForm({ name: event.target.value })}
                  required
                  value={form.name}
                />
              }
            />
            <StateRow
              label="Description"
              value={
                <textarea
                  aria-label="Repository description"
                  className="input min-h-24 w-full"
                  maxLength={500}
                  name="description"
                  onChange={(event) =>
                    updateForm({ description: event.target.value })
                  }
                  value={form.description}
                />
              }
            />
          </dl>
          <div className="mt-4">
            <SubmitRow label="Save profile" pending={pending === "profile"} />
          </div>
          <FeedbackMessage feedback={feedback} target="profile" />
        </form>
      </section>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="grid gap-5">
          <SettingsCard kicker="General" title="Repository state">
            <form onSubmit={saveState}>
              <dl>
                <StateRow
                  label="Visibility"
                  value={
                    <select
                      aria-label="Repository visibility"
                      className="input max-w-xs capitalize"
                      disabled={Boolean(visibilityLock)}
                      name="visibility"
                      onChange={(event) =>
                        updateForm({
                          visibility: event.target
                            .value as RepositorySettings["visibility"],
                        })
                      }
                      value={form.visibility}
                    >
                      <option value="public">Public</option>
                      <option value="private">Private</option>
                      <option value="internal">Internal</option>
                    </select>
                  }
                />
                {visibilityLock ? (
                  <StateRow
                    label="Policy"
                    value={
                      <Link
                        className="chip warn"
                        href={visibilityLock.settingsHref}
                      >
                        {visibilityLock.reason}
                      </Link>
                    }
                  />
                ) : null}
                <StateRow
                  label="Default branch"
                  value={
                    <select
                      aria-label="Default branch"
                      className="input max-w-xs"
                      name="defaultBranch"
                      onChange={(event) =>
                        updateForm({ defaultBranch: event.target.value })
                      }
                      value={form.defaultBranch}
                    >
                      {settings.branches.map((branch) => (
                        <option key={branch} value={branch}>
                          {branch}
                        </option>
                      ))}
                    </select>
                  }
                />
                <StateRow
                  label="Template"
                  value={
                    <SettingToggle
                      checked={form.isTemplate}
                      description="Make this repository available as a template."
                      label="Template repository"
                      name="isTemplate"
                      onChange={(checked) =>
                        updateForm({ isTemplate: checked })
                      }
                    />
                  }
                />
                <StateRow
                  label="Archive state"
                  value={
                    <span
                      className={`chip ${
                        settings.danger.isArchived ? "warn" : "soft"
                      }`}
                    >
                      {settings.danger.isArchived ? "Archived" : "Active"}
                    </span>
                  }
                />
              </dl>
              <div className="mt-4">
                <SubmitRow label="Save state" pending={pending === "state"} />
              </div>
              <FeedbackMessage feedback={feedback} target="state" />
            </form>
          </SettingsCard>

          <SettingsCard kicker="Features" title="Feature toggles">
            <form onSubmit={saveFeatures}>
              <SettingToggle
                checked={form.issuesEnabled}
                description="Issue tracking and issue templates for this repository."
                label="Issues"
                name="issuesEnabled"
                onChange={(checked) => updateForm({ issuesEnabled: checked })}
              />
              <SettingToggle
                checked={form.projectsEnabled}
                description="Repository projects and planning boards."
                label="Projects"
                name="projectsEnabled"
                onChange={(checked) => updateForm({ projectsEnabled: checked })}
              />
              <SettingToggle
                checked={form.wikiEnabled}
                description="Repository wiki pages."
                label="Wiki"
                name="wikiEnabled"
                onChange={(checked) => updateForm({ wikiEnabled: checked })}
              />
              <div className="mt-4">
                <SubmitRow
                  label="Save features"
                  pending={pending === "features"}
                />
              </div>
              <FeedbackMessage feedback={feedback} target="features" />
            </form>
          </SettingsCard>

          <SettingsCard kicker="Pull requests" title="Merge methods">
            <form onSubmit={saveMerge}>
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Default method: {mergeMethodLabel(settings.merge.defaultMethod)}
              </p>
              <label className="mt-3 block">
                <span className="t-sm font-semibold">Default merge method</span>
                <select
                  aria-label="Default merge method"
                  className="input mt-2 max-w-xs"
                  name="defaultMethod"
                  onChange={(event) =>
                    updateForm({
                      defaultMethod: event.target
                        .value as RepositorySettings["merge"]["defaultMethod"],
                    })
                  }
                  value={form.defaultMethod}
                >
                  <option value="squash">Squash</option>
                  <option value="merge_commit">Merge commit</option>
                  <option value="rebase">Rebase</option>
                </select>
              </label>
              <div className="mt-3 grid gap-1">
                <SettingToggle
                  checked={form.allowSquash}
                  description="Combine all commits into one commit before merging."
                  label="Allow squash merging"
                  name="allowSquash"
                  onChange={(checked) => updateForm({ allowSquash: checked })}
                />
                <SettingToggle
                  checked={form.allowMergeCommit}
                  description="Create a merge commit when a pull request merges."
                  label="Allow merge commits"
                  name="allowMergeCommit"
                  onChange={(checked) =>
                    updateForm({ allowMergeCommit: checked })
                  }
                />
                <SettingToggle
                  checked={form.allowRebase}
                  description="Rebase commits from the pull request branch."
                  label="Allow rebase merging"
                  name="allowRebase"
                  onChange={(checked) => updateForm({ allowRebase: checked })}
                />
              </div>
              {!mergeValid || !defaultMergeEnabled ? (
                <p
                  className="t-sm mt-3"
                  role="alert"
                  style={{ color: "var(--err)" }}
                >
                  {!mergeValid
                    ? "At least one merge method must remain enabled."
                    : "The default merge method must stay enabled."}
                </p>
              ) : null}
              <div className="mt-4">
                <SubmitRow
                  disabled={!mergeValid || !defaultMergeEnabled}
                  label="Save merge methods"
                  pending={pending === "merge"}
                />
              </div>
              <FeedbackMessage feedback={feedback} target="merge" />
            </form>
          </SettingsCard>
        </div>

        <aside className="grid content-start gap-5">
          <SettingsCard kicker="Repository behavior" title="Creation policy">
            <form onSubmit={saveBehavior}>
              <SettingToggle
                checked={form.allowForking}
                description="Allow users with access to create forks."
                disabled={Boolean(forkingLock)}
                label="Allow forking"
                name="allowForking"
                onChange={(checked) => updateForm({ allowForking: checked })}
              />
              {forkingLock ? (
                <Link
                  className="chip warn mt-2"
                  href={forkingLock.settingsHref}
                >
                  {forkingLock.reason}
                </Link>
              ) : null}
              <SettingToggle
                checked={form.webCommitSignoffRequired}
                description="Require signoff on commits created through the web editor."
                label="Require web commit signoff"
                name="webCommitSignoffRequired"
                onChange={(checked) =>
                  updateForm({ webCommitSignoffRequired: checked })
                }
              />
              <div className="mt-4">
                <SubmitRow
                  label="Save behavior"
                  pending={pending === "behavior"}
                />
              </div>
              <FeedbackMessage feedback={feedback} target="behavior" />
            </form>
          </SettingsCard>

          <SettingsCard kicker="Branches" title="Available defaults">
            <div className="flex flex-wrap gap-2">
              {settings.branches.map((branch) => (
                <span className="chip soft t-mono-sm" key={branch}>
                  {branch}
                </span>
              ))}
            </div>
            <Link className="btn sm mt-4" href={`${basePath}/branches`}>
              View branches
            </Link>
          </SettingsCard>

          <SettingsCard kicker="Audit" title="Recent setting changes">
            {settings.auditEvents.length > 0 ? (
              <div className="grid gap-3">
                {settings.auditEvents.slice(0, 3).map((event) => (
                  <div
                    className="rounded-md p-3"
                    key={event.id}
                    style={{
                      background: "var(--surface-2)",
                      border: "1px solid var(--line-soft)",
                    }}
                  >
                    <p className="t-sm font-semibold">{event.eventType}</p>
                    <p className="t-xs mt-1">
                      {event.changedFields.join(", ")} ·{" "}
                      {formattedDate(event.createdAt)}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                No setting changes recorded yet.
              </p>
            )}
          </SettingsCard>

          <SettingsCard kicker="Danger zone" title="Destructive actions">
            <form className="grid gap-2" onSubmit={saveArchive}>
              <label className="grid gap-2">
                <span className="t-sm font-semibold">
                  Type {repositoryFullName} to confirm
                </span>
                <input
                  aria-label="Archive confirmation"
                  className="input w-full"
                  onChange={(event) =>
                    updateForm({ archiveConfirmation: event.target.value })
                  }
                  value={form.archiveConfirmation}
                />
              </label>
              <button
                aria-label={
                  settings.danger.isArchived
                    ? "Unarchive repository"
                    : "Archive repository"
                }
                className="btn sm"
                disabled={
                  pending === "archive" ||
                  form.archiveConfirmation !== repositoryFullName ||
                  (!settings.danger.canArchive && !settings.danger.canUnarchive)
                }
                type="submit"
              >
                {settings.danger.isArchived ? "Unarchive" : "Archive"}
              </button>
              <FeedbackMessage feedback={feedback} target="archive" />
            </form>
            <div className="mt-2 grid gap-2">
              <p className="t-xs" style={{ color: "var(--ink-3)" }}>
                Transfer and delete confirmation flows stay disabled until the
                Rust backend owns those operations.
              </p>
              <button
                aria-label="Transfer repository unavailable"
                className="btn sm"
                disabled
                type="button"
              >
                {transferLock
                  ? "Transfer locked by organization policy"
                  : "Transfer"}
              </button>
              <button
                aria-label="Delete repository unavailable"
                className="btn sm"
                disabled
                type="button"
              >
                {deleteLock ? "Delete locked by organization policy" : "Delete"}
              </button>
              {transferLock || deleteLock ? (
                <Link
                  className="chip warn"
                  href={(transferLock ?? deleteLock)?.settingsHref ?? "#"}
                >
                  Destructive repository controls are constrained by member
                  privileges.
                </Link>
              ) : null}
            </div>
          </SettingsCard>
        </aside>
      </div>

      <section className="card p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Merge method summary
        </p>
        <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
          {enabledMergeMethods.join(", ")} enabled for pull requests.
        </p>
      </section>
    </div>
  );
}
