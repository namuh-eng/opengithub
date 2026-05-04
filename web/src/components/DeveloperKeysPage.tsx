"use client";

import Link from "next/link";
import { useState } from "react";
import type {
  CreateGpgKeyRequest,
  CreateSshKeyRequest,
  GpgKeySummary,
  KeySettingsFetchResult,
  SshKeySummary,
} from "@/lib/api";

type DeveloperKeysPageProps = {
  keySettings?: KeySettingsFetchResult;
  showHeading?: boolean;
  userEmail?: string | null;
};

const sshKeyTypes = [
  ["ssh-ed25519", "Ed25519"],
  ["ssh-rsa", "RSA"],
  ["ecdsa-sha2-nistp256", "ECDSA P-256"],
];

export function DeveloperKeysPage({
  keySettings,
  showHeading = true,
  userEmail = null,
}: DeveloperKeysPageProps = {}) {
  return (
    <article className="min-w-0">
      {showHeading ? (
        <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Developer settings
          </p>
          <h1 className="mt-2 t-h2">SSH keys</h1>
        </div>
      ) : null}
      <p className="max-w-3xl t-body" style={{ color: "var(--ink-3)" }}>
        Add public SSH keys for command-line access and future signing flows.
        opengithub stores public key fingerprints and audit history; private
        keys never leave your machine.
      </p>

      <SshKeyPanel keySettings={keySettings} userEmail={userEmail} />
      <GpgKeyPanel keySettings={keySettings} userEmail={userEmail} />
      <VigilantModePanel keySettings={keySettings} />
    </article>
  );
}

