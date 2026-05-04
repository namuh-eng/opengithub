"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type {
  AccountSecuritySettings,
  AccountSecuritySettingsFetchResult,
  AccountSignInMethod,
} from "@/lib/api";

type AccountSecurityPageProps = {
  linkGoogleHref: string;
  securitySettings?: AccountSecuritySettingsFetchResult;
  userEmail?: string | null;
};

export function AccountSecurityPage({
  linkGoogleHref,
  securitySettings,
  userEmail = null,
}: AccountSecurityPageProps) {
  const [settings, setSettings] = useState<AccountSecuritySettings | null>(
    securitySettings?.ok ? securitySettings.settings : null,
  );
  const [sudoActive, setSudoActive] = useState(
    securitySettings?.ok ? securitySettings.settings.sudo.active : false,
  );
  const [sudoConfirmation, setSudoConfirmation] = useState("");
  const [sudoSaving, setSudoSaving] = useState(false);
  const [unlinkTarget, setUnlinkTarget] = useState<AccountSignInMethod | null>(
    null,
  );
  const [unlinkConfirmation, setUnlinkConfirmation] = useState("");
  const [unlinkSaving, setUnlinkSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(
    securitySettings?.ok ? null : (securitySettings?.message ?? null),
  );

  const signInMethods = settings?.signInMethods ?? [];
  const sudoExpires = useMemo(() => {
    if (!settings?.sudo.expiresAt) {
      return null;
    }
    return new Intl.DateTimeFormat("en", {
      hour: "numeric",
      minute: "2-digit",
      month: "short",
      day: "numeric",
    }).format(new Date(settings.sudo.expiresAt));
  }, [settings?.sudo.expiresAt]);

  async function enableSudo() {
    setSudoSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/security/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ confirmation: sudoConfirmation }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Sudo mode could not be enabled.",
        );
      }
      setSettings(body);
      setSudoActive(Boolean(body.sudo?.active));
      setSudoConfirmation("");
      setMessage("Sudo mode is active for this session.");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Sudo mode could not be enabled.",
      );
    } finally {
      setSudoSaving(false);
    }
  }

  async function unlinkMethod() {
    if (!unlinkTarget || unlinkConfirmation.trim() !== unlinkTarget.email) {
      return;
    }
    setUnlinkSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/security/actions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ accountId: unlinkTarget.id }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Sign-in method could not be unlinked.",
        );
      }
      setSettings(body.settings);
      setUnlinkTarget(null);
      setUnlinkConfirmation("");
      setMessage(`${unlinkTarget.displayLabel} account unlinked.`);
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Sign-in method could not be unlinked.",
      );
    } finally {
      setUnlinkSaving(false);
    }
  }

  return (
    <article className="min-w-0">
      <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Account security
        </p>
        <h1 className="mt-2 t-h2">Security</h1>
      </div>

      <p className="mt-4 max-w-3xl t-body" style={{ color: "var(--ink-3)" }}>
        Manage Google sign-in methods and session confirmation for sensitive
        account changes. Passwords, passkeys, and recovery codes are outside
        this Google-only authentication pass.
      </p>

      {message ? (
        <p
          className="mt-4 t-sm"
          role="status"
          style={{ color: "var(--ink-2)" }}
        >
          {message}
        </p>
      ) : null}

      <section className="card mt-6 overflow-hidden">
        <div
          className="flex flex-wrap items-start justify-between gap-3 p-5"
          style={{ borderBottom: "1px solid var(--line)" }}
        >
          <div>
            <h2 className="t-h3">Sign-in methods</h2>
            <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
              Linked Google identities that can authenticate this account.
            </p>
          </div>
          <Link
            aria-disabled={!sudoActive}
            className={`btn sm ${sudoActive ? "primary" : ""}`}
            href={sudoActive ? linkGoogleHref : "#sudo-confirmation"}
          >
            Link Google account
          </Link>
        </div>

        {signInMethods.length > 0 ? (
          signInMethods.map((method) => (
            <div className="list-row p-5" key={method.id}>
              <div className="av sm" aria-hidden="true">
                {method.email.slice(0, 1).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="t-sm font-semibold">
                    {method.displayLabel}
                  </span>
                  <span className="chip ok">Linked</span>
                  {!method.canUnlink ? (
                    <span className="chip warn">Last identity</span>
                  ) : null}
                </div>
                <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
                  {method.email}
                </p>
              </div>
              <button
                className="btn sm"
                disabled={!method.canUnlink}
                onClick={() => {
                  setUnlinkTarget(method);
                  setUnlinkConfirmation("");
                }}
                type="button"
              >
                Unlink
              </button>
            </div>
          ))
        ) : (
          <div className="p-5">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No linked Google account was returned by the API.
            </p>
          </div>
        )}
      </section>

      <section className="card mt-5 p-5" id="sudo-confirmation">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="t-h3">Sudo mode</h2>
            <p
              className="mt-1 max-w-2xl t-sm"
              style={{ color: "var(--ink-3)" }}
            >
              Confirm your current account email before linking another Google
              identity or unlinking an existing one.
            </p>
          </div>
          <span className={sudoActive ? "chip ok" : "chip warn"}>
            {sudoActive ? "Sudo active" : "Sudo required"}
          </span>
        </div>
        {sudoActive ? (
          <p className="mt-4 t-sm" style={{ color: "var(--ink-3)" }}>
            Sensitive changes are enabled
            {sudoExpires ? ` until ${sudoExpires}.` : " for this session."}
          </p>
        ) : (
          <div className="mt-4 flex flex-wrap items-end gap-3">
            <label className="min-w-[260px] flex-1">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Account email
              </span>
              <input
                className="input mt-2 w-full"
                onChange={(event) => setSudoConfirmation(event.target.value)}
                placeholder={userEmail ?? "you@example.com"}
                value={sudoConfirmation}
              />
            </label>
            <button
              className="btn primary"
              disabled={sudoSaving}
              onClick={enableSudo}
              type="button"
            >
              {sudoSaving ? "Confirming" : "Enable sudo"}
            </button>
          </div>
        )}
      </section>

      <section className="card mt-5 p-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="t-h3">Two-factor authentication</h2>
            <p
              className="mt-1 max-w-2xl t-sm"
              style={{ color: "var(--ink-3)" }}
            >
              {settings?.twoFactor.reason ??
                "Two-factor authentication is not available in this pass."}
            </p>
          </div>
          <span className="chip soft">Disabled</span>
        </div>
        <button className="btn mt-4" disabled type="button">
          Configure 2FA
        </button>
      </section>

      {unlinkTarget ? (
        <section
          aria-label={`Unlink ${unlinkTarget.displayLabel}`}
          className="card mt-5 p-5"
        >
          <h2 className="t-h3">Unlink {unlinkTarget.displayLabel}</h2>
          <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
            Type <span className="t-mono-sm">{unlinkTarget.email}</span> to
            remove this sign-in method. At least one Google identity must
            remain.
          </p>
          <label className="mt-4 block">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              Confirm unlink {unlinkTarget.email}
            </span>
            <input
              className="input mt-2 w-full"
              onChange={(event) => setUnlinkConfirmation(event.target.value)}
              value={unlinkConfirmation}
            />
          </label>
          <div className="mt-4 flex flex-wrap gap-2">
            <button
              className="btn primary"
              disabled={
                unlinkSaving || unlinkConfirmation.trim() !== unlinkTarget.email
              }
              onClick={unlinkMethod}
              type="button"
            >
              {unlinkSaving ? "Unlinking" : "Unlink sign-in method"}
            </button>
            <button
              className="btn"
              onClick={() => setUnlinkTarget(null)}
              type="button"
            >
              Cancel
            </button>
          </div>
        </section>
      ) : null}
    </article>
  );
}
