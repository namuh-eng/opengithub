import Link from "next/link";
import type {
  PagesDeploymentSummary,
  PagesSource,
  RepositoryOverview,
  RepositoryPagesSettings,
  RepositoryPagesSettingsFetchResult,
} from "@/lib/api";

type RepositoryPagesSettingsPageProps = {
  repository: RepositoryOverview;
  settingsResult: RepositoryPagesSettingsFetchResult;
};

function formatDate(value: string | null) {
  if (!value) return "Pending";
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    month: "short",
  }).format(new Date(value));
}

function sourceLabel(source: PagesSource) {
  if (source.kind === "branch") {
    return `${source.branch ?? "branch"} · ${source.folder === "/" ? "/(root)" : (source.folder ?? "/")}`;
  }
  if (source.kind === "actions") {
    return source.workflowArtifactName
      ? `GitHub Actions · ${source.workflowArtifactName}`
      : "GitHub Actions";
  }
  return "None";
}

function chipForStatus(status: string) {
  const normalized = status.toLowerCase();
  if (
    ["deployed", "verified", "issued", "ready", "active"].includes(normalized)
  ) {
    return "chip ok";
  }
  if (["failed", "error", "misconfigured"].includes(normalized)) {
    return "chip err";
  }
  if (["pending", "queued", "building", "deploying"].includes(normalized)) {
    return "chip warn";
  }
  return "chip soft";
}

function PagesUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryPagesSettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${isForbidden ? "warn" : "err"}`}>
        {isForbidden ? "Access restricted" : "Unavailable"}
      </span>
      <h2 className="t-h2 mt-4">
        {isForbidden
          ? "Pages settings are restricted"
          : "Pages settings could not load"}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {isForbidden
          ? "Only repository collaborators with read access can view Pages status. Admin-only DNS challenges and cloud aliases remain hidden."
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

function SummaryCards({ settings }: { settings: RepositoryPagesSettings }) {
  const { site } = settings;
  const latest = settings.deployments[0] ?? null;
  const live = site.source.kind !== "none" && !site.unpublishedAt;
  return (
    <div className="grid gap-3 md:grid-cols-3">
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Source
        </p>
        <p className="t-h3 mt-2">{sourceLabel(site.source)}</p>
        <span className={`mt-3 inline-flex ${live ? "chip ok" : "chip soft"}`}>
          {live ? "Configured" : "Disabled"}
        </span>
      </div>
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Site URL
        </p>
        <Link
          className="t-mono-sm mt-2 block break-all"
          href={site.defaultSiteUrl}
        >
          {site.defaultSiteUrl}
        </Link>
        {site.customDomain ? (
          <p className="t-xs mt-2">Custom domain: {site.customDomain}</p>
        ) : (
          <p className="t-xs mt-2">No custom domain configured.</p>
        )}
      </div>
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Latest deployment
        </p>
        {latest ? (
          <>
            <span
              className={`mt-2 inline-flex ${chipForStatus(latest.status)}`}
            >
              {latest.status}
            </span>
            <p className="t-xs mt-2">
              {formatDate(latest.completedAt ?? latest.queuedAt)}
            </p>
          </>
        ) : (
          <>
            <span className="chip soft mt-2 inline-flex">None</span>
            <p className="t-xs mt-2">
              Deployments appear after a source is saved.
            </p>
          </>
        )}
      </div>
    </div>
  );
}

function SourceCard({ settings }: { settings: RepositoryPagesSettings }) {
  const selectedKind = settings.site.source.kind;
  const selectedBranch = settings.site.source.branch ?? "";
  const selectedFolder = settings.site.source.folder ?? "/";
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <span className="chip active">Build and deployment</span>
          <h2 className="t-h2 mt-3">Publishing source</h2>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            Choose a branch folder or an Actions artifact pipeline for static
            site publishing.
          </p>
        </div>
        <span className={settings.canEdit ? "chip ok" : "chip warn"}>
          {settings.canEdit ? "Admin editable" : "Read only"}
        </span>
      </div>

      <div className="mt-5 grid gap-4 md:grid-cols-[1fr_1fr]">
        <label className="grid gap-2">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Source
          </span>
          <select
            aria-label="Source"
            className="input"
            defaultValue={selectedKind}
            disabled
          >
            <option value="none">None</option>
            <option value="branch">Deploy from a branch</option>
            <option value="actions">GitHub Actions</option>
          </select>
        </label>
        <label className="grid gap-2">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Branch
          </span>
          <select
            aria-label="Branch"
            className="input"
            defaultValue={selectedBranch}
            disabled
          >
            <option value="">Select branch</option>
            {settings.availableRefs.map((ref) => (
              <option key={ref.name} value={ref.name}>
                {ref.name}
              </option>
            ))}
          </select>
        </label>
        <label className="grid gap-2">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Folder
          </span>
          <select
            aria-label="Folder"
            className="input"
            defaultValue={selectedFolder}
            disabled
          >
            {settings.folderOptions.map((folder) => (
              <option key={folder.value} value={folder.value}>
                {folder.label}
                {folder.exists ? "" : " (missing)"}
              </option>
            ))}
          </select>
        </label>
        <div className="grid content-end gap-2">
          <button
            aria-disabled="true"
            className="btn primary"
            disabled
            type="button"
          >
            Save source
          </button>
          <p className="t-xs">
            Source changes are confirmed by the Pages API before the UI updates.
          </p>
        </div>
      </div>

      {settings.workflowSuggestions.length > 0 ? (
        <div className="mt-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Actions templates
          </p>
          <div className="mt-2 grid gap-2">
            {settings.workflowSuggestions.map((workflow) => (
              <div className="list-row py-3" key={workflow.workflowId}>
                <div className="min-w-0">
                  <p className="t-h3">{workflow.name}</p>
                  <p
                    className="t-mono-sm mt-1 break-all"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {workflow.path}
                  </p>
                </div>
                <span className="chip soft">{workflow.artifactHint}</span>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </section>
  );
}

function DomainCard({ settings }: { settings: RepositoryPagesSettings }) {
  const { site } = settings;
  const { domain } = site;
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <span className="chip active">Custom domain</span>
          <h2 className="t-h2 mt-3">Domain and HTTPS</h2>
          <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
            DNS verification must pass before HTTPS enforcement and
            custom-domain serving can activate.
          </p>
        </div>
        <span className={chipForStatus(domain.status)}>{domain.status}</span>
      </div>

      <div className="mt-5 grid gap-4 md:grid-cols-[minmax(0,1fr)_auto]">
        <label className="grid gap-2">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Domain
          </span>
          <input
            aria-label="Domain"
            className="input"
            defaultValue={site.customDomain ?? ""}
            disabled
            placeholder="docs.example.com"
          />
        </label>
        <div className="flex items-end gap-2">
          <button
            aria-disabled="true"
            className="btn primary"
            disabled
            type="button"
          >
            Save
          </button>
          <button aria-disabled="true" className="btn" disabled type="button">
            Remove
          </button>
        </div>
      </div>

      {domain.challenge ? (
        <div className="mt-5 grid gap-3 md:grid-cols-[1fr_1fr_auto]">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              DNS record
            </p>
            <p className="t-mono-sm mt-2 break-all">{domain.challenge.name}</p>
          </div>
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Value
            </p>
            <p className="t-mono-sm mt-2 break-all">{domain.challenge.value}</p>
          </div>
          <div className="flex items-end">
            <button aria-disabled="true" className="btn" disabled type="button">
              Recheck DNS
            </button>
          </div>
        </div>
      ) : (
        <p className="t-sm mt-4" style={{ color: "var(--ink-3)" }}>
          A DNS challenge appears after a custom domain is saved.
        </p>
      )}

      <div className="mt-5 flex flex-wrap gap-2">
        <span className={chipForStatus(site.certificateStatus)}>
          Certificate: {site.certificateStatus}
        </span>
        <span className={chipForStatus(site.provisioningStatus)}>
          Provisioning: {site.provisioningStatus}
        </span>
        <span className={site.httpsEnforced ? "chip ok" : "chip soft"}>
          HTTPS {site.httpsEnforced ? "enforced" : "not enforced"}
        </span>
      </div>
      {domain.warning ? (
        <p className="t-sm mt-4" style={{ color: "var(--warn)" }}>
          {domain.warning}
        </p>
      ) : null}
    </section>
  );
}

function deploymentHref(
  repository: RepositoryOverview,
  deployment: PagesDeploymentSummary,
) {
  if (deployment.workflowRunId) {
    return `/${repository.owner_login}/${repository.name}/actions/runs/${deployment.workflowRunId}`;
  }
  return `/${repository.owner_login}/${repository.name}/settings/pages#deployment-${deployment.id}`;
}

