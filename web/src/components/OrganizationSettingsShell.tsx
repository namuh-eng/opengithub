import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import type {
  AppShellContext,
  AuthSession,
  OrganizationProfileSettings,
} from "@/lib/api";
import {
  ORGANIZATION_SETTINGS_NAV_ITEMS,
  organizationHref,
  organizationSettingsSectionHref,
} from "@/lib/navigation";

type OrganizationSettingsShellProps = {
  activeSection: string;
  children: React.ReactNode;
  session: AuthSession;
  settings: OrganizationProfileSettings;
  shellContext?: AppShellContext | null;
  title: string;
};

const GROUP_LABELS = {
  general: "General",
  access: "Access",
  integrations: "Integrations",
  danger: "Danger",
} as const;

function organizationInitial(settings: OrganizationProfileSettings) {
  return (
    settings.organization.name.trim().slice(0, 1) ||
    settings.organization.slug.trim().slice(0, 1) ||
    "O"
  ).toUpperCase();
}

export function OrganizationSettingsShell({
  activeSection,
  children,
  session,
  settings,
  shellContext,
  title,
}: OrganizationSettingsShellProps) {
  const groupedItems = ORGANIZATION_SETTINGS_NAV_ITEMS.reduce(
    (groups, item) => {
      groups[item.group].push(item);
      return groups;
    },
    {
      access: [],
      danger: [],
      general: [],
      integrations: [],
    } as Record<
      (typeof ORGANIZATION_SETTINGS_NAV_ITEMS)[number]["group"],
      (typeof ORGANIZATION_SETTINGS_NAV_ITEMS)[number][]
    >,
  );

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame
        className="grid gap-8 lg:grid-cols-[272px_minmax(0,1fr)]"
        mode="centered"
      >
        <aside className="min-w-0 lg:pr-6 lg:[border-right:1px_solid_var(--line)]">
          <div className="card p-4">
            <div className="flex min-w-0 items-center gap-3">
              {settings.avatar.avatarUrl ? (
                <span
                  aria-hidden="true"
                  className="av sm shrink-0"
                  style={{
                    backgroundImage: `url(${settings.avatar.avatarUrl})`,
                    backgroundPosition: "center",
                    backgroundSize: "cover",
                  }}
                />
              ) : (
                <span className="av sm shrink-0" aria-hidden="true">
                  {organizationInitial(settings)}
                </span>
              )}
              <div className="min-w-0">
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  Organization
                </p>
                <h1 className="t-h3 truncate">{settings.organization.name}</h1>
                <Link
                  className="t-xs break-words hover:underline"
                  href={organizationHref(settings.organization.slug)}
                >
                  @{settings.organization.slug}
                </Link>
              </div>
            </div>
            <div
              className="mt-4 grid gap-2 pt-4"
              style={{ borderTop: "1px solid var(--line-soft)" }}
            >
              <Link className="btn sm" href="/settings/profile">
                Personal settings
              </Link>
              <Link
                className="btn sm primary"
                href={settings.organization.settingsHref}
              >
                Organization settings
              </Link>
            </div>
          </div>

          <nav
            aria-label="Organization settings navigation"
            className="mt-5 grid gap-5 t-sm"
          >
            {Object.entries(groupedItems).map(([group, items]) => (
              <div className="grid gap-1" key={group}>
                <p
                  className="t-label px-3 pb-1"
                  style={{ color: "var(--ink-4)" }}
                >
                  {GROUP_LABELS[group as keyof typeof GROUP_LABELS]}
                </p>
                {items.map((item) => {
                  const active = activeSection === item.section;
                  const href = organizationSettingsSectionHref(
                    settings.organization.slug,
                    item,
                  );
                  if ("disabled" in item && item.disabled) {
                    return (
                      <span
                        aria-disabled="true"
                        className="rounded-md px-3 py-2 font-medium"
                        key={item.section}
                        title={item.description}
                        style={{
                          border: "1px solid transparent",
                          color: "var(--ink-4)",
                        }}
                      >
                        {item.label}
                      </span>
                    );
                  }

                  return (
                    <Link
                      aria-current={active ? "page" : undefined}
                      className="rounded-md px-3 py-2 font-medium hover:bg-[var(--hover)]"
                      href={href}
                      key={item.section}
                      style={
                        active
                          ? {
                              background: "var(--surface-2)",
                              border: "1px solid var(--line)",
                              color: "var(--ink-1)",
                            }
                          : {
                              border: "1px solid transparent",
                              color: "var(--ink-3)",
                            }
                      }
                    >
                      {item.label}
                    </Link>
                  );
                })}
              </div>
            ))}
          </nav>
        </aside>
        <section className="min-w-0">
          <div
            className="pb-5"
            style={{ borderBottom: "1px solid var(--line)" }}
          >
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              {settings.organization.slug} / settings
            </p>
            <h2 className="t-h1 mt-2">{title}</h2>
          </div>
          <div className="mt-6">{children}</div>
        </section>
      </AppShellFrame>
    </AppShell>
  );
}
