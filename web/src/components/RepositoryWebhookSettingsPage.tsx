"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import type {
  RepositoryOverview,
  RepositoryWebhookDetail,
  RepositoryWebhookDetailFetchResult,
  RepositoryWebhookSettings,
  RepositoryWebhookSettingsFetchResult,
  WebhookDeliveryDetailFetchResult,
  WebhookDeliverySummary,
  WebhookEventSelection,
} from "@/lib/api";

type RepositoryWebhookSettingsPageProps = {
  basePath?: string;
  deliveryResult?: WebhookDeliveryDetailFetchResult | null;
  detailResult?: RepositoryWebhookDetailFetchResult | null;
  intent?: "delete" | "edit" | "new" | "ping" | "redeliver";
  repository: RepositoryOverview;
  scopeLabel?: string;
  scopeNoun?: "organization" | "repository";
  settingsResult: RepositoryWebhookSettingsFetchResult;
};

type WebhookAction =
  | {
      action: "create-webhook" | "update-webhook";
      active: boolean;
      contentType: string;
      events: string[];
      eventSelection: WebhookEventSelection;
      hookId?: string;
      payloadUrl: string;
      secret?: string;
      sslVerify: boolean;
    }
  | { action: "delete-webhook"; hookId: string }
  | { action: "ping-webhook"; hookId: string }
  | { action: "redeliver-delivery"; hookId: string; deliveryId: string };

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

function contentTypeLabel(value: string) {
  if (value === "json") return "application/json";
  if (value === "form") return "application/x-www-form-urlencoded";
  return value;
}

function eventSummary(settings: RepositoryWebhookSettings, events: string[]) {
  if (events.length === 0) return "No events selected";
  if (events.includes("*")) return "All supported events";
  const labels = new Map(
    settings.eventDefinitions.map((event) => [event.name, event.label]),
  );
  return events.map((event) => labels.get(event) ?? event).join(", ");
}

function actionResultSettings(body: unknown): RepositoryWebhookSettings | null {
  if (!body || typeof body !== "object") return null;
  const object = body as { hooks?: unknown; settings?: unknown };
  if (object.settings && typeof object.settings === "object") {
    return object.settings as RepositoryWebhookSettings;
  }
  if (Array.isArray(object.hooks)) {
    return object as RepositoryWebhookSettings;
  }
  return null;
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
  basePath,
  result,
  scopeNoun = "repository",
}: {
  basePath: string;
  result: Exclude<RepositoryWebhookSettingsFetchResult, { ok: true }>;
  scopeNoun?: "organization" | "repository";
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
          ? `Only ${scopeNoun} owners can view webhook endpoints, secrets, and delivery history.`
          : result.message}
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Link className="btn" href={basePath.replace(/\/settings\/hooks$/, "")}>
          {scopeNoun === "organization"
            ? "Organization profile"
            : "Repository Code"}
        </Link>
        <Link className="btn" href="/dashboard">
          Dashboard
        </Link>
      </div>
    </section>
  );
}

