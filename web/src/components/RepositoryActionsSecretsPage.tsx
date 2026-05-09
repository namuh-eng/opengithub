"use client";

import Link from "next/link";
import { type FormEvent, useState } from "react";
import type {
  ActionsSecretSummary,
  ActionsSettingScope,
  ActionsVariableSummary,
  ApiErrorEnvelope,
  InheritedActionsSecretSummary,
  InheritedActionsVariableSummary,
  RepositoryActionsSecretsMutation,
  RepositoryActionsSecretsSettings,
  RepositoryActionsSecretsSettingsFetchResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsSecretsPageProps = {
  activeTab?: "secrets" | "variables";
  repository: RepositoryOverview;
  settingsResult: RepositoryActionsSecretsSettingsFetchResult;
};

type MutationKind = "secret" | "variable";
type FormMode = "create" | "update";
type SettingMutationPayload = {
  name: string;
  scopeKind: "repository" | "environment";
  scopeName: string | null;
  value: string;
};

const namePattern = /^[A-Za-z_][A-Za-z0-9_]*$/;

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    timeZone: "UTC",
    year: "numeric",
  }).format(new Date(value));
}

function scopeLabel(scope: ActionsSettingScope) {
  if (scope.kind === "repository") return "Repository";
  if (scope.name) return `${scope.kind}: ${scope.name}`;
  return scope.kind;
}

function settingActorLabel(
  item: Pick<ActionsSecretSummary | ActionsVariableSummary, "updatedBy">,
) {
  return item.updatedBy?.displayName ?? item.updatedBy?.login ?? "System";
}

function settingsHref(
  repository: RepositoryOverview,
  tab: "secrets" | "variables",
) {
  return `/${repository.owner_login}/${repository.name}/settings/secrets?tab=${tab}`;
}

function actionUrl(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}/settings/secrets/actions`;
}

function errorMessageFromPayload(payload: unknown, fallback: string) {
  const envelope = payload as ApiErrorEnvelope | null;
  return envelope?.error?.message ?? fallback;
}

function SettingsUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryActionsSecretsSettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${isForbidden ? "warn" : "err"}`}>
        {isForbidden ? "Admin access required" : "Unavailable"}
      </span>
      <h2 className="t-h2 mt-4">
        {isForbidden
          ? "Actions secrets are restricted"
          : "Actions secrets could not load"}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {isForbidden
          ? "Only repository admins can view Actions secrets, variables, inherited metadata, and write-only configuration state."
          : result.message}
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Link
          className="btn"
          href={`/${repository.owner_login}/${repository.name}`}
        >
          Repository Code
        </Link>
        <Link className="btn" href="/docs">
          API docs
        </Link>
      </div>
    </section>
  );
}

function EmptyState({
  kind,
  repository,
}: {
  kind: MutationKind;
  repository: RepositoryOverview;
}) {
  const tab = kind === "secret" ? "secrets" : "variables";
  return (
    <section className="card p-6" role="status">
      <span className="chip soft">Empty</span>
      <h3 className="t-h3 mt-3">
        No repository {kind === "secret" ? "secrets" : "variables"}
      </h3>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        {kind === "secret"
          ? "Encrypted values are write-only; this page will only show metadata after a secret is configured."
          : "Repository variables can be displayed to admins and resolved into workflow environments."}
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Link
          className="btn primary"
          href={`${settingsHref(repository, tab)}#add-${kind}`}
        >
          {kind === "secret" ? "Add secret" : "Add variable"}
        </Link>
        <Link className="btn" href="/docs">
          API docs
        </Link>
      </div>
    </section>
  );
}

function StatusMessage({
  error,
  success,
}: {
  error: string | null;
  success: string | null;
}) {
  if (!error && !success) return null;
  return (
    <p
      className="t-sm mt-3"
      role="status"
      style={{ color: error ? "var(--err)" : "var(--ok)" }}
    >
      {error ?? success}
    </p>
  );
}

