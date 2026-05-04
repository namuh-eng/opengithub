import Link from "next/link";
import type { ReactNode } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type { RepositoryOverview } from "@/lib/api";
import {
  REPOSITORY_INSIGHTS_NAV_ITEMS,
  repositoryInsightsHref,
} from "@/lib/navigation";

type RepositoryInsightsShellProps = {
  repository: RepositoryOverview;
  activeSection: string;
  children: ReactNode;
};

export function RepositoryInsightsShell({
  repository,
  activeSection,
  children,
}: RepositoryInsightsShellProps) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/pulse`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="grid gap-6 lg:grid-cols-[260px_minmax(0,1fr)]">
        <aside aria-label="Insights navigation" className="min-w-0">
          <div className="card p-2">
            <p className="t-label px-3 py-2" style={{ color: "var(--ink-3)" }}>
              Insights
            </p>
            <nav className="grid gap-1">
              {REPOSITORY_INSIGHTS_NAV_ITEMS.map((item) => {
                const active = activeSection === item.section;
                return (
                  <Link
                    aria-label={`${item.label} ${item.description}`}
                    aria-current={active ? "page" : undefined}
                    className={`rounded-md px-3 py-2 t-sm ${
                      active ? "font-semibold" : ""
                    }`}
                    href={repositoryInsightsHref(
                      repository.owner_login,
                      repository.name,
                      item,
                    )}
                    key={item.section}
                    style={{
                      background: active ? "var(--accent-soft)" : "transparent",
                      color: active ? "var(--ink-1)" : "var(--ink-3)",
                    }}
                  >
                    <span className="block">{item.label}</span>
                    <span className="t-xs mt-1 block">{item.description}</span>
                  </Link>
                );
              })}
            </nav>
          </div>
        </aside>
        <div className="min-w-0">{children}</div>
      </div>
    </RepositoryShell>
  );
}
