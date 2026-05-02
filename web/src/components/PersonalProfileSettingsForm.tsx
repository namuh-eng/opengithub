"use client";

import { useMemo, useRef, useState } from "react";
import type {
  PersonalProfileSettings,
  UpdatePersonalProfileSettingsRequest,
  UserSocialAccount,
} from "@/lib/api";

const PRONOUN_OPTIONS = ["", "they/them", "she/her", "he/him", "custom"];
const TIME_ZONES = [
  "UTC",
  "America/New_York",
  "America/Los_Angeles",
  "Europe/London",
  "Europe/Paris",
  "Asia/Seoul",
  "Asia/Tokyo",
];
const LANGUAGES = ["en", "ko", "ja", "fr", "de", "es"];
const MAX_AVATAR_BYTES = 2 * 1024 * 1024;
const AVATAR_TYPES = ["image/png", "image/jpeg", "image/webp", "image/gif"];

function settingsToForm(settings: PersonalProfileSettings) {
  const pronounsKnown = PRONOUN_OPTIONS.includes(settings.pronouns);
  return {
    displayName: settings.displayName,
    publicEmailId: settings.publicEmailId ?? "",
    bio: settings.bio,
    pronouns: pronounsKnown ? settings.pronouns : "custom",
    customPronouns: pronounsKnown ? "" : settings.pronouns,
    websiteUrl: settings.websiteUrl,
    company: settings.company,
    location: settings.location,
    displayLocalTime: settings.displayLocalTime,
    timeZone: settings.timeZone,
    preferredLanguage: settings.preferredLanguage,
    privateProfile: settings.privateProfile,
    showPrivateContributionCount: settings.showPrivateContributionCount,
    achievementsEnabled: settings.achievementsEnabled,
    socialAccounts: settings.socialAccounts.length
      ? settings.socialAccounts
      : ["x", "mastodon", "linkedin", "bluesky"].map((provider, index) => ({
          provider,
          handleOrUrl: "",
          position: index + 1,
        })),
  };
}

type FormState = ReturnType<typeof settingsToForm>;

type FieldErrors = Partial<Record<keyof FormState | "avatar", string>>;

type PersonalProfileSettingsFormProps = {
  initialSettings: PersonalProfileSettings;
};

