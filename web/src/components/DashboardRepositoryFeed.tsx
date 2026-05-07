import Link from "next/link";
import type {
  DashboardActivityItem,
  DashboardFeedEvent,
  DashboardFeedEventType,
  DashboardFeedTab,
  DashboardIssueSummary,
  DashboardReviewRequest,
  DashboardSummary,
} from "@/lib/api";

type DashboardRepositoryFeedProps = {
  activeEventTypes: DashboardFeedEventType[];
  activeFeedTab: DashboardFeedTab;
  summary: DashboardSummary;
};

const FEED_EVENT_LABELS: Record<DashboardFeedEventType, string> = {
  star: "Stars",
  follow: "Follows",
  repository_create: "Repository creation",
  help_wanted_issue: "Help wanted issues",
  help_wanted_pull_request: "Help wanted pull requests",
  push: "Pushes",
  fork: "Forks",
  release: "Releases",
};

function formatActivityDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "recently";
  }

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(date);
}

function formatRelativeActivityTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "recently";
  }

  const deltaSeconds = Math.round((date.getTime() - Date.now()) / 1000);
  const absSeconds = Math.abs(deltaSeconds);
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  if (absSeconds < 60) {
    return formatter.format(deltaSeconds, "second");
  }
  if (absSeconds < 3600) {
    return formatter.format(Math.round(deltaSeconds / 60), "minute");
  }
  if (absSeconds < 86_400) {
    return formatter.format(Math.round(deltaSeconds / 3600), "hour");
  }
  return formatter.format(Math.round(deltaSeconds / 86_400), "day");
}

function feedIconLabel(eventType: DashboardFeedEventType): string {
  switch (eventType) {
    case "star":
      return "S";
    case "follow":
      return "F";
    case "repository_create":
      return "R";
    case "help_wanted_issue":
      return "I";
    case "help_wanted_pull_request":
      return "P";
    case "push":
      return "C";
    case "fork":
      return "Y";
    case "release":
      return "V";
  }
}

function FeedIcon({ eventType }: { eventType: DashboardFeedEventType }) {
  return (
    <span
      aria-hidden="true"
      className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full t-xs font-semibold"
      style={{
        border: "1px solid var(--line)",
        background: "var(--surface-2)",
        color: "var(--ink-3)",
      }}
    >
      {feedIconLabel(eventType)}
    </span>
  );
}

function ActorAvatar({ event }: { event: DashboardFeedEvent }) {
  const initial = event.actorLogin.charAt(0).toUpperCase() || "U";

  if (event.actorAvatarUrl) {
    return (
      <span
        aria-hidden="true"
        className="av sm bg-cover bg-center"
        style={{ backgroundImage: `url(${event.actorAvatarUrl})` }}
      />
    );
  }

  return (
    <span
      aria-hidden="true"
      className="av sm inline-flex items-center justify-center t-xs font-semibold"
      style={{
        background: "var(--surface-3)",
        color: "var(--ink-3)",
      }}
    >
      {initial}
    </span>
  );
}

function ActivityAvatar({ item }: { item: DashboardActivityItem }) {
  const initial = item.actorLogin.charAt(0).toUpperCase() || "U";

  if (item.actorAvatarUrl) {
    return (
      <span
        aria-hidden="true"
        className="av sm bg-cover bg-center"
        style={{ backgroundImage: `url(${item.actorAvatarUrl})` }}
      />
    );
  }

  return (
    <span
      aria-hidden="true"
      className="av sm inline-flex items-center justify-center t-xs font-semibold"
      style={{
        background: "var(--surface-3)",
        color: "var(--ink-3)",
      }}
    >
      {initial}
    </span>
  );
}

function feedUrl(
  feedTab: DashboardFeedTab,
  eventTypes: DashboardFeedEventType[] = [],
): string {
  const params = new URLSearchParams();
  params.set("feedTab", feedTab);
  for (const eventType of eventTypes) {
    params.append("eventType", eventType);
  }
  return `/dashboard?${params.toString()}`;
}

function FeedCard({ event }: { event: DashboardFeedEvent }) {
  return (
    <li>
      <article className="card flex gap-3 p-4">
        <FeedIcon eventType={event.eventType} />
        <div className="min-w-0 flex-1">
          <div
            className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 t-xs"
            style={{ color: "var(--ink-3)" }}
          >
            <span className="font-medium">
              {FEED_EVENT_LABELS[event.eventType]}
            </span>
            <Link
              className="font-medium hover:underline"
              style={{ color: "var(--accent)" }}
              href={event.repositoryHref}
            >
              {event.repositoryName}
            </Link>
          </div>
          <h2
            className="mt-1 truncate t-sm font-semibold leading-5"
            style={{ color: "var(--ink-1)" }}
          >
            <Link
              className="hover:underline"
              style={{ color: "inherit" }}
              href={event.targetHref}
            >
              {event.title}
            </Link>
          </h2>
          {event.excerpt ? (
            <p
              className="mt-1 line-clamp-2 t-sm leading-5"
              style={{ color: "var(--ink-3)" }}
            >
              {event.excerpt}
            </p>
          ) : null}
          <div
            className="mt-2 flex min-w-0 flex-wrap items-center gap-2 t-xs"
            style={{ color: "var(--ink-3)" }}
          >
            <ActorAvatar event={event} />
            <span className="truncate">{event.actionSummary}</span>
            <time
              dateTime={event.occurredAt}
              suppressHydrationWarning
              title={formatActivityDate(event.occurredAt)}
            >
              {formatRelativeActivityTime(event.occurredAt)}
            </time>
          </div>
        </div>
      </article>
    </li>
  );
}