function WebhookForm({
  busy,
  error,
  hook,
  onCancel,
  onSubmit,
  settings,
}: {
  busy: boolean;
  error: string | null;
  hook?: RepositoryWebhookDetail["hook"];
  onCancel: () => void;
  onSubmit: (action: WebhookAction) => void;
  settings: RepositoryWebhookSettings;
}) {
  const [selection, setSelection] = useState<WebhookEventSelection>(
    hook?.eventSelection ?? "push",
  );
  const title = hook ? "Edit webhook" : "Add webhook";
  return (
    <section className="card p-5" id="webhook-editor">
      <span className="chip active">Webhook</span>
      <h2 className="t-h3 mt-3">{title}</h2>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        Saves are confirmed by the Rust API. Secrets are write-only and remain
        unchanged when the field is left blank.
      </p>
      <form
        className="mt-5 grid gap-5"
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const payloadUrl = String(form.get("payloadUrl") ?? "").trim();
          const eventSelection = String(
            form.get("eventSelection") ?? "push",
          ) as WebhookEventSelection;
          const events =
            eventSelection === "selected"
              ? form.getAll("events").map(String).filter(Boolean)
              : [];
          if (eventSelection === "selected" && events.length === 0) {
            onSubmit({
              action: hook ? "update-webhook" : "create-webhook",
              active: form.get("active") === "on",
              contentType: String(form.get("contentType") ?? "json"),
              events,
              eventSelection,
              hookId: hook?.id,
              payloadUrl: "",
              secret: "",
              sslVerify: form.get("sslVerify") === "on",
            });
            return;
          }
          onSubmit({
            action: hook ? "update-webhook" : "create-webhook",
            active: form.get("active") === "on",
            contentType: String(form.get("contentType") ?? "json"),
            events,
            eventSelection,
            hookId: hook?.id,
            payloadUrl,
            secret: String(form.get("secret") ?? ""),
            sslVerify: form.get("sslVerify") === "on",
          });
        }}
      >
        <label className="grid gap-2" htmlFor="webhook-payload-url">
          <span className="t-label">Payload URL</span>
          <input
            className="input"
            defaultValue={hook?.payloadUrl ?? ""}
            id="webhook-payload-url"
            name="payloadUrl"
            placeholder="https://receiver.example.com/hooks/opengithub"
            required
            type="url"
          />
        </label>

        <div className="grid gap-4 md:grid-cols-2">
          <fieldset className="grid gap-2">
            <legend className="t-label">Content type</legend>
            <div className="flex flex-wrap gap-2">
              {["json", "form"].map((value) => (
                <label className="chip soft" key={value}>
                  <input
                    className="mr-2"
                    defaultChecked={(hook?.contentType ?? "json") === value}
                    name="contentType"
                    type="radio"
                    value={value}
                  />
                  {contentTypeLabel(value)}
                </label>
              ))}
            </div>
          </fieldset>
          <label className="grid gap-2" htmlFor="webhook-secret">
            <span className="t-label">Secret</span>
            <input
              autoComplete="new-password"
              className="input"
              id="webhook-secret"
              minLength={8}
              name="secret"
              placeholder={
                hook?.secretConfigured
                  ? "Leave blank to keep current secret"
                  : ""
              }
              type="password"
            />
          </label>
        </div>

        <fieldset className="grid gap-2">
          <legend className="t-label">Events</legend>
          <div className="flex flex-wrap gap-2">
            {[
              ["push", "Just push"],
              ["everything", "Send me everything"],
              ["selected", "Let me select individual events"],
            ].map(([value, label]) => (
              <label className="chip soft" key={value}>
                <input
                  className="mr-2"
                  defaultChecked={(hook?.eventSelection ?? "push") === value}
                  name="eventSelection"
                  onChange={() => setSelection(value as WebhookEventSelection)}
                  type="radio"
                  value={value}
                />
                {label}
              </label>
            ))}
          </div>
        </fieldset>

        {selection === "selected" ? (
          <fieldset className="grid gap-2">
            <legend className="t-label">Individual events</legend>
            <div className="grid gap-2 md:grid-cols-2">
              {settings.eventDefinitions.map((event) => (
                <label
                  className="rounded-md p-3"
                  key={event.name}
                  style={{
                    background: "var(--surface-2)",
                    border: "1px solid var(--line)",
                  }}
                >
                  <input
                    className="mr-2"
                    defaultChecked={hook?.events.includes(event.name)}
                    name="events"
                    type="checkbox"
                    value={event.name}
                  />
                  <span className="t-h3">{event.label}</span>
                  <span
                    className="t-xs mt-1 block"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {event.description}
                  </span>
                </label>
              ))}
            </div>
          </fieldset>
        ) : null}

        <div className="flex flex-wrap gap-3">
          <label className="chip soft">
            <input
              className="mr-2"
              defaultChecked={hook?.sslVerify ?? true}
              name="sslVerify"
              type="checkbox"
            />
            Verify SSL
          </label>
          <label className="chip soft">
            <input
              className="mr-2"
              defaultChecked={hook?.active ?? true}
              name="active"
              type="checkbox"
            />
            Active
          </label>
        </div>

        {error ? (
          <p className="t-sm" role="alert" style={{ color: "var(--err)" }}>
            {error}
          </p>
        ) : null}

        <div className="flex flex-wrap justify-end gap-2">
          <button className="btn" onClick={onCancel} type="button">
            Cancel
          </button>
          <button className="btn primary" disabled={busy} type="submit">
            {busy ? "Saving..." : title}
          </button>
        </div>
      </form>
    </section>
  );
}

