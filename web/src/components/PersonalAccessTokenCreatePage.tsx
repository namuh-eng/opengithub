"use client";

import Link from "next/link";
import { type FormEvent, useMemo, useState } from "react";
import type {
  CreatePersonalAccessTokenResponse,
  PersonalAccessTokenNewContext,
  PersonalAccessTokenNewContextFetchResult,
} from "@/lib/api";

type InitialTokenQuery = {
  type?: string;
  name?: string;
  description?: string;
  target_name?: string;
  expires_in?: string;
  contents?: string;
  issues?: string;
  pull_requests?: string;
  packages?: string;
  api?: string;
  profile?: string;
};

type Props = {
  contextResult: PersonalAccessTokenNewContextFetchResult;
  userEmail: string | null;
  initialQuery?: InitialTokenQuery;
};

const permissionKeys = [
  "contents",
  "issues",
  "pull_requests",
  "packages",
  "api",
  "profile",
] as const;

type PermissionKey = (typeof permissionKeys)[number];

export function PersonalAccessTokenCreatePage({
  contextResult,
  initialQuery = {},
  userEmail,
}: Props) {
  if (!contextResult.ok) {
    return (
      <article className="min-w-0">
        <div className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Developer settings
          </p>
          <h1 className="mt-2 t-h2">New fine-grained token</h1>
          <p className="mt-3 t-body" style={{ color: "var(--ink-3)" }}>
            {contextResult.status === 401
              ? "Sign in to create personal access tokens."
              : contextResult.message}
          </p>
          <Link
            className="btn mt-4"
            href="/login?next=/settings/personal-access-tokens/new"
          >
            Sign in
          </Link>
        </div>
      </article>
    );
  }

  return (
    <TokenCreateForm
      context={contextResult.context}
      initialQuery={initialQuery}
      userEmail={userEmail}
    />
  );
}

