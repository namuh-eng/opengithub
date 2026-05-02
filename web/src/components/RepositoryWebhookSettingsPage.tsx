import Link from "next/link";
import type {
  RepositoryOverview,
  RepositoryWebhookDetailFetchResult,
  RepositoryWebhookSettings,
  RepositoryWebhookSettingsFetchResult,
  WebhookDeliveryDetailFetchResult,
  WebhookDeliverySummary,
} from "@/lib/api";

type RepositoryWebhookSettingsPageProps = {
  deliveryResult?: WebhookDeliveryDetailFetchResult | null;
  detailResult?: RepositoryWebhookDetailFetchResult | null;
  mode?: "new";
  repository: RepositoryOverview;
  settingsResult: RepositoryWebhookSettingsFetchResult;
};

function formatDate(value: string | null | undefined) {
  if (!value) return "Not delivered";
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function formatDuration(value: number | null) {
  return value === null ? "pending" : `${value}ms`;
}

function statusChipClass(status: string) {
  if (status === "delivered") return "chip ok";
  if (status === "failed") return "chip err";
  return "chip warn";
}

function statusLabel(delivery: WebhookDeliverySummary | null) {
  if (!delivery) return "No deliveries";
  const response = delivery.responseStatus
    ? ` · ${delivery.responseStatus}`
    : "";
  return `${delivery.status}${response} · ${formatDuration(delivery.durationMs)}`;
}

function eventSummary(settings: RepositoryWebhookSettings, events: string[]) {
  if (events.length === 0) return "No events selected";
  const labels = new Map(
    settings.eventDefinitions.map((event) => [event.name, event.label]),
  );
  return events.map((event) => labels.get(event) ?? event).join(", ");
}

function contentTypeLabel(value: string) {
  if (value === "json") return "application/json";
  if (value === "form") return "application/x-www-form-urlencoded";
  return value;
}

function HeadersBlock({ label, value }: { label: string; value: unknown }) {
  const text =
    value && typeof value === "object"
      ? JSON.stringify(value, null, 2)
      : String(value ?? "{}");
  return (
    <div>
      <p className="t-label">{label}</p>
      <pre
        className="t-mono-sm mt-2 overflow-auto rounded-md p-3"
        style={{
          background: "var(--surface-2)",
          border: "1px solid var(--line)",
          color: "var(--ink-2)",
          maxHeight: "180px",
        }}
      >
        {text}
      </pre>
    </div>
  );
}

function WebhookSettingsUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryWebhookSettingsFetchResult, { ok: true }>;
}) {
  const isForbidden = result.status === 403;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${isForbidden ? "warn" : "err"}`}>
        {isForbidden ? "Admin access required" : "Unavailable"}
      </span>
      <h2 className="t-h2 mt-4">
        {isForbidden
          ? "Webhook settings are restricted"
          : "Webhook settings could not load"}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {isForbidden
          ? "Only repository admins can view webhook endpoints, secrets, and delivery history."
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
  );
}

function NewWebhookPreview({
  repository,
  settings,
}: {
  repository: RepositoryOverview;
  settings: RepositoryWebhookSettings;
}) {
  return (
    <section className="card p-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label">New webhook</p>
          <h2 className="t-h2 mt-2">Add endpoint</h2>
          <p
            className="t-body mt-2 max-w-2xl"
            style={{ color: "var(--ink-3)" }}
          >
            Choose which repository events should be delivered before saving an
            HTTPS endpoint and write-only secret.
          </p>
        </div>
        <Link
          className="btn"
          href={`/${repository.owner_login}/${repository.name}/settings/hooks`}
        >
          Back to hooks
        </Link>
      </div>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        {settings.eventDefinitions.map((event) => (
          <div
            className="rounded-md p-3"
            key={event.name}
            style={{
              border: "1px solid var(--line)",
              background: "var(--surface-2)",
            }}
          >
            <p className="t-h3">{event.label}</p>
            <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
              {event.description}
            </p>
            <p className="t-mono-sm mt-2" style={{ color: "var(--ink-4)" }}>
              {event.name}
            </p>
          </div>
        ))}
      </div>
    </section>
  );
}

function WebhookRows({
  repository,
  settings,
}: {
  repository: RepositoryOverview;
  settings: RepositoryWebhookSettings;
}) {
  if (settings.hooks.length === 0) {
    return (
      <section className="card p-6">
        <span className="chip soft">No webhooks</span>
        <h2 className="t-h2 mt-4">Receive repository events elsewhere</h2>
        <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-2)" }}>
          Webhooks send push, issue, pull request, release, workflow, package,
          and Pages events to your own HTTPS endpoint. Secrets are write-only
          and never returned by the API.
        </p>
        <div className="mt-5 flex flex-wrap gap-2">
          <Link
            className="btn primary"
            href={`/${repository.owner_login}/${repository.name}/settings/hooks?new=webhook`}
          >
            Add webhook
          </Link>
          <Link className="btn" href="/docs">
            API docs
          </Link>
        </div>
      </section>
    );
  }

  return (
    <section className="card overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-3 p-4">
        <div>
          <p className="t-label">Configured endpoints</p>
          <h2 className="t-h2 mt-1">{settings.hooks.length} webhooks</h2>
        </div>
        <Link
          className="btn primary"
          href={`/${repository.owner_login}/${repository.name}/settings/hooks?new=webhook`}
        >
          Add webhook
        </Link>
      </div>
      <div style={{ borderTop: "1px solid var(--line)" }}>
        {settings.hooks.map((hook) => (
          <div className="list-row items-start" key={hook.id}>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <Link
                  className="t-mono-sm min-w-0 break-all font-semibold hover:underline"
                  href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}`}
                >
                  {hook.payloadUrl}
                </Link>
                <span className={hook.active ? "chip ok" : "chip warn"}>
                  {hook.active ? "Active" : "Inactive"}
                </span>
                {hook.secretConfigured ? (
                  <span className="chip soft">Secret configured</span>
                ) : null}
              </div>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                {eventSummary(settings, hook.events)}
              </p>
              <p className="t-xs mt-1">
                {contentTypeLabel(hook.contentType)} · SSL{" "}
                {hook.sslVerify ? "verified" : "not verified"} · updated{" "}
                {formatDate(hook.updatedAt)}
              </p>
            </div>
            <div className="flex shrink-0 flex-col items-end gap-2">
              <span
                className={
                  hook.latestDelivery
                    ? statusChipClass(hook.latestDelivery.status)
                    : "chip soft"
                }
              >
                {statusLabel(hook.latestDelivery)}
              </span>
              <div className="flex flex-wrap justify-end gap-2">
                <Link
                  className="btn sm"
                  href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?edit=webhook`}
                >
                  Edit
                </Link>
                <Link
                  className="btn sm"
                  href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?test=ping`}
                >
                  Test
                </Link>
                <Link
                  className="btn sm"
                  href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?delete=confirm`}
                >
                  Delete
                </Link>
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function DetailUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Exclude<RepositoryWebhookDetailFetchResult, { ok: true }>;
}) {
  return (
    <section className="card p-6" role="status">
      <span className="chip err">Unavailable</span>
      <h2 className="t-h2 mt-4">Webhook detail could not load</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {result.message}
      </p>
      <Link
        className="btn mt-5"
        href={`/${repository.owner_login}/${repository.name}/settings/hooks`}
      >
        Back to webhooks
      </Link>
    </section>
  );
}

function WebhookDetail({
  deliveryResult,
  detailResult,
  repository,
  settings,
}: {
  deliveryResult?: WebhookDeliveryDetailFetchResult | null;
  detailResult: RepositoryWebhookDetailFetchResult;
  repository: RepositoryOverview;
  settings: RepositoryWebhookSettings;
}) {
  if (!detailResult.ok) {
    return <DetailUnavailable repository={repository} result={detailResult} />;
  }

  const { hook, deliveries } = detailResult.detail;
  return (
    <div className="grid gap-6">
      <section className="card p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <p className="t-label">Webhook detail</p>
            <h2 className="t-h2 mt-2 break-all">{hook.payloadUrl}</h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {eventSummary(settings, hook.events)}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link
              className="btn"
              href={`/${repository.owner_login}/${repository.name}/settings/hooks`}
            >
              All webhooks
            </Link>
            <Link
              className="btn"
              href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?edit=webhook`}
            >
              Edit
            </Link>
          </div>
        </div>
        <div className="mt-5 flex flex-wrap gap-2">
          <span className={hook.active ? "chip ok" : "chip warn"}>
            {hook.active ? "Active" : "Inactive"}
          </span>
          <span className="chip soft">
            {contentTypeLabel(hook.contentType)}
          </span>
          <span className="chip soft">
            SSL {hook.sslVerify ? "verified" : "not verified"}
          </span>
          <span className="chip soft">
            Secret {hook.secretConfigured ? "configured" : "not configured"}
          </span>
        </div>
      </section>

      <section className="card overflow-hidden">
        <div className="tabs">
          <span className="tab active">Recent deliveries</span>
          <span className="tab">Configuration</span>
        </div>
        {deliveries.length === 0 ? (
          <div className="p-5">
            <p className="t-body" style={{ color: "var(--ink-2)" }}>
              No deliveries have been recorded for this webhook yet.
            </p>
          </div>
        ) : (
          <div>
            {deliveries.map((delivery) => (
              <Link
                className="list-row"
                href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?delivery=${delivery.id}`}
                key={delivery.id}
              >
                <span className={statusChipClass(delivery.status)}>
                  {delivery.status}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="t-mono-sm block break-all">
                    {delivery.guid}
                  </span>
                  <span className="t-xs">
                    {delivery.event} · attempt {delivery.attemptCount} ·{" "}
                    {formatDate(delivery.createdAt)}
                  </span>
                </span>
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {delivery.responseStatus ?? "queued"} ·{" "}
                  {formatDuration(delivery.durationMs)}
                </span>
              </Link>
            ))}
          </div>
        )}
      </section>

      {deliveryResult?.ok ? (
        <section className="grid gap-4 md:grid-cols-2">
          <div className="card p-5">
            <p className="t-label">Request</p>
            <p className="t-h3 mt-2">
              {deliveryResult.delivery.summary.event} ·{" "}
              {deliveryResult.delivery.summary.guid}
            </p>
            <HeadersBlock
              label="Headers"
              value={deliveryResult.delivery.requestHeaders}
            />
            <HeadersBlock
              label="Body"
              value={
                deliveryResult.delivery.requestBodyExcerpt ??
                deliveryResult.delivery.requestBodyStorageKey ??
                ""
              }
            />
          </div>
          <div className="card p-5">
            <p className="t-label">Response</p>
            <p className="t-h3 mt-2">
              {statusLabel(deliveryResult.delivery.summary)}
            </p>
            <HeadersBlock
              label="Headers"
              value={deliveryResult.delivery.responseHeaders}
            />
            <HeadersBlock
              label="Body"
              value={
                deliveryResult.delivery.responseBodyExcerpt ??
                deliveryResult.delivery.responseBodyStorageKey ??
                deliveryResult.delivery.terminalError ??
                ""
              }
            />
            <Link
              className="btn mt-4"
              href={`/${repository.owner_login}/${repository.name}/settings/hooks/${hook.id}?delivery=${deliveryResult.delivery.summary.id}&redeliver=confirm`}
            >
              Redeliver
            </Link>
          </div>
        </section>
      ) : deliveryResult && !deliveryResult.ok ? (
        <section className="card p-5" role="status">
          <span className="chip err">Delivery unavailable</span>
          <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
            {deliveryResult.message}
          </p>
        </section>
      ) : null}
    </div>
  );
}

export function RepositoryWebhookSettingsPage({
  deliveryResult = null,
  detailResult = null,
  mode,
  repository,
  settingsResult,
}: RepositoryWebhookSettingsPageProps) {
  if (!settingsResult.ok) {
    return (
      <WebhookSettingsUnavailable
        repository={repository}
        result={settingsResult}
      />
    );
  }

  const settings = settingsResult.settings;
  return (
    <div className="grid gap-6">
      <section className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label">Repository webhooks</p>
          <h2 className="t-h2 mt-1">
            {settings.ownerLogin}/{settings.name}
          </h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Viewer: {settings.viewerPermission} · {settings.visibility}
          </p>
        </div>
        <Link
          className="btn primary"
          href={`/${repository.owner_login}/${repository.name}/settings/hooks?new=webhook`}
        >
          Add webhook
        </Link>
      </section>

      {mode === "new" ? (
        <NewWebhookPreview repository={repository} settings={settings} />
      ) : null}

      {detailResult ? (
        <WebhookDetail
          deliveryResult={deliveryResult}
          detailResult={detailResult}
          repository={repository}
          settings={settings}
        />
      ) : (
        <WebhookRows repository={repository} settings={settings} />
      )}
    </div>
  );
}
