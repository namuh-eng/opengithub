import Link from "next/link";
import { AppShell } from "@/components/AppShell";
import { AppShellFrame } from "@/components/AppShellFrame";
import { ProfileControls } from "@/components/ProfileControls";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  AppShellContext,
  AuthSession,
  PublicProfileView,
} from "@/lib/api";
import type { QueryTab } from "@/lib/navigation";

type UserProfileOverviewProps = {
  activeTab: string;
  hrefForTab: (value: string) => string;
  profile: PublicProfileView;
  session: AuthSession;
  shellContext?: AppShellContext | null;
  tabs: readonly QueryTab[];
};

function initials(profile: PublicProfileView) {
  return (
    (profile.identity.displayName ?? profile.identity.login)
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part.slice(0, 1).toUpperCase())
      .join("") || "OG"
  );
}

function displayDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(new Date(value));
}

function monthLabel(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    timeZone: "UTC",
  }).format(new Date(value));
}

function contributionTone(intensity: number) {
  if (intensity <= 0) return "var(--surface-2)";
  if (intensity === 1)
    return "color-mix(in oklch, var(--accent) 24%, var(--surface))";
  if (intensity === 2)
    return "color-mix(in oklch, var(--accent) 46%, var(--surface))";
  if (intensity === 3)
    return "color-mix(in oklch, var(--accent) 68%, var(--surface))";
  return "var(--accent)";
}

function withTabCounts(tabs: readonly QueryTab[], profile: PublicProfileView) {
  return tabs.map((tab) => {
    const count =
      tab.value === "repositories"
        ? profile.tabs.repositories
        : tab.value === "packages"
          ? profile.tabs.packages
          : tab.value === "stars"
            ? profile.tabs.stars
            : undefined;
    return count === undefined
      ? tab
      : { ...tab, label: `${tab.label} ${count.toLocaleString()}` };
  });
}

