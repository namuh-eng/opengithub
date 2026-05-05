"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositoryCodeScanningAlertFilters } from "@/components/RepositoryCodeScanningAlertFilters";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryCodeScanningAlertRow,
  RepositoryCodeScanningAlertsFetchResult,
  RepositoryCodeScanningAlertsView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryCodeScanningAlertsHref } from "@/lib/navigation";

type RepositoryCodeScanningAlertsPageProps = {
  repository: RepositoryOverview;
  codeScanningResult: RepositoryCodeScanningAlertsFetchResult;
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

function severityClass(alert: RepositoryCodeScanningAlertRow) {
  const severity = alert.securitySeverity ?? alert.severity;
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "medium" || severity === "warning") return "chip warn";
  if (alert.state === "fixed") return "chip ok";
  return "chip soft";
}

function stateLabel(state: string) {
  if (state === "fixed") return "Fixed";
  if (state === "dismissed") return "Dismissed";
  return "Open";
}

function stateClass(state: string) {
  if (state === "open") return "chip warn";
  if (state === "fixed") return "chip ok";
  return "chip soft";
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
  view: RepositoryCodeScanningAlertsView;
}) {
  const state = view.filters.state === "closed" ? "closed" : "open";
  return (
    <nav aria-label="Code scanning alert states" className="tabs">
      <Link
        aria-current={state === "open" ? "page" : undefined}
        className={state === "open" ? "tab active" : "tab"}
        href={repositoryCodeScanningAlertsHref(owner, repo, {
          ...view.filters,
          state: "open",
        })}
      >
        Open <span className="t-num">{formatNumber(view.counts.open)}</span>
      </Link>
      <Link
        aria-current={state === "closed" ? "page" : undefined}
        className={state === "closed" ? "tab active" : "tab"}
        href={repositoryCodeScanningAlertsHref(owner, repo, {
          ...view.filters,
          state: "closed",
        })}
      >
        Closed <span className="t-num">{formatNumber(view.counts.closed)}</span>
      </Link>
    </nav>
  );
}

function AlertRow({
  alert,
  selected,
  onToggle,
}: {
  alert: RepositoryCodeScanningAlertRow;
  selected: boolean;
  onToggle: (id: string) => void;
}) {
  const severityLabel = alert.securitySeverity ?? alert.severity;
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
          <span className={severityClass(alert)}>{severityLabel}</span>
          <span className={stateClass(alert.state)}>
            {stateLabel(alert.state)}
          </span>
          <span className="chip soft">{alert.toolName}</span>
          {alert.isDefaultBranch ? (
            <span className="chip ok">Default branch</span>
          ) : null}
          {alert.linkedIssue ? (
            <Link className="chip soft" href={alert.linkedIssue.href}>
              Linked issue #{alert.linkedIssue.number}
            </Link>
          ) : null}
        </div>
        <Link
          className="mt-2 block break-words t-h3 hover:underline"
          href={alert.href}
          style={{ color: "var(--ink-1)" }}
        >
          {alert.ruleName}
        </Link>
        <p className="t-sm mt-1 break-words" style={{ color: "var(--ink-3)" }}>
          {alert.message}
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          <Link className="chip soft" href={alert.pathHref}>
            <span className="t-mono-sm">
              {alert.path}:{alert.startLine}
            </span>
          </Link>
          <span className="chip soft t-mono-sm">{alert.refName}</span>
          {alert.branchName ? (
            <span className="chip soft">{alert.branchName}</span>
          ) : null}
        </div>
        <p className="t-xs mt-2">
          Rule <span className="t-mono-sm">{alert.ruleId}</span> detected{" "}
          {formatRelativeTime(alert.detectedAt)} and updated{" "}
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

function DisabledState({ view }: { view: RepositoryCodeScanningAlertsView }) {
  return (
    <section className="card grid gap-3 p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Code scanning
      </p>
      <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
        Code scanning is not enabled.
      </h2>
      <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {view.availability.disabledReason ?? view.availability.message}
      </p>
      <div className="flex flex-wrap gap-2">
        <Link
          className="btn primary"
          href={view.availability.settingsHref ?? view.links.settingsHref}
        >
          Enable code scanning
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
  view: RepositoryCodeScanningAlertsView;
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
      activeSection="code-scanning"
      repository={repository}
    >
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Code scanning alerts
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {view.availability.message}
            </p>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            <Link className="btn" href={view.links.uploadHref}>
              Upload SARIF
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
          <span className="chip soft">{view.freshness.cadence}</span>
        </div>

        {!view.availability.enabled ? <DisabledState view={view} /> : null}

        <AlertTabs owner={owner} repo={repo} view={view} />

        <RepositoryCodeScanningAlertFilters
          branches={view.branches}
          filters={view.filters}
          owner={owner}
          repo={repo}
          tools={view.tools}
        />

        <section
          aria-label="Code scanning alert summary"
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
              Closed
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.closed)}
            </p>
          </article>
          <article className="card min-h-24 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Visible
            </p>
            <p className="t-h1 t-num mt-2" style={{ color: "var(--ink-1)" }}>
              {formatNumber(view.counts.visible)}
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

        {view.tools.length > 0 ? (
          <section className="card grid gap-3 p-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Analysis tools
              </p>
              <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
                Latest SARIF and Actions analysis
              </h2>
            </div>
            <div className="grid gap-2 md:grid-cols-2">
              {view.tools.map((tool) => (
                <div className="card grid gap-1 p-3" key={tool.name}>
                  <div className="between gap-2">
                    <span className="t-sm font-semibold">{tool.name}</span>
                    <span
                      className={
                        tool.status === "completed" ? "chip ok" : "chip warn"
                      }
                    >
                      {tool.status}
                    </span>
                  </div>
                  <p className="t-xs">
                    <span className="t-num">{tool.alertCount}</span> alerts
                    {tool.version ? ` · ${tool.version}` : ""}
                  </p>
                </div>
              ))}
            </div>
          </section>
        ) : null}

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
                Clear the search or filter menus to return to every visible code
                scanning alert.
              </p>
              <Link className="btn primary w-fit" href={view.links.listHref}>
                Clear filters
              </Link>
            </div>
          ) : (
            <ul aria-label="Code scanning alerts" className="m-0 list-none p-0">
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
  result: Extract<RepositoryCodeScanningAlertsFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell
      activeSection="code-scanning"
      repository={repository}
    >
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Code scanning
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Code scanning alerts unavailable
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

export function RepositoryCodeScanningAlertsPage({
  repository,
  codeScanningResult,
}: RepositoryCodeScanningAlertsPageProps) {
  if (!codeScanningResult.ok) {
    return (
      <AlertsUnavailablePage
        repository={repository}
        result={codeScanningResult}
      />
    );
  }

  return (
    <AlertsReadyPage
      repository={repository}
      view={codeScanningResult.codeScanning}
    />
  );
}
