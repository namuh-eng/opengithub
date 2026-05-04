"use client";

import Link from "next/link";
import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ApiErrorEnvelope,
  OrganizationMemberPrivilegesPolicies,
  OrganizationMemberPrivilegesSettings,
  OrganizationPolicyLock,
  UpdateOrganizationMemberPrivilegesRequest,
} from "@/lib/api";

type OrganizationMemberPrivilegesPageProps = {
  settings: OrganizationMemberPrivilegesSettings;
};

type SaveState = {
  card: string | null;
  error: string | null;
  message: string | null;
  pending: boolean;
};

type ConfirmationState = {
  fields: string[];
  label: string;
  payload: UpdateOrganizationMemberPrivilegesRequest;
  successMessage: string;
} | null;

const PERMISSION_OPTIONS = [
  {
    value: "none",
    label: "None",
    description: "Members receive no automatic repository access.",
  },
  {
    value: "read",
    label: "Read",
    description: "Members can view and clone organization repositories.",
  },
  {
    value: "write",
    label: "Write",
    description: "Members can push to repositories unless access is narrowed.",
  },
  {
    value: "admin",
    label: "Admin",
    description: "Members receive broad repository administration by default.",
  },
] as const;

const APP_ACCESS_OPTIONS = [
  {
    value: "owners_only",
    label: "Owners only",
    description: "Only owners can request or approve third-party app access.",
  },
  {
    value: "owners_and_members",
    label: "Owners and members",
    description: "Members can request app access for owner review.",
  },
] as const;

const FIELD_LABELS: Record<string, string> = {
  appAccessRequestPolicy: "App access requests",
  baseRepositoryPermission: "Base repository permission",
  membersCanChangeRepositoryVisibility: "Repository visibility changes",
  membersCanCreateInternalRepositories: "Internal repository creation",
  membersCanCreatePrivateRepositories: "Private repository creation",
  membersCanCreatePublicRepositories: "Public repository creation",
  membersCanCreateTeams: "Team creation",
  membersCanDeleteIssues: "Issue deletion",
  membersCanDeleteRepositories: "Repository deletion",
  membersCanForkPrivateRepositories: "Private repository forking",
  membersCanTransferRepositories: "Repository transfers",
  pagesPrivatePublishing: "Private Pages publishing",
  pagesPublicPublishing: "Public Pages publishing",
  projectsBasePermission: "Projects base permission",
  repositoryDiscussionsEnabled: "Repository discussions",
};

function clonePolicies(
  policies: OrganizationMemberPrivilegesPolicies,
): OrganizationMemberPrivilegesPolicies {
  return { ...policies };
}

function lockForField(
  locks: OrganizationPolicyLock[],
  field: keyof OrganizationMemberPrivilegesPolicies,
) {
  return locks.find((lock) => lock.field === field) ?? null;
}

function errorCause(error: unknown): ApiErrorEnvelope | null {
  return error instanceof Error &&
    error.cause &&
    typeof error.cause === "object"
    ? (error.cause as ApiErrorEnvelope)
    : null;
}

function labelForField(field: string) {
  return FIELD_LABELS[field] ?? field;
}

function SettingsCard({
  children,
  kicker,
  title,
}: {
  children: React.ReactNode;
  kicker: string;
  title: string;
}) {
  return (
    <section className="card p-5">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {kicker}
      </p>
      <h3 className="t-h2 mt-2">{title}</h3>
      <div className="mt-5">{children}</div>
    </section>
  );
}

function LockNotice({ lock }: { lock: OrganizationPolicyLock | null }) {
  if (!lock) return null;
  return (
    <div
      className="mt-3 rounded-md p-3 t-sm"
      style={{
        background: "var(--surface-2)",
        border: "1px solid var(--line)",
        color: "var(--ink-2)",
      }}
    >
      <span className="chip warn mr-2">Locked</span>
      {lock.reason}
      {lock.href ? (
        <Link className="ml-2 underline" href={lock.href}>
          Why
        </Link>
      ) : (
        <span className="ml-2 t-xs">Enforced by {lock.enforcedBy}</span>
      )}
    </div>
  );
}

function SaveButton({
  disabled,
  label,
  pending,
}: {
  disabled?: boolean;
  label: string;
  pending: boolean;
}) {
  return (
    <button
      className="btn sm primary"
      disabled={disabled || pending}
      type="submit"
    >
      {pending ? "Saving..." : label}
    </button>
  );
}