function DeploymentHistory({
  repository,
  settings,
}: {
  repository: RepositoryOverview;
  settings: RepositoryPagesSettings;
}) {
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <span className="chip active">Deployments</span>
          <h2 className="t-h2 mt-3">Recent activity</h2>
        </div>
        <Link
          className="btn"
          href={`/${repository.owner_login}/${repository.name}/actions`}
        >
          Actions
        </Link>
      </div>
      <div className="mt-4 grid gap-1">
        {settings.deployments.length > 0 ? (
          settings.deployments.map((deployment) => (
            <Link
              className="list-row grid gap-3 py-3 md:grid-cols-[auto_minmax(0,1fr)_auto]"
              href={deploymentHref(repository, deployment)}
              id={`deployment-${deployment.id}`}
              key={deployment.id}
            >
              <span className={chipForStatus(deployment.status)}>
                {deployment.status}
              </span>
              <span className="min-w-0">
                <span className="t-h3 block">
                  {sourceLabel(deployment.source)}
                </span>
                <span className="t-xs mt-1 block break-all">
                  {deployment.customDomainUrl ?? deployment.defaultUrl}
                </span>
                {deployment.failureReason ? (
                  <span
                    className="t-xs mt-1 block"
                    style={{ color: "var(--err)" }}
                  >
                    {deployment.failureReason}
                  </span>
                ) : null}
              </span>
              <span className="t-xs">
                {formatDate(deployment.completedAt ?? deployment.queuedAt)}
              </span>
            </Link>
          ))
        ) : (
          <div className="py-5" role="status">
            <p className="t-h3">No deployments yet</p>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Pages deployment history will appear after a branch source or
              Actions artifact is deployed.
            </p>
          </div>
        )}
      </div>
    </section>
  );
}

export function RepositoryPagesSettingsPage({
  repository,
  settingsResult,
}: RepositoryPagesSettingsPageProps) {
  if (!settingsResult.ok) {
    return <PagesUnavailable repository={repository} result={settingsResult} />;
  }

  const { settings } = settingsResult;
  const live =
    settings.site.source.kind !== "none" && !settings.site.unpublishedAt;

  return (
    <div className="grid gap-6">
      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <span className={live ? "chip ok" : "chip soft"}>
              {live ? "Live" : "Not published"}
            </span>
            <h2 className="t-h2 mt-3">
              {settings.ownerLogin}/{settings.name} Pages
            </h2>
            <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
              Publish a static site from repository content or workflow
              artifacts, then verify a custom domain before HTTPS and aliases
              activate.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip soft">{settings.viewerPermission}</span>
            <span className="chip soft">{settings.visibility}</span>
          </div>
        </div>
        {live ? (
          <div className="mt-5 flex flex-wrap items-center gap-2">
            <Link
              className="btn primary"
              href={
                settings.site.customDomain
                  ? `https://${settings.site.customDomain}`
                  : settings.site.defaultSiteUrl
              }
            >
              Visit site
            </Link>
            <Link
              className="btn"
              href={`/${repository.owner_login}/${repository.name}/actions`}
            >
              View builds
            </Link>
          </div>
        ) : null}
      </section>

      <SummaryCards settings={settings} />
      <SourceCard settings={settings} />
      <DomainCard settings={settings} />
      <DeploymentHistory repository={repository} settings={settings} />
    </div>
  );
}
