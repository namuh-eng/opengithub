import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { OrganizationPeopleAdminPage } from "@/components/OrganizationPeopleAdminPage";
import { OrganizationPeoplePage } from "@/components/OrganizationPeoplePage";
import { OrganizationRepositoriesPage } from "@/components/OrganizationRepositoriesPage";
import { OrganizationTeamsPage } from "@/components/OrganizationTeamsPage";
import { OwnerPackagesPage } from "@/components/OwnerPackagesPage";
import { ProjectsListPage } from "@/components/ProjectsListPage";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  AppShellContext,
  AuthSession,
  OrganizationPeopleAdmin,
  OrganizationPeopleList,
  OrganizationRepositoryList,
  OrganizationTeamsDirectory,
  OwnerPackageList,
  ProjectList,
  PublicOrganizationProfile,
} from "@/lib/api";
import {
  activeOrganizationTab,
  ORGANIZATION_TABS,
  organizationProjectHref,
  organizationSettingsHref,
  organizationTabHref,
  type QueryTab,
} from "@/lib/navigation";

type OrganizationProfilePageProps = {
  activeTab: string;
  adminPeople?: OrganizationPeopleAdmin | null;
  peopleList?: OrganizationPeopleList | null;
  teamsDirectory?: OrganizationTeamsDirectory | null;
  profile: PublicOrganizationProfile;
  packageList?: OwnerPackageList | null;
  projectList?: ProjectList | null;
  repositoryList?: OrganizationRepositoryList | null;
  session: AuthSession;
  shellContext?: AppShellContext | null;
};

function organizationInitial(profile: PublicOrganizationProfile) {
  return (
    profile.identity.name.trim().slice(0, 1) ||
    profile.identity.slug.trim().slice(0, 1) ||
    "O"
  ).toUpperCase();
}

function compactDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function relativeDate(value: string) {
  const date = new Date(value);
  const diffMs = Date.now() - date.getTime();
  const diffDays = Math.max(0, Math.floor(diffMs / 86_400_000));
  if (diffDays < 1) {
    return "Updated today";
  }
  if (diffDays < 31) {
    return `Updated ${diffDays} day${diffDays === 1 ? "" : "s"} ago`;
  }
  return `Updated ${compactDate(value)}`;
}

function formatCount(value: number) {
  return value.toLocaleString();
}

function languagePercent(
  language: PublicOrganizationProfile["topLanguages"][number],
  languages: PublicOrganizationProfile["topLanguages"],
) {
  const total = languages.reduce((sum, item) => sum + item.byteCount, 0);
  if (total <= 0) {
    return 0;
  }
  return Math.max(1, Math.round((language.byteCount / total) * 100));
}

function tabLabel(tab: QueryTab, profile: PublicOrganizationProfile) {
  if (tab.value === "repositories") {
    return `Repositories ${profile.tabCounts.repositories.toLocaleString()}`;
  }
  if (tab.value === "projects") {
    return `Projects ${profile.tabCounts.projects.toLocaleString()}`;
  }
  if (tab.value === "packages") {
    return `Packages ${profile.tabCounts.packages.toLocaleString()}`;
  }
  if (tab.value === "people") {
    return `People ${profile.tabCounts.people.toLocaleString()}`;
  }
  if (tab.value === "teams") {
    return "Teams";
  }
  return tab.label;
}

function organizationTabs(profile: PublicOrganizationProfile): QueryTab[] {
  if (profile.identity.isPrivate) {
    return ORGANIZATION_TABS.filter((tab) => tab.value === "overview").map(
      (tab) => ({
        ...tab,
        label: tabLabel(tab, profile),
      }),
    );
  }

  return ORGANIZATION_TABS.map((tab) => ({
    ...tab,
    label: tabLabel(tab, profile),
  }));
}

function visibleTab(activeTab: string, profile: PublicOrganizationProfile) {
  const tabs = organizationTabs(profile);
  return tabs.some((tab) => tab.value === activeTab)
    ? activeTab
    : tabs[0].value;
}

function Avatar({ profile }: { profile: PublicOrganizationProfile }) {
  if (profile.identity.avatarUrl) {
    return (
      <span
        aria-hidden="true"
        className="av xl shrink-0"
        style={{
          backgroundImage: `url(${profile.identity.avatarUrl})`,
          backgroundPosition: "center",
          backgroundSize: "cover",
        }}
      />
    );
  }

  return (
    <span className="av xl shrink-0" aria-hidden="true">
      {organizationInitial(profile)}
    </span>
  );
}

