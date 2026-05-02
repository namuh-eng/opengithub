import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  AppShellContext,
  AuthSession,
  OrganizationMemberPreview,
  OrganizationOverview,
  OrganizationRepositoryPreview,
} from "@/lib/api";
import {
  activeOrganizationTab,
  ORGANIZATION_TABS,
  organizationTabHref,
} from "@/lib/navigation";

export type OrganizationOverviewPageProps = {
  activeTab?: string;
  organization: OrganizationOverview;
  session: AuthSession;
  shellContext?: AppShellContext | null;
};

function avatarInitial(label: string) {
  return label.trim().slice(0, 1).toUpperCase() || "O";
}

function compactCount(value: number) {
  return new Intl.NumberFormat("en", { notation: "compact" }).format(value);
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function RepositoryCard({
  repository,
}: {
  repository: OrganizationRepositoryPreview;
}) {
  return (
    <Link
      aria-label={`${repository.name} repository`}
      className="card block p-4 transition hover:-translate-y-0.5 hover:shadow-sm focus:outline-none focus:ring-2 focus:ring-[color:var(--accent-soft)]"
      href={repository.href}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="t-h3 truncate">{repository.name}</p>
          <p className="t-xs mt-1">
            Updated {formatDate(repository.updatedAt)}
          </p>
        </div>
        <span className="chip soft">{repository.visibility}</span>
      </div>
      {repository.description ? (
        <p className="t-sm mt-3 line-clamp-2" style={{ color: "var(--ink-2)" }}>
          {repository.description}
        </p>
      ) : null}
      <div className="mt-4 flex flex-wrap items-center gap-3 t-xs">
        {repository.primaryLanguage ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="dot live" aria-hidden="true" />
            {repository.primaryLanguage.language}
          </span>
        ) : null}
        <span>{compactCount(repository.starsCount)} stars</span>
        <span>{compactCount(repository.forksCount)} forks</span>
      </div>
      {repository.topics.length > 0 ? (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {repository.topics.slice(0, 3).map((topic) => (
            <span className="chip" key={topic}>
              {topic}
            </span>
          ))}
        </div>
      ) : null}
    </Link>
  );
}

function RepositoryRow({
  repository,
}: {
  repository: OrganizationRepositoryPreview;
}) {
  return (
    <Link className="list-row block py-4" href={repository.href}>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="t-h3">{repository.name}</span>
            <span className="chip soft">{repository.visibility}</span>
            {repository.isPinned ? (
              <span className="chip accent">Pinned</span>
            ) : null}
          </div>
          {repository.description ? (
            <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
              {repository.description}
            </p>
          ) : null}
          <div className="mt-2 flex flex-wrap gap-3 t-xs">
            {repository.primaryLanguage ? (
              <span>{repository.primaryLanguage.language}</span>
            ) : null}
            <span>{repository.openIssuesCount} open issues</span>
            <span>{repository.openPullRequestsCount} pull requests</span>
            <span>Updated {formatDate(repository.updatedAt)}</span>
          </div>
        </div>
        <div className="flex gap-2 t-xs">
          <span>{compactCount(repository.starsCount)} stars</span>
          <span>{compactCount(repository.forksCount)} forks</span>
        </div>
      </div>
    </Link>
  );
}

function MemberPill({ member }: { member: OrganizationMemberPreview }) {
  return (
    <Link
      className="flex items-center gap-2 rounded-[var(--radius)] p-2 hover:bg-[color:var(--surface-2)]"
      href={member.href}
    >
      <span className="av sm" aria-hidden="true">
        {avatarInitial(member.displayName ?? member.login)}
      </span>
      <span className="min-w-0">
        <span className="t-sm block truncate">
          {member.displayName ?? member.login}
        </span>
        <span className="t-xs block truncate">{member.role}</span>
      </span>
    </Link>
  );
}

function EmptyCard({ title, body }: { title: string; body: string }) {
  return (
    <div className="card p-5">
      <p className="t-h3">{title}</p>
      <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
        {body}
      </p>
    </div>
  );
}

