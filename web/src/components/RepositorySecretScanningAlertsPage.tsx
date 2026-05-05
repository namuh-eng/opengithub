"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositorySecretScanningAlertFilters } from "@/components/RepositorySecretScanningAlertFilters";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecretScanningAlertRow,
  RepositorySecretScanningAlertsFetchResult,
  RepositorySecretScanningAlertsView,
} from "@/lib/api";
import { repositorySecretScanningAlertsHref } from "@/lib/navigation";

type RepositorySecretScanningAlertsPageProps = {
  repository: RepositoryOverview;
  secretScanningResult: RepositorySecretScanningAlertsFetchResult;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) return "recently";
  const diffMs = Date.now() - timestamp;
  const absMs = Math.abs(diffMs);
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 1000 * 60 * 60 * 24 * 365],
    ["month", 1000 * 60 * 60 * 24 * 30],
    ["day", 1000 * 60 * 60 * 24],
    ["hour", 1000 * 60 * 60],
    ["minute", 1000 * 60],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, unitMs] of units) {
    if (absMs >= unitMs) {
      return formatter.format(Math.round(-diffMs / unitMs), unit);
    }
  }
  return "just now";
}

function stateLabel(alert: RepositorySecretScanningAlertRow) {
  if (alert.state === "resolved") return "Resolved";
  if (alert.state === "reopened") return "Reopened";
  return "Open";
}

function stateClass(alert: RepositorySecretScanningAlertRow) {
  if (alert.state === "resolved") return "chip ok";
  if (alert.state === "open" || alert.state === "reopened") return "chip warn";
  return "chip soft";
}

function validityClass(status: string) {
  if (status === "active") return "chip err";
  if (status === "inactive") return "chip ok";
  return "chip soft";
}

function resultKindLabel(resultKind: string) {
  if (resultKind === "generic") return "Generic";
  return "Provider";
}

function AlertSettingsMenu({ settingsHref }: { settingsHref: string }) {
  return (
    <details className="relative">
      <summary className="btn cursor-pointer" aria-label="Alert settings menu">
        Alert settings
      </summary>
      <div
        className="card absolute right-0 z-20 mt-2 grid min-w-64 gap-1 p-2"
        style={{ background: "var(--surface)" }}
      >
        <Link className="btn sm ghost justify-start" href={settingsHref}>
          Code security settings
        </Link>
        <Link className="btn sm ghost justify-start" href="/contact">
          Give feedback
        </Link>
      </div>
    </details>
  );
}

function AlertTabs({
  owner,
  repo,
  view,
}: {
  owner: string;
  repo: string;
  view: RepositorySecretScanningAlertsView;
}) {
  const state = view.filters.state === "resolved" ? "resolved" : "open";
  const resultKind = view.filters.topic === "generic" ? "generic" : "provider";
  return (
    <div className="grid gap-2">
      <nav aria-label="Secret scanning alert states" className="tabs">
        <Link
          aria-current={state === "open" ? "page" : undefined}
          className={state === "open" ? "tab active" : "tab"}
          href={repositorySecretScanningAlertsHref(owner, repo, {
            ...view.filters,
            state: "open",
          })}
        >
          Open <span className="t-num">{formatNumber(view.counts.open)}</span>
        </Link>
        <Link
          aria-current={state === "resolved" ? "page" : undefined}
          className={state === "resolved" ? "tab active" : "tab"}
          href={repositorySecretScanningAlertsHref(owner, repo, {
            ...view.filters,
            state: "resolved",
          })}
        >
          Resolved{" "}
          <span className="t-num">{formatNumber(view.counts.resolved)}</span>
        </Link>
      </nav>
      <nav
        aria-label="Secret scanning result kinds"
        className="flex flex-wrap gap-2"
      >
        <Link
          aria-current={resultKind === "provider" ? "page" : undefined}
          className={resultKind === "provider" ? "chip active" : "chip soft"}
          href={view.links.providerHref}
        >
          Provider and default{" "}
          <span className="t-num">{formatNumber(view.counts.provider)}</span>
        </Link>
        <Link
          aria-current={resultKind === "generic" ? "page" : undefined}
          className={resultKind === "generic" ? "chip active" : "chip soft"}
          href={view.links.genericHref}
        >
          Generic{" "}
          <span className="t-num">{formatNumber(view.counts.generic)}</span>
        </Link>
      </nav>
    </div>
  );
}