function SettingMutationForm({
  disabled,
  initialName = "",
  initialScopeKind = "repository",
  initialScopeName = "",
  initialValue = "",
  kind,
  mode,
  onCancel,
  onSubmit,
}: {
  disabled: boolean;
  initialName?: string;
  initialScopeKind?: "repository" | "environment";
  initialScopeName?: string;
  initialValue?: string;
  kind: MutationKind;
  mode: FormMode;
  onCancel?: () => void;
  onSubmit: (payload: SettingMutationPayload) => Promise<void>;
}) {
  const [name, setName] = useState(initialName);
  const [scopeKind, setScopeKind] = useState<"repository" | "environment">(
    initialScopeKind,
  );
  const [scopeName, setScopeName] = useState(initialScopeName);
  const [value, setValue] = useState(initialValue);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const title = `${mode === "create" ? "Add" : "Update"} ${kind}`;
  const valueLabel = kind === "secret" ? "Secret value" : "Variable value";

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setSuccess(null);
    const normalizedName = name.trim();
    const nextValue = value;
    if (!namePattern.test(normalizedName)) {
      setError(
        "Use letters, numbers, and underscores; start with a letter or underscore.",
      );
      return;
    }
    if (!nextValue.trim()) {
      setError(
        kind === "secret"
          ? "Secret updates require a new value; existing values are never reused."
          : "Variable value is required.",
      );
      return;
    }
    if (scopeKind === "environment" && !scopeName.trim()) {
      setError("Environment scoped settings require an environment name.");
      return;
    }
    setSaving(true);
    try {
      await onSubmit({
        name: normalizedName,
        scopeKind,
        scopeName: scopeKind === "environment" ? scopeName.trim() : null,
        value: nextValue,
      });
      setSuccess(`${title} saved.`);
      if (mode === "create") {
        setName("");
        setScopeKind("repository");
        setScopeName("");
        setValue("");
      }
    } catch (formError) {
      setError(
        formError instanceof Error
          ? formError.message
          : "Actions setting could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <form
      className="card p-4"
      id={`${mode === "create" ? "add" : "update"}-${kind}`}
      onSubmit={submit}
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="t-label">
            {mode === "create" ? "New" : "Edit"} {kind}
          </p>
          <h3 className="t-h3 mt-1">{title}</h3>
        </div>
        <span className={kind === "secret" ? "chip accent" : "chip info"}>
          {kind === "secret" ? "Write-only" : "Visible to admins"}
        </span>
      </div>
      <div className="mt-4 grid gap-3 md:grid-cols-[minmax(0,220px)_minmax(0,220px)_1fr]">
        <label className="grid gap-1">
          <span className="t-label">Name</span>
          <input
            className="input"
            disabled={disabled || saving}
            name="name"
            onChange={(event) => setName(event.target.value)}
            placeholder={kind === "secret" ? "DEPLOY_KEY" : "PUBLIC_BASE_URL"}
            value={name}
          />
        </label>
        <label className="grid gap-1">
          <span className="t-label">Scope</span>
          <select
            className="input"
            disabled={disabled || saving || mode === "update"}
            onChange={(event) =>
              setScopeKind(
                event.target.value === "environment"
                  ? "environment"
                  : "repository",
              )
            }
            value={scopeKind}
          >
            <option value="repository">Repository</option>
            <option value="environment">Environment</option>
          </select>
        </label>
        <label className="grid gap-1">
          <span className="t-label">{valueLabel}</span>
          <textarea
            className="input"
            disabled={disabled || saving}
            name="value"
            onChange={(event) => setValue(event.target.value)}
            placeholder={
              kind === "secret"
                ? "Paste a new secret value"
                : "https://opengithub.namuh.co"
            }
            rows={3}
            value={value}
          />
        </label>
      </div>
      {scopeKind === "environment" ? (
        <label className="mt-3 grid gap-1 md:max-w-[220px]">
          <span className="t-label">Environment</span>
          <input
            className="input"
            disabled={disabled || saving || mode === "update"}
            onChange={(event) => setScopeName(event.target.value)}
            placeholder="production"
            value={scopeName}
          />
        </label>
      ) : null}
      <p className="t-xs mt-3">
        {kind === "secret"
          ? "Secret values are encrypted by the Rust API and never rendered back. Updating a secret always requires a fresh value."
          : "Variable values are stored as repository metadata and can be shown to repository admins."}
      </p>
      <div className="mt-4 flex flex-wrap gap-2">
        <button
          aria-disabled={disabled || saving}
          className="btn primary"
          disabled={disabled || saving}
          type="submit"
        >
          {saving ? "Saving..." : title}
        </button>
        {onCancel ? (
          <button
            aria-disabled={saving}
            className="btn"
            disabled={saving}
            onClick={onCancel}
            type="button"
          >
            Cancel
          </button>
        ) : null}
      </div>
      <StatusMessage error={error} success={success} />
    </form>
  );
}

function DeleteConfirmation({
  disabled,
  kind,
  name,
  onCancel,
  onDelete,
}: {
  disabled: boolean;
  kind: MutationKind;
  name: string;
  onCancel: () => void;
  onDelete: () => Promise<void>;
}) {
  const [confirmation, setConfirmation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    if (confirmation.trim() !== name) {
      setError(`Type ${name} to confirm deletion.`);
      return;
    }
    setDeleting(true);
    try {
      await onDelete();
    } catch (deleteError) {
      setError(
        deleteError instanceof Error
          ? deleteError.message
          : `Repository ${kind} could not be deleted.`,
      );
      setDeleting(false);
    }
  }

  return (
    <form className="mt-3 grid gap-3" onSubmit={submit}>
      <p className="t-xs">
        Type <span className="t-mono-sm">{name}</span> to delete this {kind}.
      </p>
      <input
        aria-label={`Confirm delete ${name}`}
        className="input"
        disabled={disabled || deleting}
        onChange={(event) => setConfirmation(event.target.value)}
        value={confirmation}
      />
      <div className="flex flex-wrap gap-2">
        <button
          aria-disabled={disabled || deleting}
          className="btn sm"
          disabled={disabled || deleting}
          type="submit"
        >
          {deleting ? "Deleting..." : `Delete ${kind}`}
        </button>
        <button
          aria-disabled={deleting}
          className="btn sm"
          disabled={deleting}
          onClick={onCancel}
          type="button"
        >
          Cancel
        </button>
      </div>
      <StatusMessage error={error} success={null} />
    </form>
  );
}

function SecretRows({
  canEdit,
  deletingName,
  editingName,
  items,
  onDelete,
  onEdit,
  onSave,
  setDeletingName,
  setEditingName,
  title,
}: {
  canEdit: boolean;
  deletingName: string | null;
  editingName: string | null;
  items: ActionsSecretSummary[];
  onDelete: (name: string) => Promise<void>;
  onEdit: (name: string) => void;
  onSave: (
    currentName: string,
    payload: SettingMutationPayload,
  ) => Promise<void>;
  setDeletingName: (name: string | null) => void;
  setEditingName: (name: string | null) => void;
  title: string;
}) {
  if (items.length === 0) return null;
  return (
    <section className="card p-0" id="repository-secrets">
      <div
        className="flex flex-wrap items-start justify-between gap-3 p-4"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label">{title}</p>
          <h3 className="t-h3 mt-1">{items.length} configured</h3>
        </div>
        <span className="chip accent">Write-only values</span>
      </div>
      <div>
        {items.map((secret) => (
          <div className="list-row flex-wrap gap-4 p-4" key={secret.id}>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="t-mono-sm font-semibold">{secret.name}</span>
                <span className="chip soft">{scopeLabel(secret.scope)}</span>
                <span className="chip ok">Configured</span>
              </div>
              <p className="t-xs mt-2">
                Updated {formatDate(secret.updatedAt)} by{" "}
                {settingActorLabel(secret)} · {secret.storageKind} ·{" "}
                {secret.visibilityPolicy}
              </p>
              {editingName === secret.name ? (
                <div className="mt-3">
                  <SettingMutationForm
                    disabled={!canEdit}
                    initialName={secret.name}
                    initialScopeKind={
                      secret.scope.kind === "environment"
                        ? "environment"
                        : "repository"
                    }
                    initialScopeName={secret.scope.name ?? ""}
                    kind="secret"
                    mode="update"
                    onCancel={() => setEditingName(null)}
                    onSubmit={(payload) => onSave(secret.name, payload)}
                  />
                </div>
              ) : null}
              {deletingName === secret.name ? (
                <DeleteConfirmation
                  disabled={!canEdit}
                  kind="secret"
                  name={secret.name}
                  onCancel={() => setDeletingName(null)}
                  onDelete={() => onDelete(secret.name)}
                />
              ) : null}
            </div>
            <div className="flex shrink-0 flex-wrap gap-2">
              <button
                aria-disabled={!canEdit}
                className="btn sm"
                disabled={!canEdit}
                onClick={() => onEdit(secret.name)}
                type="button"
              >
                Update
              </button>
              <button
                aria-disabled={!canEdit}
                className="btn sm"
                disabled={!canEdit}
                onClick={() => setDeletingName(secret.name)}
                type="button"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function InheritedSecretRows({
  items,
}: {
  items: InheritedActionsSecretSummary[];
}) {
  if (items.length === 0) return null;
  return (
    <section className="card p-4">
      <p className="t-label">Inherited secrets</p>
      <div className="mt-3 grid gap-2">
        {items.map((secret) => (
          <div
            className="flex flex-wrap items-center justify-between gap-3"
            key={`${secret.scope.kind}-${secret.scope.name}-${secret.name}`}
          >
            <div>
              <span className="t-mono-sm font-semibold">{secret.name}</span>
              <p className="t-xs mt-1">
                {scopeLabel(secret.scope)} · updated{" "}
                {formatDate(secret.updatedAt)}
              </p>
            </div>
            <span className="chip soft">{secret.visibilityPolicy}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function VariableRows({
  canEdit,
  deletingName,
  editingName,
  items,
  onDelete,
  onEdit,
  onSave,
  setDeletingName,
  setEditingName,
  title,
}: {
  canEdit: boolean;
  deletingName: string | null;
  editingName: string | null;
  items: ActionsVariableSummary[];
  onDelete: (name: string) => Promise<void>;
  onEdit: (name: string) => void;
  onSave: (
    currentName: string,
    payload: SettingMutationPayload,
  ) => Promise<void>;
  setDeletingName: (name: string | null) => void;
  setEditingName: (name: string | null) => void;
  title: string;
}) {
  if (items.length === 0) return null;
  return (
    <section className="card p-0" id="repository-variables">
      <div
        className="flex flex-wrap items-start justify-between gap-3 p-4"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label">{title}</p>
          <h3 className="t-h3 mt-1">{items.length} configured</h3>
        </div>
        <span className="chip info">Workflow environment</span>
      </div>
      <div>
        {items.map((variable) => (
          <div className="list-row flex-wrap gap-4 p-4" key={variable.id}>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="t-mono-sm font-semibold">{variable.name}</span>
                <span className="chip soft">{scopeLabel(variable.scope)}</span>
              </div>
              {variable.value !== null ? (
                <code
                  className="t-mono-sm mt-2 block overflow-hidden text-ellipsis rounded-md px-2 py-1"
                  style={{
                    background: "var(--surface-2)",
                    border: "1px solid var(--line)",
                    color: "var(--ink-2)",
                  }}
                >
                  {variable.value}
                </code>
              ) : (
                <p className="t-xs mt-2">Value hidden by scope policy.</p>
              )}
              <p className="t-xs mt-2">
                Updated {formatDate(variable.updatedAt)} by{" "}
                {settingActorLabel(variable)} · {variable.visibilityPolicy}
              </p>
              {editingName === variable.name ? (
                <div className="mt-3">
                  <SettingMutationForm
                    disabled={!canEdit}
                    initialName={variable.name}
                    initialScopeKind={
                      variable.scope.kind === "environment"
                        ? "environment"
                        : "repository"
                    }
                    initialScopeName={variable.scope.name ?? ""}
                    initialValue={variable.value ?? ""}
                    kind="variable"
                    mode="update"
                    onCancel={() => setEditingName(null)}
                    onSubmit={(payload) => onSave(variable.name, payload)}
                  />
                </div>
              ) : null}
              {deletingName === variable.name ? (
                <DeleteConfirmation
                  disabled={!canEdit}
                  kind="variable"
                  name={variable.name}
                  onCancel={() => setDeletingName(null)}
                  onDelete={() => onDelete(variable.name)}
                />
              ) : null}
            </div>
            <div className="flex shrink-0 flex-wrap gap-2">
              <button
                aria-disabled={!canEdit}
                className="btn sm"
                disabled={!canEdit}
                onClick={() => onEdit(variable.name)}
                type="button"
              >
                Update
              </button>
              <button
                aria-disabled={!canEdit}
                className="btn sm"
                disabled={!canEdit}
                onClick={() => setDeletingName(variable.name)}
                type="button"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function InheritedVariableRows({
  items,
}: {
  items: InheritedActionsVariableSummary[];
}) {
  if (items.length === 0) return null;
  return (
    <section className="card p-4">
      <p className="t-label">Inherited variables</p>
      <div className="mt-3 grid gap-2">
        {items.map((variable) => (
          <div
            className="flex flex-wrap items-start justify-between gap-3"
            key={`${variable.scope.kind}-${variable.scope.name}-${variable.name}`}
          >
            <div className="min-w-0">
              <span className="t-mono-sm font-semibold">{variable.name}</span>
              <p className="t-xs mt-1">
                {scopeLabel(variable.scope)} · updated{" "}
                {formatDate(variable.updatedAt)}
              </p>
              {variable.value !== null ? (
                <code
                  className="t-mono-sm mt-2 block overflow-hidden text-ellipsis rounded-md px-2 py-1"
                  style={{
                    background: "var(--surface-2)",
                    border: "1px solid var(--line)",
                    color: "var(--ink-2)",
                  }}
                >
                  {variable.value}
                </code>
              ) : null}
            </div>
            <span className="chip soft">{variable.visibilityPolicy}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function SummaryCards({
  settings,
}: {
  settings: RepositoryActionsSecretsSettings;
}) {
  const totalInherited =
    settings.inheritedSecrets.length + settings.inheritedVariables.length;
  return (
    <div className="grid gap-3 md:grid-cols-3">
      <div className="card p-4">
        <p className="t-label">Repository secrets</p>
        <p className="t-h2 mt-2">{settings.secrets.length}</p>
        <p className="t-xs mt-2">Values are encrypted and never displayed.</p>
      </div>
      <div className="card p-4">
        <p className="t-label">Repository variables</p>
        <p className="t-h2 mt-2">{settings.variables.length}</p>
        <p className="t-xs mt-2">Visible values can enter workflow context.</p>
      </div>
      <div className="card p-4">
        <p className="t-label">Inherited metadata</p>
        <p className="t-h2 mt-2">{totalInherited}</p>
        <p className="t-xs mt-2">Organization and environment scope only.</p>
      </div>
    </div>
  );
}

export function RepositoryActionsSecretsPage({
  activeTab = "secrets",
  repository,
  settingsResult,
}: RepositoryActionsSecretsPageProps) {
  const [settings, setSettings] = useState(
    settingsResult.ok ? settingsResult.settings : null,
  );
  const [pageError, setPageError] = useState<string | null>(null);
  const [pageSuccess, setPageSuccess] = useState<string | null>(null);
  const [editingSecret, setEditingSecret] = useState<string | null>(null);
  const [editingVariable, setEditingVariable] = useState<string | null>(null);
  const [deletingSecret, setDeletingSecret] = useState<string | null>(null);
  const [deletingVariable, setDeletingVariable] = useState<string | null>(null);

  if (!settingsResult.ok) {
    return (
      <SettingsUnavailable repository={repository} result={settingsResult} />
    );
  }

  if (!settings) return null;

  const showSecrets = activeTab !== "variables";

  async function mutate(
    mutation: RepositoryActionsSecretsMutation,
    message: string,
  ) {
    setPageError(null);
    setPageSuccess(null);
    const response = await fetch(actionUrl(repository), {
      body: JSON.stringify(mutation),
      headers: { "content-type": "application/json" },
      method: "POST",
    });
    const payload = (await response.json().catch(() => null)) as unknown;
    if (!response.ok) {
      throw new Error(
        errorMessageFromPayload(
          payload,
          "Repository Actions setting update failed.",
        ),
      );
    }
    setSettings(payload as RepositoryActionsSecretsSettings);
    setEditingSecret(null);
    setEditingVariable(null);
    setDeletingSecret(null);
    setDeletingVariable(null);
    setPageSuccess(message);
  }

  const canEdit = settings.canEdit;

  return (
    <div className="grid gap-6">
      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <span className="chip active">Actions</span>
            <h2 className="t-h2 mt-3">
              {settings.ownerLogin}/{settings.name}
            </h2>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Store encrypted secrets and repository variables for workflow
              jobs. Secret values are write-only and never rendered back to
              admins, logs, audit rows, or browser state.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip soft">{settings.viewerPermission}</span>
            <span className="chip soft">{settings.visibility}</span>
            <span className={canEdit ? "chip ok" : "chip warn"}>
              {canEdit ? "Admin editable" : "Read only"}
            </span>
          </div>
        </div>
        <StatusMessage error={pageError} success={pageSuccess} />
      </section>

      <SummaryCards settings={settings} />

      <nav aria-label="Secrets and variables tabs" className="tabs">
        <Link
          aria-current={showSecrets ? "page" : undefined}
          className={`tab ${showSecrets ? "active" : ""}`}
          href={settingsHref(repository, "secrets")}
        >
          Secrets{" "}
          <span className="t-num">
            {settings.secrets.length + settings.inheritedSecrets.length}
          </span>
        </Link>
        <Link
          aria-current={!showSecrets ? "page" : undefined}
          className={`tab ${showSecrets ? "" : "active"}`}
          href={settingsHref(repository, "variables")}
        >
          Variables{" "}
          <span className="t-num">
            {settings.variables.length + settings.inheritedVariables.length}
          </span>
        </Link>
      </nav>

      {showSecrets ? (
        <div className="grid gap-4">
          <SettingMutationForm
            disabled={!canEdit}
            kind="secret"
            mode="create"
            onSubmit={(payload) =>
              mutate(
                { action: "create-secret", ...payload },
                `${payload.name.toUpperCase()} created.`,
              )
            }
          />
          {settings.secrets.length === 0 ? (
            <EmptyState kind="secret" repository={repository} />
          ) : (
            <SecretRows
              canEdit={canEdit}
              deletingName={deletingSecret}
              editingName={editingSecret}
              items={settings.secrets}
              onDelete={(name) =>
                mutate({ action: "delete-secret", name }, `${name} deleted.`)
              }
              onEdit={(name) => {
                setDeletingSecret(null);
                setEditingSecret(name);
              }}
              onSave={(currentName, payload) =>
                mutate(
                  { action: "update-secret", currentName, ...payload },
                  `${payload.name.toUpperCase()} updated.`,
                )
              }
              setDeletingName={(name) => {
                setEditingSecret(null);
                setDeletingSecret(name);
              }}
              setEditingName={setEditingSecret}
              title="Repository secrets"
            />
          )}
          <InheritedSecretRows items={settings.inheritedSecrets} />
        </div>
      ) : (
        <div className="grid gap-4">
          <SettingMutationForm
            disabled={!canEdit}
            kind="variable"
            mode="create"
            onSubmit={(payload) =>
              mutate(
                { action: "create-variable", ...payload },
                `${payload.name.toUpperCase()} created.`,
              )
            }
          />
          {settings.variables.length === 0 ? (
            <EmptyState kind="variable" repository={repository} />
          ) : (
            <VariableRows
              canEdit={canEdit}
              deletingName={deletingVariable}
              editingName={editingVariable}
              items={settings.variables}
              onDelete={(name) =>
                mutate({ action: "delete-variable", name }, `${name} deleted.`)
              }
              onEdit={(name) => {
                setDeletingVariable(null);
                setEditingVariable(name);
              }}
              onSave={(currentName, payload) =>
                mutate(
                  { action: "update-variable", currentName, ...payload },
                  `${payload.name.toUpperCase()} updated.`,
                )
              }
              setDeletingName={(name) => {
                setEditingVariable(null);
                setDeletingVariable(name);
              }}
              setEditingName={setEditingVariable}
              title="Repository variables"
            />
          )}
          <InheritedVariableRows items={settings.inheritedVariables} />
        </div>
      )}
    </div>
  );
}
