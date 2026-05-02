"use client";

import { useMemo, useState } from "react";
import type { Webhook, WebhookCatalog, WebhookDelivery } from "@/lib/api";

const EVENT_LABELS: Record<string, string> = {
  push: "Push",
  pull_request: "Pull request",
  pull_request_review: "Pull request review",
  issues: "Issues",
  issue_comment: "Issue comment",
  release: "Release",
  workflow_run: "Workflow run",
  check_run: "Check run",
  ping: "Ping",
};

type Props = {
  catalog: WebhookCatalog | null;
  ownerLabel: string;
  endpointBase: string;
};

export function WebhooksSettingsPage({
  catalog,
  ownerLabel,
  endpointBase,
}: Props) {
  const supportedEvents = catalog?.supportedEvents ?? [
    "push",
    "pull_request",
    "issues",
    "release",
    "workflow_run",
    "check_run",
    "ping",
  ];
  const [hooks, setHooks] = useState<Webhook[]>(catalog?.hooks ?? []);
  const [mode, setMode] = useState<"push" | "everything" | "select">("push");
  const [selectedEvents, setSelectedEvents] = useState<string[]>(["push"]);
  const [contentType, setContentType] = useState<"json" | "form">("json");
  const [url, setUrl] = useState("");
  const [secret, setSecret] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [selectedDelivery, setSelectedDelivery] =
    useState<WebhookDelivery | null>(hooks[0]?.deliveries[0] ?? null);

  const effectiveEvents = useMemo(() => {
    if (mode === "push") return ["push"];
    if (mode === "everything") return supportedEvents;
    return selectedEvents.length ? selectedEvents : ["push"];
  }, [mode, selectedEvents, supportedEvents]);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setFeedback(null);
    if (!url.trim()) {
      setFeedback("Enter a payload URL before adding a webhook.");
      return;
    }
    setPending(true);
    try {
      const response = await fetch(endpointBase, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          url: url.trim(),
          contentType,
          secret: secret.trim() || null,
          events: effectiveEvents,
          active: true,
          sslVerify: true,
        }),
      });
      if (!response.ok)
        throw new Error(
          (await response.json().catch(() => null))?.error?.message ??
            "Webhook could not be created.",
        );
      const hook = (await response.json()) as Webhook;
      setHooks((current) => [hook, ...current]);
      setUrl("");
      setSecret("");
      setFeedback(
        "Webhook added. A ping event can now be delivered by the API.",
      );
    } catch (error) {
      setFeedback(
        error instanceof Error
          ? error.message
          : "Webhook could not be created.",
      );
    } finally {
      setPending(false);
    }
  }

  async function toggleHook(hook: Webhook) {
    setFeedback(null);
    const nextActive = !hook.active;
    setHooks((current) =>
      current.map((item) =>
        item.id === hook.id ? { ...item, active: nextActive } : item,
      ),
    );
    try {
      const response = await fetch(`${endpointBase}/${hook.id}`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ active: nextActive }),
      });
      if (!response.ok) throw new Error("Active state could not be saved.");
      const updated = (await response.json()) as Webhook;
      setHooks((current) =>
        current.map((item) =>
          item.id === hook.id
            ? { ...item, ...updated, deliveries: item.deliveries }
            : item,
        ),
      );
    } catch (error) {
      setHooks((current) =>
        current.map((item) =>
          item.id === hook.id ? { ...item, active: hook.active } : item,
        ),
      );
      setFeedback(
        error instanceof Error
          ? error.message
          : "Active state could not be saved.",
      );
    }
  }

  async function redeliver(hook: Webhook, delivery?: WebhookDelivery) {
    setFeedback(null);
    try {
      const response = await fetch(`${endpointBase}/${hook.id}/redeliveries`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ deliveryId: delivery?.id }),
      });
      if (!response.ok) throw new Error("Delivery could not be retried.");
      const nextDelivery = (await response.json()) as WebhookDelivery;
      setHooks((current) =>
        current.map((item) =>
          item.id === hook.id
            ? { ...item, deliveries: [nextDelivery, ...item.deliveries] }
            : item,
        ),
      );
      setSelectedDelivery(nextDelivery);
      setFeedback(
        "Redelivery queued and attempted with the stored raw payload.",
      );
    } catch (error) {
      setFeedback(
        error instanceof Error
          ? error.message
          : "Delivery could not be retried.",
      );
    }
  }

  return (
    <div className="space-y-6">
      <section className="card p-6" aria-labelledby="webhook-catalog-title">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Event catalog
        </p>
        <h2 className="t-h2 mt-2" id="webhook-catalog-title">
          Webhooks for {ownerLabel}
        </h2>
        <p className="t-body mt-2" style={{ color: "var(--ink-2)" }}>
          OpenGitHub signs every outbound POST with{" "}
          <span className="t-mono-sm">X-Hub-Signature-256</span> and includes{" "}
          <span className="t-mono-sm">X-GitHub-Event</span> plus{" "}
          <span className="t-mono-sm">X-GitHub-Delivery</span> headers.
        </p>
        <div className="mt-4 flex flex-wrap gap-2">
          {supportedEvents.map((event) => (
            <span className="chip" key={event}>
              {EVENT_LABELS[event] ?? event}
            </span>
          ))}
        </div>
      </section>

      <form
        className="card p-6"
        onSubmit={submit}
        aria-labelledby="add-webhook-title"
      >
        <h3 className="t-h3" id="add-webhook-title">
          Add webhook
        </h3>
        <div className="mt-5 grid gap-4 lg:grid-cols-2">
          <label className="grid gap-2 t-sm">
            Payload URL
            <input
              className="input"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://example.com/opengithub"
            />
          </label>
          <label className="grid gap-2 t-sm">
            Content type
            <select
              className="input"
              value={contentType}
              onChange={(event) =>
                setContentType(event.target.value as "json" | "form")
              }
            >
              <option value="json">application/json</option>
              <option value="form">application/x-www-form-urlencoded</option>
            </select>
          </label>
          <label className="grid gap-2 t-sm lg:col-span-2">
            Secret
            <input
              className="input"
              type="password"
              value={secret}
              onChange={(event) => setSecret(event.target.value)}
              placeholder="Used for HMAC-SHA256 signing"
            />
          </label>
        </div>
        <fieldset className="mt-5 grid gap-3">
          <legend className="t-label">
            Which events trigger this webhook?
          </legend>
          {[
            ["push", "Just the push event"],
            ["everything", "Send me everything"],
            ["select", "Let me select"],
          ].map(([value, label]) => (
            <label className="flex items-center gap-2 t-sm" key={value}>
              <input
                type="radio"
                name="event-mode"
                checked={mode === value}
                onChange={() =>
                  setMode(value as "push" | "everything" | "select")
                }
              />
              {label}
            </label>
          ))}
          {mode === "select" ? (
            <div className="mt-2 grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
              {supportedEvents.map((event) => (
                <label className="flex items-center gap-2 t-sm" key={event}>
                  <input
                    type="checkbox"
                    checked={selectedEvents.includes(event)}
                    onChange={(change) =>
                      setSelectedEvents((current) =>
                        change.target.checked
                          ? [...current, event]
                          : current.filter((item) => item !== event),
                      )
                    }
                  />
                  {EVENT_LABELS[event] ?? event}
                </label>
              ))}
            </div>
          ) : null}
        </fieldset>
        <div className="mt-5 flex items-center gap-3">
          <button className="btn primary" disabled={pending} type="submit">
            {pending ? "Adding…" : "Add webhook"}
          </button>
          {feedback ? (
            <p className="t-sm" role="status">
              {feedback}
            </p>
          ) : null}
        </div>
      </form>

      <section
        className="card overflow-hidden"
        aria-labelledby="hooks-table-title"
      >
        <div className="p-5" style={{ borderBottom: "1px solid var(--line)" }}>
          <h3 className="t-h3" id="hooks-table-title">
            Hooks
          </h3>
        </div>
        {hooks.length ? (
          hooks.map((hook) => (
            <div
              className="list-row grid gap-4 p-5 lg:grid-cols-[120px_minmax(0,1fr)_160px_160px]"
              key={hook.id}
            >
              <label className="flex items-center gap-2 t-sm">
                <input
                  type="checkbox"
                  checked={hook.active}
                  onChange={() => void toggleHook(hook)}
                />
                Active
              </label>
              <div>
                <p className="t-mono-sm break-all">{hook.url}</p>
                <p className="t-xs mt-1">
                  {hook.contentType} · {hook.events.join(", ")} ·{" "}
                  {hook.hasSecret ? "signed" : "unsigned"}
                </p>
              </div>
              <span
                className={`chip ${hook.deliveries[0]?.status === "delivered" ? "ok" : hook.deliveries[0]?.status === "failed" ? "err" : "warn"}`}
              >
                {hook.deliveries[0]?.responseStatus ?? "No deliveries"}
              </span>
              <button
                className="btn sm"
                type="button"
                onClick={() => void redeliver(hook, hook.deliveries[0])}
              >
                Redeliver
              </button>
            </div>
          ))
        ) : (
          <div className="p-6">
            <p className="t-body">
              No webhooks yet. Add an endpoint to start receiving signed event
              payloads.
            </p>
          </div>
        )}
      </section>

      <section className="card p-6" aria-labelledby="deliveries-title">
        <h3 className="t-h3" id="deliveries-title">
          Recent deliveries
        </h3>
        <div className="mt-4 grid gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
          <div className="grid gap-2">
            {hooks
              .flatMap((hook) =>
                hook.deliveries.map((delivery) => ({ hook, delivery })),
              )
              .map(({ hook, delivery }) => (
                <button
                  className="btn ghost justify-start"
                  key={delivery.id}
                  type="button"
                  onClick={() => setSelectedDelivery(delivery)}
                >
                  {delivery.event} ·{" "}
                  {delivery.responseStatus ?? delivery.status}
                  <span className="t-xs ml-auto">
                    {hook.url.replace(/^https?:\/\//, "")}
                  </span>
                </button>
              ))}
          </div>
          <pre
            className="t-mono-sm min-h-48 overflow-auto rounded-md p-4"
            style={{
              background: "var(--surface-2)",
              border: "1px solid var(--line)",
              color: "var(--ink-2)",
            }}
          >
            {selectedDelivery
              ? JSON.stringify(
                  {
                    headers: selectedDelivery.requestHeaders,
                    payload: safeJson(selectedDelivery.requestBody),
                    responseStatus: selectedDelivery.responseStatus,
                    responseBody: selectedDelivery.responseBody,
                  },
                  null,
                  2,
                )
              : "Select a delivery to inspect the signed request and response."}
          </pre>
        </div>
      </section>
    </div>
  );
}

function safeJson(value: string) {
  try {
    return JSON.parse(value);
  } catch {
    return value;
  }
}
