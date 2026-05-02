import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  AppShellContext,
  AuthSession,
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
  profile: PublicOrganizationProfile;
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
            Repository cards are backed by the organization profile contract and
            will gain full repository metrics in the next slice.
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            {profile.pinnedRepositories.slice(0, 4).map((repository) => (
              <Link
                className="chip soft"
                href={repository.href}
                key={repository.id}
              >
                {repository.fullName}
              </Link>
            ))}
            {profile.pinnedRepositories.length === 0 ? (
              <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                No pinned repositories are visible yet.
              </span>
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
          <div className="mt-4 grid gap-2">
            {profile.repositoryPreview.slice(0, 3).map((repository) => (
              <Link
                className="list-row block py-3 no-underline"
                href={repository.href}
                key={repository.id}
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="t-h3">{repository.fullName}</span>
                  {repository.visibility !== "public" ? (
                    <span className="chip soft">{repository.visibility}</span>
                  ) : null}
                </div>
                {repository.description ? (
                  <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                    {repository.description}
                  </p>
                ) : null}
              </Link>
            ))}
            {profile.repositoryPreview.length === 0 ? (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                No repositories are visible to this viewer.
              </p>
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
          <div className="mt-3 flex flex-wrap gap-2">
            {profile.peoplePreview.slice(0, 8).map((person) => (
              <Link
                aria-label={person.name || person.login}
                className="av sm no-underline"
                href={person.href}
                key={person.id}
                title={person.name || person.login}
              >
                {(person.name || person.login).slice(0, 1).toUpperCase()}
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
              <div
                className="flex items-center justify-between gap-3"
                key={language.language}
              >
                <span className="t-sm truncate">{language.language}</span>
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {language.byteCount.toLocaleString()}
                </span>
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
              <a className="chip soft" href={topic.href} key={topic.topic}>
                {topic.topic}
              </a>
            ))}
            {profile.topTopics.length === 0 ? (
              <span className="t-sm" style={{ color: "var(--ink-3)" }}>
                No topics have been published.
              </span>
            ) : null}
          </div>
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
  profile,
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
            ) : (
              <SecondaryTab activeTab={selectedTab} profile={profile} />
            )}
          </main>
        </div>
      </AppShellFrame>
    </AppShell>
  );
}