export function OrganizationOverviewPage({
  activeTab = "overview",
  organization,
  session,
  shellContext,
}: OrganizationOverviewPageProps) {
  const currentTab = activeOrganizationTab(activeTab);
  const pinnedRepositories = organization.pinnedRepositories;
  const repositoryPreview = organization.repositories;

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame className="max-w-[1240px]" mode="centered">
        <section className="grid gap-6">
          <header className="card overflow-hidden">
            <div className="grid gap-6 p-6 md:grid-cols-[1fr_auto]">
              <div className="flex min-w-0 gap-4">
                <span className="av xl" aria-hidden="true">
                  {avatarInitial(organization.displayName || organization.slug)}
                </span>
                <div className="min-w-0">
                  <p className="t-label">Organization</p>
                  <div className="mt-1 flex flex-wrap items-center gap-2">
                    <h1 className="t-h1">{organization.displayName}</h1>
                    {organization.verifiedDomain ? (
                      <Link
                        className="chip ok"
                        href={organization.verifiedDomain.href}
                        title={`Verified domain: ${organization.verifiedDomain.domain}`}
                      >
                        Verified
                      </Link>
                    ) : null}
                  </div>
                  <p
                    className="t-mono-sm mt-1"
                    style={{ color: "var(--ink-3)" }}
                  >
                    @{organization.slug}
                  </p>
                  {organization.description ? (
                    <p
                      className="t-body mt-3 max-w-2xl"
                      style={{ color: "var(--ink-2)" }}
                    >
                      {organization.description}
                    </p>
                  ) : null}
                  <div className="mt-4 flex flex-wrap gap-2">
                    {organization.websiteUrl ? (
                      <a className="chip" href={organization.websiteUrl}>
                        {organization.websiteUrl.replace(/^https?:\/\//, "")}
                      </a>
                    ) : null}
                    <span className="chip soft">
                      {compactCount(organization.followerCount)} followers
                    </span>
                    <Link className="chip soft" href={organization.peopleHref}>
                      {compactCount(organization.memberCount)} people
                    </Link>
                    <Link
                      className="chip soft"
                      href={organization.repositoriesHref}
                    >
                      {compactCount(organization.repositoryCount)} repositories
                    </Link>
                  </div>
                </div>
              </div>
              <div className="flex flex-wrap content-start gap-2 md:justify-end">
                {organization.sponsorship.enabled &&
                organization.sponsorship.sponsorHref ? (
                  <Link
                    className="btn accent"
                    href={organization.sponsorship.sponsorHref}
                  >
                    Sponsor
                  </Link>
                ) : (
                  <button
                    className="btn ghost"
                    disabled
                    type="button"
                    title={organization.sponsorship.note}
                  >
                    Sponsor unavailable
                  </button>
                )}
                <Link className="btn ghost" href={organization.projectsHref}>
                  Projects
                </Link>
                {organization.settingsHref ? (
                  <Link
                    className="btn primary"
                    href={organization.settingsHref}
                  >
                    Settings
                  </Link>
                ) : null}
              </div>
            </div>
            <div className="border-t border-[color:var(--line)] px-6 py-3">
              <QueryTabNavigation
                activeValue={currentTab}
                ariaLabel="Organization sections"
                hrefForTab={(value) =>
                  organizationTabHref(organization.slug, value)
                }
                tabs={ORGANIZATION_TABS}
              />
            </div>
          </header>

          <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
            <main className="grid gap-6">
              <section
                className="grid gap-3"
                aria-labelledby="pinned-repositories-heading"
              >
                <div className="flex items-end justify-between gap-3">
                  <div>
                    <p className="t-label">Pinned</p>
                    <h2 className="t-h2" id="pinned-repositories-heading">
                      Pinned repositories
                    </h2>
                  </div>
                  <Link
                    className="btn sm ghost"
                    href={organization.repositoriesHref}
                  >
                    View repositories
                  </Link>
                </div>
                {pinnedRepositories.length > 0 ? (
                  <div className="grid gap-3 md:grid-cols-2">
                    {pinnedRepositories.map((repository) => (
                      <RepositoryCard
                        key={repository.id}
                        repository={repository}
                      />
                    ))}
                  </div>
                ) : (
                  <EmptyCard
                    title="No pinned repositories yet"
                    body="Pinned repositories will appear here once organization owners select them."
                  />
                )}
              </section>

              <section
                className="card p-5"
                aria-labelledby="repository-preview-heading"
              >
                <div className="flex items-end justify-between gap-3">
                  <div>
                    <p className="t-label">Repository preview</p>
                    <h2 className="t-h2" id="repository-preview-heading">
                      Active public work
                    </h2>
                  </div>
                  <span className="chip soft">
                    {organization.repositoryCount} visible
                  </span>
                </div>
                <div className="mt-4 divide-y divide-[color:var(--line)]">
                  {repositoryPreview.length > 0 ? (
                    repositoryPreview.map((repository) => (
                      <RepositoryRow
                        key={repository.id}
                        repository={repository}
                      />
                    ))
                  ) : (
                    <p className="t-sm" style={{ color: "var(--ink-2)" }}>
                      No repositories are visible to this viewer.
                    </p>
                  )}
                </div>
              </section>
            </main>

            <aside className="grid content-start gap-4">
              <section className="card p-5" aria-labelledby="people-heading">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="t-label">People</p>
                    <h2 className="t-h3" id="people-heading">
                      Members
                    </h2>
                  </div>
                  <Link className="btn sm ghost" href={organization.peopleHref}>
                    View all
                  </Link>
                </div>
                <div className="mt-3 grid gap-1">
                  {organization.members.map((member) => (
                    <MemberPill key={member.id} member={member} />
                  ))}
                </div>
              </section>

              <section className="card p-5" aria-labelledby="languages-heading">
                <p className="t-label">Languages</p>
                <h2 className="t-h3 mt-1" id="languages-heading">
                  Top languages
                </h2>
                <div className="mt-4 grid gap-3">
                  {organization.languages.map((language) => (
                    <div key={language.language}>
                      <div className="flex justify-between gap-3 t-xs">
                        <span>{language.language}</span>
                        <span>{language.percentage}%</span>
                      </div>
                      <div className="mt-1 h-1.5 rounded-[var(--radius-pill)] bg-[color:var(--surface-3)]">
                        <div
                          className="h-1.5 rounded-[var(--radius-pill)] bg-[color:var(--accent)]"
                          style={{
                            width: `${Math.max(language.percentage, 4)}%`,
                          }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </section>

              <section className="card p-5" aria-labelledby="topics-heading">
                <p className="t-label">Topics</p>
                <h2 className="t-h3 mt-1" id="topics-heading">
                  Most used topics
                </h2>
                <div className="mt-3 flex flex-wrap gap-2">
                  {organization.topics.map((topic) => (
                    <Link className="chip" href={topic.href} key={topic.topic}>
                      {topic.topic} · {topic.repositoryCount}
                    </Link>
                  ))}
                </div>
              </section>
            </aside>
          </div>
        </section>
      </AppShellFrame>
    </AppShell>
  );
}
