import type { OrganizationProfileSettings } from "@/lib/api";

type OrganizationProfileSettingsFormProps = {
  settings: OrganizationProfileSettings;
};

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
  help,
  label,
  name,
  readOnly = true,
  type = "text",
  value,
}: {
  help?: string;
  label: string;
  name: string;
  readOnly?: boolean;
  type?: string;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="t-sm font-semibold">{label}</span>
      <input
        aria-label={label}
        className="input"
        name={name}
        readOnly={readOnly}
        type={type}
        value={value}
      />
      {help ? <span className="t-xs">{help}</span> : null}
    </label>
  );
}

function TextArea({
  help,
  label,
  name,
  value,
}: {
  help?: string;
  label: string;
  name: string;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="t-sm font-semibold">{label}</span>
      <textarea
        aria-label={label}
        className="input min-h-28 resize-y"
        name={name}
        readOnly
        value={value}
      />
      {help ? <span className="t-xs">{help}</span> : null}
    </label>
  );
}

function DisabledSave({ label }: { label: string }) {
  return (
    <button className="btn sm" disabled type="button">
      {label}
    </button>
  );
}

export function OrganizationProfileSettingsForm({
  settings,
}: OrganizationProfileSettingsFormProps) {
  const profile = settings.profile;
  const avatarLabel = settings.organization.name || settings.organization.slug;

  return (
    <div className="grid gap-6">
      <div className="chip soft w-fit" role="status">
        Owner access confirmed
      </div>

      <SettingsCard kicker="Identity" title="Profile picture">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex min-w-0 items-center gap-4">
            {settings.avatar.avatarUrl ? (
              <span
                aria-label={`${avatarLabel} avatar`}
                className="av lg shrink-0"
                role="img"
                style={{
                  backgroundImage: `url(${settings.avatar.avatarUrl})`,
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
                {settings.avatar.unavailableReason ??
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
        <div className="grid gap-4">
          <Field
            help="Shown on the organization profile and repository owner headers."
            label="Organization display name"
            name="displayName"
            value={profile.displayName}
          />
          <TextArea
            help="A short public description for people visiting the organization."
            label="Description"
            name="description"
            value={valueOrEmpty(profile.description)}
          />
          <div className="grid gap-4 md:grid-cols-2">
            <Field
              label="URL"
              name="websiteUrl"
              type="url"
              value={valueOrEmpty(profile.websiteUrl)}
            />
            <Field
              label="Location"
              name="location"
              value={valueOrEmpty(profile.location)}
            />
            <Field
              label="Public email"
              name="publicEmail"
              type="email"
              value={valueOrEmpty(profile.publicEmail)}
            />
            <Field
              label="Company"
              name="companyName"
              value={valueOrEmpty(profile.companyName)}
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
          <DisabledSave label="Save profile changes" />
        </div>
      </SettingsCard>

      <SettingsCard kicker="Contact" title="Administrative contact">
        <div className="grid gap-4 md:grid-cols-2">
          <Field
            help="Used for organization administration and policy notices."
            label="Contact email"
            name="contactEmail"
            type="email"
            value={valueOrEmpty(profile.contactEmail)}
          />
          <Field
            help="Billing pages are outside this clone's current scope."
            label="Billing email"
            name="billingEmail"
            type="email"
            value={valueOrEmpty(profile.billingEmail)}
          />
        </div>
        <div className="mt-4">
          <DisabledSave label="Save contact changes" />
        </div>
      </SettingsCard>

      <SettingsCard kicker="Social" title="Social accounts">
        <div className="grid gap-4 md:grid-cols-2">
          {SOCIAL_PROVIDERS.map((provider) => (
            <Field
              key={provider}
              label={SOCIAL_LABELS[provider]}
              name={`social-${provider}`}
              value={socialValue(settings, provider)}
            />
          ))}
        </div>
        <div className="mt-4">
          <DisabledSave label="Save social accounts" />
        </div>
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