function RepositoryMeta({
  compact = false,
  repository,
}: {
  compact?: boolean;
  repository: PublicOrganizationProfile["repositoryPreview"][number];
}) {
  const stats = [
    repository.primaryLanguage
      ? {
          label: repository.primaryLanguage.language,
          node: (
            <>
              <span
                aria-hidden="true"
                className="inline-block size-2 rounded-full"
                style={{ background: repository.primaryLanguage.color }}
              />
              <span>{repository.primaryLanguage.language}</span>
            </>
          ),
        }
      : null,
    repository.starsCount > 0
      ? { label: `${formatCount(repository.starsCount)} stars` }
      : null,
    repository.forksCount > 0
      ? { label: `${formatCount(repository.forksCount)} forks` }
      : null,
    repository.openIssuesCount > 0
      ? { label: `${formatCount(repository.openIssuesCount)} open issues` }
      : null,
    repository.openPullRequestsCount > 0
      ? {
          label: `${formatCount(repository.openPullRequestsCount)} open pull requests`,
        }
      : null,
    repository.license ? { label: repository.license.name } : null,
    { label: relativeDate(repository.updatedAt) },
  ].filter(Boolean);

  return (
    <div
      className={`mt-3 flex flex-wrap ${compact ? "gap-x-3 gap-y-1" : "gap-x-4 gap-y-2"}`}
      style={{ color: "var(--ink-3)" }}
    >
      {stats.map((stat) => (
        <span
          className="t-xs inline-flex min-w-0 items-center gap-1.5"
          key={stat?.label}
        >
          {stat?.node ?? stat?.label}
        </span>
      ))}
    </div>
  );
}

function RepositoryBadges({
  repository,
}: {
  repository: PublicOrganizationProfile["repositoryPreview"][number];
}) {
  return (
    <>
      {repository.visibility !== "public" ? (
        <span className="chip soft">{repository.visibility}</span>
      ) : null}
      {repository.isArchived ? (
        <span className="chip warn">Archived</span>
      ) : null}
      {repository.isTemplate ? (
        <span className="chip soft">Template</span>
      ) : null}
      {repository.isMirror ? <span className="chip soft">Mirror</span> : null}
    </>
  );
}

function TopicChips({
  repository,
}: {
  repository: PublicOrganizationProfile["repositoryPreview"][number];
}) {
  if (repository.topics.length === 0) {
    return null;
  }
  return (
    <div className="mt-3 flex flex-wrap gap-1.5">
      {repository.topics.slice(0, 4).map((topic) => (
        <span className="chip soft" key={topic}>
          {topic}
        </span>
      ))}
    </div>
  );
}

function PinnedRepositoryCard({
  repository,
}: {
  repository: PublicOrganizationProfile["repositoryPreview"][number];
}) {
  return (
    <Link
      aria-label={`Open ${repository.fullName}`}
      className="card block min-w-0 p-4 no-underline transition-colors hover:bg-[var(--hover)]"
      href={repository.href}
    >
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <span className="t-h3 min-w-0 truncate">{repository.fullName}</span>
        <RepositoryBadges repository={repository} />
      </div>
      {repository.description ? (
        <p className="t-sm mt-2 line-clamp-2" style={{ color: "var(--ink-2)" }}>
          {repository.description}
        </p>
      ) : (
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          No description published.
        </p>
      )}
      <TopicChips repository={repository} />
      <RepositoryMeta repository={repository} />
    </Link>
  );
}

function RepositoryPreviewRow({
  repository,
}: {
  repository: PublicOrganizationProfile["repositoryPreview"][number];
}) {
  return (
    <Link
      aria-label={`Open ${repository.fullName}`}
      className="list-row block min-w-0 py-4 no-underline"
      href={repository.href}
    >
      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <span className="t-h3 min-w-0 truncate">{repository.fullName}</span>
            <RepositoryBadges repository={repository} />
          </div>
          {repository.description ? (
            <p
              className="t-sm mt-1 line-clamp-2"
              style={{ color: "var(--ink-3)" }}
            >
              {repository.description}
            </p>
          ) : null}
          <TopicChips repository={repository} />
          <RepositoryMeta compact repository={repository} />
        </div>
        <span className="t-mono-sm" style={{ color: "var(--ink-4)" }}>
          {repository.defaultBranch}
        </span>
      </div>
    </Link>
  );
}