function TokenCreateForm({
  context,
  initialQuery,
  userEmail,
}: {
  context: PersonalAccessTokenNewContext;
  initialQuery: InitialTokenQuery;
  userEmail: string | null;
}) {
  const initialOwnerId =
    context.resourceOwners.find(
      (owner) => owner.login === initialQuery.target_name,
    )?.id ??
    context.resourceOwners[0]?.id ??
    "";
  const [tokenType, setTokenType] = useState<"fine_grained" | "classic">(
    initialQuery.type === "classic" ? "classic" : "fine_grained",
  );
  const [sudoActive, setSudoActive] = useState(context.sudo.active);
  const [sudoConfirmation, setSudoConfirmation] = useState("");
  const [sudoMessage, setSudoMessage] = useState<string | null>(null);
  const [sudoSaving, setSudoSaving] = useState(false);
  const [name, setName] = useState(initialQuery.name ?? "");
  const [description, setDescription] = useState(
    initialQuery.description ?? "",
  );
  const [resourceOwnerId, setResourceOwnerId] = useState(initialOwnerId);
  const [repositoryAccess, setRepositoryAccess] = useState<
    "all" | "selected" | "none"
  >(initialQuery.type === "classic" ? "all" : "selected");
  const [selectedRepositories, setSelectedRepositories] = useState<string[]>(
    [],
  );
  const [expiresIn, setExpiresIn] = useState(
    initialQuery.expires_in ?? String(context.defaultExpirationDays),
  );
  const [permissions, setPermissions] = useState<Record<PermissionKey, string>>(
    {
      contents: sanitizePermission(initialQuery.contents, "read"),
      issues: sanitizePermission(initialQuery.issues, "none"),
      pull_requests: sanitizePermission(initialQuery.pull_requests, "none"),
      packages: sanitizePermission(initialQuery.packages, "none"),
      api: sanitizePermission(initialQuery.api, "read"),
      profile: sanitizePermission(initialQuery.profile, "none"),
    },
  );
  const [createMessage, setCreateMessage] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [created, setCreated] =
    useState<CreatePersonalAccessTokenResponse | null>(null);
  const [copyMessage, setCopyMessage] = useState<string | null>(null);

  const selectedOwner = context.resourceOwners.find(
    (owner) => owner.id === resourceOwnerId,
  );
  const ownerRepositories = useMemo(() => {
    if (!selectedOwner) {
      return [];
    }
    return context.repositories.filter(
      (repository) => repository.owner === selectedOwner.login,
    );
  }, [context.repositories, selectedOwner]);

  async function submitSudo(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSudoSaving(true);
    setSudoMessage(null);
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
      setSudoMessage("Sudo mode is active for this session.");
    } catch (error) {
      setSudoMessage(
        error instanceof Error
          ? error.message
          : "Sudo mode could not be enabled.",
      );
    } finally {
      setSudoSaving(false);
    }
  }

  async function submitToken(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreateMessage(null);
    setCreated(null);
    const selectedPermissions = Object.entries(permissions).map(
      ([key, level]) => ({ key, level }),
    );
    const repositoryIds =
      repositoryAccess === "selected" ? selectedRepositories : [];
    setCreating(true);
    try {
      const response = await fetch("/settings/personal-access-tokens/actions", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          name,
          description,
          type: tokenType,
          resourceOwnerId,
          repositoryAccess: tokenType === "classic" ? "all" : repositoryAccess,
          repositoryIds,
          expires_in_days:
            expiresIn === "never" ? "never" : Number.parseInt(expiresIn, 10),
          permissions: selectedPermissions,
        }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Personal access token could not be created.",
        );
      }
      setCreated(body as CreatePersonalAccessTokenResponse);
      setCreateMessage(
        "Token created. Copy it now; it will not be shown again.",
      );
    } catch (error) {
      setCreateMessage(
        error instanceof Error
          ? error.message
          : "Personal access token could not be created.",
      );
    } finally {
      setCreating(false);
    }
  }

  async function copyToken() {
    if (!created) {
      return;
    }
    try {
      await navigator.clipboard.writeText(created.plainTextToken);
      setCopyMessage("Copied");
    } catch {
      setCopyMessage("Copy unavailable");
    }
  }

  return (
    <article className="min-w-0">
      <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Developer settings
        </p>
        <h1 className="mt-2 t-h2">
          {tokenType === "classic"
            ? "New classic token"
            : "New fine-grained token"}
        </h1>
        <p className="mt-3 max-w-3xl t-body" style={{ color: "var(--ink-3)" }}>
          {tokenType === "classic"
            ? "Create a broad-scope token for older Git, REST, and package automation. The secret appears once."
            : "Create a repository-scoped token for Git over HTTPS, REST API calls, package registry auth, and automation. The secret appears once."}
        </p>
      </div>

      <section className="mt-6 card p-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Sudo mode
            </p>
            <h2 className="mt-2 t-h3">
              {sudoActive ? "Session confirmed" : "Confirm this session"}
            </h2>
            <p
              className="mt-2 max-w-2xl t-body"
              style={{ color: "var(--ink-3)" }}
            >
              Token creation requires a fresh confirmation for the signed-in
              account. Enter your email address to enable sudo mode.
            </p>
          </div>
          <span className={sudoActive ? "chip ok" : "chip warn"}>
            {sudoActive ? "Sudo active" : "Sudo required"}
          </span>
        </div>
        {!sudoActive ? (
          <form
            className="mt-4 flex flex-col gap-3 md:flex-row"
            onSubmit={submitSudo}
          >
            <label className="flex-1">
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
              className="btn primary self-end"
              disabled={sudoSaving}
              type="submit"
            >
              {sudoSaving ? "Confirming" : "Enable sudo"}
            </button>
          </form>
        ) : null}
        {sudoMessage ? (
          <p
            className="mt-3 t-sm"
            role="status"
            style={{ color: "var(--ink-3)" }}
          >
            {sudoMessage}
          </p>
        ) : null}
      </section>

      <form className="mt-6 grid gap-6" onSubmit={submitToken}>
        <section className="card p-4">
          <h2 className="t-h3">Token details</h2>
          <div className="mt-4 grid gap-4 lg:grid-cols-2">
            <fieldset className="lg:col-span-2">
              <legend className="t-label" style={{ color: "var(--ink-4)" }}>
                Token type
              </legend>
              <div className="mt-2 flex flex-wrap gap-2">
                {[
                  ["fine_grained", "Fine-grained"],
                  ["classic", "Classic"],
                ].map(([value, label]) => (
                  <label className="chip" key={value}>
                    <input
                      checked={tokenType === value}
                      className="mr-2"
                      name="tokenType"
                      onChange={() => {
                        const nextType = value as "fine_grained" | "classic";
                        setTokenType(nextType);
                        setRepositoryAccess(
                          nextType === "classic" ? "all" : "selected",
                        );
                        setSelectedRepositories([]);
                      }}
                      type="radio"
                    />
                    {label}
                  </label>
                ))}
              </div>
            </fieldset>
            <label>
              <span className="t-label" style={{ color: "var(--ink-4)" }}>
                Token name
              </span>
              <input
                className="input mt-2 w-full"
                onChange={(event) => setName(event.target.value)}
                required
                value={name}
              />
            </label>
            <label>
              <span className="t-label" style={{ color: "var(--ink-4)" }}>
                Expiration
              </span>
              <select
                className="input mt-2 w-full"
                onChange={(event) => setExpiresIn(event.target.value)}
                value={expiresIn}
              >
                <option value="7">7 days</option>
                <option value={String(context.defaultExpirationDays)}>
                  {context.defaultExpirationDays} days
                </option>
                <option value="90">90 days</option>
                <option value="366">366 days</option>
                <option value="never">Never</option>
              </select>
            </label>
            <label className="lg:col-span-2">
              <span className="t-label" style={{ color: "var(--ink-4)" }}>
                Description
              </span>
              <textarea
                className="input mt-2 min-h-24 w-full"
                onChange={(event) => setDescription(event.target.value)}
                value={description}
              />
            </label>
          </div>
        </section>

        <section className="card p-4">
          <h2 className="t-h3">Resource owner and repositories</h2>
          <div className="mt-4 grid gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
            <label>
              <span className="t-label" style={{ color: "var(--ink-4)" }}>
                Resource owner
              </span>
              <select
                className="input mt-2 w-full"
                onChange={(event) => {
                  setResourceOwnerId(event.target.value);
                  setSelectedRepositories([]);
                }}
                value={resourceOwnerId}
              >
                {context.resourceOwners.map((owner) => (
                  <option key={owner.id} value={owner.id}>
                    {owner.login} ({owner.kind})
                  </option>
                ))}
              </select>
            </label>
            {tokenType === "classic" ? (
              <div>
                <p className="t-label" style={{ color: "var(--ink-4)" }}>
                  Repository access
                </p>
                <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                  Classic tokens use broad access for every repository the
                  resource owner can reach.
                </p>
              </div>
            ) : (
              <fieldset>
                <legend className="t-label" style={{ color: "var(--ink-4)" }}>
                  Repository access
                </legend>
                <div className="mt-2 flex flex-wrap gap-2">
                  {[
                    ["selected", "Selected repositories"],
                    ["all", "All repositories"],
                    ["none", "No repository access"],
                  ].map(([value, label]) => (
                    <label className="chip" key={value}>
                      <input
                        checked={repositoryAccess === value}
                        className="mr-2"
                        name="repositoryAccess"
                        onChange={() =>
                          setRepositoryAccess(
                            value as "all" | "selected" | "none",
                          )
                        }
                        type="radio"
                      />
                      {label}
                    </label>
                  ))}
                </div>
              </fieldset>
            )}
          </div>
          {tokenType !== "classic" && repositoryAccess === "selected" ? (
            <div className="mt-4 grid gap-2">
              {ownerRepositories.length === 0 ? (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No repositories are available for this owner.
                </p>
              ) : (
                ownerRepositories.map((repository) => (
                  <label
                    className="list-row rounded-md px-3 py-2"
                    key={repository.id}
                  >
                    <input
                      checked={selectedRepositories.includes(repository.id)}
                      className="mr-3"
                      onChange={(event) => {
                        setSelectedRepositories((current) =>
                          event.target.checked
                            ? [...current, repository.id]
                            : current.filter((id) => id !== repository.id),
                        );
                      }}
                      type="checkbox"
                    />
                    <span className="t-mono-sm">{repository.fullName}</span>
                    <span className="chip soft ml-auto">
                      {repository.visibility}
                    </span>
                  </label>
                ))
              )}
            </div>
          ) : null}
        </section>

        <section className="card p-4">
          <h2 className="t-h3">Permissions</h2>
          <div className="mt-4 grid gap-3">
            {context.permissionGroups.flatMap((group) =>
              group.permissions.map((permission) => (
                <label
                  className="grid gap-3 rounded-md p-3 md:grid-cols-[1fr_180px]"
                  key={permission.key}
                  style={{ border: "1px solid var(--line)" }}
                >
                  <span>
                    <span className="block t-sm font-semibold">
                      {permission.label}
                    </span>
                    <span className="t-xs">{group.label}</span>
                  </span>
                  <select
                    className="input"
                    onChange={(event) =>
                      setPermissions((current) => ({
                        ...current,
                        [permission.key]: event.target.value,
                      }))
                    }
                    value={
                      permissions[permission.key as PermissionKey] ?? "none"
                    }
                  >
                    {permission.levels.map((level) => (
                      <option key={level} value={level}>
                        {level}
                      </option>
                    ))}
                  </select>
                </label>
              )),
            )}
          </div>
        </section>

        <div className="flex flex-wrap items-center gap-3">
          <button
            className="btn primary"
            disabled={!sudoActive || creating}
            type="submit"
          >
            {creating ? "Generating" : "Generate token"}
          </button>
          <Link className="btn" href="/settings/tokens">
            Cancel
          </Link>
          {createMessage ? (
            <span
              className="t-sm"
              role="status"
              style={{ color: "var(--ink-3)" }}
            >
              {createMessage}
            </span>
          ) : null}
        </div>
      </form>

      {created ? (
        <section className="mt-6 card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            One-time reveal
          </p>
          <h2 className="mt-2 t-h3">Copy your new token</h2>
          <p className="mt-2 t-body" style={{ color: "var(--ink-3)" }}>
            This token will not be shown again. Store it before leaving this
            page.
          </p>
          <div
            className="mt-4 flex flex-col gap-3 rounded-md p-3 md:flex-row md:items-center"
            style={{ background: "var(--surface-2)" }}
          >
            <code className="t-mono-sm min-w-0 flex-1 break-all">
              {created.plainTextToken}
            </code>
            <button className="btn" onClick={copyToken} type="button">
              Copy token
            </button>
          </div>
          {copyMessage ? (
            <p
              className="mt-3 t-sm"
              role="status"
              style={{ color: "var(--ink-3)" }}
            >
              {copyMessage}
            </p>
          ) : null}
          <Link className="btn mt-4" href="/settings/tokens">
            Return to token list
          </Link>
        </section>
      ) : null}
    </article>
  );
}

function sanitizePermission(value: string | undefined, fallback: string) {
  return ["none", "read", "write"].includes(value ?? "")
    ? (value as string)
    : fallback;
}