function SshKeyPanel({
  keySettings,
  userEmail,
}: {
  keySettings?: KeySettingsFetchResult;
  userEmail?: string | null;
}) {
  const [sshKeys, setSshKeys] = useState(
    keySettings?.ok ? keySettings.settings.sshKeys : [],
  );
  const [formOpen, setFormOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [keyType, setKeyType] = useState("ssh-ed25519");
  const [accessMode, setAccessMode] =
    useState<CreateSshKeyRequest["accessMode"]>("read_write");
  const [publicKey, setPublicKey] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleteKey, setDeleteKey] = useState<SshKeySummary | null>(null);
  const [deleteTitle, setDeleteTitle] = useState("");
  const [deleting, setDeleting] = useState(false);
  const [sudoActive, setSudoActive] = useState(
    keySettings?.ok ? keySettings.settings.sudo.active : false,
  );
  const [sudoConfirmation, setSudoConfirmation] = useState("");
  const [sudoSaving, setSudoSaving] = useState(false);

  async function addSshKey() {
    setSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/keys/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          title,
          keyType,
          publicKey,
          accessMode,
        } satisfies CreateSshKeyRequest),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "SSH key could not be added.");
      }
      setSshKeys((current) => [body.sshKey, ...current]);
      setTitle("");
      setPublicKey("");
      setKeyType("ssh-ed25519");
      setAccessMode("read_write");
      setFormOpen(false);
      setMessage(`${body.sshKey.title} added.`);
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "SSH key could not be added.",
      );
    } finally {
      setSaving(false);
    }
  }

  async function confirmDeleteSshKey() {
    if (!deleteKey || deleteTitle.trim() !== deleteKey.title) {
      return;
    }
    setDeleting(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/keys/actions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ keyId: deleteKey.id }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "SSH key could not be deleted.",
        );
      }
      setSshKeys((current) =>
        current.map((key) => (key.id === deleteKey.id ? body.sshKey : key)),
      );
      setMessage(`${deleteKey.title} deleted.`);
      setDeleteKey(null);
      setDeleteTitle("");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "SSH key could not be deleted.",
      );
    } finally {
      setDeleting(false);
    }
  }

  async function enableSudo() {
    setSudoSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/personal-access-tokens/sudo", {
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

  return (
    <section className="mt-6 card">
      <div
        className="flex flex-col gap-4 p-4 md:flex-row md:items-start md:justify-between"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Authentication keys
          </p>
          <h2 className="mt-2 t-h3">SSH keys</h2>
          <p
            className="mt-2 max-w-2xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            Public SSH keys unlock authenticated Git operations when SSH
            transport is enabled. Deleted keys stay in audit history and cannot
            be used again.
          </p>
        </div>
        <button
          className="btn primary"
          onClick={() => {
            setFormOpen((open) => !open);
            setMessage(null);
          }}
          type="button"
        >
          New SSH key
        </button>
      </div>

      {keySettings && !keySettings.ok ? (
        <div className="p-4">
          <div
            className="rounded-md p-4"
            style={{ background: "var(--surface-2)" }}
          >
            <p className="t-sm font-semibold" style={{ color: "var(--ink-1)" }}>
              SSH keys could not be loaded.
            </p>
            <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
              {keySettings.status === 401
                ? "Sign in to manage SSH keys."
                : keySettings.message}
            </p>
            <Link className="btn mt-4" href="/login?next=/settings/keys">
              Sign in
            </Link>
          </div>
        </div>
      ) : (
        <>
          {formOpen ? (
            <div
              className="m-4 rounded-md p-4"
              style={{
                background: "var(--surface-2)",
                border: "1px solid var(--line)",
              }}
            >
              <h3 className="t-h3">Add new SSH key</h3>
              <div className="mt-4 grid gap-4 md:grid-cols-2">
                <label className="block">
                  <span className="t-label" style={{ color: "var(--ink-4)" }}>
                    Title
                  </span>
                  <input
                    className="input mt-2 w-full"
                    onChange={(event) => setTitle(event.target.value)}
                    placeholder="Work laptop"
                    value={title}
                  />
                </label>
                <label className="block">
                  <span className="t-label" style={{ color: "var(--ink-4)" }}>
                    Key type
                  </span>
                  <select
                    className="input mt-2 w-full"
                    onChange={(event) => setKeyType(event.target.value)}
                    value={keyType}
                  >
                    {sshKeyTypes.map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <label className="mt-4 block">
                <span className="t-label" style={{ color: "var(--ink-4)" }}>
                  Public key
                </span>
                <textarea
                  className="input mt-2 min-h-32 w-full font-mono"
                  onChange={(event) => setPublicKey(event.target.value)}
                  placeholder="ssh-ed25519 AAAA..."
                  value={publicKey}
                />
              </label>
              <fieldset className="mt-4">
                <legend className="t-label" style={{ color: "var(--ink-4)" }}>
                  Access
                </legend>
                <div className="mt-2 flex flex-wrap gap-2">
                  {[
                    ["read_write", "Read/write"],
                    ["read_only", "Read only"],
                  ].map(([value, label]) => (
                    <label className="chip soft cursor-pointer" key={value}>
                      <input
                        checked={accessMode === value}
                        className="mr-2"
                        name="ssh-access"
                        onChange={() =>
                          setAccessMode(
                            value as CreateSshKeyRequest["accessMode"],
                          )
                        }
                        type="radio"
                      />
                      {label}
                    </label>
                  ))}
                </div>
              </fieldset>
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  className="btn primary"
                  disabled={saving}
                  onClick={addSshKey}
                  type="button"
                >
                  {saving ? "Adding" : "Add SSH key"}
                </button>
                <button
                  className="btn"
                  onClick={() => setFormOpen(false)}
                  type="button"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : null}

          {sshKeys.length > 0 ? (
            <div className="divide-y" style={{ borderColor: "var(--line)" }}>
              {sshKeys.map((key) => (
                <SshKeyRow
                  key={key.id}
                  onDelete={() => {
                    setDeleteKey(key);
                    setDeleteTitle("");
                    setMessage(null);
                  }}
                  sshKey={key}
                />
              ))}
            </div>
          ) : (
            <div className="p-4">
              <div
                className="rounded-md p-5"
                style={{ background: "var(--surface-2)" }}
              >
                <h3 className="t-h3">No SSH keys yet</h3>
                <p
                  className="mt-2 max-w-2xl t-body"
                  style={{ color: "var(--ink-3)" }}
                >
                  Add a public key from a trusted machine before using SSH for
                  repository operations.
                </p>
                <button
                  className="btn primary mt-4"
                  onClick={() => setFormOpen(true)}
                  type="button"
                >
                  New SSH key
                </button>
              </div>
            </div>
          )}
        </>
      )}

      <div
        className="flex flex-col gap-2 p-4 t-sm md:flex-row md:items-center md:justify-between"
        style={{ borderTop: "1px solid var(--line)", color: "var(--ink-3)" }}
      >
        <span>
          Sudo mode is{" "}
          {sudoActive
            ? "active for destructive key changes"
            : "required before deleting keys"}
          .
        </span>
        <Link
          className="font-semibold"
          href="/docs/git"
          style={{ color: "var(--accent)" }}
        >
          SSH key guide
        </Link>
      </div>

      {message ? (
        <p
          className="px-4 pb-4 t-sm"
          role="status"
          style={{ color: "var(--ink-3)" }}
        >
          {message}
        </p>
      ) : null}

      {deleteKey ? (
        <div
          aria-labelledby="ssh-key-delete-title"
          aria-modal="true"
          className="m-4 rounded-md p-4"
          role="alertdialog"
          style={{
            background: "var(--surface-2)",
            border: "1px solid var(--line)",
          }}
        >
          <p className="chip err">Confirm delete</p>
          <h3 className="mt-3 t-h3" id="ssh-key-delete-title">
            Delete {deleteKey.title}
          </h3>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            This revokes the key for future SSH authentication while preserving
            audit history. Type the key title to confirm.
          </p>
          {!sudoActive ? (
            <div
              className="mt-4 rounded-md p-3"
              style={{
                background: "var(--surface)",
                border: "1px solid var(--line)",
              }}
            >
              <p className="t-sm font-semibold">Confirm this session</p>
              <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
                Enter your account email to enable sudo mode before deleting SSH
                keys.
              </p>
              <label className="mt-3 block">
                <span className="t-label" style={{ color: "var(--ink-4)" }}>
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
                className="btn mt-3"
                disabled={sudoSaving}
                onClick={enableSudo}
                type="button"
              >
                {sudoSaving ? "Confirming" : "Enable sudo"}
              </button>
            </div>
          ) : null}
          <label className="mt-4 block">
            <span className="t-label" style={{ color: "var(--ink-4)" }}>
              Key title
            </span>
            <input
              aria-label={`Confirm delete ${deleteKey.title}`}
              className="input mt-2 w-full"
              onChange={(event) => setDeleteTitle(event.target.value)}
              value={deleteTitle}
            />
          </label>
          <div className="mt-4 flex flex-wrap gap-2">
            <button
              className="btn primary"
              disabled={
                !sudoActive ||
                deleteTitle.trim() !== deleteKey.title ||
                deleting
              }
              onClick={confirmDeleteSshKey}
              type="button"
            >
              {deleting ? "Deleting" : "Delete SSH key"}
            </button>
            <button
              className="btn"
              onClick={() => setDeleteKey(null)}
              type="button"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}
    </section>
  );
}

function GpgKeyPanel({
  keySettings,
  userEmail,
}: {
  keySettings?: KeySettingsFetchResult;
  userEmail?: string | null;
}) {
  const [gpgKeys, setGpgKeys] = useState(
    keySettings?.ok ? keySettings.settings.gpgKeys : [],
  );
  const [formOpen, setFormOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [armoredPublicKey, setArmoredPublicKey] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleteKey, setDeleteKey] = useState<GpgKeySummary | null>(null);
  const [deleteTitle, setDeleteTitle] = useState("");
  const [deleting, setDeleting] = useState(false);
  const [sudoActive, setSudoActive] = useState(
    keySettings?.ok ? keySettings.settings.sudo.active : false,
  );
  const [sudoConfirmation, setSudoConfirmation] = useState("");
  const [sudoSaving, setSudoSaving] = useState(false);

  async function addGpgKey() {
    setSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/keys/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          title,
          armoredPublicKey,
        } satisfies CreateGpgKeyRequest),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "GPG key could not be added.");
      }
      setGpgKeys((current) => [body.gpgKey, ...current]);
      setTitle("");
      setArmoredPublicKey("");
      setFormOpen(false);
      setMessage(`${body.gpgKey.title} added.`);
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "GPG key could not be added.",
      );
    } finally {
      setSaving(false);
    }
  }

  async function confirmDeleteGpgKey() {
    if (!deleteKey || deleteTitle.trim() !== deleteKey.title) {
      return;
    }
    setDeleting(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/keys/actions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ keyId: deleteKey.id, keyKind: "gpg" }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "GPG key could not be deleted.",
        );
      }
      setGpgKeys((current) =>
        current.map((key) => (key.id === deleteKey.id ? body.gpgKey : key)),
      );
      setMessage(`${deleteKey.title} deleted.`);
      setDeleteKey(null);
      setDeleteTitle("");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "GPG key could not be deleted.",
      );
    } finally {
      setDeleting(false);
    }
  }

  async function enableSudo() {
    setSudoSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/personal-access-tokens/sudo", {
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

  return (
    <section className="mt-6 card">
      <div
        className="flex flex-col gap-4 p-4 md:flex-row md:items-start md:justify-between"
        style={{ borderBottom: "1px solid var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Signing keys
          </p>
          <h2 className="mt-2 t-h3">GPG keys</h2>
          <p
            className="mt-2 max-w-2xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            Add armored public GPG keys for commit and tag verification. Raw key
            armor is accepted by the API but never rendered back to the browser.
          </p>
        </div>
        <button
          className="btn primary"
          disabled={keySettings ? !keySettings.ok : false}
          onClick={() => {
            setFormOpen((open) => !open);
            setMessage(null);
          }}
          type="button"
        >
          New GPG key
        </button>
      </div>

      {keySettings && !keySettings.ok ? (
        <div className="p-4">
          <div
            className="rounded-md p-4"
            style={{ background: "var(--surface-2)" }}
          >
            <p className="t-sm font-semibold" style={{ color: "var(--ink-1)" }}>
              GPG keys could not be loaded.
            </p>
            <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
              {keySettings.status === 401
                ? "Sign in to manage GPG keys."
                : keySettings.message}
            </p>
          </div>
        </div>
      ) : (
        <>
          {formOpen ? (
            <div
              className="m-4 rounded-md p-4"
              style={{
                background: "var(--surface-2)",
                border: "1px solid var(--line)",
              }}
            >
              <h3 className="t-h3">Add new GPG key</h3>
              <label className="mt-4 block">
                <span className="t-label" style={{ color: "var(--ink-4)" }}>
                  Title
                </span>
                <input
                  className="input mt-2 w-full"
                  onChange={(event) => setTitle(event.target.value)}
                  placeholder="Release signing"
                  value={title}
                />
              </label>
              <label className="mt-4 block">
                <span className="t-label" style={{ color: "var(--ink-4)" }}>
                  Armored public key
                </span>
                <textarea
                  className="input mt-2 min-h-40 w-full font-mono"
                  onChange={(event) => setArmoredPublicKey(event.target.value)}
                  placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
                  value={armoredPublicKey}
                />
              </label>
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  className="btn primary"
                  disabled={saving}
                  onClick={addGpgKey}
                  type="button"
                >
                  {saving ? "Adding" : "Add GPG key"}
                </button>
                <button
                  className="btn"
                  onClick={() => setFormOpen(false)}
                  type="button"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : null}

          {gpgKeys.length > 0 ? (
            <div className="divide-y" style={{ borderColor: "var(--line)" }}>
              {gpgKeys.map((key) => (
                <GpgKeyRow
                  gpgKey={key}
                  key={key.id}
                  onDelete={() => {
                    setDeleteKey(key);
                    setDeleteTitle("");
                    setMessage(null);
                  }}
                />
              ))}
            </div>
          ) : (
            <div className="p-4">
              <div
                className="rounded-md p-5"
                style={{ background: "var(--surface-2)" }}
              >
                <h3 className="t-h3">No GPG keys yet</h3>
                <p
                  className="mt-2 max-w-2xl t-body"
                  style={{ color: "var(--ink-3)" }}
                >
                  Add a public signing key to verify commits and release tags
                  attributed to your account.
                </p>
                <button
                  className="btn primary mt-4"
                  onClick={() => setFormOpen(true)}
                  type="button"
                >
                  New GPG key
                </button>
              </div>
            </div>
          )}
        </>
      )}

      <div
        className="flex flex-col gap-2 p-4 t-sm md:flex-row md:items-center md:justify-between"
        style={{ borderTop: "1px solid var(--line)", color: "var(--ink-3)" }}
      >
        <span>
          Sudo mode is{" "}
          {sudoActive
            ? "active for destructive signing-key changes"
            : "required before deleting signing keys"}
          .
        </span>
        <Link
          className="font-semibold"
          href="/docs/git"
          style={{ color: "var(--accent)" }}
        >
          GPG signing guide
        </Link>
      </div>

      {message ? (
        <p
          className="px-4 pb-4 t-sm"
          role="status"
          style={{ color: "var(--ink-3)" }}
        >
          {message}
        </p>
      ) : null}

      {deleteKey ? (
        <div
          aria-labelledby="gpg-key-delete-title"
          aria-modal="true"
          className="m-4 rounded-md p-4"
          role="alertdialog"
          style={{
            background: "var(--surface-2)",
            border: "1px solid var(--line)",
          }}
        >
          <p className="chip err">Confirm delete</p>
          <h3 className="mt-3 t-h3" id="gpg-key-delete-title">
            Delete {deleteKey.title}
          </h3>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            This revokes the key for future signature verification while
            preserving audit history. Type the key title to confirm.
          </p>
          {!sudoActive ? (
            <div
              className="mt-4 rounded-md p-3"
              style={{
                background: "var(--surface)",
                border: "1px solid var(--line)",
              }}
            >
              <p className="t-sm font-semibold">Confirm this session</p>
              <p className="mt-1 t-sm" style={{ color: "var(--ink-3)" }}>
                Enter your account email to enable sudo mode before deleting GPG
                keys.
              </p>
              <label className="mt-3 block">
                <span className="t-label" style={{ color: "var(--ink-4)" }}>
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
                className="btn mt-3"
                disabled={sudoSaving}
                onClick={enableSudo}
                type="button"
              >
                {sudoSaving ? "Confirming" : "Enable sudo"}
              </button>
            </div>
          ) : null}
          <label className="mt-4 block">
            <span className="t-label" style={{ color: "var(--ink-4)" }}>
              Key title
            </span>
            <input
              aria-label={`Confirm delete ${deleteKey.title}`}
              className="input mt-2 w-full"
              onChange={(event) => setDeleteTitle(event.target.value)}
              value={deleteTitle}
            />
          </label>
          <div className="mt-4 flex flex-wrap gap-2">
            <button
              className="btn primary"
              disabled={
                !sudoActive ||
                deleteTitle.trim() !== deleteKey.title ||
                deleting
              }
              onClick={confirmDeleteGpgKey}
              type="button"
            >
              {deleting ? "Deleting" : "Delete GPG key"}
            </button>
            <button
              className="btn"
              onClick={() => setDeleteKey(null)}
              type="button"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}
    </section>
  );
}

function VigilantModePanel({
  keySettings,
}: {
  keySettings?: KeySettingsFetchResult;
}) {
  const [enabled, setEnabled] = useState(
    keySettings?.ok ? keySettings.settings.vigilantMode : false,
  );
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  async function updateVigilantMode(nextEnabled: boolean) {
    const previous = enabled;
    setEnabled(nextEnabled);
    setSaving(true);
    setMessage(null);
    try {
      const response = await fetch("/settings/keys/actions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ enabled: nextEnabled }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Vigilant mode could not be updated.",
        );
      }
      setEnabled(Boolean(body.vigilantMode));
      setMessage(
        body.vigilantMode
          ? "Vigilant mode enabled."
          : "Vigilant mode disabled.",
      );
    } catch (error) {
      setEnabled(previous);
      setMessage(
        error instanceof Error
          ? error.message
          : "Vigilant mode could not be updated.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="mt-6 card p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Commit verification
          </p>
          <h2 className="mt-2 t-h3">Vigilant mode</h2>
          <p
            className="mt-2 max-w-2xl t-body"
            style={{ color: "var(--ink-3)" }}
          >
            Flag unsigned commits attributed to you as unverified. This setting
            is stored on your account and will feed commit-signature
            presentation in the next integration slice.
          </p>
        </div>
        <span className={enabled ? "chip ok" : "chip soft"}>
          {enabled ? "Enabled" : "Disabled"}
        </span>
      </div>
      <label
        className="mt-4 flex cursor-pointer items-start gap-3 rounded-md p-3"
        style={{
          background: "var(--surface-2)",
          border: "1px solid var(--line)",
        }}
      >
        <input
          checked={enabled}
          className="mt-1"
          disabled={saving || Boolean(keySettings && !keySettings.ok)}
          onChange={(event) => updateVigilantMode(event.target.checked)}
          type="checkbox"
        />
        <span>
          <span className="block t-sm font-semibold">
            Flag unsigned commits as unverified
          </span>
          <span className="mt-1 block t-sm" style={{ color: "var(--ink-3)" }}>
            Signed commits remain verified when they match an active GPG key.
            Unsigned commits receive stricter verification copy.
          </span>
        </span>
      </label>
      {message ? (
        <p
          className="mt-3 t-sm"
          role="status"
          style={{ color: "var(--ink-3)" }}
        >
          {message}
        </p>
      ) : null}
    </section>
  );
}

function SshKeyRow({
  onDelete,
  sshKey,
}: {
  onDelete: () => void;
  sshKey: SshKeySummary;
}) {
  const revoked = Boolean(sshKey.revokedAt);
  return (
    <div className="list-row gap-4 p-4">
      <div
        aria-hidden="true"
        className="grid h-10 w-10 shrink-0 place-items-center rounded-md"
        style={{
          background: "var(--surface-2)",
          border: "1px solid var(--line)",
          color: "var(--ink-2)",
        }}
      >
        <span className="t-mono-sm">SSH</span>
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="t-h3">{sshKey.title}</h3>
          <span className={revoked ? "chip err" : "chip ok"}>
            {revoked ? "Deleted" : "Active"}
          </span>
          <span className="chip soft">{formatKeyType(sshKey.keyType)}</span>
          <span className="chip soft">
            {formatAccessMode(sshKey.accessMode)}
          </span>
        </div>
        <p
          className="mt-2 break-all t-mono-sm"
          style={{ color: "var(--ink-2)" }}
        >
          {sshKey.fingerprintSha256}
        </p>
        <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
          Added {formatDate(sshKey.createdAt)} from{" "}
          {formatSource(sshKey.source)}
          {" · "}
          {sshKey.lastUsedAt
            ? `Last used ${formatDate(sshKey.lastUsedAt)}`
            : "Never used"}
        </p>
      </div>
      <button
        className="btn sm"
        disabled={revoked}
        onClick={onDelete}
        type="button"
      >
        Delete
      </button>
    </div>
  );
}

function GpgKeyRow({
  gpgKey,
  onDelete,
}: {
  gpgKey: GpgKeySummary;
  onDelete: () => void;
}) {
  const revoked = Boolean(gpgKey.revokedAt);
  return (
    <div className="list-row gap-4 p-4">
      <div
        aria-hidden="true"
        className="grid h-10 w-10 shrink-0 place-items-center rounded-md"
        style={{
          background: "var(--surface-2)",
          border: "1px solid var(--line)",
          color: "var(--ink-2)",
        }}
      >
        <span className="t-mono-sm">GPG</span>
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="t-h3">{gpgKey.title}</h3>
          <span className={revoked ? "chip err" : "chip ok"}>
            {revoked ? "Deleted" : "Active"}
          </span>
          {gpgKey.keyId ? (
            <span className="chip soft">{gpgKey.keyId}</span>
          ) : null}
        </div>
        <p
          className="mt-2 break-all t-mono-sm"
          style={{ color: "var(--ink-2)" }}
        >
          {gpgKey.primaryFingerprint}
        </p>
        {gpgKey.emails.length > 0 ? (
          <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
            {gpgKey.emails.join(", ")}
          </p>
        ) : null}
        <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
          Added {formatDate(gpgKey.createdAt)} from{" "}
          {formatSource(gpgKey.source)}
          {" · "}
          {gpgKey.lastUsedAt
            ? `Last used ${formatDate(gpgKey.lastUsedAt)}`
            : "Never used"}
        </p>
      </div>
      <button
        className="btn sm"
        disabled={revoked}
        onClick={onDelete}
        type="button"
      >
        Delete
      </button>
    </div>
  );
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "unknown";
  }
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

function formatKeyType(value: string) {
  if (value === "ssh-ed25519") return "Ed25519";
  if (value === "ssh-rsa") return "RSA";
  if (value.startsWith("ecdsa-")) return "ECDSA";
  return value;
}

function formatAccessMode(value: string) {
  return value === "read_only" ? "Read only" : "Read/write";
}

function formatSource(value: string | null | undefined) {
  return (value ?? "user_upload").replaceAll("_", " ");
}