export function PersonalProfileSettingsForm({
  initialSettings,
}: PersonalProfileSettingsFormProps) {
  const [settings, setSettings] = useState(initialSettings);
  const [form, setForm] = useState<FormState>(() =>
    settingsToForm(initialSettings),
  );
  const [savedForm, setSavedForm] = useState<FormState>(() =>
    settingsToForm(initialSettings),
  );
  const [errors, setErrors] = useState<FieldErrors>({});
  const [toast, setToast] = useState<string | null>(null);
  const [savingProfile, setSavingProfile] = useState(false);
  const [savingPrivacy, setSavingPrivacy] = useState(false);
  const [savingAvatar, setSavingAvatar] = useState(false);
  const [avatarPreview, setAvatarPreview] = useState<string | null>(
    settings.avatar?.url ?? null,
  );
  const fileInputRef = useRef<HTMLInputElement>(null);

  const profileDirty = useMemo(
    () =>
      JSON.stringify(profilePayload(form)) !==
      JSON.stringify(profilePayload(savedForm)),
    [form, savedForm],
  );
  const privacyDirty = useMemo(
    () =>
      JSON.stringify(privacyPayload(form)) !==
      JSON.stringify(privacyPayload(savedForm)),
    [form, savedForm],
  );

  function patchForm(patch: Partial<FormState>) {
    setForm((current) => ({ ...current, ...patch }));
    setErrors((current) => ({
      ...current,
      ...Object.fromEntries(Object.keys(patch).map((key) => [key, undefined])),
    }));
  }

  function updateSocial(index: number, patch: Partial<UserSocialAccount>) {
    const socialAccounts = form.socialAccounts.map((account, accountIndex) =>
      accountIndex === index ? { ...account, ...patch } : account,
    );
    patchForm({ socialAccounts });
  }

  async function saveProfile() {
    const nextErrors = validateProfile(form);
    setErrors(nextErrors);
    if (Object.keys(nextErrors).length) return;
    setSavingProfile(true);
    try {
      const response = await fetch("/settings/profile/actions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(profilePayload(form)),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(body?.error?.message ?? "Profile could not be saved");
      }
      applySettings(body as PersonalProfileSettings);
      setToast("Public profile updated");
    } catch (error) {
      setErrors({
        displayName:
          error instanceof Error ? error.message : "Profile could not be saved",
      });
    } finally {
      setSavingProfile(false);
    }
  }

  async function savePrivacy(nextForm = form) {
    setSavingPrivacy(true);
    try {
      const response = await fetch("/settings/profile/actions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(privacyPayload(nextForm)),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Privacy settings could not be saved",
        );
      }
      applySettings(body as PersonalProfileSettings);
      setToast("Profile privacy updated");
    } catch (error) {
      setErrors({
        privateProfile:
          error instanceof Error
            ? error.message
            : "Privacy settings could not be saved",
      });
    } finally {
      setSavingPrivacy(false);
    }
  }

  async function handleAvatarFile(file: File | undefined) {
    if (!file) return;
    if (!AVATAR_TYPES.includes(file.type)) {
      setErrors({ avatar: "Avatar must be a PNG, JPEG, WebP, or GIF image." });
      return;
    }
    if (file.size > MAX_AVATAR_BYTES) {
      setErrors({ avatar: "Avatar must be smaller than 2 MB." });
      return;
    }
    const previewUrl = await fileToDataUrl(file);
    setAvatarPreview(previewUrl);
    setSavingAvatar(true);
    setErrors((current) => ({ ...current, avatar: undefined }));
    try {
      const response = await fetch("/settings/profile/avatar", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          action: "upload",
          fileName: file.name,
          contentType: file.type,
          byteSize: file.size,
          previewUrl,
        }),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok)
        throw new Error(body?.error?.message ?? "Avatar could not be saved");
      applySettings(body as PersonalProfileSettings);
      setAvatarPreview(
        (body as PersonalProfileSettings).avatar?.url ?? previewUrl,
      );
      setToast("Profile picture updated");
    } catch (error) {
      setErrors({
        avatar:
          error instanceof Error ? error.message : "Avatar could not be saved",
      });
    } finally {
      setSavingAvatar(false);
    }
  }

  async function removeAvatar() {
    setSavingAvatar(true);
    try {
      const response = await fetch("/settings/profile/avatar", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ action: "remove" }),
      });
      const body = await response.json().catch(() => null);
      if (!response.ok)
        throw new Error(body?.error?.message ?? "Avatar could not be removed");
      applySettings(body as PersonalProfileSettings);
      setAvatarPreview(null);
      setToast("Profile picture removed");
    } catch (error) {
      setErrors({
        avatar:
          error instanceof Error
            ? error.message
            : "Avatar could not be removed",
      });
    } finally {
      setSavingAvatar(false);
    }
  }

  function resetAvatarPreview() {
    setAvatarPreview(settings.avatar?.url ?? null);
    setErrors((current) => ({ ...current, avatar: undefined }));
    setToast("Profile picture reset");
  }

  function applySettings(next: PersonalProfileSettings) {
    const nextForm = settingsToForm(next);
    setSettings(next);
    setForm(nextForm);
    setSavedForm(nextForm);
  }

  const avatarLabel = form.displayName || settings.login;

  return (
    <div className="grid gap-6">
      {toast ? (
        <div className="chip ok w-fit" role="status">
          {toast}
        </div>
      ) : null}

      <section className="card p-5" aria-labelledby="avatar-heading">
        <div className="flex flex-col gap-5 md:flex-row md:items-start md:justify-between">
          <div>
            <p className="t-label">Profile picture</p>
            <h3 className="t-h2 mt-2" id="avatar-heading">
              Avatar
            </h3>
            <p className="t-sm mt-2 max-w-xl" style={{ color: "var(--ink-3)" }}>
              Upload a square public avatar. Images are validated before saving
              and represented as S3 avatar objects in the API contract.
            </p>
            {errors.avatar ? (
              <p className="t-sm mt-2" style={{ color: "var(--err)" }}>
                {errors.avatar}
              </p>
            ) : null}
          </div>
          <div className="flex items-center gap-4">
            <div
              className="av xl overflow-hidden bg-[length:cover] bg-center"
              role="img"
              aria-label={`${avatarLabel} avatar`}
              style={
                avatarPreview
                  ? { backgroundImage: `url(${avatarPreview})` }
                  : undefined
              }
            >
              {avatarPreview ? "" : avatarLabel.slice(0, 2).toUpperCase()}
            </div>
            <div className="grid gap-2">
              <input
                accept={AVATAR_TYPES.join(",")}
                className="sr-only"
                onChange={(event) => handleAvatarFile(event.target.files?.[0])}
                ref={fileInputRef}
                type="file"
              />
              <button
                className="btn primary sm"
                disabled={savingAvatar}
                onClick={() => fileInputRef.current?.click()}
                type="button"
              >
                {savingAvatar ? "Saving…" : "Upload image"}
              </button>
              <button
                className="btn sm"
                disabled={savingAvatar}
                onClick={resetAvatarPreview}
                type="button"
              >
                Reset preview
              </button>
              <button
                className="btn ghost sm"
                disabled={savingAvatar || !avatarPreview}
                onClick={removeAvatar}
                type="button"
              >
                Remove
              </button>
            </div>
          </div>
        </div>
      </section>

      <section className="card p-5" aria-labelledby="public-profile-heading">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label">Identity</p>
            <h3 className="t-h2 mt-2" id="public-profile-heading">
              Public profile
            </h3>
          </div>
          <button
            className="btn primary sm"
            disabled={!profileDirty || savingProfile}
            onClick={saveProfile}
            type="button"
          >
            {savingProfile ? "Updating…" : "Update profile"}
          </button>
        </div>

        <div className="mt-5 grid gap-4 md:grid-cols-2">
          <FieldError label="Name" error={errors.displayName}>
            <input
              className="input w-full"
              name="displayName"
              onChange={(event) =>
                patchForm({ displayName: event.target.value })
              }
              value={form.displayName}
            />
          </FieldError>
          <FieldError label="Public email" error={errors.publicEmailId}>
            <select
              className="input w-full"
              name="publicEmailId"
              onChange={(event) =>
                patchForm({ publicEmailId: event.target.value })
              }
              value={form.publicEmailId}
            >
              <option value="">Do not show my email</option>
              {settings.emails.map((email) => (
                <option key={email.id} value={email.id}>
                  {email.email}
                  {email.verified ? " · verified" : ""}
                </option>
              ))}
            </select>
          </FieldError>
          <FieldError className="md:col-span-2" label="Bio" error={errors.bio}>
            <textarea
              className="input min-h-28 w-full"
              maxLength={280}
              name="bio"
              onChange={(event) => patchForm({ bio: event.target.value })}
              value={form.bio}
            />
          </FieldError>
          <FieldError label="Pronouns" error={errors.pronouns}>
            <div className="grid gap-2 sm:grid-cols-2">
              <select
                className="input w-full"
                onChange={(event) =>
                  patchForm({ pronouns: event.target.value })
                }
                value={form.pronouns}
              >
                {PRONOUN_OPTIONS.map((option) => (
                  <option key={option || "none"} value={option}>
                    {option || "Prefer not to say"}
                  </option>
                ))}
              </select>
              {form.pronouns === "custom" ? (
                <input
                  className="input w-full"
                  aria-label="Custom pronouns"
                  onChange={(event) =>
                    patchForm({ customPronouns: event.target.value })
                  }
                  placeholder="Custom"
                  value={form.customPronouns}
                />
              ) : null}
            </div>
          </FieldError>
          <FieldError label="URL" error={errors.websiteUrl}>
            <input
              className="input w-full"
              inputMode="url"
              name="websiteUrl"
              onChange={(event) =>
                patchForm({ websiteUrl: event.target.value })
              }
              placeholder="https://example.com"
              value={form.websiteUrl}
            />
          </FieldError>
          <FieldError label="Company" error={errors.company}>
            <input
              className="input w-full"
              name="company"
              onChange={(event) => patchForm({ company: event.target.value })}
              value={form.company}
            />
          </FieldError>
          <FieldError label="Location" error={errors.location}>
            <input
              className="input w-full"
              name="location"
              onChange={(event) => patchForm({ location: event.target.value })}
              value={form.location}
            />
          </FieldError>
          <label className="flex items-center gap-2 t-sm">
            <input
              checked={form.displayLocalTime}
              onChange={(event) =>
                patchForm({ displayLocalTime: event.target.checked })
              }
              type="checkbox"
            />
            Display current local time
          </label>
          <FieldError label="Time zone" error={errors.timeZone}>
            <select
              className="input w-full"
              name="timeZone"
              onChange={(event) => patchForm({ timeZone: event.target.value })}
              value={form.timeZone}
            >
              {TIME_ZONES.map((zone) => (
                <option key={zone} value={zone}>
                  {zone}
                </option>
              ))}
            </select>
          </FieldError>
          <FieldError
            label="Preferred language"
            error={errors.preferredLanguage}
          >
            <select
              className="input w-full"
              name="preferredLanguage"
              onChange={(event) =>
                patchForm({ preferredLanguage: event.target.value })
              }
              value={form.preferredLanguage}
            >
              {LANGUAGES.map((language) => (
                <option key={language} value={language}>
                  {language}
                </option>
              ))}
            </select>
          </FieldError>
        </div>

        <div className="mt-6">
          <p className="t-label">Social accounts</p>
          <div className="mt-3 grid gap-3 md:grid-cols-2">
            {form.socialAccounts.map((account, index) => (
              <label className="block" key={account.position}>
                <span className="t-sm flex items-center gap-2 font-medium">
                  <span className="chip soft">
                    {providerIcon(account.provider)}
                  </span>
                  {providerLabel(account.provider)}
                </span>
                <input
                  className="input mt-2 w-full"
                  onChange={(event) =>
                    updateSocial(index, { handleOrUrl: event.target.value })
                  }
                  placeholder="Handle or profile URL"
                  value={account.handleOrUrl}
                />
              </label>
            ))}
          </div>
        </div>
      </section>

      <section className="card p-5" aria-labelledby="privacy-heading">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label">Contributions & activity</p>
            <h3 className="t-h2 mt-2" id="privacy-heading">
              Profile activity privacy
            </h3>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Privacy changes save in place and are recorded as security audit
              events.
            </p>
          </div>
          <button
            className="btn primary sm"
            disabled={!privacyDirty || savingPrivacy}
            onClick={() => savePrivacy()}
            type="button"
          >
            {savingPrivacy ? "Saving…" : "Save privacy"}
          </button>
        </div>
        {errors.privateProfile ? (
          <p className="t-sm mt-3" style={{ color: "var(--err)" }}>
            {errors.privateProfile}
          </p>
        ) : null}
        <div className="mt-5 grid gap-3">
          <ToggleRow
            checked={form.privateProfile}
            description="Hide profile metadata and activity from anonymous viewers."
            label="Make my profile private"
            onChange={(checked) => patchForm({ privateProfile: checked })}
          />
          <ToggleRow
            checked={form.showPrivateContributionCount}
            description="Include private contribution counts in public contribution totals."
            label="Show private contribution counts"
            onChange={(checked) =>
              patchForm({ showPrivateContributionCount: checked })
            }
          />
          <ToggleRow
            checked={form.achievementsEnabled}
            description="Display earned achievements on your public profile."
            label="Show achievements on my profile"
            onChange={(checked) => patchForm({ achievementsEnabled: checked })}
          />
        </div>
      </section>
    </div>
  );
}

