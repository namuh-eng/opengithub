import Link from "next/link";
import type { ReactNode } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type { RepositoryOverview } from "@/lib/api";

type RepositorySecurityShellProps = {
  repository: RepositoryOverview;
  activeSection: string;
  children: ReactNode;
};

const SECURITY_NAV_GROUPS = [
  {
    label: "Overview",
    items: [
      {
        section: "overview",
        label: "Overview",
        description: "Policy, feature state, and advisories",
        hrefSuffix: "/security",
      },
    ],
  },
  {
    label: "Findings",
    items: [
      {
        section: "dependabot",
        label: "Dependabot",
        description: "Dependency alerts and updates",
        hrefSuffix: "/security/dependabot",
      },
      {
        section: "code-scanning",
        label: "Code scanning",
        description: "Static analysis findings",
        hrefSuffix: "/security/code-scanning",
      },
      {
        section: "secret-scanning",
        label: "Secret scanning",
        description: "Credential exposure findings",
        hrefSuffix: "/security/secret-scanning",
      },
    ],
  },
  {
    label: "Reporting",
    items: [
      {
        section: "policy",
        label: "Security policy",
        description: "Responsible disclosure guidance",
        hrefSuffix: "/security/policy",
      },
      {
        section: "advisories",
        label: "Advisories",
        description: "Published security advisories",
        hrefSuffix: "/security/advisories",
      },
    ],
  },
] as const;

function repositoryBase(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}`;
}

export function RepositorySecurityShell({
  repository,
  activeSection,
  children,
}: RepositorySecurityShellProps) {
  const base = repositoryBase(repository);

  return (
    <RepositoryShell
      activePath={`${base}/security`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="grid gap-6 lg:grid-cols-[260px_minmax(0,1fr)]">
        <aside aria-label="Security and quality navigation" className="min-w-0">
          <div className="card p-2">
            <p className="t-label px-3 py-2" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <nav className="grid gap-3">
              {SECURITY_NAV_GROUPS.map((group) => (
                <div className="grid gap-1" key={group.label}>
                  <p
                    className="t-xs px-3 pt-1"
                    style={{ color: "var(--ink-4)" }}
                  >
                    {group.label}
                  </p>
                  {group.items.map((item) => {
                    const active = activeSection === item.section;
                    return (
                      <Link
                        aria-current={active ? "page" : undefined}
                        aria-label={`${item.label} ${item.description}`}
                        className={`rounded-md px-3 py-2 t-sm ${
                          active ? "font-semibold" : ""
                        }`}
                        href={`${base}${item.hrefSuffix}`}
                        key={item.section}
                        style={{
                          background: active
                            ? "var(--accent-soft)"
                            : "transparent",
                          color: active ? "var(--ink-1)" : "var(--ink-3)",
                        }}
                      >
                        <span className="block">{item.label}</span>
                        <span className="t-xs mt-1 block">
                          {item.description}
                        </span>
                      </Link>
                    );
                  })}
                </div>
              ))}
            </nav>
          </div>
        </aside>
        <div className="min-w-0">{children}</div>
      </div>
    </RepositoryShell>
  );
}