export function UserProfileOverview({
  activeTab,
  hrefForTab,
  profile,
  session,
  shellContext,
  tabs,
}: UserProfileOverviewProps) {
  const identity = profile.identity;
  const tabItems = withTabCounts(tabs, profile);

  return (
    <AppShell session={session} shellContext={shellContext}>
      <AppShellFrame className="max-w-[1240px]" mode="centered">
        <div className="grid gap-8 lg:grid-cols-[300px_1fr]">
          <aside className="grid content-start gap-5">
            <div className="card grid gap-4 p-5">
              <div className="flex items-start gap-4 lg:grid lg:gap-4">
                <span className="av xl" aria-hidden="true">
                  {initials(profile)}
                </span>
                <div className="min-w-0">
                  <p className="t-label" style={{ color: "var(--ink-3)" }}>
                    Public profile
                  </p>
                  <h1 className="t-h1 mt-1 truncate">
                    {identity.displayName ?? identity.login}
                  </h1>
                  <p
                    className="t-mono-sm mt-1"
                    style={{ color: "var(--ink-3)" }}
                  >
                    @{identity.login}
                  </p>
                </div>
              </div>

              {identity.bio ? <p className="t-body">{identity.bio}</p> : null}

              <ProfileControls
                initialFollowerCount={identity.followerCount}
                login={identity.login}
                viewer={profile.viewer}
              />

              <dl className="grid gap-2 t-sm" style={{ color: "var(--ink-2)" }}>
                {identity.company ? (
                  <div className="flex gap-2">
                    <dt className="t-label min-w-16">Company</dt>
                    <dd>{identity.company}</dd>
                  </div>
                ) : null}
                {identity.location ? (
                  <div className="flex gap-2">
                    <dt className="t-label min-w-16">Location</dt>
                    <dd>{identity.location}</dd>
                  </div>
                ) : null}
                {identity.websiteUrl ? (
                  <div className="flex gap-2">
                    <dt className="t-label min-w-16">Web</dt>
                    <dd>
                      <a
                        className="hover:underline"
                        href={identity.websiteUrl}
                        rel="noreferrer"
                        target="_blank"
                      >
                        {identity.websiteUrl.replace(/^https?:\/\//, "")}
                      </a>
                    </dd>
                  </div>
                ) : null}
                <div className="flex gap-2">
                  <dt className="t-label min-w-16">Joined</dt>
                  <dd>{displayDate(identity.createdAt)}</dd>
                </div>
              </dl>

              <div
                className="grid grid-cols-2 gap-3 border-t pt-4"
                style={{ borderColor: "var(--line)" }}
              >
                <div>
                  <p className="t-num text-lg">
                    {identity.followingCount.toLocaleString()}
                  </p>
                  <p className="t-xs">following</p>
                </div>
                <div>
                  <p className="t-num text-lg">
                    {profile.tabs.stars.toLocaleString()}
                  </p>
                  <p className="t-xs">starred</p>
                </div>
              </div>
            </div>

            {profile.achievements.length ? (
              <section
                className="card p-5"
                aria-labelledby="achievements-heading"
              >
                <h2 className="t-h3" id="achievements-heading">
                  Achievements
                </h2>
                <div className="mt-4 grid gap-3">
                  {profile.achievements.map((achievement) => (
                    <div className="flex gap-3" key={achievement.slug}>
                      <span className="chip accent h-fit" aria-hidden="true">
                        ◆
                      </span>
                      <div>
                        <p className="t-sm font-semibold">{achievement.name}</p>
                        <p className="t-xs mt-1">{achievement.description}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            ) : null}
          </aside>

          <main className="grid content-start gap-6">
            <QueryTabNavigation
              activeValue={activeTab}
              ariaLabel="Profile sections"
              hrefForTab={hrefForTab}
              tabs={tabItems}
            />

            {identity.privateProfile ? (
              <PrivateProfilePanel profile={profile} />
            ) : activeTab === "overview" ? (
              <OverviewPanel profile={profile} />
            ) : (
              <TabPlaceholder activeTab={activeTab} login={identity.login} />
            )}
          </main>
        </div>
      </AppShellFrame>
    </AppShell>
  );
}

function OverviewPanel({ profile }: { profile: PublicProfileView }) {
  return (
    <div className="grid gap-6">
      {profile.readme ? (
        <section className="card p-6" aria-labelledby="readme-heading">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Profile README
          </p>
          <h2 className="t-h2 mt-2" id="readme-heading">
            Hello from {profile.identity.login}
          </h2>
          <p
            className="t-body mt-4 whitespace-pre-wrap"
            style={{ color: "var(--ink-2)" }}
          >
            {profile.readme.body}
          </p>
        </section>
      ) : null}

      <section aria-labelledby="pinned-heading">
        <div className="mb-3 flex items-center justify-between gap-3">
          <h2 className="t-h3" id="pinned-heading">
            Pinned
          </h2>
          <span className="t-xs">Up to six public repositories and gists</span>
        </div>
        {profile.pinnedItems.length ? (
          <div className="grid gap-3 md:grid-cols-2">
            {profile.pinnedItems.map((item) => (
              <article className="card p-4" key={item.id}>
                <div className="flex items-start justify-between gap-3">
                  <div>
                    {item.href ? (
                      <Link className="t-h3 hover:underline" href={item.href}>
                        {item.title}
                      </Link>
                    ) : (
                      <h3 className="t-h3">{item.title}</h3>
                    )}
                    <p className="t-xs mt-1 capitalize">{item.kind}</p>
                  </div>
                  <span className="chip soft">Pinned</span>
                </div>
                {item.description ? (
                  <p
                    className="t-sm mt-3 line-clamp-2"
                    style={{ color: "var(--ink-2)" }}
                  >
                    {item.description}
                  </p>
                ) : null}
                <div className="mt-4 flex flex-wrap gap-3 t-xs">
                  {item.language ? <span>{item.language}</span> : null}
                  <span className="t-num">
                    {item.starsCount.toLocaleString()} stars
                  </span>
                  <span className="t-num">
                    {item.forksCount.toLocaleString()} forks
                  </span>
                  <span>Updated {displayDate(item.updatedAt)}</span>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <div className="card p-6">
            <p className="t-body" style={{ color: "var(--ink-2)" }}>
              No public pins yet. Public repositories can appear here once this
              profile has activity.
            </p>
          </div>
        )}
      </section>

      <ContributionGraph profile={profile} />
    </div>
  );
}

function ContributionGraph({ profile }: { profile: PublicProfileView }) {
  const days = profile.contributions.days;
  const monthStarts = days.filter(
    (day, index) => index === 0 || new Date(day.date).getUTCDate() === 1,
  );
  return (
    <section className="card p-5" aria-labelledby="contributions-heading">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Contributions
          </p>
          <h2 className="t-h2 mt-1" id="contributions-heading">
            {profile.contributions.total.toLocaleString()} contributions in{" "}
            {profile.contributions.year}
          </h2>
        </div>
        <Link
          className="btn sm ghost"
          href={`/${encodeURIComponent(profile.identity.login)}?year=${profile.contributions.year - 1}`}
        >
          Previous year
        </Link>
      </div>
      <div className="mt-5 overflow-x-auto pb-2">
        <div className="grid min-w-[760px] gap-2">
          <div className="grid grid-cols-12 gap-2 t-xs" aria-hidden="true">
            {monthStarts.slice(0, 12).map((day) => (
              <span key={day.date}>{monthLabel(day.date)}</span>
            ))}
          </div>
          <div
            className="grid grid-flow-col grid-rows-7 gap-1"
            style={{ gridAutoColumns: "minmax(10px, 1fr)" }}
          >
            {days.map((day) => (
              <span
                className="h-3 rounded-[2px] border"
                key={day.date}
                style={{
                  background: contributionTone(day.intensity),
                  borderColor: "var(--line-soft)",
                }}
                title={`${day.count} contributions on ${displayDate(day.date)}`}
              >
                <span className="sr-only">
                  {day.count} contributions on {displayDate(day.date)}
                </span>
              </span>
            ))}
          </div>
        </div>
      </div>
      <div
        className="mt-4 flex items-center justify-end gap-2 t-xs"
        aria-hidden="true"
      >
        <span>Less</span>
        {[0, 1, 2, 3, 4].map((level) => (
          <span
            className="h-3 w-3 rounded-[2px] border"
            key={level}
            style={{
              background: contributionTone(level),
              borderColor: "var(--line-soft)",
            }}
          />
        ))}
        <span>More</span>
      </div>
    </section>
  );
}

function PrivateProfilePanel({ profile }: { profile: PublicProfileView }) {
  return (
    <section className="card p-6" aria-labelledby="private-profile-heading">
      <span className="chip soft">Private profile</span>
      <h2 className="t-h2 mt-4" id="private-profile-heading">
        {profile.identity.login} keeps activity private
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        Bio and identity stay visible, while achievements, follower counts,
        activity, contribution graph, and secondary tab content are hidden.
      </p>
      {profile.readme ? (
        <p className="t-body mt-4 whitespace-pre-wrap">{profile.readme.body}</p>
      ) : null}
    </section>
  );
}

function TabPlaceholder({
  activeTab,
  login,
}: {
  activeTab: string;
  login: string;
}) {
  return (
    <section className="card p-6">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Profile tab
      </p>
      <h2 className="t-h2 mt-2 capitalize">{activeTab}</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        The {activeTab} tab for @{login} keeps the profile identity column in
        place and is ready for the next vertical slice.
      </p>
    </section>
  );
}
