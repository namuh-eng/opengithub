"use client";

import { useState } from "react";
import type {
  ActionsRunner,
  RepositoryActionsRunnerSettings,
  RepositoryActionsRunnerSettingsFetchResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsRunnersPageProps = {
  repository: RepositoryOverview;
  settingsResult: RepositoryActionsRunnerSettingsFetchResult;
};

function dateTimeLabel(value: string | null | undefined) {
  if (!value) return "Never";
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "Recently";
  return date.toLocaleString("en", {
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    month: "short",
  });
}

function statusClass(status: string) {
  if (status === "online") return "chip ok";
  if (status === "busy") return "chip accent";
  if (status === "offline") return "chip warn";
  return "chip soft";
}

function labelsFromInput(value: string) {
  return value
    .split(",")
    .map((label) => label.trim().toLowerCase())
    .filter(Boolean);
}

function RunnerRow({ runner }: { runner: ActionsRunner }) {
  return (
    <div className="list-row items-start px-5 py-4">
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="font-medium">{runner.name}</span>
          <span className={statusClass(runner.status)}>{runner.status}</span>
          {runner.labels.map((label) => (
            <span className="chip soft t-mono-sm" key={label}>
              {label}
            </span>
          ))}
        </div>
        <p className="t-xs mt-2">
          Last heartbeat {dateTimeLabel(runner.lastHeartbeat)}
          {runner.busySince
            ? ` · busy since ${dateTimeLabel(runner.busySince)}`
            : ""}
        </p>
        {runner.currentJob ? (
          <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
            Running {runner.currentJob.workflowName} #
            {runner.currentJob.runNumber}: {runner.currentJob.jobName}
          </p>
        ) : null}
      </div>
    </div>
  );
}

export function RepositoryActionsRunnersPage({
  repository,
  settingsResult,
}: RepositoryActionsRunnersPageProps) {
  const [settings, setSettings] =
    useState<RepositoryActionsRunnerSettings | null>(
      settingsResult.ok ? settingsResult.settings : null,
    );
  const [runnerName, setRunnerName] = useState("linux-build-1");
  const [runnerLabels, setRunnerLabels] = useState(
    "self-hosted, ubuntu-latest",
  );
  const [concurrencyLimit, setConcurrencyLimit] = useState(
    settings?.queue.concurrencyLimit ?? 4,
  );
  const [cancelInProgress, setCancelInProgress] = useState(
    settings?.queue.cancelInProgress ?? false,
  );
  const [githubTokenPermission, setGithubTokenPermission] = useState(
    settings?.workflowPermissions.githubTokenPermission ?? "read",
  );
  const [allowPullRequestApproval, setAllowPullRequestApproval] = useState(
    settings?.workflowPermissions.allowPullRequestApproval ?? false,
  );
  const [message, setMessage] = useState(
    settingsResult.ok ? "" : settingsResult.message,
  );
  const [pending, setPending] = useState<string | null>(null);

  async function postAction(body: Record<string, unknown>) {
    setPending(String(body.action));
    setMessage("");
    const response = await fetch(
      `/${repository.owner_login}/${repository.name}/settings/actions/runners/actions`,
      {
        body: JSON.stringify(body),
        headers: { "content-type": "application/json" },
        method: "POST",
      },
    );
    const payload = await response.json().catch(() => null);
    setPending(null);
    if (!response.ok) {
      setMessage(
        payload?.error?.message ?? "Runner settings could not be saved.",
      );
      return null;
    }
    return payload;
  }

  async function createRunner() {
    const payload = await postAction({
      action: "create-runner",
      labels: labelsFromInput(runnerLabels),
      name: runnerName,
    });
    if (payload?.runners) {
      setSettings(payload as RepositoryActionsRunnerSettings);
      setMessage("Runner registered. Start it with the setup command below.");
    }
  }

  async function saveSettings() {
    const payload = await postAction({
      action: "update-settings",
      allowPullRequestApproval,
      cancelInProgress,
      concurrencyLimit,
      githubTokenPermission,
    });
    if (payload?.queue) {
      setSettings(payload as RepositoryActionsRunnerSettings);
      setMessage("Actions workflow settings saved.");
    }
  }

  async function scheduleJobs() {
    const payload = await postAction({ action: "schedule-jobs" });
    if (payload?.assigned) {
      setMessage(
        `${payload.assigned.length} queued ${payload.assigned.length === 1 ? "job" : "jobs"} assigned. ${payload.queuedJobs} still queued.`,
      );
    }
  }

  if (!settings) {
    return (
      <div className="card p-6">
        <p className="t-label mb-2">Actions runners unavailable</p>
        <p className="t-body" style={{ color: "var(--ink-2)" }}>
          {message || "Runner settings could not be loaded."}
        </p>
      </div>
    );
  }

  return (
    <div className="grid gap-6">
      <div className="grid gap-3 md:grid-cols-4">
        {[
          ["Queued jobs", settings.queue.queuedJobs],
          ["Online runners", settings.queue.onlineRunners],
          ["Busy runners", settings.queue.busyRunners],
          ["Offline runners", settings.queue.offlineRunners],
        ].map(([label, value]) => (
          <div className="card p-4" key={label}>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              {label}
            </p>
            <p className="t-h2 t-num mt-2">{value}</p>
          </div>
        ))}
      </div>

      {message ? (
        <div className="chip info w-fit" role="status">
          {message}
        </div>
      ) : null}

      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Runner pool
            </p>
            <h2 className="t-h2 mt-2">Self-hosted runners</h2>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-2)" }}
            >
              Repository runners accept queued workflow jobs when every
              requested runs-on label matches the runner labels.
            </p>
          </div>
          <button
            className="btn primary"
            disabled={pending === "schedule-jobs"}
            onClick={scheduleJobs}
            type="button"
          >
            Assign queued jobs
          </button>
        </div>
        <div className="mt-4 overflow-hidden rounded-[var(--radius)] border border-[var(--line)]">
          {settings.runners.length ? (
            settings.runners.map((runner) => (
              <RunnerRow key={runner.id} runner={runner} />
            ))
          ) : (
            <div className="p-5">
              <p className="t-sm">No runners registered yet.</p>
            </div>
          )}
        </div>
      </section>

      <section className="grid gap-6 lg:grid-cols-[1fr_1fr]">
        <div className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            New runner
          </p>
          <label className="mt-4 block">
            <span className="t-sm font-medium">Runner name</span>
            <input
              className="input mt-2 w-full"
              onChange={(event) => setRunnerName(event.target.value)}
              value={runnerName}
            />
          </label>
          <label className="mt-4 block">
            <span className="t-sm font-medium">Labels</span>
            <input
              className="input mt-2 w-full"
              onChange={(event) => setRunnerLabels(event.target.value)}
              value={runnerLabels}
            />
          </label>
          <button
            className="btn primary mt-4"
            disabled={pending === "create-runner"}
            onClick={createRunner}
            type="button"
          >
            Register runner
          </button>
          {settings.setup.dockerCommand ? (
            <pre className="t-mono-sm mt-4 overflow-auto rounded-[var(--radius)] bg-[var(--surface-2)] p-3">
              {settings.setup.dockerCommand}
            </pre>
          ) : null}
        </div>

        <div className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Scheduling
          </p>
          <label className="mt-4 block">
            <span className="t-sm font-medium">Concurrency limit</span>
            <input
              className="input mt-2 w-full"
              max={64}
              min={1}
              onChange={(event) =>
                setConcurrencyLimit(Number(event.target.value))
              }
              type="number"
              value={concurrencyLimit}
            />
          </label>
          <label className="mt-4 flex items-center gap-3">
            <input
              checked={cancelInProgress}
              onChange={(event) => setCancelInProgress(event.target.checked)}
              type="checkbox"
            />
            <span className="t-sm">
              Cancel older in-progress runs in the same concurrency group
            </span>
          </label>
          <button
            className="btn mt-4"
            disabled={pending === "update-settings"}
            onClick={saveSettings}
            type="button"
          >
            Save scheduling settings
          </button>
        </div>
      </section>

      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Workflow permissions
            </p>
            <h2 className="t-h2 mt-2">GITHUB_TOKEN policy</h2>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-2)" }}
            >
              New workflow runs mint an opengithub-owned token with these
              repository scopes. Secrets remain masked and environment-scoped
              values are held until protection rules approve the job.
            </p>
          </div>
          <span className="chip soft">
            {settings.workflowPermissions.githubTokenScopes.length} scopes
          </span>
        </div>

        <div className="mt-5 grid gap-4 lg:grid-cols-[1fr_1fr]">
          <fieldset className="grid gap-3">
            <legend className="t-label">Default token access</legend>
            {[
              ["read", "Read repository contents and metadata"],
              [
                "write",
                "Read and write repository contents, checks, packages, issues, and pull requests",
              ],
            ].map(([value, label]) => (
              <label className="flex items-start gap-3" key={value}>
                <input
                  checked={githubTokenPermission === value}
                  name="github-token-permission"
                  onChange={() => setGithubTokenPermission(value)}
                  type="radio"
                  value={value}
                />
                <span className="t-sm">{label}</span>
              </label>
            ))}
            <label className="mt-2 flex items-start gap-3">
              <input
                checked={allowPullRequestApproval}
                disabled={githubTokenPermission !== "write"}
                onChange={(event) =>
                  setAllowPullRequestApproval(event.target.checked)
                }
                type="checkbox"
              />
              <span className="t-sm">
                Allow Actions to create and approve pull requests
              </span>
            </label>
          </fieldset>

          <div>
            <p className="t-label">Minted scopes</p>
            <div className="mt-3 flex flex-wrap gap-2">
              {settings.workflowPermissions.githubTokenScopes.map((scope) => (
                <span className="chip soft t-mono-sm" key={scope}>
                  {scope}
                </span>
              ))}
            </div>
            <p className="t-xs mt-4">
              The Rust API persists this policy with runner settings and
              includes it in the Actions settings contract used by job
              provisioning.
            </p>
          </div>
        </div>

        <button
          className="btn primary mt-5"
          disabled={pending === "update-settings"}
          onClick={saveSettings}
          type="button"
        >
          Save workflow permissions
        </button>
      </section>
    </div>
  );
}