function ConfirmPanel({
  busy,
  error,
  hook,
  kind,
  onCancel,
  onConfirm,
  selectedDelivery,
}: {
  busy: boolean;
  error: string | null;
  hook: RepositoryWebhookDetail["hook"];
  kind: "delete" | "ping" | "redeliver";
  onCancel: () => void;
  onConfirm: () => void;
  selectedDelivery?: WebhookDeliverySummary;
}) {
  const [confirmation, setConfirmation] = useState("");
  const requiresText = kind === "delete";
  const canConfirm = !requiresText || confirmation === hook.payloadUrl;
  const copy =
    kind === "delete"
      ? "Delete this webhook and remove it from repository settings."
      : kind === "redeliver"
        ? "Create a new delivery linked to the selected delivery."
        : "Create a new ping delivery for this webhook.";
  return (
    <section
      className="card p-5"
      role={kind === "delete" ? "alertdialog" : "dialog"}
    >
      <span className={kind === "delete" ? "chip err" : "chip warn"}>
        {kind === "delete" ? "Confirm delete" : "Confirm delivery"}
      </span>
      <h2 className="t-h3 mt-3">
        {kind === "delete"
          ? "Delete webhook"
          : kind === "redeliver"
            ? "Redeliver webhook event"
            : "Test webhook"}
      </h2>
      <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
        {copy}
      </p>
      <p className="t-mono-sm mt-3 break-all" style={{ color: "var(--ink-3)" }}>
        {selectedDelivery?.guid ?? hook.payloadUrl}
      </p>
      {requiresText ? (
        <label className="mt-4 grid gap-2" htmlFor="webhook-delete-confirm">
          <span className="t-label">Type payload URL to confirm</span>
          <input
            className="input"
            id="webhook-delete-confirm"
            onChange={(event) => setConfirmation(event.currentTarget.value)}
            value={confirmation}
          />
        </label>
      ) : null}
      {error ? (
        <p className="t-sm mt-3" role="alert" style={{ color: "var(--err)" }}>
          {error}
        </p>
      ) : null}
      <div className="mt-5 flex flex-wrap justify-end gap-2">
        <button className="btn" onClick={onCancel} type="button">
          Cancel
        </button>
        <button
          className="btn primary"
          disabled={busy || !canConfirm}
          onClick={onConfirm}
          type="button"
        >
          {busy
            ? "Working..."
            : kind === "delete"
              ? "Delete webhook"
              : kind === "redeliver"
                ? "Redeliver"
                : "Send ping"}
        </button>
      </div>
    </section>
  );
}

