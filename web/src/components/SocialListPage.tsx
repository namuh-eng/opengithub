import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import type {
  AuthSession,
  ProfileSocialList,
  RepositoryStargazerList,
} from "@/lib/api";

type SocialListPageProps = {
  session: AuthSession;
  title: string;
  eyebrow: string;
  empty: string;
  list: ProfileSocialList | RepositoryStargazerList | null;
  backHref: string;
  backLabel: string;
};

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
    year: "numeric",
  }).format(date);
}

function initials(login: string) {
  return login.slice(0, 2).toUpperCase() || "OG";
}

function items(list: ProfileSocialList | RepositoryStargazerList | null) {
  if (!list) return [];
  return list.items.map((item) => ({
    ...item,
    date:
      "starredAt" in item
        ? `Starred ${formatDate(item.starredAt)}`
        : `Followed ${formatDate(item.followedAt)}`,
  }));
}

export function SocialListPage({
  backHref,
  backLabel,
  empty,
  eyebrow,
  list,
  session,
  title,
}: SocialListPageProps) {
  const rows = items(list);
  const total = list?.total ?? 0;

  return (
    <AppShell session={session}>
      <AppShellFrame>
        <main className="mx-auto grid max-w-5xl gap-6">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                {eyebrow}
              </p>
              <h1 className="t-h1 mt-2">{title}</h1>
              <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
                {total.toLocaleString()} social connections
              </p>
            </div>
            <Link className="btn" href={backHref}>
              {backLabel}
            </Link>
          </header>

          <section className="card overflow-hidden" aria-label={title}>
            {rows.length ? (
              rows.map((item) => (
                <article className="list-row px-5 py-4" key={item.id}>
                  <Link className="av" href={item.href}>
                    {initials(item.login)}
                  </Link>
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <Link className="t-h3 no-underline" href={item.href}>
                        {item.login}
                      </Link>
                      {item.name ? (
                        <span
                          className="t-sm"
                          style={{ color: "var(--ink-3)" }}
                        >
                          {item.name}
                        </span>
                      ) : null}
                    </div>
                    {item.bio ? (
                      <p
                        className="t-sm mt-1"
                        style={{ color: "var(--ink-3)" }}
                      >
                        {item.bio}
                      </p>
                    ) : null}
                  </div>
                  <span className="t-mono-sm shrink-0">{item.date}</span>
                </article>
              ))
            ) : (
              <div className="p-6">
                <p className="t-body" style={{ color: "var(--ink-3)" }}>
                  {empty}
                </p>
              </div>
            )}
          </section>
        </main>
      </AppShellFrame>
    </AppShell>
  );
}