function OrganizationHeader({
  profile,
}: {
  profile: PublicOrganizationProfile;
}) {
  const identity = profile.identity;
  const verifiedDomain = profile.verifiedDomains[0];
  const sponsorUnavailable =
    !profile.sponsorship.enabled && profile.sponsorship.unavailableReason;

  return (
    <section className="card p-5" aria-labelledby="organization-title">
      <div className="flex flex-wrap items-start justify-between gap-5">
        <div className="flex min-w-0 gap-4">
          <Avatar profile={profile} />
          <div className="min-w-0">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Organization
            </p>
            <div className="mt-1 flex flex-wrap items-center gap-2">
              <h1 className="t-h1 min-w-0 truncate" id="organization-title">
                {identity.name}
              </h1>
              {verifiedDomain ? (
                <a
                  className="chip ok no-underline"
                  href={verifiedDomain.href}
                  title={`Verified domain ${verifiedDomain.domain}`}
                >
                  Verified
                </a>
              ) : null}
              {identity.isPrivate ? (
                <span className="chip warn">Private</span>
              ) : null}
              {profile.viewerState.role ? (
                <span className="chip soft">
                  {profile.viewerState.role === "owner"
                    ? "Owner view"
                    : `${profile.viewerState.role} view`}
                </span>
              ) : null}
            </div>
            <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
              @{identity.slug}
            </p>
            {identity.description ? (
              <p
                className="t-body mt-3 max-w-3xl"
                style={{ color: "var(--ink-2)" }}
              >
                {identity.description}
              </p>
            ) : null}
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <Link
            className="btn primary"
            href={organizationTabHref(identity.slug, "repositories")}
          >
            Repositories
          </Link>
          {profile.sponsorship.enabled && profile.sponsorship.href ? (
            <a className="btn accent" href={profile.sponsorship.href}>
              Sponsor
            </a>
          ) : (
            <button
              aria-describedby="organization-sponsor-unavailable"
              className="btn"
              disabled
              type="button"
            >
              Sponsor
            </button>
          )}
          {profile.viewerState.canAdmin ? (
            <Link
              className="btn ghost"
              href={organizationSettingsHref(identity.slug)}
            >
              Settings
            </Link>
          ) : null}
        </div>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <dl className="flex flex-wrap gap-x-5 gap-y-2">
          <div>
            <dt className="sr-only">Followers</dt>
            <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
              {identity.followerCount.toLocaleString()} followers
            </dd>
          </div>
          <div>
            <dt className="sr-only">Public members</dt>
            <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
              {identity.publicMemberCount.toLocaleString()} public members
            </dd>
          </div>
          <div>
            <dt className="sr-only">Repositories</dt>
            <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
              {identity.repositoryCount.toLocaleString()} repositories
            </dd>
          </div>
          <div>
            <dt className="sr-only">Created</dt>
            <dd className="t-sm" style={{ color: "var(--ink-3)" }}>
              Since {compactDate(identity.createdAt)}
            </dd>
          </div>
        </dl>

        <div className="flex flex-wrap gap-2 md:justify-end">
          {identity.websiteUrl ? (
            <a
              aria-label={`Website ${identity.websiteUrl.replace(/^https?:\/\//, "")}`}
              className="chip soft"
              href={identity.websiteUrl}
            >
              {identity.websiteUrl.replace(/^https?:\/\//, "")}
            </a>
          ) : null}
          {identity.location ? (
            <span className="chip soft">{identity.location}</span>
          ) : null}
          {verifiedDomain ? (
            <a
              aria-label={`Verified domain ${verifiedDomain.domain}`}
              className="chip soft"
              href={verifiedDomain.href}
            >
              {verifiedDomain.domain}
            </a>
          ) : null}
        </div>
      </div>

      {sponsorUnavailable ? (
        <p
          className="t-xs mt-4"
          id="organization-sponsor-unavailable"
          style={{ color: "var(--ink-3)" }}
        >
          Sponsorships are unavailable: {profile.sponsorship.unavailableReason}
        </p>
      ) : null}
    </section>
  );
}

function OverviewShell({ profile }: { profile: PublicOrganizationProfile }) {
  return (
    <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_280px]">
      <div className="grid gap-6">
        <section className="card p-5" aria-labelledby="organization-pinned">
          <div className="flex items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Spotlight
              </p>
              <h2 className="t-h2 mt-1" id="organization-pinned">
                Pinned repositories
              </h2>
            </div>
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {profile.pinnedRepositories.length}/6
            </span>
          </div>
          <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
            Curated repositories from this organization, ordered by profile pin
            position and filtered to what this viewer can open.
          </p>
          <div className="mt-4 grid gap-3 md:grid-cols-2">
            {profile.pinnedRepositories.slice(0, 6).map((repository) => (
              <PinnedRepositoryCard
                key={repository.id}
                repository={repository}
              />
            ))}
            {profile.pinnedRepositories.length === 0 ? (
              <div
                className="rounded-[var(--radius)] border border-dashed p-4"
                style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
              >
                <p className="t-sm">No pinned repositories are visible yet.</p>
                <Link
                  className="btn sm ghost mt-3"
                  href={organizationTabHref(
                    profile.identity.slug,
                    "repositories",
                  )}
                >
                  Browse repositories
                </Link>
              </div>
            ) : null}
          </div>
        </section>

        <section className="card p-5" aria-labelledby="organization-repos">
          <div className="flex items-end justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Activity preview
              </p>
              <h2 className="t-h2 mt-1" id="organization-repos">
                Repositories
              </h2>
            </div>
            <Link
              className="btn sm ghost"
              href={organizationTabHref(profile.identity.slug, "repositories")}
            >
              View all
            </Link>
          </div>
          <div className="mt-2 grid">
            {profile.repositoryPreview.slice(0, 3).map((repository) => (
              <RepositoryPreviewRow
                key={repository.id}
                repository={repository}
              />
            ))}
            {profile.repositoryPreview.length === 0 ? (
              <div
                className="mt-4 rounded-[var(--radius)] border border-dashed p-4"
                style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
              >
                <p className="t-sm">
                  No repositories are visible to this viewer.
                </p>
                {profile.viewerState.canAdmin ? (
                  <Link className="btn sm ghost mt-3" href="/new">
                    Create repository
                  </Link>
                ) : null}
              </div>
            ) : null}
          </div>
        </section>
      </div>

      <aside className="grid content-start gap-4">
        <section className="card p-4" aria-labelledby="organization-people">
          <div className="flex items-center justify-between gap-3">
            <h2 className="t-label" id="organization-people">
              People
            </h2>
            <Link
              className="t-xs underline"
              href={organizationTabHref(profile.identity.slug, "people")}
            >
              View people
            </Link>
          </div>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            {profile.tabCounts.people.toLocaleString()} visible{" "}
            {profile.tabCounts.people === 1 ? "person" : "people"}
            {profile.viewerState.isMember ? " including private members." : "."}
          </p>
          <div className="mt-3 grid gap-2">
            {profile.peoplePreview.slice(0, 6).map((person) => (
              <Link
                aria-label={`Open ${person.name || person.login}`}
                className="list-row grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 py-2 no-underline"
                href={person.href}
                key={person.id}
              >
                {person.avatarUrl ? (
                  <span
                    aria-hidden="true"
                    className="av sm"
                    style={{
                      backgroundImage: `url(${person.avatarUrl})`,
                      backgroundPosition: "center",
                      backgroundSize: "cover",
                    }}
                  />
                ) : (
                  <span aria-hidden="true" className="av sm">
                    {(person.name || person.login).slice(0, 1).toUpperCase()}
                  </span>
                )}
                <span className="min-w-0">
                  <span className="t-sm block truncate">
                    {person.name || person.login}
                  </span>
                  <span
                    className="t-mono-sm block truncate"
                    style={{ color: "var(--ink-3)" }}
                  >
                    @{person.login}
                  </span>
                </span>
                {person.role ? (
                  <span className="chip soft">{person.role}</span>
                ) : null}
              </Link>
            ))}
            {profile.peoplePreview.length === 0 ? (
              <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                No public members are visible.
              </span>
            ) : null}
          </div>
        </section>

        <section className="card p-4" aria-labelledby="organization-languages">
          <h2 className="t-label" id="organization-languages">
            Top languages
          </h2>
          <div className="mt-3 grid gap-2">
            {profile.topLanguages.slice(0, 5).map((language) => (
              <div className="grid gap-1" key={language.language}>
                <div className="flex items-center justify-between gap-3">
                  <span className="t-sm inline-flex min-w-0 items-center gap-2 truncate">
                    <span
                      aria-hidden="true"
                      className="inline-block size-2 rounded-full"
                      style={{ background: language.color }}
                    />
                    <span className="truncate">{language.language}</span>
                  </span>
                  <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                    {language.byteCount.toLocaleString()}
                  </span>
                </div>
                <div
                  aria-label={`${language.language} ${languagePercent(
                    language,
                    profile.topLanguages,
                  )}% of visible organization code`}
                  aria-valuemax={100}
                  aria-valuemin={0}
                  aria-valuenow={languagePercent(
                    language,
                    profile.topLanguages,
                  )}
                  className="h-1.5 overflow-hidden rounded-[var(--radius-pill)]"
                  role="progressbar"
                  style={{ background: "var(--surface-2)" }}
                >
                  <span
                    className="block h-full rounded-[var(--radius-pill)]"
                    style={{
                      background: language.color,
                      width: `${languagePercent(language, profile.topLanguages)}%`,
                    }}
                  />
                </div>
              </div>
            ))}
            {profile.topLanguages.length === 0 ? (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                No language data yet.
              </p>
            ) : null}
          </div>
        </section>

        <section className="card p-4" aria-labelledby="organization-topics">
          <h2 className="t-label" id="organization-topics">
            Topics
          </h2>
          <div className="mt-3 flex flex-wrap gap-2">
            {profile.topTopics.slice(0, 10).map((topic) => (
              <a
                aria-label={`${topic.topic}, ${topic.count.toLocaleString()} repositories`}
                className="chip soft"
                href={topic.href}
                key={topic.topic}
              >
                {topic.topic}
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {topic.count.toLocaleString()}
                </span>
              </a>
            ))}
            {profile.topTopics.length === 0 ? (
              <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                No topics have been published.
              </span>
            ) : null}
          </div>
        </section>

        <section className="card p-4" aria-labelledby="organization-sponsoring">
          <h2 className="t-label" id="organization-sponsoring">
            Sponsoring
          </h2>
          <p className="t-sm mt-3" style={{ color: "var(--ink-2)" }}>
            {profile.sponsorship.enabled
              ? `${profile.sponsorship.sponsorCount.toLocaleString()} active sponsors`
              : "Sponsorships are not available for organizations yet."}
          </p>
          {profile.sponsorship.enabled && profile.sponsorship.href ? (
            <a className="btn sm accent mt-3" href={profile.sponsorship.href}>
              Sponsor this organization
            </a>
          ) : (
            <button className="btn sm mt-3" disabled type="button">
              Sponsor preview unavailable
            </button>
          )}
        </section>
      </aside>
    </div>
  );
}

function SecondaryTab({
  activeTab,
  profile,
}: {
  activeTab: string;
  profile: PublicOrganizationProfile;
}) {
  const label =
    ORGANIZATION_TABS.find((tab) => tab.value === activeTab)?.label ??
    "Overview";
  return (
    <section className="card p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {label}
      </p>
      <h2 className="t-h2 mt-2">
        {label} for {profile.identity.slug}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        This tab keeps organization navigation concrete while the dedicated
        {` ${label.toLowerCase()} `}surface is built.
      </p>
      <div className="mt-4 flex flex-wrap gap-2">
        <Link
          className="btn sm ghost"
          href={organizationProjectHref(profile.identity.slug)}
        >
          Projects
        </Link>
        <Link
          className="btn sm ghost"
          href={organizationTabHref(profile.identity.slug, "overview")}
        >
          Overview
        </Link>
      </div>
    </section>
  );
}

export function OrganizationProfilePage({
  activeTab,
  adminPeople,
  peopleList,
  teamsDirectory,
  profile,
  packageList,
  projectList,
  repositoryList,
  session,
  shellContext,
}: OrganizationProfilePageProps) {
  const selectedTab = visibleTab(activeOrganizationTab(activeTab), profile);
  const tabs = organizationTabs(profile);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame className="max-w-[1240px]" mode="centered">
        <div className="grid gap-6">
          <OrganizationHeader profile={profile} />
          <QueryTabNavigation
            activeValue={selectedTab}
            ariaLabel="Organization sections"
            hrefForTab={(value) =>
              organizationTabHref(profile.identity.slug, value)
            }
            tabs={tabs}
          />
          <main className="min-w-0">
            {selectedTab === "overview" ? (
              <OverviewShell profile={profile} />
            ) : selectedTab === "repositories" && repositoryList ? (
              <OrganizationRepositoriesPage
                list={repositoryList}
                org={profile.identity.slug}
              />
            ) : selectedTab === "people" && adminPeople ? (
              <OrganizationPeopleAdminPage
                admin={adminPeople}
                org={profile.identity.slug}
              />
            ) : selectedTab === "people" && peopleList ? (
              <OrganizationPeoplePage
                list={peopleList}
                org={profile.identity.slug}
              />
            ) : selectedTab === "teams" && teamsDirectory ? (
              <OrganizationTeamsPage
                directory={teamsDirectory}
                org={profile.identity.slug}
              />
            ) : selectedTab === "packages" && packageList ? (
              <OwnerPackagesPage
                list={packageList}
                owner={profile.identity.slug}
                ownerKind="organization"
              />
            ) : selectedTab === "projects" && projectList ? (
              <ProjectsListPage
                list={projectList}
                scopeLabel={`${profile.identity.slug} projects`}
              />
            ) : (
              <SecondaryTab activeTab={selectedTab} profile={profile} />
            )}
          </main>
        </div>
      </AppShellFrame>
    </AppShell>
  );
}