function WebhookRows({
  basePath,
  onDelete,
  onEdit,
  onPing,
  scopeNoun = "repository",
  settings,
}: {
  basePath: string;
  onDelete: (hook: RepositoryWebhookDetail["hook"]) => void;
  onEdit: (hook: RepositoryWebhookDetail["hook"]) => void;
  onPing: (hook: RepositoryWebhookDetail["hook"]) => void;
  scopeNoun?: "organization" | "repository";
  settings: RepositoryWebhookSettings;
}) {
  if (settings.hooks.length === 0) {
    return (
      <section className="card p-6">
        <span className="chip soft">No webhooks</span>
        <h2 className="t-h2 mt-4">Receive {scopeNoun} events elsewhere</h2>
        <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-2)" }}>
          Webhooks send push, issue, pull request, release, workflow, package,
          and Pages events to your own HTTPS endpoint. Secrets are write-only
          and never returned by the API.
        </p>
        <div className="mt-5 flex flex-wrap gap-2">
          <Link className="btn primary" href={`${basePath}?new=webhook`}>
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
        <Link className="btn primary" href={`${basePath}?new=webhook`}>
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
                  href={`${basePath}/${hook.id}`}
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
                <button
                  className="btn sm"
                  onClick={() => onEdit(hook)}
                  type="button"
                >
                  Edit
                </button>
                <button
                  className="btn sm"
                  onClick={() => onPing(hook)}
                  type="button"
                >
                  Test
                </button>
                <button
                  className="btn sm"
                  onClick={() => onDelete(hook)}
                  type="button"
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function DetailUnavailable({
  basePath,
  result,
}: {
  basePath: string;
  result: Exclude<RepositoryWebhookDetailFetchResult, { ok: true }>;
}) {
  return (
    <section className="card p-6" role="status">
      <span className="chip err">Unavailable</span>
      <h2 className="t-h2 mt-4">Webhook detail could not load</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {result.message}
      </p>
      <Link className="btn mt-5" href={basePath}>
        Back to webhooks
      </Link>
    </section>
  );
}

function WebhookDetail({
  basePath,
  deliveryResult,
  detail,
  onEdit,
  onPing,
  onRedeliver,
  settings,
}: {
  basePath: string;
  deliveryResult?: WebhookDeliveryDetailFetchResult | null;
  detail: RepositoryWebhookDetail;
  onEdit: (hook: RepositoryWebhookDetail["hook"]) => void;
  onPing: (hook: RepositoryWebhookDetail["hook"]) => void;
  onRedeliver: (
    hook: RepositoryWebhookDetail["hook"],
    delivery: WebhookDeliverySummary,
  ) => void;
  settings: RepositoryWebhookSettings;
}) {
  const { hook, deliveries } = detail;
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
            <Link className="btn" href={basePath}>
              All webhooks
            </Link>
            <button className="btn" onClick={() => onEdit(hook)} type="button">
              Edit
            </button>
            <button className="btn" onClick={() => onPing(hook)} type="button">
              Test
            </button>
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
                href={`${basePath}/${hook.id}?delivery=${delivery.id}`}
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
            <button
              className="btn mt-4"
              onClick={() => onRedeliver(hook, deliveryResult.delivery.summary)}
              type="button"
            >
              Redeliver
            </button>
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
  basePath: providedBasePath,
  deliveryResult = null,
  detailResult = null,
  intent,
  repository,
  scopeLabel = "Repository webhooks",
  scopeNoun = "repository",
  settingsResult,
}: RepositoryWebhookSettingsPageProps) {
  const basePath =
    providedBasePath ??
    `/${repository.owner_login}/${repository.name}/settings/hooks`;
  if (!settingsResult.ok) {
    return (
      <WebhookSettingsUnavailable
        basePath={basePath}
        result={settingsResult}
        scopeNoun={scopeNoun}
      />
    );
  }

  return (
    <RepositoryWebhookSettingsContent
      basePath={basePath}
      deliveryResult={deliveryResult}
      detailResult={detailResult}
      initialIntent={intent}
      initialSettings={settingsResult.settings}
      scopeLabel={scopeLabel}
      scopeNoun={scopeNoun}
    />
  );
}

function RepositoryWebhookSettingsContent({
  basePath,
  deliveryResult,
  detailResult,
  initialIntent,
  initialSettings,
  scopeLabel,
  scopeNoun,
}: {
  basePath: string;
  deliveryResult?: WebhookDeliveryDetailFetchResult | null;
  detailResult?: RepositoryWebhookDetailFetchResult | null;
  initialIntent?: "delete" | "edit" | "new" | "ping" | "redeliver";
  initialSettings: RepositoryWebhookSettings;
  scopeLabel: string;
  scopeNoun: "organization" | "repository";
}) {
  const [settings, setSettings] = useState(initialSettings);
  const detail = detailResult?.ok ? detailResult.detail : null;
  const [editor, setEditor] = useState<
    RepositoryWebhookDetail["hook"] | "new" | null
  >(
    initialIntent === "new"
      ? "new"
      : initialIntent === "edit" && detail
        ? detail.hook
        : null,
  );
  const [confirmation, setConfirmation] = useState<{
    delivery?: WebhookDeliverySummary;
    hook: RepositoryWebhookDetail["hook"];
    kind: "delete" | "ping" | "redeliver";
  } | null>(
    detail && initialIntent === "delete"
      ? { hook: detail.hook, kind: "delete" }
      : detail && initialIntent === "ping"
        ? { hook: detail.hook, kind: "ping" }
        : detail && initialIntent === "redeliver" && deliveryResult?.ok
          ? {
              delivery: deliveryResult.delivery.summary,
              hook: detail.hook,
              kind: "redeliver",
            }
          : null,
  );
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const actionUrl = `${basePath}/actions`;
  const router = useRouter();
  const currentDetail = useMemo(() => {
    if (!detail) return null;
    const hook = settings.hooks.find((item) => item.id === detail.hook.id);
    return hook ? { ...detail, hook } : detail;
  }, [detail, settings.hooks]);

  async function mutate(action: WebhookAction, success: string) {
    if (
      (action.action === "create-webhook" ||
        action.action === "update-webhook") &&
      (!action.payloadUrl ||
        (action.eventSelection === "selected" && action.events.length === 0))
    ) {
      setError(
        action.eventSelection === "selected" && action.events.length === 0
          ? "Select at least one individual event."
          : "Payload URL is required.",
      );
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    const response = await fetch(actionUrl, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(action),
    });
    const body = await response.json().catch(() => null);
    setBusy(false);
    if (!response.ok) {
      setError(body?.error?.message ?? "Repository webhook update failed.");
      return;
    }
    const nextSettings = actionResultSettings(body);
    if (nextSettings) setSettings(nextSettings);
    setEditor(null);
    setConfirmation(null);
    setNotice(success);
    if (
      action.action === "create-webhook" &&
      nextSettings &&
      body &&
      typeof body === "object" &&
      "delivery" in body
    ) {
      const createdHook = nextSettings.hooks.find(
        (hook) => hook.payloadUrl === action.payloadUrl,
      );
      const delivery = (body as { delivery?: WebhookDeliverySummary }).delivery;
      if (createdHook && delivery) {
        router.push(`${basePath}/${createdHook.id}?delivery=${delivery.id}`);
      }
    }
  }

  return (
    <div className="grid gap-6">
      <section className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label">{scopeLabel}</p>
          <h2 className="t-h2 mt-1">
            {settings.ownerLogin}/{settings.name}
          </h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Viewer: {settings.viewerPermission} · {settings.visibility}
          </p>
        </div>
        <Link className="btn primary" href={`${basePath}?new=webhook`}>
          Add webhook
        </Link>
      </section>

      {notice ? (
        <p className="chip ok w-fit" role="status">
          {notice}
        </p>
      ) : null}

      {editor ? (
        <WebhookForm
          busy={busy}
          error={error}
          hook={editor === "new" ? undefined : editor}
          onCancel={() => {
            setEditor(null);
            setError(null);
          }}
          onSubmit={(action) => {
            const normalized =
              action.action === "update-webhook"
                ? {
                    ...action,
                    hookId:
                      action.hookId ?? (editor !== "new" ? editor.id : ""),
                  }
                : action;
            void mutate(
              normalized,
              action.action === "create-webhook"
                ? "Webhook created and ping delivery queued."
                : "Webhook settings saved.",
            );
          }}
          settings={settings}
        />
      ) : null}

      {confirmation ? (
        <ConfirmPanel
          busy={busy}
          error={error}
          hook={confirmation.hook}
          kind={confirmation.kind}
          onCancel={() => {
            setConfirmation(null);
            setError(null);
          }}
          onConfirm={() => {
            const hookId = confirmation.hook.id;
            if (confirmation.kind === "delete") {
              void mutate(
                { action: "delete-webhook", hookId },
                "Webhook deleted.",
              );
            } else if (
              confirmation.kind === "redeliver" &&
              confirmation.delivery
            ) {
              void mutate(
                {
                  action: "redeliver-delivery",
                  deliveryId: confirmation.delivery.id,
                  hookId,
                },
                "Webhook delivery queued again.",
              );
            } else {
              void mutate(
                { action: "ping-webhook", hookId },
                "Webhook ping delivery queued.",
              );
            }
          }}
          selectedDelivery={confirmation.delivery}
        />
      ) : null}

      {detailResult && !detailResult.ok ? (
        <DetailUnavailable basePath={basePath} result={detailResult} />
      ) : currentDetail ? (
        <WebhookDetail
          basePath={basePath}
          deliveryResult={deliveryResult}
          detail={currentDetail}
          onEdit={(hook) => {
            setEditor(hook);
            setError(null);
          }}
          onPing={(hook) => {
            setConfirmation({ hook, kind: "ping" });
            setError(null);
          }}
          onRedeliver={(hook, delivery) => {
            setConfirmation({ delivery, hook, kind: "redeliver" });
            setError(null);
          }}
          settings={settings}
        />
      ) : (
        <WebhookRows
          basePath={basePath}
          onDelete={(hook) => {
            setConfirmation({ hook, kind: "delete" });
            setError(null);
          }}
          onEdit={(hook) => {
            setEditor(hook);
            setError(null);
          }}
          onPing={(hook) => {
            setConfirmation({ hook, kind: "ping" });
            setError(null);
          }}
          scopeNoun={scopeNoun}
          settings={settings}
        />
      )}
    </div>
  );
}