function activityIconLabel(kind: DashboardActivityItem["kind"]): string {
  return kind === "pull_request" ? "P" : "I";
}

function stateChipClass(state: DashboardActivityItem["state"]): string {
  if (state === "closed") {
    return "chip err";
  }
  if (state === "merged") {
    return "chip info";
  }
  return "chip ok";
}

function WorkActivityRow({ item }: { item: DashboardActivityItem }) {
  return (
    <li className="list-row gap-3 py-3">
      <span
        aria-hidden="true"
        className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full t-xs font-semibold"
        style={{
          border: "1px solid var(--line)",
          background: "var(--surface-2)",
          color: "var(--ink-3)",
        }}
      >
        {activityIconLabel(item.kind)}
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Link
            className="min-w-0 truncate t-sm font-semibold leading-5 hover:underline"
            style={{ color: "var(--accent)" }}
            href={item.href}
          >
            {item.title}
          </Link>
          <span className={stateChipClass(item.state)}>{item.state}</span>
        </div>
        <div
          className="mt-1 flex min-w-0 flex-wrap items-center gap-2 t-xs"
          style={{ color: "var(--ink-3)" }}
        >
          <ActivityAvatar item={item} />
          <Link
            className="font-medium hover:underline"
            style={{ color: "inherit" }}
            href={item.repositoryHref}
          >
            {item.repositoryName}
          </Link>
          <span>#{item.number}</span>
          <span>{item.description ?? "updated"}</span>
          <span>{item.actorLogin}</span>
          <time
            dateTime={item.occurredAt}
            suppressHydrationWarning
            title={formatActivityDate(item.occurredAt)}
          >
            {formatRelativeActivityTime(item.occurredAt)}
          </time>
        </div>
      </div>
    </li>
  );
}

function RecentWorkActivity({ summary }: { summary: DashboardSummary }) {
  return (
    <section aria-labelledby="recent-activity-heading" className="card p-5">
      <h2 className="t-h3" id="recent-activity-heading">
        Recent activity
      </h2>
      {summary.recentActivity.length > 0 ? (
        <ul className="mt-2">
          {summary.recentActivity.map((item) => (
            <WorkActivityRow item={item} key={item.id} />
          ))}
        </ul>
      ) : (
        <div className="mt-2">
          <p className="t-sm leading-6" style={{ color: "var(--ink-3)" }}>
            There is no recent activity involving you yet.
          </p>
          <div className="mt-3 flex flex-wrap gap-3">
            <Link
              className="t-sm font-semibold hover:underline"
              style={{ color: "var(--accent)" }}
              href="/new"
            >
              Create repository
            </Link>
            <Link
              className="t-sm font-semibold hover:underline"
              style={{ color: "var(--accent)" }}
              href="/explore"
            >
              Explore repositories
            </Link>
          </div>
        </div>
      )}
    </section>
  );
}

function DashboardFeedControls({
  activeEventTypes,
  activeFeedTab,
  supportedEventTypes,
}: {
  activeEventTypes: DashboardFeedEventType[];
  activeFeedTab: DashboardFeedTab;
  supportedEventTypes: DashboardFeedEventType[];
}) {
  return (
    <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div
        aria-label="Dashboard feed"
        className="tabs inline-flex w-fit"
        role="tablist"
      >
        {[
          ["following", "Following"],
          ["for_you", "For you"],
        ].map(([feedTab, label]) => {
          const selected = activeFeedTab === feedTab;
          return (
            <Link
              aria-selected={selected}
              className={`tab${selected ? " active" : ""}`}
              href={feedUrl(feedTab as DashboardFeedTab, activeEventTypes)}
              key={feedTab}
              role="tab"
            >
              {label}
            </Link>
          );
        })}
      </div>

      <details className="relative">
        <summary className="btn ghost sm inline-flex cursor-pointer list-none items-center justify-center">
          Filter
          {activeEventTypes.length > 0 ? (
            <span className="chip accent ml-2">{activeEventTypes.length}</span>
          ) : null}
        </summary>
        <form
          action="/dashboard"
          className="card absolute right-0 z-10 mt-2 w-72 p-3"
          style={{ boxShadow: "var(--shadow-md)" }}
          method="get"
        >
          <input name="feedTab" type="hidden" value={activeFeedTab} />
          <fieldset>
            <legend className="t-label" style={{ color: "var(--ink-3)" }}>
              Event types
            </legend>
            <div className="mt-2 grid gap-1">
              {supportedEventTypes.map((eventType) => (
                <label
                  className="flex min-h-8 items-center gap-2 rounded-md px-2 t-sm hover:bg-[var(--hover)]"
                  style={{ color: "var(--ink-1)" }}
                  key={eventType}
                >
                  <input
                    className="h-4 w-4"
                    defaultChecked={activeEventTypes.includes(eventType)}
                    name="eventType"
                    type="checkbox"
                    value={eventType}
                  />
                  <span>{FEED_EVENT_LABELS[eventType]}</span>
                </label>
              ))}
            </div>
          </fieldset>
          <div
            className="mt-3 flex items-center justify-between pt-3"
            style={{ borderTop: "1px solid var(--line)" }}
          >
            <Link
              className="t-sm font-semibold hover:underline"
              style={{ color: "var(--accent)" }}
              href={feedUrl(activeFeedTab)}
            >
              Clear filters
            </Link>
            <button className="btn primary sm" type="submit">
              Apply
            </button>
          </div>
        </form>
      </details>
    </div>
  );
}

