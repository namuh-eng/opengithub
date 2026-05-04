"use client";

import { useMemo, useState } from "react";
import type { OrganizationProfileSettings } from "@/lib/api";

type OrganizationProfileSettingsFormProps = {
  settings: OrganizationProfileSettings;
};

type FormState = {
  billingEmail: string;
  companyName: string;
  contactEmail: string;
  description: string;
  displayName: string;
  location: string;
  publicEmail: string;
  socialAccounts: Record<string, string>;
  websiteUrl: string;
};

type FormErrors = Partial<Record<keyof FormState | "socialAccounts", string>>;

type SaveSection = "profile" | "contact" | "social";

const SOCIAL_LABELS: Record<string, string> = {
  bluesky: "Bluesky",
  linkedin: "LinkedIn",
  mastodon: "Mastodon",
  x: "X",
};

const SOCIAL_PROVIDERS = ["x", "mastodon", "linkedin", "bluesky"];

function valueOrEmpty(value: string | null) {
  return value ?? "";
}

function socialValue(settings: OrganizationProfileSettings, provider: string) {
  return (
    settings.socialAccounts.find((account) => account.provider === provider)
      ?.value ?? ""
  );
}

function settingsToForm(settings: OrganizationProfileSettings): FormState {
  return {
    billingEmail: valueOrEmpty(settings.profile.billingEmail),
    companyName: valueOrEmpty(settings.profile.companyName),
    contactEmail: valueOrEmpty(settings.profile.contactEmail),
    description: valueOrEmpty(settings.profile.description),
    displayName: settings.profile.displayName,
    location: valueOrEmpty(settings.profile.location),
    publicEmail: valueOrEmpty(settings.profile.publicEmail),
    socialAccounts: Object.fromEntries(
      SOCIAL_PROVIDERS.map((provider) => [
        provider,
        socialValue(settings, provider),
      ]),
    ),
    websiteUrl: valueOrEmpty(settings.profile.websiteUrl),
  };
}

