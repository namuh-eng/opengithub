import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import { UserProfileActions } from "@/components/UserProfileActions";
import type {
  AppShellContext,
  AuthSession,
  PublicUserProfile,
} from "@/lib/api";
import {
  activeProfileTab,
  PROFILE_TABS,
  profileTabHref,
  type QueryTab,
} from "@/lib/navigation";

type UserProfilePageProps = {
  activeTab: string;
  profile: PublicUserProfile;
  session: AuthSession;
  shellContext?: AppShellContext | null;
};

function profileInitial(profile: PublicUserProfile) {
  return (
    profile.identity.name?.trim().slice(0, 1) ??
    profile.identity.login.trim().slice(0, 1) ??
    "U"
  ).toUpperCase();
}

function displayName(profile: PublicUserProfile) {
  return profile.identity.name?.trim() || profile.identity.login;
}

function compactDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    year: "numeric",
  }).format(new Date(value));
}

function tabLabel(tab: QueryTab, profile: PublicUserProfile) {
  if (tab.value === "repositories") {
    return `Repositories ${profile.tabCounts.repositories.toLocaleString()}`;
  }
  if (tab.value === "projects") {
    return `Projects ${profile.tabCounts.projects.toLocaleString()}`;
  }
  if (tab.value === "packages") {
    return `Packages ${profile.tabCounts.packages.toLocaleString()}`;
  }
  if (tab.value === "stars") {
    return `Stars ${profile.tabCounts.stars.toLocaleString()}`;
  }
  return tab.label;
}

function profileTabs(profile: PublicUserProfile): QueryTab[] {
  if (profile.identity.isPrivate) {
    return PROFILE_TABS.filter((tab) => tab.value === "overview").map(
      (tab) => ({
        ...tab,
        label: tabLabel(tab, profile),
      }),
    );
  }

  return PROFILE_TABS.map((tab) => ({
    ...tab,
    label: tabLabel(tab, profile),
  }));
}

function visibleTab(activeTab: string, profile: PublicUserProfile) {
  const tabs = profileTabs(profile);
  return tabs.some((tab) => tab.value === activeTab)
    ? activeTab
    : tabs[0].value;
}

function Avatar({ profile }: { profile: PublicUserProfile }) {
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
      {profileInitial(profile)}
    </span>
  );
}