function PermissionMenu({
  disabled,
  label,
  name,
  onChange,
  value,
}: {
  disabled?: boolean;
  label: string;
  name: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <fieldset className="grid gap-2">
      <legend className="t-label mb-1" style={{ color: "var(--ink-3)" }}>
        {label}
      </legend>
      <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
        {PERMISSION_OPTIONS.map((option) => (
          <label
            className="rounded-md p-3"
            key={option.value}
            style={{
              background:
                value === option.value ? "var(--surface-2)" : "transparent",
              border:
                value === option.value
                  ? "1px solid var(--line-strong)"
                  : "1px solid var(--line)",
              color: disabled ? "var(--ink-4)" : "var(--ink-1)",
            }}
          >
            <span className="flex items-center gap-2">
              <input
                aria-label={option.label}
                checked={value === option.value}
                disabled={disabled}
                name={name}
                onChange={() => onChange(option.value)}
                type="radio"
              />
              <span className="font-medium">{option.label}</span>
            </span>
            <span className="mt-2 block t-xs">{option.description}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}

function ToggleField({
  checked,
  description,
  disabled,
  label,
  name,
  onChange,
}: {
  checked: boolean;
  description: string;
  disabled?: boolean;
  label: string;
  name: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label
      className="flex min-w-0 items-start gap-3 rounded-md p-3"
      style={{ border: "1px solid var(--line)", color: "var(--ink-1)" }}
    >
      <input
        aria-label={label}
        checked={checked}
        className="mt-1 shrink-0"
        disabled={disabled}
        name={name}
        onChange={(event) => onChange(event.currentTarget.checked)}
        type="checkbox"
      />
      <span className="min-w-0">
        <span className="block font-medium break-words">{label}</span>
        <span className="mt-1 block t-xs break-words">{description}</span>
      </span>
    </label>
  );
}

export function OrganizationMemberPrivilegesPage({
  settings,
}: OrganizationMemberPrivilegesPageProps) {
  const [currentSettings, setCurrentSettings] = useState(settings);
  const [form, setForm] = useState(() => clonePolicies(settings.policies));
  const [saveState, setSaveState] = useState<SaveState>({
    card: null,
    error: null,
    message: null,
    pending: false,
  });
  const [confirmation, setConfirmation] = useState<ConfirmationState>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const errorRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (confirmation) {
      dialogRef.current?.focus();
    }
  }, [confirmation]);

  useEffect(() => {
    if (saveState.error) {
      errorRef.current?.focus();
    }
  }, [saveState.error]);

  const locks = currentSettings.capabilities.locks;
  const canUpdate = currentSettings.capabilities.canUpdate;
  const actionPath = `/organizations/${encodeURIComponent(
    currentSettings.organization.slug,
  )}/settings/member_privileges/actions`;

  const lockMap = useMemo(
    () =>
      Object.fromEntries(
        Object.keys(currentSettings.policies).map((field) => [
          field,
          lockForField(
            locks,
            field as keyof OrganizationMemberPrivilegesPolicies,
          ),
        ]),
      ) as Record<
        keyof OrganizationMemberPrivilegesPolicies,
        OrganizationPolicyLock | null
      >,
    [currentSettings.policies, locks],
  );

  function updateField<K extends keyof OrganizationMemberPrivilegesPolicies>(
    field: K,
    value: OrganizationMemberPrivilegesPolicies[K],
  ) {
    setForm((previous) => ({ ...previous, [field]: value }));
    setSaveState({ card: null, error: null, message: null, pending: false });
  }

  async function submitPatch(
    label: string,
    payload: UpdateOrganizationMemberPrivilegesRequest,
    successMessage: string,
  ) {
    setSaveState({ card: label, error: null, message: null, pending: true });
    const response = await fetch(actionPath, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload),
    });
    const body = await response.json().catch(() => null);
    if (!response.ok) {
      const envelope = body as ApiErrorEnvelope | null;
      if (envelope?.error.code === "confirmation_required") {
        const fields = Array.isArray(envelope.details?.fields)
          ? envelope.details.fields.map(String)
          : [];
        setConfirmation({ fields, label, payload, successMessage });
        setSaveState({
          card: label,
          error: envelope.error.message,
          message: null,
          pending: false,
        });
        return;
      }
      throw new Error(
        envelope?.error.message ?? "Organization policy update failed",
        { cause: envelope },
      );
    }

    const nextSettings = body as OrganizationMemberPrivilegesSettings;
    setCurrentSettings(nextSettings);
    setForm(clonePolicies(nextSettings.policies));
    setSaveState({
      card: label,
      error: null,
      message: successMessage,
      pending: false,
    });
  }

  async function saveCard(
    label: string,
    payload: UpdateOrganizationMemberPrivilegesRequest,
    successMessage: string,
    confirmFirst = false,
  ) {
    if (confirmFirst) {
      setConfirmation({
        fields: Object.keys(payload),
        label,
        payload,
        successMessage,
      });
      setSaveState({ card: label, error: null, message: null, pending: false });
      return;
    }
    try {
      await submitPatch(label, payload, successMessage);
    } catch (error) {
      const envelope = errorCause(error);
      setSaveState({
        card: label,
        error:
          envelope?.error.message ??
          (error instanceof Error
            ? error.message
            : "Organization policy update failed"),
        message: null,
        pending: false,
      });
    }
  }

  async function confirmSave() {
    if (!confirmation) return;
    const next = {
      ...confirmation.payload,
      confirmation: "confirm",
    };
    const label = confirmation.label;
    const message = confirmation.successMessage;
    setConfirmation(null);
    try {
      await submitPatch(label, next, message);
    } catch (error) {
      const envelope = errorCause(error);
      setSaveState({
        card: label,
        error:
          envelope?.error.message ??
          (error instanceof Error
            ? error.message
            : "Organization policy update failed"),
        message: null,
        pending: false,
      });
    }
  }

  const disabled = !canUpdate || saveState.pending;

  return (
    <div className="grid min-w-0 gap-5">
      <div className="flex flex-wrap items-center gap-2">
        <Link
          className="btn sm"
          href="/docs/api#organization-member-privileges"
        >
          API docs
        </Link>
        <span className="chip soft">{currentSettings.viewerState.role}</span>
        <span className="chip info">
          {canUpdate ? "Owner editable" : "Read only"}
        </span>
      </div>

      {saveState.message ? (
        <p className="chip ok" role="status">
          {saveState.message}
        </p>
      ) : null}
      {saveState.error ? (
        <p className="chip err" ref={errorRef} role="alert" tabIndex={-1}>
          {saveState.error}
        </p>
      ) : null}

      <SettingsCard kicker="Repository access" title="Base permissions">
        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "base-permission",
              { baseRepositoryPermission: form.baseRepositoryPermission },
              "Base repository permission updated",
              form.baseRepositoryPermission !==
                currentSettings.policies.baseRepositoryPermission,
            );
          }}
        >
          <PermissionMenu
            disabled={disabled || Boolean(lockMap.baseRepositoryPermission)}
            label="Default repository role"
            name="baseRepositoryPermission"
            onChange={(value) => updateField("baseRepositoryPermission", value)}
            value={form.baseRepositoryPermission}
          />
          <LockNotice lock={lockMap.baseRepositoryPermission} />
          <div className="flex justify-end">
            <SaveButton
              disabled={
                disabled ||
                Boolean(lockMap.baseRepositoryPermission) ||
                form.baseRepositoryPermission ===
                  currentSettings.policies.baseRepositoryPermission
              }
              label="Save base permission"
              pending={
                saveState.pending && saveState.card === "base-permission"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Repository creation" title="Creation visibility">
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "repository-creation",
              {
                membersCanCreateInternalRepositories:
                  form.membersCanCreateInternalRepositories,
                membersCanCreatePrivateRepositories:
                  form.membersCanCreatePrivateRepositories,
                membersCanCreatePublicRepositories:
                  form.membersCanCreatePublicRepositories,
              },
              "Repository creation policy updated",
            );
          }}
        >
          <ToggleField
            checked={form.membersCanCreatePublicRepositories}
            description="Allow members to create public organization repositories."
            disabled={
              disabled || Boolean(lockMap.membersCanCreatePublicRepositories)
            }
            label="Public repositories"
            name="membersCanCreatePublicRepositories"
            onChange={(value) =>
              updateField("membersCanCreatePublicRepositories", value)
            }
          />
          <ToggleField
            checked={form.membersCanCreatePrivateRepositories}
            description="Allow members to create private organization repositories."
            disabled={
              disabled || Boolean(lockMap.membersCanCreatePrivateRepositories)
            }
            label="Private repositories"
            name="membersCanCreatePrivateRepositories"
            onChange={(value) =>
              updateField("membersCanCreatePrivateRepositories", value)
            }
          />
          <ToggleField
            checked={form.membersCanCreateInternalRepositories}
            description="Allow internal repositories when the organization supports that visibility."
            disabled={
              disabled || Boolean(lockMap.membersCanCreateInternalRepositories)
            }
            label="Internal repositories"
            name="membersCanCreateInternalRepositories"
            onChange={(value) =>
              updateField("membersCanCreateInternalRepositories", value)
            }
          />
          <LockNotice lock={lockMap.membersCanCreatePublicRepositories} />
          <LockNotice lock={lockMap.membersCanCreatePrivateRepositories} />
          <LockNotice lock={lockMap.membersCanCreateInternalRepositories} />
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled}
              label="Save repository creation"
              pending={
                saveState.pending && saveState.card === "repository-creation"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard
        kicker="Repository features"
        title="Forking and discussions"
      >
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "repository-features",
              {
                membersCanForkPrivateRepositories:
                  form.membersCanForkPrivateRepositories,
                repositoryDiscussionsEnabled: form.repositoryDiscussionsEnabled,
              },
              "Repository feature policy updated",
            );
          }}
        >
          <ToggleField
            checked={form.membersCanForkPrivateRepositories}
            description="Members can fork private repositories they can read."
            disabled={
              disabled || Boolean(lockMap.membersCanForkPrivateRepositories)
            }
            label="Private repository forking"
            name="membersCanForkPrivateRepositories"
            onChange={(value) =>
              updateField("membersCanForkPrivateRepositories", value)
            }
          />
          <ToggleField
            checked={form.repositoryDiscussionsEnabled}
            description="New repositories can use Discussions when the feature is available."
            disabled={disabled || Boolean(lockMap.repositoryDiscussionsEnabled)}
            label="Repository discussions"
            name="repositoryDiscussionsEnabled"
            onChange={(value) =>
              updateField("repositoryDiscussionsEnabled", value)
            }
          />
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled}
              label="Save repository features"
              pending={
                saveState.pending && saveState.card === "repository-features"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Projects" title="Projects base permission">
        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "projects-permission",
              { projectsBasePermission: form.projectsBasePermission },
              "Projects base permission updated",
              form.projectsBasePermission !==
                currentSettings.policies.projectsBasePermission,
            );
          }}
        >
          <PermissionMenu
            disabled={disabled || Boolean(lockMap.projectsBasePermission)}
            label="Default project role"
            name="projectsBasePermission"
            onChange={(value) => updateField("projectsBasePermission", value)}
            value={form.projectsBasePermission}
          />
          <div className="flex justify-end">
            <SaveButton
              disabled={
                disabled ||
                Boolean(lockMap.projectsBasePermission) ||
                form.projectsBasePermission ===
                  currentSettings.policies.projectsBasePermission
              }
              label="Save Projects permission"
              pending={
                saveState.pending && saveState.card === "projects-permission"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Pages" title="Publishing policy">
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "pages-publishing",
              {
                pagesPrivatePublishing: form.pagesPrivatePublishing,
                pagesPublicPublishing: form.pagesPublicPublishing,
              },
              "Pages publishing policy updated",
            );
          }}
        >
          <ToggleField
            checked={form.pagesPublicPublishing}
            description="Allow Pages publication from public repositories."
            disabled={disabled || Boolean(lockMap.pagesPublicPublishing)}
            label="Public Pages publishing"
            name="pagesPublicPublishing"
            onChange={(value) => updateField("pagesPublicPublishing", value)}
          />
          <ToggleField
            checked={form.pagesPrivatePublishing}
            description="Allow Pages publication from private repositories."
            disabled={disabled || Boolean(lockMap.pagesPrivatePublishing)}
            label="Private Pages publishing"
            name="pagesPrivatePublishing"
            onChange={(value) => updateField("pagesPrivatePublishing", value)}
          />
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled}
              label="Save Pages policy"
              pending={
                saveState.pending && saveState.card === "pages-publishing"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Integrations" title="App access requests">
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "app-access",
              { appAccessRequestPolicy: form.appAccessRequestPolicy },
              "App access request policy updated",
            );
          }}
        >
          <fieldset className="grid gap-2">
            <legend className="t-label mb-1" style={{ color: "var(--ink-3)" }}>
              Request policy
            </legend>
            {APP_ACCESS_OPTIONS.map((option) => (
              <label
                className="rounded-md p-3"
                key={option.value}
                style={{ border: "1px solid var(--line)" }}
              >
                <span className="flex items-center gap-2">
                  <input
                    aria-label={option.label}
                    checked={form.appAccessRequestPolicy === option.value}
                    disabled={
                      disabled || Boolean(lockMap.appAccessRequestPolicy)
                    }
                    name="appAccessRequestPolicy"
                    onChange={() =>
                      updateField("appAccessRequestPolicy", option.value)
                    }
                    type="radio"
                  />
                  <span className="font-medium">{option.label}</span>
                </span>
                <span className="mt-1 block t-xs">{option.description}</span>
              </label>
            ))}
          </fieldset>
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled || Boolean(lockMap.appAccessRequestPolicy)}
              label="Save app access"
              pending={saveState.pending && saveState.card === "app-access"}
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard
        kicker="Destructive actions"
        title="Visibility, delete, and transfer"
      >
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "destructive-actions",
              {
                membersCanChangeRepositoryVisibility:
                  form.membersCanChangeRepositoryVisibility,
                membersCanDeleteIssues: form.membersCanDeleteIssues,
                membersCanDeleteRepositories: form.membersCanDeleteRepositories,
                membersCanTransferRepositories:
                  form.membersCanTransferRepositories,
              },
              "Destructive action policy updated",
            );
          }}
        >
          <ToggleField
            checked={form.membersCanChangeRepositoryVisibility}
            description="Members can change repository visibility where they already administer the repository."
            disabled={
              disabled || Boolean(lockMap.membersCanChangeRepositoryVisibility)
            }
            label="Repository visibility changes"
            name="membersCanChangeRepositoryVisibility"
            onChange={(value) =>
              updateField("membersCanChangeRepositoryVisibility", value)
            }
          />
          <ToggleField
            checked={form.membersCanDeleteRepositories}
            description="Members can delete organization repositories they administer."
            disabled={disabled || Boolean(lockMap.membersCanDeleteRepositories)}
            label="Repository deletion"
            name="membersCanDeleteRepositories"
            onChange={(value) =>
              updateField("membersCanDeleteRepositories", value)
            }
          />
          <ToggleField
            checked={form.membersCanTransferRepositories}
            description="Members can transfer repositories out of the organization."
            disabled={
              disabled || Boolean(lockMap.membersCanTransferRepositories)
            }
            label="Repository transfers"
            name="membersCanTransferRepositories"
            onChange={(value) =>
              updateField("membersCanTransferRepositories", value)
            }
          />
          <ToggleField
            checked={form.membersCanDeleteIssues}
            description="Members can delete issues in repositories where they can triage."
            disabled={disabled || Boolean(lockMap.membersCanDeleteIssues)}
            label="Issue deletion"
            name="membersCanDeleteIssues"
            onChange={(value) => updateField("membersCanDeleteIssues", value)}
          />
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled}
              label="Save destructive actions"
              pending={
                saveState.pending && saveState.card === "destructive-actions"
              }
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Teams" title="Team creation">
        <form
          className="grid gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            saveCard(
              "team-creation",
              { membersCanCreateTeams: form.membersCanCreateTeams },
              "Team creation policy updated",
            );
          }}
        >
          <ToggleField
            checked={form.membersCanCreateTeams}
            description="Members can create teams when they can administer organization access."
            disabled={disabled || Boolean(lockMap.membersCanCreateTeams)}
            label="Members can create teams"
            name="membersCanCreateTeams"
            onChange={(value) => updateField("membersCanCreateTeams", value)}
          />
          <div className="flex justify-end">
            <SaveButton
              disabled={disabled || Boolean(lockMap.membersCanCreateTeams)}
              label="Save team creation"
              pending={saveState.pending && saveState.card === "team-creation"}
            />
          </div>
        </form>
      </SettingsCard>

      {confirmation ? (
        <div
          aria-labelledby="policy-confirmation-title"
          aria-modal="true"
          className="fixed inset-0 z-50 grid place-items-center p-4"
          role="dialog"
          style={{
            background: "color-mix(in oklch, var(--ink-1) 28%, transparent)",
          }}
        >
          <div className="card max-w-lg p-5" ref={dialogRef} tabIndex={-1}>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Confirmation
            </p>
            <h3 className="t-h2 mt-2" id="policy-confirmation-title">
              Confirm organization policy change
            </h3>
            <p className="mt-3 t-sm" style={{ color: "var(--ink-2)" }}>
              This change can immediately alter access for organization members.
              Confirm before saving{" "}
              {confirmation.fields.map(labelForField).join(", ")}.
            </p>
            <div className="mt-5 flex flex-wrap justify-end gap-2">
              <button
                className="btn sm"
                onClick={() => setConfirmation(null)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn sm primary"
                onClick={confirmSave}
                type="button"
              >
                Confirm and save
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