function AlertRow({
  alert,
  selected,
  onToggle,
}: {
  alert: RepositorySecretScanningAlertRow;
  selected: boolean;
  onToggle: (id: string) => void;
}) {
  const location = alert.primaryLocation;
  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[auto_minmax(0,1fr)_auto]">
      <label
        className="flex items-start pt-1"
        aria-label={`Select alert ${alert.number}`}
      >
        <input
          checked={selected}
          onChange={() => onToggle(alert.id)}
          type="checkbox"
        />
      </label>
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <span className={stateClass(alert)}>{stateLabel(alert)}</span>
          <span className={validityClass(alert.validity.status)}>
            Validity {alert.validity.status}
          </span>
          <span className="chip soft">
            {resultKindLabel(alert.pattern.resultKind)}
          </span>
          <span className="chip soft">{alert.pattern.provider}</span>
          {alert.bypassed ? (
            <span className="chip warn">Bypassed push protection</span>
          ) : null}
          {alert.resolution ? (
            <span className="chip ok">{alert.resolution}</span>
          ) : null}
        </div>
        <Link
          className="mt-2 block break-words t-h3 hover:underline"
          href={alert.href}
          style={{ color: "var(--ink-1)" }}
        >
          {alert.pattern.displayName}
        </Link>
        <p className="t-sm mt-1 break-words" style={{ color: "var(--ink-3)" }}>
          <span className="t-mono-sm">{alert.redactedSecret}</span>
          {alert.redactedContext ? ` · ${alert.redactedContext}` : ""}
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          {location ? (
            <>
              <Link className="chip soft" href={location.pathHref}>
                <span className="t-mono-sm">
                  {location.path}:{location.startLine}
                </span>
              </Link>
              <Link className="chip soft" href={location.rawHref}>
                Raw file
              </Link>
              {location.commitHref ? (
                <Link className="chip soft" href={location.commitHref}>
                  Commit
                </Link>
              ) : null}
              <span className="chip soft t-mono-sm">{location.refName}</span>
            </>
          ) : (
            <span className="chip soft">No indexed file location</span>
          )}
        </div>
        <p className="t-xs mt-2">
          Fingerprint <span className="t-mono-sm">{alert.fingerprint}</span>{" "}
          detected {formatRelativeTime(alert.detectedAt)} and updated{" "}
          {formatRelativeTime(alert.updatedAt)}.
        </p>
      </div>
      <div className="flex flex-wrap gap-2 md:justify-end">
        {alert.assignees.map((assignee) => (
          <Link className="chip soft" href={assignee.href} key={assignee.id}>
            {assignee.login}
          </Link>
        ))}
        <Link className="btn sm" href={alert.href}>
          View alert
        </Link>
      </div>
    </article>
  );
}

function DisabledState({ view }: { view: RepositorySecretScanningAlertsView }) {
  return (
    <section className="card grid gap-3 p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Secret scanning
      </p>
      <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
        Secret scanning alerts
      </h2>
      <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {view.availability.disabledReason ?? view.availability.message}
      </p>
      <div className="flex flex-wrap gap-2">
        <Link
          className="btn primary"
          href={view.availability.settingsHref ?? view.links.settingsHref}
        >
          Enable secret scanning
        </Link>
        <Link className="btn" href={view.repository.securityHref}>
          Back to security overview
        </Link>
      </div>
    </section>
  );
}