function IdentityColumn({ profile }: { profile: PublicUserProfile }) {
  const identity = profile.identity;
  return (
    <aside className="grid content-start gap-4 lg:sticky lg:top-[calc(var(--header-h)+24px)]">
      <div className="card p-5">
        <div className="flex items-start gap-4 lg:grid">
          <Avatar profile={profile} />
          <div className="min-w-0">
            <h1 className="t-h1 truncate">{displayName(profile)}</h1>
            <p className="t-mono-sm mt-1" style={{ color: "var(--ink-3)" }}>
              @{identity.login}
            </p>
          </div>
        </div>

        {identity.bio ? (
          <p className="t-body mt-4" style={{ color: "var(--ink-2)" }}>
            {identity.bio}
          </p>
        ) : null}

        <dl className="mt-4 grid gap-2">
          {identity.company ? (
            <div>
              <dt className="sr-only">Company</dt>
              <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
                {identity.company}
              </dd>
            </div>
          ) : null}
          {identity.location ? (
            <div>
              <dt className="sr-only">Location</dt>
              <dd className="t-sm" style={{ color: "var(--ink-2)" }}>
                {identity.location}
              </dd>
            </div>
          ) : null}
          {identity.websiteUrl ? (
            <div>
              <dt className="sr-only">Website</dt>
              <dd>
                <a className="t-sm underline" href={identity.websiteUrl}>
                  {identity.websiteUrl.replace(/^https?:\/\//, "")}
                </a>
              </dd>
            </div>
          ) : null}
          <div>
            <dt className="sr-only">Joined</dt>
            <dd className="t-sm" style={{ color: "var(--ink-3)" }}>
              Joined {compactDate(identity.joinedAt)}
            </dd>
          </div>
        </dl>

        <UserProfileActions
          followerCount={identity.followerCount}
          followingCount={identity.followingCount}
          isPrivate={identity.isPrivate}
          login={identity.login}
          viewerState={profile.viewerState}
        />
      </div>

      {!identity.isPrivate && profile.organizations.length > 0 ? (
        <section className="card p-4" aria-labelledby="profile-organizations">
          <h2 className="t-label" id="profile-organizations">
            Organizations
          </h2>
          <div className="mt-3 flex flex-wrap gap-2">
            {profile.organizations.map((organization) => (
              <Link
                className="chip soft"
                href={organization.href}
                key={organization.id}
              >
                {organization.name || organization.slug}
              </Link>
            ))}
          </div>
        </section>
      ) : null}
    </aside>
  );
}

function ReadmeCard({ profile }: { profile: PublicUserProfile }) {
  const readmeBody =
    profile.readme?.body?.trim() ||
    profile.identity.bio ||
    `${displayName(profile)} has not added a profile README yet.`;

  return (
    <section className="card p-5" aria-labelledby="profile-readme">
      <div className="flex items-center justify-between gap-3">
        <h2 className="t-h2" id="profile-readme">
          README
        </h2>
        {profile.identity.isPrivate ? (
          <span className="chip warn">Private profile</span>
        ) : null}
      </div>
      <div
        className="t-body mt-4 whitespace-pre-wrap"
        style={{ color: "var(--ink-2)" }}
      >
        {readmeBody}
      </div>
    </section>
  );
}

function PinnedRepositories({ profile }: { profile: PublicUserProfile }) {
  if (profile.identity.isPrivate) {
    return null;
  }

  return (
    <section aria-labelledby="profile-pins">
      <div className="flex items-end justify-between gap-4">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Spotlight
          </p>
          <h2 className="t-h2 mt-1" id="profile-pins">
            Pinned repositories
          </h2>
        </div>
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {profile.pinnedRepositories.length}/6
        </span>
      </div>

      {profile.pinnedRepositories.length > 0 ? (
        <div className="mt-4 grid gap-3 md:grid-cols-2">
          {profile.pinnedRepositories.slice(0, 6).map((repository) => (
            <Link
              className="card block p-4 no-underline"
              href={repository.href}
              key={repository.id}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <h3 className="t-h3 truncate">{repository.name}</h3>
                  <p
                    className="t-mono-sm mt-1"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {repository.owner}/{repository.defaultBranch}
                  </p>
                </div>
                {repository.visibility !== "public" ? (
                  <span className="chip soft">{repository.visibility}</span>
                ) : null}
              </div>
              {repository.description ? (
                <p
                  className="t-sm mt-3 line-clamp-2"
                  style={{ color: "var(--ink-2)" }}
                >
                  {repository.description}
                </p>
              ) : null}
              <div className="mt-4 flex flex-wrap items-center gap-3">
                {repository.primaryLanguage ? (
                  <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                    <span aria-hidden="true">●</span>{" "}
                    {repository.primaryLanguage.language}
                  </span>
                ) : null}
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {repository.starsCount.toLocaleString()} stars
                </span>
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {repository.forksCount.toLocaleString()} forks
                </span>
              </div>
            </Link>
          ))}
        </div>
      ) : (
        <div className="card mt-4 p-5">
          <p className="t-body" style={{ color: "var(--ink-3)" }}>
            No pinned repositories are visible yet.
          </p>
        </div>
      )}
    </section>
  );
}

function Achievements({ profile }: { profile: PublicUserProfile }) {
  if (profile.identity.isPrivate || profile.achievements.length === 0) {
    return null;
  }

  return (
    <section className="card p-4" aria-labelledby="profile-achievements">
      <h2 className="t-label" id="profile-achievements">
        Achievements
      </h2>
      <div className="mt-3 grid gap-2">
        {profile.achievements.map((achievement) => (
          <div className="flex items-center gap-3" key={achievement.slug}>
            <span className="chip accent" aria-hidden="true">
              {achievement.icon || "Award"}
            </span>
            <div className="min-w-0">
              <p className="t-sm font-semibold">{achievement.name}</p>
              {achievement.description ? (
                <p className="t-xs truncate">{achievement.description}</p>
              ) : null}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function ContributionSummary({ profile }: { profile: PublicUserProfile }) {
  if (profile.identity.isPrivate) {
    return null;
  }

  const days = profile.contributionSummary.days.slice(-84);

  return (
    <section className="card p-5" aria-labelledby="profile-contributions">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Contributions
          </p>
          <h2 className="t-h2 mt-1" id="profile-contributions">
            {profile.contributionSummary.total.toLocaleString()} contributions
            this year
          </h2>
        </div>
        <span className="chip soft">Overview</span>
      </div>
      {days.length > 0 ? (
        <fieldset className="mt-4 grid grid-cols-[repeat(12,minmax(0,1fr))] gap-1 border-0 p-0 sm:grid-cols-[repeat(21,minmax(0,1fr))]">
          <legend className="sr-only">
            {profile.contributionSummary.total.toLocaleString()} contributions
            this year
          </legend>
          {days.map((day) => (
            <span
              aria-label={`${day.count} contributions on ${day.date}`}
              className="h-3 rounded-[2px] border"
              key={day.date}
              role="img"
              style={{
                background:
                  day.intensity > 0 ? "var(--accent-soft)" : "var(--surface-2)",
                borderColor: "var(--line-soft)",
                opacity: day.intensity > 0 ? 0.35 + day.intensity * 0.13 : 1,
              }}
              title={`${day.count} contributions on ${day.date}`}
            />
          ))}
        </fieldset>
      ) : (
        <p className="t-body mt-4" style={{ color: "var(--ink-3)" }}>
          No public contributions are visible for this period.
        </p>
      )}
    </section>
  );
}

function Overview({ profile }: { profile: PublicUserProfile }) {
  return (
    <div className="grid gap-6">
      <ReadmeCard profile={profile} />
      <PinnedRepositories profile={profile} />
      <ContributionSummary profile={profile} />
    </div>
  );
}

function SecondaryTab({
  activeTab,
  profile,
}: {
  activeTab: string;
  profile: PublicUserProfile;
}) {
  const label =
    PROFILE_TABS.find((tab) => tab.value === activeTab)?.label ?? "Overview";
  return (
    <section className="card p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {label}
      </p>
      <h2 className="t-h2 mt-2">
        {label} for {profile.identity.login}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        This section keeps the identity column and profile tabs stable while the
        dedicated {label.toLowerCase()} surface is built.
      </p>
    </section>
  );
}

export function UserProfilePage({
  activeTab,
  profile,
  session,
  shellContext,
}: UserProfilePageProps) {
  const tabs = profileTabs(profile);
  const selectedTab = visibleTab(activeProfileTab(activeTab), profile);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame>
        <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
          <IdentityColumn profile={profile} />
          <main className="min-w-0">
            <QueryTabNavigation
              activeValue={selectedTab}
              ariaLabel="Profile sections"
              hrefForTab={(value) =>
                profileTabHref(profile.identity.login, value)
              }
              tabs={tabs}
            />
            <div className="mt-6 grid gap-6 xl:grid-cols-[minmax(0,1fr)_260px]">
              <div className="min-w-0">
                {selectedTab === "overview" ? (
                  <Overview profile={profile} />
                ) : (
                  <SecondaryTab activeTab={selectedTab} profile={profile} />
                )}
              </div>
              <div className="grid content-start gap-4">
                <Achievements profile={profile} />
                {!profile.identity.isPrivate ? (
                  <section
                    className="card p-4"
                    aria-labelledby="profile-activity"
                  >
                    <h2 className="t-label" id="profile-activity">
                      Recent activity
                    </h2>
                    {profile.contributionSummary.recentEvents.length > 0 ? (
                      <div className="mt-3 grid gap-3">
                        {profile.contributionSummary.recentEvents
                          .slice(0, 4)
                          .map((event) => (
                            <Link
                              className="list-row block py-2 no-underline"
                              href={
                                event.targetHref || profile.identity.htmlUrl
                              }
                              key={event.id}
                            >
                              <p className="t-sm">{event.title}</p>
                              <p className="t-xs mt-1">
                                {event.repository
                                  ? `${event.repository.owner}/${event.repository.name}`
                                  : event.eventType}
                              </p>
                            </Link>
                          ))}
                      </div>
                    ) : (
                      <p
                        className="t-sm mt-3"
                        style={{ color: "var(--ink-3)" }}
                      >
                        No public activity is visible yet.
                      </p>
                    )}
                  </section>
                ) : null}
              </div>
            </div>
          </main>
        </div>
      </AppShellFrame>
    </AppShell>
  );
}