function FieldError({
  children,
  className = "",
  error,
  label,
}: {
  children: React.ReactNode;
  className?: string;
  error?: string;
  label: string;
}) {
  return (
    // biome-ignore lint/a11y/noLabelWithoutControl: children always include the section control.
    <label className={`block ${className}`}>
      <span className="t-sm font-medium">{label}</span>
      <span className="mt-2 block">{children}</span>
      {error ? (
        <span className="t-xs mt-1 block" style={{ color: "var(--err)" }}>
          {error}
        </span>
      ) : null}
    </label>
  );
}

function ToggleRow({
  checked,
  description,
  label,
  onChange,
}: {
  checked: boolean;
  description: string;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="list-row flex cursor-pointer items-center justify-between gap-4 py-3">
      <span>
        <span className="t-sm block font-medium">{label}</span>
        <span className="t-xs mt-1 block">{description}</span>
      </span>
      <input
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        type="checkbox"
      />
    </label>
  );
}

function profilePayload(form: FormState): UpdatePersonalProfileSettingsRequest {
  return {
    displayName: form.displayName,
    publicEmailId: form.publicEmailId || null,
    bio: form.bio,
    pronouns: form.pronouns === "custom" ? form.customPronouns : form.pronouns,
    websiteUrl: form.websiteUrl,
    company: form.company,
    location: form.location,
    displayLocalTime: form.displayLocalTime,
    timeZone: form.timeZone,
    preferredLanguage: form.preferredLanguage,
    socialAccounts: form.socialAccounts,
  };
}