function AlertsReadyPage({
  repository,
  view,
}: {
  repository: RepositoryOverview;
  view: RepositorySecretScanningAlertsView;
}) {
  const owner = view.repository.ownerLogin;
  const repo = view.repository.name;
  const [selected, setSelected] = useState<string[]>([]);
  const selectedSet = useMemo(() => new Set(selected), [selected]);

  function toggle(id: string) {
    setSelected((current) =>
      current.includes(id)
        ? current.filter((value) => value !== id)
        : [...current, id],
    );
  }

  function toggleAllVisible() {
    setSelected((current) => {
      const visibleIds = view.alerts.map((alert) => alert.id);
      if (visibleIds.every((id) => current.includes(id))) {
        return current.filter((id) => !visibleIds.includes(id));
      }
      return Array.from(new Set([...current, ...visibleIds]));
    });
  }

  return (
    <RepositorySecurityShell
      activeSection="secret-scanning"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Secret scanning alerts
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {view.availability.message}
            </p>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            <Link className="btn" href={view.pushProtection.settingsHref}>
              Push protection
            </Link>
            <AlertSettingsMenu settingsHref={view.links.settingsHref} />
          </div>
        </section>

        <div className="flex flex-wrap gap-2">
          <span className={view.availability.enabled ? "chip ok" : "chip warn"}>
            {view.availability.enabled ? "Enabled" : "Disabled"}
          </span>
          <span className={view.availability.indexed ? "chip ok" : "chip warn"}>
            {view.availability.indexed ? "Indexed" : "Unindexed"}
          </span>
          <span
            className={
              view.availability.pushProtectionEnabled ? "chip ok" : "chip soft"
            }
          >
            Push protection{" "}
            {view.availability.pushProtectionEnabled ? "on" : "off"}
          </span>
          <span className="chip soft">{view.freshness.cadence}</span>
        </div>

        {!view.availability.enabled ? <DisabledState view={view} /> : null}

        <AlertTabs owner={owner} repo={repo} view={view} />

        <RepositorySecretScanningAlertFilters
          filters={view.filters}
          owner={owner}
          providers={view.providers}
          repo={repo}
          secretTypes={view.secretTypes}
        />

        <section
          aria-label="Secret scanning alert summary"
          className="grid gap-4 md:grid-cols-4"
        >
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Open
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.open)}
            </p>
          </article>
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Resolved
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.resolved)}
            </p>
          </article>
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Bypassed
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.bypassed)}
            </p>
          </article>
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Selected
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(selected.length)}
            </p>
          </article>
        </section>

        <section className="card grid gap-3 p-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Push protection
            </p>
            <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
              Protected pushes and bypass outcomes
            </h2>
          </div>
          <div className="grid gap-3 md:grid-cols-3">
            <div className="card p-3">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Status
              </p>
              <p className="t-sm mt-1">
                {view.pushProtection.enabled ? "Enabled" : "Disabled"}
              </p>
            </div>
            <div className="card p-3">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Protected patterns
              </p>
              <p className="t-sm t-num mt-1">
                {formatNumber(view.pushProtection.protectedPatternCount)}
              </p>
            </div>
            <div className="card p-3">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Pending review
              </p>
              <p className="t-sm t-num mt-1">
                {formatNumber(view.pushProtection.pendingReviewCount)}
              </p>
            </div>
          </div>
        </section>

        <section className="card overflow-hidden">
          <div
            className="between flex-wrap gap-3 px-4 py-3"
            style={{ borderBottom: "1px solid var(--line)" }}
          >
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Alert queue
              </p>
              <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
                {formatNumber(view.counts.visible)} matching alerts
              </h2>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <button
                className="btn sm"
                disabled={view.alerts.length === 0}
                onClick={toggleAllVisible}
                type="button"
              >
                {selected.length === view.alerts.length &&
                view.alerts.length > 0
                  ? "Clear visible"
                  : "Select all visible"}
              </button>
              <span className="chip soft">
                {formatNumber(selected.length)} selected
              </span>
            </div>
          </div>
          {view.alerts.length === 0 ? (
            <div className="grid gap-3 p-5">
              <h3 className="t-h2" style={{ color: "var(--ink-1)" }}>
                No matching alerts.
              </h3>
              <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
                Clear the search or filter menus to return to every visible
                secret scanning alert.
              </p>
              <Link className="btn primary w-fit" href={view.links.listHref}>
                Clear filters
              </Link>
            </div>
          ) : (
            <ul
              aria-label="Secret scanning alerts"
              className="m-0 list-none p-0"
            >
              {view.alerts.map((alert) => (
                <li key={alert.id}>
                  <AlertRow
                    alert={alert}
                    onToggle={toggle}
                    selected={selectedSet.has(alert.id)}
                  />
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </RepositorySecurityShell>
  );
}

function AlertsUnavailablePage({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositorySecretScanningAlertsFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell
      activeSection="secret-scanning"
      repository={repository}
    >
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Secret scanning
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Secret scanning alerts unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security`}
        >
          Back to security overview
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

export function RepositorySecretScanningAlertsPage({
  repository,
  secretScanningResult,
}: RepositorySecretScanningAlertsPageProps) {
  if (!secretScanningResult.ok) {
    return (
      <AlertsUnavailablePage
        repository={repository}
        result={secretScanningResult}
      />
    );
  }

  return (
    <AlertsReadyPage
      repository={repository}
      view={secretScanningResult.secretScanning}
    />
  );
}
