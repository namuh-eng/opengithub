import Link from "next/link";
import type { PackageSettingsFetchResult } from "@/lib/api";
import { ownerPackagesHref } from "@/lib/navigation";

type PackageSettingsPageProps = {
  owner: string;
  ownerKind: "user" | "organization";
  result: PackageSettingsFetchResult;
};

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "recently";
  }
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

function Unavailable({
  result,
}: {
  result: Exclude<PackageSettingsFetchResult, { ok: true }>;
}) {
  const forbidden = result.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${forbidden ? "warn" : "err"}`}>
        {forbidden ? "Admin access required" : "Unavailable"}
      </span>
      <h1 className="t-h1 mt-4">Package settings could not load</h1>
      <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-2)" }}>
        {forbidden
          ? "Package settings are visible only to package admins, owner accounts, organization owners, or linked repository admins."
          : result.message}
      </p>
    </section>
  );
}

export function PackageSettingsPage({
  owner,
  ownerKind,
  result,
}: PackageSettingsPageProps) {
  if (!result.ok) {
    return <Unavailable result={result} />;
  }

  const settings = result.settings;
  const detail = settings.package;
  return (
    <div className="grid gap-6">
      <header className="grid gap-4">
        <div className="flex flex-wrap items-center gap-2">
          <Link
            className="t-sm underline"
            href={ownerPackagesHref(ownerKind, owner)}
          >
            Packages
          </Link>
          <span className="t-xs">/</span>
          <Link className="t-sm underline" href={detail.href}>
            {detail.name}
          </Link>
          <span className="t-xs">/ settings</span>
        </div>
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Package settings
            </p>
            <h1 className="t-h1 mt-1 min-w-0 break-words">{detail.name}</h1>
            <div className="mt-3 flex flex-wrap gap-2">
              <span className="chip soft">{detail.typeLabel}</span>
              <span className="chip soft">{detail.visibility}</span>
              {detail.latestVersion ? (
                <span className="chip accent">
                  Latest {detail.latestVersion}
                </span>
              ) : null}
              <span className="chip soft">
                {detail.downloadCount.toLocaleString()} downloads
              </span>
            </div>
          </div>
          <Link className="btn" href={detail.href}>
            View package
          </Link>
        </div>
      </header>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]">
        <main className="grid min-w-0 gap-6">
          <section className="card p-5" aria-labelledby="access-heading">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Access
            </p>
            <h2 className="t-h2 mt-1" id="access-heading">
              Explicit package access
            </h2>
            <div className="mt-4 grid gap-3">
              {settings.explicitPermissions.length > 0 ? (
                settings.explicitPermissions.map((permission) => (
                  <div className="list-row py-3" key={permission.userId}>
                    <div className="flex flex-wrap items-center justify-between gap-3">
                      <Link className="t-sm underline" href={permission.href}>
                        {permission.displayName ?? permission.login}
                      </Link>
                      <span className="chip soft">{permission.role}</span>
                    </div>
                    <p className="t-xs mt-1">
                      Granted {formatDate(permission.grantedAt)}
                    </p>
                  </div>
                ))
              ) : (
                <p className="t-body" style={{ color: "var(--ink-2)" }}>
                  No direct package grants are recorded. Access currently comes
                  from the owner account or linked repositories.
                </p>
              )}
            </div>
          </section>

          <section className="card p-5" aria-labelledby="repository-heading">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Provenance
            </p>
            <h2 className="t-h2 mt-1" id="repository-heading">
              Linked repositories
            </h2>
            <div className="mt-4 grid gap-3">
              {settings.linkedRepositories.map((repository) => (
                <div className="list-row py-3" key={repository.id}>
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <Link className="t-sm underline" href={repository.href}>
                      {repository.fullName}
                    </Link>
                    <span className="chip soft">{repository.visibility}</span>
                  </div>
                </div>
              ))}
            </div>
            <h3 className="t-h3 mt-5">Inherited repository access</h3>
            <div className="mt-3 grid gap-3">
              {settings.inheritedRepositoryAccess.length > 0 ? (
                settings.inheritedRepositoryAccess.map((access) => (
                  <div
                    className="list-row py-3"
                    key={`${access.repository.id}-${access.userId}`}
                  >
                    <div className="flex flex-wrap items-center justify-between gap-3">
                      <span className="t-sm">
                        <Link className="underline" href={access.href}>
                          {access.login}
                        </Link>{" "}
                        through{" "}
                        <Link
                          className="underline"
                          href={access.repository.href}
                        >
                          {access.repository.fullName}
                        </Link>
                      </span>
                      <span className="chip soft">{access.role}</span>
                    </div>
                    <p className="t-xs mt-1">Source: {access.source}</p>
                  </div>
                ))
              ) : (
                <p className="t-body" style={{ color: "var(--ink-2)" }}>
                  No inherited repository permissions are currently attached to
                  this package.
                </p>
              )}
            </div>
          </section>

          <section className="card p-5" aria-labelledby="capability-heading">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Future writes
            </p>
            <h2 className="t-h2 mt-1" id="capability-heading">
              Registry management controls
            </h2>
            <div className="mt-4 grid gap-3">
              {settings.registryWriteCapabilities.map((capability) => (
                <div
                  className="list-row grid gap-3 py-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center"
                  key={capability.key}
                >
                  <div className="min-w-0">
                    <p className="t-sm font-semibold">{capability.label}</p>
                    <p className="t-xs mt-1">{capability.reason}</p>
                  </div>
                  <button
                    aria-disabled="true"
                    className="btn sm"
                    disabled
                    type="button"
                  >
                    Not available
                  </button>
                </div>
              ))}
            </div>
          </section>
        </main>

        <aside className="grid content-start gap-4">
          <section className="card p-5" aria-labelledby="summary-heading">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Overview
            </p>
            <h2 className="t-h3 mt-1" id="summary-heading">
              Current state
            </h2>
            <dl className="mt-4 grid gap-3">
              <div>
                <dt className="t-xs">Owner</dt>
                <dd>
                  <Link className="t-sm underline" href={settings.owner.href}>
                    {settings.owner.login}
                  </Link>
                </dd>
              </div>
              <div>
                <dt className="t-xs">Latest digest</dt>
                <dd className="t-mono-sm break-all">
                  {detail.latestDigest ?? "No digest recorded"}
                </dd>
              </div>
              <div>
                <dt className="t-xs">Updated</dt>
                <dd className="t-sm">{formatDate(detail.updatedAt)}</dd>
              </div>
            </dl>
          </section>

          <section className="card p-5" aria-labelledby="activity-heading">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Activity
            </p>
            <h2 className="t-h3 mt-1" id="activity-heading">
              Recent package activity
            </h2>
            <div className="mt-4 grid gap-3">
              {settings.recentActivity.length > 0 ? (
                settings.recentActivity.map((activity) => (
                  <div
                    className="list-row py-3"
                    key={`${activity.kind}-${activity.occurredAt}-${activity.label}`}
                  >
                    <p className="t-sm">{activity.label}</p>
                    <p className="t-xs mt-1">
                      {formatDate(activity.occurredAt)}
                      {activity.actor ? ` by ${activity.actor.login}` : ""}
                    </p>
                  </div>
                ))
              ) : (
                <p className="t-body" style={{ color: "var(--ink-2)" }}>
                  No package activity has been recorded yet.
                </p>
              )}
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}
