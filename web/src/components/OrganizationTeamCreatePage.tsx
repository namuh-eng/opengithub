"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { type FormEvent, useMemo, useState } from "react";
import type { OrganizationTeamsDirectory } from "@/lib/api";
import { organizationTeamsHref } from "@/lib/navigation";

type OrganizationTeamCreatePageProps = {
  directory: OrganizationTeamsDirectory;
  org: string;
};

type SubmitState =
  | { status: "idle"; message: string | null }
  | { status: "submitting"; message: string | null }
  | { status: "error"; message: string };

function slugPreview(name: string) {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80);
  return slug || "team-slug";
}

export function OrganizationTeamCreatePage({
  directory,
  org,
}: OrganizationTeamCreatePageProps) {
  const router = useRouter();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [parentTeamId, setParentTeamId] = useState("");
  const [visibility, setVisibility] = useState<"visible" | "secret">("visible");
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [submitState, setSubmitState] = useState<SubmitState>({
    status: "idle",
    message: null,
  });
  const parentOptions = useMemo(
    () =>
      directory.parentOptions.filter(
        (option) => option.visibility === "visible",
      ),
    [directory.parentOptions],
  );
  const secretWithParent = visibility === "secret" && Boolean(parentTeamId);
  const canSubmit =
    submitState.status !== "submitting" &&
    name.trim().length > 0 &&
    !secretWithParent;

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) {
      setSubmitState({
        status: "error",
        message: secretWithParent
          ? "Secret teams cannot be nested under another team."
          : "Team name is required.",
      });
      return;
    }

    setSubmitState({ status: "submitting", message: "Creating team..." });
    const response = await fetch(
      `/orgs/${encodeURIComponent(org)}/teams/actions`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          name,
          description,
          parentTeamId: parentTeamId || null,
          visibility,
          notificationsEnabled,
        }),
      },
    );
    const body = (await response.json().catch(() => null)) as {
      destinationHref?: string;
      error?: { message?: string };
    } | null;
    if (!response.ok) {
      setSubmitState({
        status: "error",
        message: body?.error?.message ?? "Team creation failed.",
      });
      return;
    }

    router.push(body?.destinationHref ?? organizationTeamsHref(org));
  }

  return (
    <section
      aria-labelledby="team-create-title"
      className="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]"
    >
      <aside className="card self-start p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Team setup
        </p>
        <div className="mt-3 grid gap-2">
          <Link
            className="chip soft justify-start no-underline"
            href={organizationTeamsHref(org)}
          >
            Back to teams
          </Link>
          <Link
            className="chip soft justify-start no-underline"
            href="/docs/api#organization-teams"
          >
            Learn more
          </Link>
        </div>
        <p className="t-sm mt-4" style={{ color: "var(--ink-2)" }}>
          Visible teams are discoverable and mentionable by organization
          members. Secret teams stay limited to their members and owners.
        </p>
      </aside>

      <form className="card min-w-0 overflow-hidden" onSubmit={onSubmit}>
        <div className="border-b border-[var(--line)] p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            New organization team
          </p>
          <h2 className="t-h2 mt-1" id="team-create-title">
            Create team
          </h2>
          <p
            className="t-body mt-2 max-w-2xl"
            style={{ color: "var(--ink-2)" }}
          >
            Create a team for repository access, review ownership, and mention
            notifications.
          </p>
        </div>

        <div className="grid gap-5 p-5">
          <div className="grid gap-1">
            <label className="t-label" htmlFor="team-name">
              Team name
            </label>
            <input
              aria-describedby="team-slug-preview"
              className="input"
              id="team-name"
              name="name"
              onChange={(event) => setName(event.target.value)}
              placeholder="Platform Maintainers"
              value={name}
            />
            <span
              className="t-mono-sm"
              id="team-slug-preview"
              style={{ color: "var(--ink-3)" }}
            >
              @{slugPreview(name)}
            </span>
          </div>

          <div className="grid gap-1">
            <label className="t-label" htmlFor="team-description">
              Description
            </label>
            <textarea
              className="input min-h-28"
              id="team-description"
              maxLength={280}
              name="description"
              onChange={(event) => setDescription(event.target.value)}
              placeholder="What this team owns and when to mention it."
              value={description}
            />
          </div>

          <div className="grid gap-1">
            {parentOptions.length > 0 ? (
              <>
                <label className="t-label" htmlFor="parent-team">
                  Parent team
                </label>
                <select
                  id="parent-team"
                  className="input"
                  name="parentTeamId"
                  onChange={(event) => setParentTeamId(event.target.value)}
                  value={parentTeamId}
                >
                  <option value="">No parent team</option>
                  {parentOptions.map((option) => (
                    <option key={option.id} value={option.id}>
                      {option.name} (@{option.slug})
                    </option>
                  ))}
                </select>
              </>
            ) : (
              <>
                <span className="t-label">Parent team</span>
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  No visible parent teams are available yet.
                </p>
              </>
            )}
          </div>

          <fieldset className="grid gap-3">
            <legend className="t-label">Visibility</legend>
            <label className="card flex gap-3 p-4">
              <input
                aria-label="Visible"
                checked={visibility === "visible"}
                name="visibility"
                onChange={() => setVisibility("visible")}
                type="radio"
                value="visible"
              />
              <span>
                <span className="t-h3">Visible</span>
                <span className="chip ok ml-2">Recommended</span>
                <span className="t-sm block" style={{ color: "var(--ink-2)" }}>
                  Organization members can discover and mention this team.
                </span>
              </span>
            </label>
            <label className="card flex gap-3 p-4">
              <input
                aria-label="Secret"
                checked={visibility === "secret"}
                name="visibility"
                onChange={() => {
                  setVisibility("secret");
                  setParentTeamId("");
                }}
                type="radio"
                value="secret"
              />
              <span>
                <span className="t-h3">Secret</span>
                <span className="t-sm block" style={{ color: "var(--ink-2)" }}>
                  Only owners and team members can see this team. Secret teams
                  cannot be nested.
                </span>
              </span>
            </label>
          </fieldset>

          <fieldset className="grid gap-3">
            <legend className="t-label">Team notifications</legend>
            <label className="flex items-start gap-3">
              <input
                aria-label="Enabled"
                checked={notificationsEnabled}
                name="notificationsEnabled"
                onChange={() => setNotificationsEnabled(true)}
                type="radio"
              />
              <span>
                <span className="t-h3">Enabled</span>
                <span className="t-sm block" style={{ color: "var(--ink-2)" }}>
                  Notify team members when the team is mentioned.
                </span>
              </span>
            </label>
            <label className="flex items-start gap-3">
              <input
                aria-label="Disabled"
                checked={!notificationsEnabled}
                name="notificationsEnabled"
                onChange={() => setNotificationsEnabled(false)}
                type="radio"
              />
              <span>
                <span className="t-h3">Disabled</span>
                <span className="t-sm block" style={{ color: "var(--ink-2)" }}>
                  Keep mentions indexed without automatic member fanout.
                </span>
              </span>
            </label>
          </fieldset>

          {submitState.status === "error" ? (
            <div className="chip err justify-start" role="alert">
              {submitState.message}
            </div>
          ) : null}

          <div className="flex flex-wrap justify-end gap-2 border-t border-[var(--line)] pt-5">
            <Link
              className="btn ghost no-underline"
              href={organizationTeamsHref(org)}
            >
              Cancel
            </Link>
            <button className="btn primary" disabled={!canSubmit} type="submit">
              {submitState.status === "submitting"
                ? "Creating..."
                : "Create team"}
            </button>
          </div>
        </div>
      </form>
    </section>
  );
}
