import Link from "next/link";
import { RepositoryShell } from "@/components/RepositoryShell";
import type { RepositoryOverview } from "@/lib/api";
import {
  REPOSITORY_SETTINGS_NAV_ITEMS,
  repositorySettingsHref,
} from "@/lib/navigation";

const REPOSITORY_SETTINGS_GROUPS = [
  {
    label: "General",
    sections: ["general"],
  },
  {
    label: "Access",
    sections: ["access"],
  },
  {
    label: "Code and automation",
    sections: ["branches", "actions", "hooks", "pages", "tags"],
  },
  {
    label: "Security",
    sections: ["secrets", "security"],
  },
] as const;

function repositorySettingsGroupItems(sections: readonly string[]) {
  return REPOSITORY_SETTINGS_NAV_ITEMS.filter((item) =>
    sections.includes(item.section),
  );
}

type RepositorySettingsShellProps = {
  activeSection: string;
  children: React.ReactNode;
  repository: RepositoryOverview;
  title: string;
};

export function RepositorySettingsShell({
  activeSection,
  children,
  repository,
  title,
}: RepositorySettingsShellProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/settings/${activeSection === "general" ? "" : activeSection}`}
      frameClassName="grid gap-8 lg:grid-cols-[248px_minmax(0,1fr)]"
      repository={repository}
    >
      <aside className="lg:pr-6 lg:[border-right:1px_solid_var(--line)]">
        <div className="mb-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository
          </p>
          <h2 className="t-h2 mt-2">Settings</h2>
        </div>
        <nav
          aria-label="Repository settings navigation"
          className="grid gap-1 t-sm"
        >
          {REPOSITORY_SETTINGS_GROUPS.map((group) => (
            <div className="grid gap-1" key={group.label}>
              <p
                className="t-label px-3 pt-3 first:pt-0"
                style={{ color: "var(--ink-4)" }}
              >
                {group.label}
              </p>
              {repositorySettingsGroupItems(group.sections).map((item) => {
                const active = activeSection === item.section;
                return (
                  <Link
                    aria-current={active ? "page" : undefined}
                    className="rounded-md px-3 py-2 font-medium hover:bg-[var(--hover)]"
                    href={repositorySettingsHref(
                      repository.owner_login,
                      repository.name,
                      item,
                    )}
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
          <div className="grid gap-1">
            <p className="t-label px-3 pt-3" style={{ color: "var(--ink-4)" }}>
              Danger Zone
            </p>
            <Link
              className="rounded-md px-3 py-2 font-medium hover:bg-[var(--hover)]"
              href={`/${repository.owner_login}/${repository.name}/settings#danger-zone`}
              style={{ border: "1px solid transparent", color: "var(--ink-3)" }}
            >
              Destructive actions
            </Link>
          </div>
        </nav>
      </aside>
      <section className="min-w-0">
        <div className="pb-5" style={{ borderBottom: "1px solid var(--line)" }}>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            {repository.owner_login} / {repository.name}
          </p>
          <h1 className="t-h1 mt-2">{title}</h1>
        </div>
        <div className="mt-6">{children}</div>
      </section>
    </RepositoryShell>
  );
}

type RepositorySettingsPlaceholderContentProps = {
  actions?: { href: string; label: string; primary?: boolean }[];
  children?: React.ReactNode;
  message: string;
};

export function RepositorySettingsPlaceholderContent({
  actions = [],
  children,
  message,
}: RepositorySettingsPlaceholderContentProps) {
  return (
    <div className="card p-6">
      <p className="t-body" style={{ color: "var(--ink-2)" }}>
        {message}
      </p>
      {children}
      {actions.length > 0 ? (
        <div className="mt-6 flex flex-wrap gap-2">
          {actions.map((action) => (
            <Link
              className={action.primary ? "btn primary" : "btn"}
              href={action.href}
              key={`${action.href}-${action.label}`}
            >
              {action.label}
            </Link>
          ))}
        </div>
      ) : null}
    </div>
  );
}
