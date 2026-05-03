"use client";

import { useMemo, useState } from "react";
import type {
  NotificationDeliveryPreference,
  NotificationDeliverySettings,
} from "@/lib/api";

type NotificationDeliverySettingsPageProps = {
  initialSettings: NotificationDeliverySettings;
};

type ChannelDraft = {
  key: string;
  channels: string[];
};

const CHANNEL_LABELS: Record<string, string> = {
  web: "On GitHub",
  email: "Email",
  cli: "CLI",
};

export function NotificationDeliverySettingsPage({
  initialSettings,
}: NotificationDeliverySettingsPageProps) {
  const [settings, setSettings] = useState(initialSettings);
  const [draft, setDraft] = useState<ChannelDraft | null>(null);
  const [pendingEmailId, setPendingEmailId] = useState(
    initialSettings.defaultEmailId ?? "",
  );
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedPreference = useMemo(
    () =>
      draft
        ? settings.preferences.find(
            (preference) => preference.key === draft.key,
          )
        : null,
    [draft, settings.preferences],
  );
  const verifiedEmails = settings.emails.filter((email) => email.verified);

  async function saveDefaultEmail() {
    setSaving(true);
    setError(null);
    setStatus(null);
    try {
      const next = await persist({
        defaultEmailId: pendingEmailId || null,
      });
      setSettings(next);
      setPendingEmailId(next.defaultEmailId ?? "");
      setStatus("Default notifications email saved.");
    } catch (saveError) {
      setError(errorMessage(saveError));
    } finally {
      setSaving(false);
    }
  }

  function openPanel(preference: NotificationDeliveryPreference) {
    setDraft({ key: preference.key, channels: preference.channels });
    setError(null);
    setStatus(null);
  }

  function toggleChannel(channel: string) {
    setDraft((current) => {
      if (!current) return current;
      const selected = current.channels.includes(channel);
      const channels = selected
        ? current.channels.filter((value) => value !== channel)
        : [...current.channels, channel];
      return { ...current, channels };
    });
  }

  async function saveChannels() {
    if (!draft || draft.channels.length === 0) return;
    setSaving(true);
    setError(null);
    try {
      const next = await persist({
        defaultEmailId: pendingEmailId || settings.defaultEmailId,
        preferences: [{ key: draft.key, channels: draft.channels }],
      });
      setSettings(next);
      setPendingEmailId(next.defaultEmailId ?? "");
      setDraft(null);
      setStatus("Notification channels saved.");
    } catch (saveError) {
      setError(errorMessage(saveError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="grid gap-6">
      {status ? (
        <div className="chip ok w-fit" role="status">
          {status}
        </div>
      ) : null}
      {error ? (
        <div className="chip err w-fit" role="alert">
          {error}
        </div>
      ) : null}

      <section className="card p-5" aria-labelledby="delivery-email-heading">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <p className="t-label">Delivery</p>
            <h3 className="t-h2 mt-2" id="delivery-email-heading">
              Default notifications email
            </h3>
            <p
              className="t-body mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Email delivery uses verified account addresses. Unverified
              addresses stay visible for account context but cannot receive
              notification mail.
            </p>
          </div>
          <span className={settings.sesSenderReady ? "chip ok" : "chip warn"}>
            {settings.sesSenderReady ? "SES ready" : "Email unavailable"}
          </span>
        </div>

        <div className="mt-5 grid gap-3 md:grid-cols-[minmax(0,1fr)_auto]">
          <label className="grid gap-2">
            <span className="t-sm font-medium">Email address</span>
            <select
              className="input"
              disabled={!settings.emailChannelAvailable || saving}
              onChange={(event) => setPendingEmailId(event.target.value)}
              value={pendingEmailId}
            >
              <option value="">No email delivery</option>
              {settings.emails.map((email) => (
                <option
                  disabled={!email.verified}
                  key={email.id}
                  value={email.id}
                >
                  {email.email}
                  {email.isPrimary ? " - primary" : ""}
                  {email.verified ? "" : " - unverified"}
                </option>
              ))}
            </select>
          </label>
          <div className="flex items-end">
            <button
              className="btn primary"
              disabled={saving || !settings.emailChannelAvailable}
              onClick={saveDefaultEmail}
              type="button"
            >
              Save email
            </button>
          </div>
        </div>
        {verifiedEmails.length === 0 ? (
          <p className="t-xs mt-3">
            Add and verify an email address before enabling email channels.
          </p>
        ) : null}
      </section>

      <PreferenceSection
        heading="Subscriptions"
        hrefs={[
          ["Custom routing", settings.customRoutingHref],
          ["Watched repositories", settings.watchedRepositoriesHref],
          ["Ignored repositories", settings.ignoredRepositoriesHref],
        ]}
        onOpen={openPanel}
        preferences={settings.preferences.filter(
          (preference) => preference.section === "subscriptions",
        )}
      />

      <PreferenceSection
        heading="System"
        hrefs={[]}
        onOpen={openPanel}
        preferences={settings.preferences.filter(
          (preference) => preference.section === "system",
        )}
      />

      {draft && selectedPreference ? (
        <div
          aria-labelledby="delivery-panel-heading"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center bg-[color-mix(in_oklch,var(--ink-1)_42%,transparent)] p-4"
          role="dialog"
        >
          <div className="card w-full max-w-md bg-[var(--surface)] p-5">
            <p className="t-label">Notify me</p>
            <h3 className="t-h2 mt-2" id="delivery-panel-heading">
              {selectedPreference.label}
            </h3>
            <div className="mt-4 grid gap-2">
              {selectedPreference.supportedChannels.map((channel) => {
                const disabled =
                  channel === "email" && !settings.emailChannelAvailable;
                return (
                  <label
                    className="flex items-center justify-between gap-4 rounded-md border border-[var(--line)] px-3 py-2"
                    key={channel}
                  >
                    <span>
                      <span className="block t-sm font-medium">
                        {CHANNEL_LABELS[channel] ?? channel}
                      </span>
                      {disabled ? (
                        <span className="t-xs">Requires verified email</span>
                      ) : null}
                    </span>
                    <input
                      checked={draft.channels.includes(channel)}
                      disabled={disabled}
                      onChange={() => toggleChannel(channel)}
                      type="checkbox"
                    />
                  </label>
                );
              })}
            </div>
            <div className="mt-5 flex flex-wrap gap-2">
              <button
                className="btn primary"
                disabled={saving || draft.channels.length === 0}
                onClick={saveChannels}
                type="button"
              >
                Save
              </button>
              <button
                className="btn"
                disabled={saving}
                onClick={() => setDraft(null)}
                type="button"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function PreferenceSection({
  heading,
  hrefs,
  onOpen,
  preferences,
}: {
  heading: string;
  hrefs: [string, string][];
  onOpen: (preference: NotificationDeliveryPreference) => void;
  preferences: NotificationDeliveryPreference[];
}) {
  return (
    <section className="card p-5" aria-labelledby={`${heading}-heading`}>
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <p className="t-label">Preferences</p>
          <h3 className="t-h2 mt-2" id={`${heading}-heading`}>
            {heading}
          </h3>
        </div>
        {hrefs.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {hrefs.map(([label, href]) => (
              <a className="btn sm" href={href} key={href}>
                {label}
              </a>
            ))}
          </div>
        ) : null}
      </div>
      <div className="mt-4 grid gap-2">
        {preferences.map((preference) => (
          <div
            className="list-row flex flex-col gap-3 py-4 md:flex-row md:items-center md:justify-between"
            key={preference.key}
          >
            <div>
              <div className="font-medium">{preference.label}</div>
              <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                {preference.description}
              </p>
              <div className="mt-2 flex flex-wrap gap-2">
                {preference.channels.map((channel) => (
                  <span className="chip soft" key={channel}>
                    {CHANNEL_LABELS[channel] ?? channel}
                  </span>
                ))}
                {preference.disabledReason ? (
                  <span className="chip warn">{preference.disabledReason}</span>
                ) : null}
              </div>
            </div>
            <button
              className="btn sm"
              disabled={preference.disabled}
              onClick={() => onOpen(preference)}
              type="button"
            >
              Notify me
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}

async function persist(input: {
  defaultEmailId?: string | null;
  preferences?: { key: string; channels: string[] }[];
}) {
  const response = await fetch("/settings/notifications/delivery", {
    method: "PATCH",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(input),
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw new Error(
      body?.error?.message ??
        "Notification delivery settings could not be saved.",
    );
  }
  return body as NotificationDeliverySettings;
}

function errorMessage(error: unknown) {
  return error instanceof Error
    ? error.message
    : "Notification delivery settings could not be saved.";
}