function optionalValue(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function profilePayload(form: FormState) {
  return {
    companyName: optionalValue(form.companyName),
    description: optionalValue(form.description),
    displayName: form.displayName.trim(),
    location: optionalValue(form.location),
    publicEmail: optionalValue(form.publicEmail),
    websiteUrl: optionalValue(form.websiteUrl),
  };
}

function contactPayload(form: FormState) {
  return {
    billingEmail: optionalValue(form.billingEmail),
    contactEmail: optionalValue(form.contactEmail),
  };
}

function socialPayload(form: FormState) {
  return {
    socialAccounts: SOCIAL_PROVIDERS.map((provider) => ({
      provider,
      value: form.socialAccounts[provider]?.trim() ?? "",
    })).filter((account) => account.value),
  };
}

function validateEmail(value: string, label: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const valid =
    trimmed.length <= 254 &&
    trimmed.split("@").length === 2 &&
    trimmed.split("@")[0].length > 0 &&
    trimmed.split("@")[1]?.includes(".") &&
    !trimmed.split("@")[1]?.startsWith(".") &&
    !trimmed.split("@")[1]?.endsWith(".");
  return valid ? undefined : `Enter a valid ${label.toLowerCase()}.`;
}

function validateProfile(form: FormState): FormErrors {
  const errors: FormErrors = {};
  if (!form.displayName.trim()) {
    errors.displayName = "Organization display name is required.";
  }
  const website = form.websiteUrl.trim().toLowerCase();
  if (
    website &&
    !(website.startsWith("https://") || website.startsWith("http://"))
  ) {
    errors.websiteUrl = "URL must start with http:// or https://.";
  }
  const publicEmail = validateEmail(form.publicEmail, "public email");
  if (publicEmail) errors.publicEmail = publicEmail;
  return errors;
}

function validateContact(form: FormState): FormErrors {
  const errors: FormErrors = {};
  const contactEmail = validateEmail(form.contactEmail, "contact email");
  const billingEmail = validateEmail(form.billingEmail, "billing email");
  if (contactEmail) errors.contactEmail = contactEmail;
  if (billingEmail) errors.billingEmail = billingEmail;
  return errors;
}

function validateSocial(form: FormState): FormErrors {
  const tooLong = SOCIAL_PROVIDERS.find(
    (provider) => (form.socialAccounts[provider]?.trim().length ?? 0) > 160,
  );
  return tooLong
    ? {
        socialAccounts: `${SOCIAL_LABELS[tooLong]} must be 160 characters or fewer.`,
      }
    : {};
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

function Field({
  disabled = false,
  error,
  help,
  label,
  name,
  onChange,
  type = "text",
  value,
}: {
  disabled?: boolean;
  error?: string;
  help?: string;
  label: string;
  name: string;
  onChange?: (value: string) => void;
  type?: string;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="t-sm font-semibold">{label}</span>
      <input
        aria-label={label}
        aria-describedby={error ? `${name}-error` : undefined}
        aria-invalid={error ? "true" : undefined}
        className="input"
        disabled={disabled}
        name={name}
        onChange={(event) => onChange?.(event.target.value)}
        type={type}
        value={value}
      />
      {error ? (
        <span
          className="t-xs"
          id={`${name}-error`}
          style={{ color: "var(--err)" }}
        >
          {error}
        </span>
      ) : null}
      {help ? <span className="t-xs">{help}</span> : null}
    </label>
  );
}

function TextArea({
  disabled = false,
  error,
  help,
  label,
  name,
  onChange,
  value,
}: {
  disabled?: boolean;
  error?: string;
  help?: string;
  label: string;
  name: string;
  onChange?: (value: string) => void;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="t-sm font-semibold">{label}</span>
      <textarea
        aria-label={label}
        aria-describedby={error ? `${name}-error` : undefined}
        aria-invalid={error ? "true" : undefined}
        className="input min-h-28 resize-y"
        disabled={disabled}
        name={name}
        onChange={(event) => onChange?.(event.target.value)}
        value={value}
      />
      {error ? (
        <span
          className="t-xs"
          id={`${name}-error`}
          style={{ color: "var(--err)" }}
        >
          {error}
        </span>
      ) : null}
      {help ? <span className="t-xs">{help}</span> : null}
    </label>
  );
}

function SectionSave({
  disabled,
  label,
  saving,
}: {
  disabled: boolean;
  label: string;
  saving: boolean;
}) {
  return (
    <button
      className="btn sm primary"
      disabled={disabled || saving}
      type="submit"
    >
      {saving ? "Saving..." : label}
    </button>
  );
}

export function OrganizationProfileSettingsForm({
  settings,
}: OrganizationProfileSettingsFormProps) {
  const [currentSettings, setCurrentSettings] = useState(settings);
  const [form, setForm] = useState(() => settingsToForm(settings));
  const [savedForm, setSavedForm] = useState(() => settingsToForm(settings));
  const [errors, setErrors] = useState<FormErrors>({});
  const [saving, setSaving] = useState<SaveSection | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const profile = currentSettings.profile;
  const avatarLabel =
    currentSettings.organization.name || currentSettings.organization.slug;
  const canEdit = currentSettings.viewerState.canEditProfile;
  const profileDirty = useMemo(
    () =>
      JSON.stringify(profilePayload(form)) !==
      JSON.stringify(profilePayload(savedForm)),
    [form, savedForm],
  );
  const contactDirty = useMemo(
    () =>
      JSON.stringify(contactPayload(form)) !==
      JSON.stringify(contactPayload(savedForm)),
    [form, savedForm],
  );
  const socialDirty = useMemo(
    () =>
      JSON.stringify(socialPayload(form)) !==
      JSON.stringify(socialPayload(savedForm)),
    [form, savedForm],
  );

  function patchForm(patch: Partial<FormState>) {
    setForm((current) => ({ ...current, ...patch }));
    setErrors((current) => ({
      ...current,
      ...Object.fromEntries(Object.keys(patch).map((key) => [key, undefined])),
    }));
  }

  function updateSocial(provider: string, value: string) {
    setForm((current) => ({
      ...current,
      socialAccounts: { ...current.socialAccounts, [provider]: value },
    }));
    setErrors((current) => ({ ...current, socialAccounts: undefined }));
  }

  function applySettings(next: OrganizationProfileSettings, message: string) {
    const nextForm = settingsToForm(next);
    setCurrentSettings(next);
    setForm(nextForm);
    setSavedForm(nextForm);
    setToast(message);
    setErrors({});
  }

  async function saveSection(section: SaveSection) {
    const nextErrors =
      section === "profile"
        ? validateProfile(form)
        : section === "contact"
          ? validateContact(form)
          : validateSocial(form);
    setErrors(nextErrors);
    if (Object.keys(nextErrors).length) return;

    const payload =
      section === "profile"
        ? profilePayload(form)
        : section === "contact"
          ? contactPayload(form)
          : socialPayload(form);
    setSaving(section);
    try {
      const response = await fetch(
        `/organizations/${encodeURIComponent(
          currentSettings.organization.slug,
        )}/settings/profile/actions`,
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        },
      );
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Organization settings could not be saved",
        );
      }
      applySettings(
        body as OrganizationProfileSettings,
        section === "profile"
          ? "Public profile updated"
          : section === "contact"
            ? "Administrative contact updated"
            : "Social accounts updated",
      );
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "Organization settings could not be saved";
      setErrors(
        section === "social"
          ? { socialAccounts: message }
          : section === "contact"
            ? { contactEmail: message }
            : { displayName: message },
      );
    } finally {
      setSaving(null);
    }
  }

  return (
    <div className="grid gap-6">
      {toast ? (
        <div className="chip ok w-fit" role="status">
          {toast}
        </div>
      ) : (
        <div className="chip soft w-fit" role="status">
          Owner access confirmed
        </div>
      )}

      <SettingsCard kicker="Identity" title="Profile picture">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex min-w-0 items-center gap-4">
            {currentSettings.avatar.avatarUrl ? (
              <span
                aria-label={`${avatarLabel} avatar`}
                className="av lg shrink-0"
                role="img"
                style={{
                  backgroundImage: `url(${currentSettings.avatar.avatarUrl})`,
                  backgroundPosition: "center",
                  backgroundSize: "cover",
                }}
              />
            ) : (
              <span
                className="av lg shrink-0"
                aria-label={avatarLabel}
                role="img"
              >
                {avatarLabel.trim().slice(0, 1).toUpperCase() || "O"}
              </span>
            )}
            <div className="min-w-0">
              <p className="t-sm font-semibold">Organization avatar</p>
              <p className="t-xs mt-1 break-words">
                {currentSettings.avatar.unavailableReason ??
                  "Avatar upload is not available for this organization yet."}
              </p>
            </div>
          </div>
          <button className="btn sm" disabled type="button">
            Upload unavailable
          </button>
        </div>
      </SettingsCard>

      <SettingsCard kicker="Public profile" title="Organization profile">
        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            void saveSection("profile");
          }}
        >
          <Field
            disabled={!canEdit}
            error={errors.displayName}
            help="Shown on the organization profile and repository owner headers."
            label="Organization display name"
            name="displayName"
            onChange={(value) => patchForm({ displayName: value })}
            value={form.displayName}
          />
          <TextArea
            disabled={!canEdit}
            error={errors.description}
            help="A short public description for people visiting the organization."
            label="Description"
            name="description"
            onChange={(value) => patchForm({ description: value })}
            value={form.description}
          />
          <div className="grid gap-4 md:grid-cols-2">
            <Field
              disabled={!canEdit}
              error={errors.websiteUrl}
              label="URL"
              name="websiteUrl"
              onChange={(value) => patchForm({ websiteUrl: value })}
              type="url"
              value={form.websiteUrl}
            />
            <Field
              disabled={!canEdit}
              error={errors.location}
              label="Location"
              name="location"
              onChange={(value) => patchForm({ location: value })}
              value={form.location}
            />
            <Field
              disabled={!canEdit}
              error={errors.publicEmail}
              label="Public email"
              name="publicEmail"
              onChange={(value) => patchForm({ publicEmail: value })}
              type="email"
              value={form.publicEmail}
            />
            <Field
              disabled={!canEdit}
              error={errors.companyName}
              label="Company"
              name="companyName"
              onChange={(value) => patchForm({ companyName: value })}
              value={form.companyName}
            />
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <span className="chip soft">{profile.profileVisibility}</span>
            <span className="chip soft">{profile.ownershipType}</span>
            {profile.publicMembersVisible ? (
              <span className="chip ok">Public members visible</span>
            ) : (
              <span className="chip warn">Members private</span>
            )}
          </div>
          <SectionSave
            disabled={!canEdit || !profileDirty}
            label="Save profile changes"
            saving={saving === "profile"}
          />
        </form>
      </SettingsCard>

      <SettingsCard kicker="Contact" title="Administrative contact">
        <form
          onSubmit={(event) => {
            event.preventDefault();
            void saveSection("contact");
          }}
        >
          <div className="grid gap-4 md:grid-cols-2">
            <Field
              disabled={!canEdit}
              error={errors.contactEmail}
              help="Used for organization administration and policy notices."
              label="Contact email"
              name="contactEmail"
              onChange={(value) => patchForm({ contactEmail: value })}
              type="email"
              value={form.contactEmail}
            />
            <Field
              disabled={!canEdit}
              error={errors.billingEmail}
              help="Billing pages are outside this clone's current scope."
              label="Billing email"
              name="billingEmail"
              onChange={(value) => patchForm({ billingEmail: value })}
              type="email"
              value={form.billingEmail}
            />
          </div>
          <div className="mt-4">
            <SectionSave
              disabled={!canEdit || !contactDirty}
              label="Save contact changes"
              saving={saving === "contact"}
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Social" title="Social accounts">
        <form
          onSubmit={(event) => {
            event.preventDefault();
            void saveSection("social");
          }}
        >
          <div className="grid gap-4 md:grid-cols-2">
            {SOCIAL_PROVIDERS.map((provider) => (
              <Field
                disabled={!canEdit}
                key={provider}
                label={SOCIAL_LABELS[provider]}
                name={`social-${provider}`}
                onChange={(value) => updateSocial(provider, value)}
                value={form.socialAccounts[provider] ?? ""}
              />
            ))}
          </div>
          {errors.socialAccounts ? (
            <p
              className="t-xs mt-3"
              role="alert"
              style={{ color: "var(--err)" }}
            >
              {errors.socialAccounts}
            </p>
          ) : null}
          <div className="mt-4">
            <SectionSave
              disabled={!canEdit || !socialDirty}
              label="Save social accounts"
              saving={saving === "social"}
            />
          </div>
        </form>
      </SettingsCard>

      <SettingsCard kicker="Danger zone" title="Organization controls">
        <div className="grid gap-3">
          <div
            className="flex flex-col gap-3 rounded-md p-4 sm:flex-row sm:items-center sm:justify-between"
            style={{ border: "1px solid var(--line)" }}
          >
            <div>
              <p className="t-sm font-semibold">Rename organization</p>
              <p className="t-xs mt-1">
                Slug validation and confirmation are scheduled for the next
                organization settings phase.
              </p>
            </div>
            <button className="btn sm" disabled type="button">
              Rename unavailable
            </button>
          </div>
          <div
            className="flex flex-col gap-3 rounded-md p-4 sm:flex-row sm:items-center sm:justify-between"
            style={{ border: "1px solid var(--line)" }}
          >
            <div>
              <p className="t-sm font-semibold">Archive or delete</p>
              <p className="t-xs mt-1">
                Destructive actions stay disabled until retention guardrails are
                implemented.
              </p>
            </div>
            <button className="btn sm" disabled type="button">
              Danger actions unavailable
            </button>
          </div>
        </div>
      </SettingsCard>
    </div>
  );
}
