"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import { RepositoryDependabotAlertFilters } from "@/components/RepositoryDependabotAlertFilters";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryDependabotAlertRow,
  RepositoryDependabotAlertsFetchResult,
  RepositoryDependabotAlertsView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryDependabotAlertsHref } from "@/lib/navigation";

type RepositoryDependabotAlertsPageProps = {
  repository: RepositoryOverview;
  dependabotResult: RepositoryDependabotAlertsFetchResult;
};

const DISMISSAL_REASONS = [
  { value: "fix_started", label: "A fix has already started" },
  { value: "inaccurate", label: "This alert is inaccurate" },
  { value: "no_bandwidth", label: "No bandwidth to fix this" },
  { value: "not_used", label: "Vulnerable code is not used" },
  { value: "tolerable_risk", label: "Risk is tolerable" },
];

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

function severityClass(severity: string) {
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "moderate") return "chip warn";
  return "chip soft";
}

function stateLabel(state: string) {
  if (state === "fixed") return "Fixed";
  if (state === "dismissed") return "Dismissed";
  return state === "closed" ? "Closed" : "Open";
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
          Vulnerability settings
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
  view: RepositoryDependabotAlertsView;
}) {
  const state = view.filters.state === "closed" ? "closed" : "open";
  return (
    <nav aria-label="Dependabot alert states" className="tabs">
      <Link
        aria-current={state === "open" ? "page" : undefined}
        className={state === "open" ? "tab active" : "tab"}
        href={repositoryDependabotAlertsHref(owner, repo, {
          ...view.filters,
          state: "open",
        })}
      >
        Open <span className="t-num">{formatNumber(view.counts.open)}</span>
      </Link>
      <Link
        aria-current={state === "closed" ? "page" : undefined}
        className={state === "closed" ? "tab active" : "tab"}
        href={repositoryDependabotAlertsHref(owner, repo, {
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
  alert: RepositoryDependabotAlertRow;
  selected: boolean;
  onToggle: (id: string) => void;
}) {
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
          <span className="chip soft">{alert.package.ecosystem}</span>
          <Link
            className="break-words t-sm font-semibold hover:underline"
            href={alert.href}
          >
            {alert.package.name}
          </Link>
          <span className={severityClass(alert.advisory.severity)}>
            {alert.advisory.severity}
          </span>
          <span className="chip soft">{alert.scope}</span>
          <span className={stateClass(alert.state)}>
            {stateLabel(alert.state)}
          </span>
        </div>
        <Link
          className="mt-2 block break-words t-h3 hover:underline"
          href={alert.href}
          style={{ color: "var(--ink-1)" }}
        >
          {alert.advisory.title}
        </Link>
        <div className="mt-3 flex flex-wrap gap-2">
          <Link className="chip soft" href={alert.manifestHref}>
            <span className="t-mono-sm">{alert.manifestPath}</span>
          </Link>
          {alert.lockfilePath && alert.lockfileHref ? (
            <Link className="chip soft" href={alert.lockfileHref}>
              <span className="t-mono-sm">{alert.lockfilePath}</span>
            </Link>
          ) : null}
          {alert.currentVersion ? (
            <span className="chip soft t-mono-sm">
              Current {alert.currentVersion}
            </span>
          ) : null}
          {alert.fixedVersion ? (
            <span className="chip ok t-mono-sm">
              Fixed in {alert.fixedVersion}
            </span>
          ) : (
            <span className="chip warn">No fix published</span>
          )}
        </div>
        <p className="t-xs mt-2">
          {alert.advisory.identifier} detected{" "}
          {formatRelativeTime(alert.detectedAt)} in a {alert.relationship}{" "}
          dependency.
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

function DisabledState({ view }: { view: RepositoryDependabotAlertsView }) {
  return (
    <section className="card grid gap-3 p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Dependabot alerts
      </p>
      <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
        Vulnerability alerts are disabled.
      </h2>
      <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {view.availability.disabledReason ?? view.availability.message}
      </p>
      <div className="flex flex-wrap gap-2">
        <Link
          className="btn primary"
          href={view.availability.settingsHref ?? view.links.settingsHref}
        >
          Open vulnerability settings
        </Link>
        <Link className="btn" href={view.repository.securityHref}>
          Back to security overview
        </Link>
      </div>
    </section>
  );
}

function RepositoryDependabotBulkActions({
  canWrite,
  owner,
  repo,
  selectedIds,
  selectedState,
  visibleAlerts,
  onClear,
  onToggleAll,
}: {
  canWrite: boolean;
  owner: string;
  repo: string;
  selectedIds: string[];
  selectedState: string;
  visibleAlerts: RepositoryDependabotAlertRow[];
  onClear: () => void;
  onToggleAll: () => void;
}) {
  const router = useRouter();
  const [reason, setReason] = useState(DISMISSAL_REASONS[0]?.value ?? "");
  const [comment, setComment] = useState("");
  const [pending, setPending] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const selectedCount = selectedIds.length;

  async function submit(action: "dismiss" | "reopen") {
    setPending(action);
    setMessage(null);
    setError(null);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/security/dependabot/bulk`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            action,
            alertIds: selectedIds,
            dismissalComment: action === "dismiss" ? comment : null,
            dismissalReason: action === "dismiss" ? reason : null,
          }),
        },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Bulk update failed.");
      }
      setMessage(body?.message ?? "Bulk update saved.");
      onClear();
      router.refresh();
    } catch (caught) {
      setError(
        caught instanceof Error ? caught.message : "Bulk update failed.",
      );
    } finally {
      setPending(null);
    }
  }

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <button
          className="btn sm"
          disabled={visibleAlerts.length === 0}
          onClick={onToggleAll}
          type="button"
        >
          {selectedCount === visibleAlerts.length && visibleAlerts.length > 0
            ? "Clear visible"
            : "Select all visible"}
        </button>
        <span className="chip soft">
          {formatNumber(selectedCount)} selected
        </span>
      </div>
      {canWrite ? (
        <div className="flex flex-wrap items-end gap-2">
          {selectedState === "open" ? (
            <>
              <label className="grid gap-1 t-xs" htmlFor="bulk-dismiss-reason">
                Dismiss reason
                <select
                  className="input min-w-56"
                  id="bulk-dismiss-reason"
                  onChange={(event) => setReason(event.target.value)}
                  value={reason}
                >
                  {DISMISSAL_REASONS.map((item) => (
                    <option key={item.value} value={item.value}>
                      {item.label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="grid gap-1 t-xs" htmlFor="bulk-dismiss-comment">
                Comment
                <input
                  className="input min-w-64"
                  id="bulk-dismiss-comment"
                  maxLength={500}
                  onChange={(event) => setComment(event.target.value)}
                  value={comment}
                />
              </label>
              <button
                className="btn primary sm"
                disabled={selectedCount === 0 || pending !== null}
                onClick={() => submit("dismiss")}
                type="button"
              >
                Dismiss selected
              </button>
            </>
          ) : (
            <button
              className="btn primary sm"
              disabled={selectedCount === 0 || pending !== null}
              onClick={() => submit("reopen")}
              type="button"
            >
              Reopen selected
            </button>
          )}
        </div>
      ) : (
        <span className="chip soft">Write access required for bulk triage</span>
      )}
      {pending ? <span className="chip soft">Saving {pending}</span> : null}
      {message ? <span className="chip ok">{message}</span> : null}
      {error ? <span className="chip err">{error}</span> : null}
    </div>
  );
}

function AlertsReadyPage({
  repository,
  view,
}: {
  repository: RepositoryOverview;
  view: RepositoryDependabotAlertsView;
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
    <RepositorySecurityShell activeSection="dependabot" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Dependabot alerts
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {view.availability.message}
            </p>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            <Link className="btn" href="/contact">
              Give feedback
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

        <RepositoryDependabotAlertFilters
          filters={view.filters}
          manifests={view.manifests}
          owner={owner}
          packages={view.packages}
          repo={repo}
        />

        <section
          aria-label="Dependabot alert summary"
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
            <div className="flex flex-wrap gap-2">
              <RepositoryDependabotBulkActions
                canWrite={view.viewer.canWrite}
                onClear={() => setSelected([])}
                onToggleAll={toggleAllVisible}
                owner={owner}
                repo={repo}
                selectedIds={selected}
                selectedState={
                  view.filters.state === "closed" ? "closed" : "open"
                }
                visibleAlerts={view.alerts}
              />
            </div>
          </div>
          {view.alerts.length === 0 ? (
            <div className="grid gap-3 p-5">
              <h3 className="t-h2" style={{ color: "var(--ink-1)" }}>
                No matching alerts.
              </h3>
              <p className="t-sm max-w-2xl" style={{ color: "var(--ink-3)" }}>
                Clear the search or filter menus to return to every visible
                Dependabot alert.
              </p>
              <Link className="btn primary w-fit" href={view.links.listHref}>
                Clear filters
              </Link>
            </div>
          ) : (
            <ul
              aria-label="Dependabot vulnerability alerts"
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
  result: Extract<RepositoryDependabotAlertsFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="dependabot" repository={repository}>
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Dependabot alerts
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Dependabot alerts unavailable
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

export function RepositoryDependabotAlertsPage({
  repository,
  dependabotResult,
}: RepositoryDependabotAlertsPageProps) {
  if (!dependabotResult.ok) {
    return (
      <AlertsUnavailablePage
        repository={repository}
        result={dependabotResult}
      />
    );
  }

  return (
    <AlertsReadyPage
      repository={repository}
      view={dependabotResult.dependabot}
    />
  );
}
