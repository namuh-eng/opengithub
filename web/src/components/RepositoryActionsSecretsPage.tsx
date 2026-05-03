import Link from "next/link";
import type {
  ActionsSecretSummary,
  ActionsSettingScope,
  ActionsVariableSummary,
  InheritedActionsSecretSummary,
  InheritedActionsVariableSummary,
  RepositoryActionsSecretsSettings,
  RepositoryActionsSecretsSettingsFetchResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsSecretsPageProps = {
  activeTab?: "secrets" | "variables";
  repository: RepositoryOverview;
  settingsResult: RepositoryActionsSecretsSettingsFetchResult;
};

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
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

function DisabledAction({ label }: { label: string }) {
  return (
    <button
      aria-disabled="true"
      className="btn sm"
      disabled
      title="Mutation forms are implemented in the next settings phase."
      type="button"
    >
      {label}
    </button>
  );
}

function EmptyState({
  kind,
  repository,
}: {
  kind: "secret" | "variable";
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
          href={`${settingsHref(repository, tab)}#repository-${tab}`}
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

function SecretRows({
  items,
  title,
}: {
  items: ActionsSecretSummary[];
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
            </div>
            <div className="flex shrink-0 flex-wrap gap-2">
              <DisabledAction label="Update" />
              <DisabledAction label="Delete" />
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
  items,
  title,
}: {
  items: ActionsVariableSummary[];
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
            </div>
            <div className="flex shrink-0 flex-wrap gap-2">
              <DisabledAction label="Update" />
              <DisabledAction label="Delete" />
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
  if (!settingsResult.ok) {
    return (
      <SettingsUnavailable repository={repository} result={settingsResult} />
    );
  }

  const { settings } = settingsResult;
  const showSecrets = activeTab !== "variables";
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
            <span className={settings.canEdit ? "chip ok" : "chip warn"}>
              {settings.canEdit ? "Admin editable" : "Read only"}
            </span>
          </div>
        </div>
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
          {settings.secrets.length === 0 ? (
            <EmptyState kind="secret" repository={repository} />
          ) : (
            <SecretRows items={settings.secrets} title="Repository secrets" />
          )}
          <InheritedSecretRows items={settings.inheritedSecrets} />
        </div>
      ) : (
        <div className="grid gap-4">
          {settings.variables.length === 0 ? (
            <EmptyState kind="variable" repository={repository} />
          ) : (
            <VariableRows
              items={settings.variables}
              title="Repository variables"
            />
          )}
          <InheritedVariableRows items={settings.inheritedVariables} />
        </div>
      )}
    </div>
  );
}