function privacyPayload(form: FormState): UpdatePersonalProfileSettingsRequest {
  return {
    privateProfile: form.privateProfile,
    showPrivateContributionCount: form.showPrivateContributionCount,
    achievementsEnabled: form.achievementsEnabled,
  };
}

function validateProfile(form: FormState): FieldErrors {
  const errors: FieldErrors = {};
  if (form.displayName.length > 80)
    errors.displayName = "Name must be 80 characters or fewer.";
  if (form.bio.length > 280)
    errors.bio = "Bio must be 280 characters or fewer.";
  const pronouns =
    form.pronouns === "custom" ? form.customPronouns : form.pronouns;
  if (pronouns.length > 40)
    errors.pronouns = "Pronouns must be 40 characters or fewer.";
  if (form.websiteUrl && !/^https?:\/\//.test(form.websiteUrl))
    errors.websiteUrl = "URL must start with http:// or https://.";
  if (!form.timeZone) errors.timeZone = "Choose a time zone.";
  return errors;
}

function fileToDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("File could not be read"));
    reader.readAsDataURL(file);
  });
}

function providerIcon(provider: string) {
  return provider.slice(0, 1).toUpperCase();
}

function providerLabel(provider: string) {
  return provider.replace(
    /(^|-)(\w)/g,
    (_match, separator: string, letter: string) =>
      `${separator ? " " : ""}${letter.toUpperCase()}`,
  );
}
