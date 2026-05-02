import Link from "next/link";
import type {
  RepositoryOverview,
  RepositorySettings,
  RepositorySettingsFetchResult,
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

function DisabledSave({ label }: { label: string }) {
  return (
    <button aria-label={label} className="btn sm" disabled type="button">
      Save
    </button>
  );
}

function SettingToggle({
  checked,
  description,
  label,
}: {
  checked: boolean;
  description: string;
  label: string;
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
        disabled
        readOnly
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

  const { settings } = settingsResult;
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const enabledMergeMethods = [
    settings.merge.allowSquash ? "Squash" : null,
    settings.merge.allowMergeCommit ? "Merge commit" : null,
    settings.merge.allowRebase ? "Rebase" : null,
  ].filter(Boolean);

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
        <dl className="mt-4">
          <StateRow
            label="Repository name"
            value={
              <input
                aria-label="Repository name"
                className="input w-full max-w-md"
                disabled
                readOnly
                value={settings.name}
              />
            }
          />
          <StateRow
            label="Description"
            value={
              <textarea
                aria-label="Repository description"
                className="input min-h-24 w-full"
                disabled
                readOnly
                value={settings.description ?? ""}
              />
            }
          />
        </dl>
        <div className="mt-4">
          <DisabledSave label="Save repository profile unavailable" />
        </div>
      </section>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="grid gap-5">
          <SettingsCard kicker="General" title="Repository state">
            <dl>
              <StateRow
                label="Visibility"
                value={
                  <span className="chip soft capitalize">
                    {settings.visibility}
                  </span>
                }
              />
              <StateRow
                label="Default branch"
                value={
                  <select
                    aria-label="Default branch"
                    className="input max-w-xs"
                    disabled
                    value={settings.defaultBranch}
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
                  <span
                    className={`chip ${settings.isTemplate ? "ok" : "soft"}`}
                  >
                    {settings.isTemplate ? "Enabled" : "Disabled"}
                  </span>
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
              <DisabledSave label="Save repository state unavailable" />
            </div>
          </SettingsCard>

          <SettingsCard kicker="Features" title="Feature toggles">
            <SettingToggle
              checked={settings.features.issuesEnabled}
              description="Issue tracking and issue templates for this repository."
              label="Issues"
            />
            <SettingToggle
              checked={settings.features.projectsEnabled}
              description="Repository projects and planning boards."
              label="Projects"
            />
            <SettingToggle
              checked={settings.features.wikiEnabled}
              description="Repository wiki pages."
              label="Wiki"
            />
            <div className="mt-4">
              <DisabledSave label="Save feature toggles unavailable" />
            </div>
          </SettingsCard>

          <SettingsCard kicker="Pull requests" title="Merge methods">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              Default method: {mergeMethodLabel(settings.merge.defaultMethod)}
            </p>
            <div className="mt-3 grid gap-1">
              <SettingToggle
                checked={settings.merge.allowSquash}
                description="Combine all commits into one commit before merging."
                label="Allow squash merging"
              />
              <SettingToggle
                checked={settings.merge.allowMergeCommit}
                description="Create a merge commit when a pull request merges."
                label="Allow merge commits"
              />
              <SettingToggle
                checked={settings.merge.allowRebase}
                description="Rebase commits from the pull request branch."
                label="Allow rebase merging"
              />
            </div>
            <div className="mt-4">
              <DisabledSave label="Save merge methods unavailable" />
            </div>
          </SettingsCard>
        </div>

        <aside className="grid content-start gap-5">
          <SettingsCard kicker="Repository behavior" title="Creation policy">
            <div className="flex flex-wrap gap-2">
              <span className={`chip ${settings.allowForking ? "ok" : "soft"}`}>
                Forking {settings.allowForking ? "enabled" : "disabled"}
              </span>
              <span
                className={`chip ${
                  settings.webCommitSignoffRequired ? "warn" : "soft"
                }`}
              >
                Web signoff{" "}
                {settings.webCommitSignoffRequired ? "required" : "optional"}
              </span>
            </div>
            <div className="mt-4">
              <DisabledSave label="Save repository behavior unavailable" />
            </div>
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
            <div className="grid gap-2">
              <button
                aria-label={
                  settings.danger.isArchived
                    ? "Unarchive repository unavailable"
                    : "Archive repository unavailable"
                }
                className="btn sm"
                disabled
                type="button"
              >
                {settings.danger.isArchived ? "Unarchive" : "Archive"}
              </button>
              <button
                aria-label="Transfer repository unavailable"
                className="btn sm"
                disabled
                type="button"
              >
                Transfer
              </button>
              <button
                aria-label="Delete repository unavailable"
                className="btn sm"
                disabled
                type="button"
              >
                Delete
              </button>
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
