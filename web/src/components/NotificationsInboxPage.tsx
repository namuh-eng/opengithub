import type { ApiErrorEnvelope, NotificationInboxView } from "@/lib/api";

type NotificationsInboxPageProps = {
  view: NotificationInboxView | ApiErrorEnvelope;
};

function isError(
  view: NotificationInboxView | ApiErrorEnvelope,
): view is ApiErrorEnvelope {
  return "error" in view;
}

function withQuery(overrides: Record<string, string | null | undefined>) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(overrides)) {
    if (value?.trim()) {
      params.set(key, value.trim());
    }
  }
  const suffix = params.toString();
  return suffix ? `/notifications?${suffix}` : "/notifications";
}

export function NotificationsInboxPage({ view }: NotificationsInboxPageProps) {
  if (isError(view)) {
    return (
      <section className="card p-8" aria-labelledby="notifications-error-title">
        <p className="t-label" style={{ color: "var(--err)" }}>
          Notifications unavailable
        </p>
        <h1 className="t-h2 mt-2" id="notifications-error-title">
          {view.error.message}
        </h1>
        <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
          Try refreshing the inbox after the API connection recovers.
        </p>
      </section>
    );
  }

  const { query } = view;
  const allHref = withQuery({
    folder: query.folder === "inbox" ? null : query.folder,
    q: query.q,
    sort: query.sort === "newest" ? null : query.sort,
    group: query.group === "date" ? null : query.group,
    repo: query.repo ?? null,
  });
  const unreadHref = withQuery({
    folder: query.folder === "inbox" ? null : query.folder,
    tab: "unread",
    q: query.q,
    sort: query.sort === "newest" ? null : query.sort,
    group: query.group === "date" ? null : query.group,
    repo: query.repo ?? null,
  });

  return (
    <section aria-labelledby="notifications-title">
      <div className="mb-6 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Inbox
          </p>
          <h1 className="t-h1 mt-1" id="notifications-title">
            {view.total} {query.tab === "unread" ? "unread" : "notifications"}
          </h1>
          <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
            Triage repository activity, mentions, review requests, and delivery
            states.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span className="chip accent">{view.unreadCount} unread</span>
          <a className="btn sm" href="/settings/notifications">
            Manage notifications
          </a>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
        <aside className="space-y-5" aria-label="Notification navigation">
          <FacetSection title="Folders" facets={view.folders} />
          <FacetSection title="Default filters" facets={view.filters} />
          <div className="card p-4">
            <div className="mb-3 flex items-center justify-between gap-3">
              <h2 className="t-label">Repositories</h2>
              <a
                className="t-xs"
                href="/settings/notifications"
                style={{ color: "var(--ink-3)" }}
              >
                Manage
              </a>
            </div>
            {view.repositories.length ? (
              <nav
                className="space-y-1"
                aria-label="Repository notification buckets"
              >
                {view.repositories.map((repo) => (
                  <a
                    className={`flex items-center justify-between gap-3 rounded-[var(--radius)] px-2 py-2 t-sm ${
                      repo.active ? "chip active" : ""
                    }`}
                    href={repo.href}
                    key={repo.id}
                  >
                    <span className="truncate t-mono-sm">{repo.label}</span>
                    <span className="t-num" style={{ color: "var(--ink-3)" }}>
                      {repo.count}
                    </span>
                  </a>
                ))}
              </nav>
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Repository buckets appear once notifications are delivered.
              </p>
            )}
          </div>
        </aside>

        <div className="min-w-0 space-y-4">
          <div className="card p-4">
            <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
              <nav className="tabs" aria-label="Notification read state">
                <a
                  className={`tab ${query.tab === "all" ? "active" : ""}`}
                  href={allHref}
                >
                  All
                </a>
                <a
                  className={`tab ${query.tab === "unread" ? "active" : ""}`}
                  href={unreadHref}
                >
                  Unread
                </a>
              </nav>
              <form
                action="/notifications"
                className="flex min-w-0 flex-1 flex-wrap items-center gap-2"
              >
                <input
                  name="folder"
                  type="hidden"
                  value={query.folder === "inbox" ? "" : query.folder}
                />
                <input
                  name="tab"
                  type="hidden"
                  value={query.tab === "all" ? "" : query.tab}
                />
                <input
                  name="sort"
                  type="hidden"
                  value={query.sort === "newest" ? "" : query.sort}
                />
                <input
                  name="group"
                  type="hidden"
                  value={query.group === "date" ? "" : query.group}
                />
                <input name="repo" type="hidden" value={query.repo ?? ""} />
                <label className="sr-only" htmlFor="notifications-query">
                  Search notifications
                </label>
                <input
                  className="input min-w-[260px] flex-1"
                  type="search"
                  defaultValue={query.q}
                  id="notifications-query"
                  name="q"
                  placeholder="is:unread repo:mona/octo-app reason:mention"
                />
                <button className="btn sm primary" type="submit">
                  Search
                </button>
              </form>
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <ChoiceGroup label="Sort by" options={view.sortOptions} />
              <ChoiceGroup label="Group by" options={view.groupOptions} />
              <span className="chip soft">
                Cleanup: unread + saved stay visible until done
              </span>
            </div>
          </div>

          <div className="card p-3">
            <label
              className="flex items-center gap-2 px-2 py-1 t-sm"
              style={{ color: "var(--ink-3)" }}
            >
              <input
                aria-label="Select all visible notifications"
                type="checkbox"
              />
              Select all visible notifications
            </label>
          </div>

          {view.groups.length ? (
            view.groups.map((group) => (
              <section
                aria-labelledby={`notification-group-${group.id}`}
                key={group.id}
              >
                <div className="mb-2 flex items-center gap-2 px-1">
                  <h2 className="t-label" id={`notification-group-${group.id}`}>
                    {group.label}
                  </h2>
                  <span className="t-num t-xs">{group.count}</span>
                </div>
                <div className="card overflow-hidden">
                  {group.rows.map((row) => (
                    <article className="list-row items-start" key={row.id}>
                      <label className="mt-1">
                        <span className="sr-only">Select {row.title}</span>
                        <input type="checkbox" />
                      </label>
                      <span
                        role="img"
                        aria-label={row.unread ? "Unread" : "Read"}
                        className={
                          row.unread
                            ? "dot live mt-2"
                            : "mt-2 inline-block h-[6px] w-[6px]"
                        }
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          {row.repositoryHref ? (
                            <a
                              className="t-mono-sm"
                              href={row.repositoryHref}
                              style={{ color: "var(--ink-3)" }}
                            >
                              {row.repositoryName}
                            </a>
                          ) : (
                            <span
                              className="t-mono-sm"
                              style={{ color: "var(--ink-3)" }}
                            >
                              {row.repositoryName}
                            </span>
                          )}
                          <span className="chip soft">{row.reasonLabel}</span>
                          <span className="t-xs">{row.relativeTime}</span>
                        </div>
                        <a
                          className="mt-1 block t-body"
                          href={row.openHref}
                          style={{
                            color: "var(--ink-1)",
                            fontWeight: row.unread ? 600 : 400,
                          }}
                        >
                          {row.title}
                          {row.subjectNumber ? (
                            <span
                              className="t-mono-sm"
                              style={{ color: "var(--ink-4)" }}
                            >
                              {" #"}
                              {row.subjectNumber}
                            </span>
                          ) : null}
                        </a>
                      </div>
                      <div className="flex flex-wrap justify-end gap-1 text-right">
                        <span
                          className={row.saved ? "chip accent" : "chip soft"}
                        >
                          {row.saved ? "Saved" : "Save"}
                        </span>
                        <span className={row.done ? "chip ok" : "chip soft"}>
                          {row.done ? "Done" : "Done"}
                        </span>
                        <span
                          className={row.subscribed ? "chip info" : "chip soft"}
                        >
                          {row.subscribed ? "Subscribed" : "Unsubscribed"}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              </section>
            ))
          ) : (
            <div className="card p-10 text-center">
              <p className="t-display text-[28px]">{view.emptyTitle}</p>
              <p className="t-body mt-2" style={{ color: "var(--ink-3)" }}>
                {view.emptyMessage}
              </p>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}

function FacetSection({
  title,
  facets,
}: {
  title: string;
  facets: NotificationInboxView["folders"];
}) {
  return (
    <div className="card p-4">
      <h2 className="t-label mb-3">{title}</h2>
      <nav className="space-y-1" aria-label={title}>
        {facets.map((facet) => (
          <a
            className={`flex items-center justify-between gap-3 rounded-[var(--radius)] px-2 py-2 t-sm ${
              facet.active ? "chip active" : ""
            }`}
            href={facet.href}
            key={facet.id}
          >
            <span>{facet.label}</span>
            <span className="t-num" style={{ color: "var(--ink-3)" }}>
              {facet.count}
            </span>
          </a>
        ))}
      </nav>
      {title === "Default filters" ? (
        <a className="btn sm mt-3 w-full" href="/settings/notifications">
          Add new filter
        </a>
      ) : null}
    </div>
  );
}

function ChoiceGroup({
  label,
  options,
}: {
  label: string;
  options: NotificationInboxView["sortOptions"];
}) {
  return (
    <fieldset className="flex flex-wrap items-center gap-1">
      <legend className="t-xs mr-1">{label}</legend>
      {options.map((option) => (
        <a
          className={`chip ${option.active ? "active" : "soft"}`}
          href={option.href}
          key={option.id}
        >
          {option.label}
        </a>
      ))}
    </fieldset>
  );
}