function CompactWorkItem({
  item,
  type,
}: {
  item: DashboardIssueSummary | DashboardReviewRequest;
  type: "issue" | "review";
}) {
  return (
    <li className="list-row py-3">
      <Link
        className="t-sm font-semibold leading-5 hover:underline"
        style={{ color: "var(--accent)" }}
        href={item.href}
      >
        {item.title}
      </Link>
      <p className="mt-1 t-xs" style={{ color: "var(--ink-3)" }}>
        {item.repositoryName} #{item.number} ·{" "}
        {type === "issue" ? "Assigned" : "Review requested"}{" "}
        {formatActivityDate(item.updatedAt)}
      </p>
    </li>
  );
}

export function DashboardRepositoryFeed({
  activeEventTypes,
  activeFeedTab,
  summary,
}: DashboardRepositoryFeedProps) {
  const hasFeedEvents = summary.feedEvents.length > 0;

  return (
    <div className="grid gap-6 xl:grid-cols-[minmax(0,720px)_minmax(240px,1fr)]">
      <main className="min-w-0 max-w-[720px] space-y-5">
        <RecentWorkActivity summary={summary} />

        <section aria-labelledby="dashboard-feed-heading">
          <h1 className="mb-3 t-h3" id="dashboard-feed-heading">
            Dashboard feed
          </h1>
          <DashboardFeedControls
            activeEventTypes={activeEventTypes}
            activeFeedTab={activeFeedTab}
            supportedEventTypes={summary.supportedFeedEventTypes}
          />
          {hasFeedEvents ? (
            <ul className="space-y-3">
              {summary.feedEvents.map((event) => (
                <FeedCard event={event} key={event.id} />
              ))}
            </ul>
          ) : (
            <div className="card p-5">
              <p className="t-sm leading-6" style={{ color: "var(--ink-3)" }}>
                No dashboard feed events match the current filters.
              </p>
              <div className="mt-3 flex flex-wrap gap-3">
                <Link
                  className="t-sm font-semibold hover:underline"
                  style={{ color: "var(--accent)" }}
                  href={feedUrl(activeFeedTab)}
                >
                  Clear filters
                </Link>
                <Link
                  className="t-sm font-semibold hover:underline"
                  style={{ color: "var(--accent)" }}
                  href="/new"
                >
                  Create repository
                </Link>
                <Link
                  className="t-sm font-semibold hover:underline"
                  style={{ color: "var(--accent)" }}
                  href="/explore"
                >
                  Explore repositories
                </Link>
              </div>
            </div>
          )}
        </section>

        <section aria-labelledby="assigned-issues-heading" className="card p-5">
          <h2 className="t-h3">
            <span id="assigned-issues-heading">Assigned issues</span>
          </h2>
          {summary.assignedIssues.length > 0 ? (
            <ul className="mt-1">
              {summary.assignedIssues.map((item) => (
                <CompactWorkItem item={item} key={item.id} type="issue" />
              ))}
            </ul>
          ) : (
            <p
              className="mt-2 t-sm leading-6"
              style={{ color: "var(--ink-3)" }}
            >
              Issues assigned to you will appear here when issue tracking ships.
            </p>
          )}
        </section>
      </main>

      <aside className="space-y-5">
        <section aria-labelledby="review-requests-heading" className="card p-5">
          <h2 className="t-h3" id="review-requests-heading">
            Review requests
          </h2>
          {summary.reviewRequests.length > 0 ? (
            <ul className="mt-1">
              {summary.reviewRequests.map((item) => (
                <CompactWorkItem item={item} key={item.id} type="review" />
              ))}
            </ul>
          ) : (
            <p
              className="mt-2 t-sm leading-6"
              style={{ color: "var(--ink-3)" }}
            >
              Pull requests waiting for your review will appear here.
            </p>
          )}
        </section>
        <section className="card p-5">
          <h2 className="t-h3">Explore repositories</h2>
          <p className="mt-2 t-sm leading-6" style={{ color: "var(--ink-3)" }}>
            Use the repository list to jump back into active projects and open
            the latest code, issues, and pull requests.
          </p>
        </section>
      </aside>
    </div>
  );
}
