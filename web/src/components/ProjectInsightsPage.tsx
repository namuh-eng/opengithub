"use client";

import Link from "next/link";
import type { ProjectInsights, ProjectInsightsChartSummary } from "@/lib/api";
import {
  organizationProjectInsightsHref,
  organizationProjectSettingsHref,
  organizationProjectWorkspaceHref,
  type ProjectInsightsRouteQuery,
  userProjectInsightsHref,
  userProjectSettingsHref,
  userProjectWorkspaceHref,
} from "@/lib/navigation";

type ProjectInsightsPageProps = {
  insights: ProjectInsights;
  scope: "user" | "organization";
  owner: string;
};

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
}

function projectInsightsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
  query: ProjectInsightsRouteQuery = {},
) {
  return scope === "organization"
    ? organizationProjectInsightsHref(owner, projectNumber, query)
    : userProjectInsightsHref(owner, projectNumber, query);
}

function projectWorkspaceHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
) {
  return scope === "organization"
    ? organizationProjectWorkspaceHref(owner, projectNumber, 1)
    : userProjectWorkspaceHref(owner, projectNumber, 1);
}

function projectSettingsHref(
  scope: "user" | "organization",
  owner: string,
  projectNumber: number,
) {
  return scope === "organization"
    ? organizationProjectSettingsHref(owner, projectNumber)
    : userProjectSettingsHref(owner, projectNumber);
}

function rangeQuery(
  insights: ProjectInsights,
  range: string,
): ProjectInsightsRouteQuery {
  return {
    chart: insights.selectedChart.id,
    range,
    filter: insights.filter.query,
    table: insights.selectedChart.configuration.table === true,
  };
}

function chartQuery(
  insights: ProjectInsights,
  chart: ProjectInsightsChartSummary,
): ProjectInsightsRouteQuery {
  return {
    chart: chart.id,
    range: insights.range.key,
    filter: insights.filter.query,
    table: insights.selectedChart.configuration.table === true,
  };
}

function activeQuery(insights: ProjectInsights): ProjectInsightsRouteQuery {
  return {
    chart: insights.selectedChart.id,
    range: insights.range.key,
    start: insights.range.key === "custom" ? insights.range.start : null,
    end: insights.range.key === "custom" ? insights.range.end : null,
    filter: insights.filter.query,
    table: insights.selectedChart.configuration.table === true,
  };
}

function seriesColor(index: number) {
  return index === 0 ? "var(--accent)" : "var(--ink-2)";
}

function maxSeriesValue(insights: ProjectInsights) {
  return Math.max(
    1,
    ...insights.series.flatMap((series) =>
      series.points.map((point) => point.value),
    ),
  );
}

export function ProjectInsightsPage({
  insights,
  scope,
  owner,
}: ProjectInsightsPageProps) {
  const projectNumber = insights.project.number;
  const canCreate = insights.viewerPermissions.canCreateCharts;
  const maxValue = maxSeriesValue(insights);
  const status = insights.latestStatus;
  const currentChartId = insights.selectedChart.id;
  const showingTable = insights.selectedChart.configuration.table === true;
  const hasMatches = insights.matchingItemCount > 0;

  return (
    <main
      style={{ maxWidth: 1240, margin: "0 auto", padding: "24px 32px 48px" }}
    >
      <div className="mb-5 flex flex-wrap items-center gap-2">
        <Link
          className="chip soft"
          href={projectWorkspaceHref(scope, owner, projectNumber)}
        >
          Return to project view
        </Link>
        <span className="chip active">Insights</span>
        <span className="t-xs t-mono-sm">#{projectNumber}</span>
      </div>

      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="t-label">Project insights</div>
          <h1 className="t-h1 mt-1">{insights.project.title}</h1>
          {insights.project.description ? (
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {insights.project.description}
            </p>
          ) : null}
        </div>
        <nav className="flex flex-wrap gap-2" aria-label="Project sections">
          <Link
            className="btn sm"
            href={projectWorkspaceHref(scope, owner, projectNumber)}
          >
            View
          </Link>
          <Link
            aria-current="page"
            className="btn sm primary"
            href={projectInsightsHref(scope, owner, projectNumber)}
          >
            Insights
          </Link>
          <Link
            className="btn sm"
            href={projectSettingsHref(scope, owner, projectNumber)}
          >
            Settings
          </Link>
        </nav>
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(220px, 280px) minmax(0, 1fr)",
          gap: 20,
        }}
      >
        <aside className="min-w-0" aria-label="Project charts">
          <div className="card p-2">
            <div className="t-label px-2 pb-2 pt-1">Default charts</div>
            {insights.defaultCharts.map((chart) => (
              <Link
                aria-current={chart.id === currentChartId ? "page" : undefined}
                className="list-row"
                href={projectInsightsHref(
                  scope,
                  owner,
                  projectNumber,
                  chartQuery(insights, chart),
                )}
                key={chart.id}
                style={{
                  borderRadius: "var(--radius)",
                  borderBottom: 0,
                  padding: "10px 8px",
                  background:
                    chart.id === currentChartId
                      ? "var(--surface-2)"
                      : "transparent",
                }}
              >
                <span className="t-sm" style={{ fontWeight: 600 }}>
                  {chart.title}
                </span>
                <span className="t-xs">{chart.description}</span>
              </Link>
            ))}
          </div>

          <div className="card mt-4 p-2">
            <div className="flex items-center justify-between gap-2 px-2 pb-2 pt-1">
              <div className="t-label">Custom charts</div>
              <button
                className="btn sm"
                disabled
                title={
                  canCreate
                    ? "Custom chart creation is implemented in a later phase."
                    : "Your project role cannot create charts."
                }
                type="button"
              >
                New
              </button>
            </div>
            {insights.customCharts.length ? (
              insights.customCharts.map((chart) => (
                <Link
                  aria-current={
                    chart.id === currentChartId ? "page" : undefined
                  }
                  className="list-row"
                  href={projectInsightsHref(
                    scope,
                    owner,
                    projectNumber,
                    chartQuery(insights, chart),
                  )}
                  key={chart.id}
                  style={{
                    borderRadius: "var(--radius)",
                    borderBottom: 0,
                    padding: "10px 8px",
                  }}
                >
                  <span className="t-sm" style={{ fontWeight: 600 }}>
                    {chart.title}
                  </span>
                  <span className="t-xs">
                    {chart.sharedWithViewers ? "Shared" : "Private"} ·{" "}
                    {chart.chartType}
                  </span>
                </Link>
              ))
            ) : (
              <p className="t-sm px-2 py-3" style={{ color: "var(--ink-3)" }}>
                No custom charts yet.
              </p>
            )}
          </div>
        </aside>

        <section className="min-w-0" aria-label="Selected chart">
          <div className="card overflow-hidden">
            <div
              className="flex flex-wrap items-start justify-between gap-4"
              style={{ borderBottom: "1px solid var(--line)", padding: 18 }}
            >
              <div className="min-w-0">
                <div className="t-label">Burn up chart</div>
                <h2 className="t-h2 mt-1">{insights.selectedChart.title}</h2>
                {insights.selectedChart.description ? (
                  <p
                    className="t-sm mt-2 max-w-2xl"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {insights.selectedChart.description}
                  </p>
                ) : null}
              </div>
              <div className="flex flex-wrap gap-2">
                <button
                  className="btn sm"
                  disabled={!insights.viewerPermissions.canEditCharts}
                  title="Chart editing is implemented in a later phase."
                  type="button"
                >
                  Edit
                </button>
                <button
                  className="btn sm"
                  disabled={!insights.viewerPermissions.canShareCharts}
                  title="Chart sharing is implemented in a later phase."
                  type="button"
                >
                  Share
                </button>
              </div>
            </div>

            <div style={{ padding: 18 }}>
              <form
                action={projectInsightsHref(scope, owner, projectNumber)}
                className="mb-4 flex flex-wrap items-end gap-2"
                method="get"
              >
                <input name="chart" type="hidden" value={currentChartId} />
                <input name="range" type="hidden" value={insights.range.key} />
                {insights.range.key === "custom" ? (
                  <>
                    <input
                      name="start"
                      type="hidden"
                      value={insights.range.start}
                    />
                    <input
                      name="end"
                      type="hidden"
                      value={insights.range.end}
                    />
                  </>
                ) : null}
                {showingTable ? (
                  <input name="table" type="hidden" value="true" />
                ) : null}
                <label className="min-w-[240px] flex-1">
                  <span className="t-label mb-1 block">Filter</span>
                  <input
                    className="input w-full"
                    defaultValue={insights.filter.query ?? ""}
                    name="filter"
                    placeholder="is:open label:bug assignee:@me"
                  />
                </label>
                <button className="btn" type="submit">
                  Apply filter
                </button>
              </form>

              <div className="mb-4 flex flex-wrap items-center gap-2">
                <span className="t-label">Range</span>
                {insights.range.options.map((option) => (
                  <Link
                    aria-current={option.active ? "page" : undefined}
                    className={`chip ${option.active ? "active" : "soft"}`}
                    href={projectInsightsHref(
                      scope,
                      owner,
                      projectNumber,
                      rangeQuery(insights, option.key),
                    )}
                    key={option.key}
                  >
                    {option.label}
                  </Link>
                ))}
                <details className="relative">
                  <summary className="chip soft cursor-pointer list-none">
                    Custom range
                  </summary>
                  <form
                    action={projectInsightsHref(scope, owner, projectNumber)}
                    className="card absolute right-0 z-10 mt-2 w-[280px] p-3"
                    method="get"
                  >
                    <input name="chart" type="hidden" value={currentChartId} />
                    <input name="range" type="hidden" value="custom" />
                    {insights.filter.query ? (
                      <input
                        name="filter"
                        type="hidden"
                        value={insights.filter.query}
                      />
                    ) : null}
                    {showingTable ? (
                      <input name="table" type="hidden" value="true" />
                    ) : null}
                    <label className="block">
                      <span className="t-label mb-1 block">Start date</span>
                      <input
                        className="input w-full"
                        defaultValue={insights.range.start}
                        name="start"
                        required
                        type="date"
                      />
                    </label>
                    <label className="mt-3 block">
                      <span className="t-label mb-1 block">End date</span>
                      <input
                        className="input w-full"
                        defaultValue={insights.range.end}
                        name="end"
                        required
                        type="date"
                      />
                    </label>
                    <button className="btn sm mt-3 w-full" type="submit">
                      Apply dates
                    </button>
                  </form>
                </details>
                <span className="t-xs ml-auto">
                  {insights.matchingItemCount} matching items ·{" "}
                  {formatDate(insights.range.start)} to{" "}
                  {formatDate(insights.range.end)}
                </span>
              </div>

              {!hasMatches ? (
                <div className="card mb-4 p-4">
                  <div className="t-label">No matching items</div>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    Adjust the filter or date range to include project items in
                    this chart.
                  </p>
                </div>
              ) : null}

              {showingTable ? (
                <div className="card overflow-auto">
                  <table className="w-full min-w-[640px] text-left">
                    <caption className="sr-only">
                      {insights.selectedChart.title} chart data table
                    </caption>
                    <thead>
                      <tr style={{ borderBottom: "1px solid var(--line)" }}>
                        <th className="t-label p-3">Item</th>
                        <th className="t-label p-3">Type</th>
                        <th className="t-label p-3">State</th>
                        <th className="t-label p-3">Repository</th>
                        <th className="t-label p-3">Created</th>
                        <th className="t-label p-3">Completed</th>
                      </tr>
                    </thead>
                    <tbody>
                      {insights.dataRows.length ? (
                        insights.dataRows.map((row) => (
                          <tr
                            key={row.itemId}
                            style={{ borderBottom: "1px solid var(--line)" }}
                          >
                            <td className="t-sm p-3">{row.title}</td>
                            <td className="t-mono-sm p-3">{row.itemType}</td>
                            <td className="t-sm p-3">{row.state ?? "open"}</td>
                            <td className="t-sm p-3">
                              {row.repository?.fullName ?? "Project draft"}
                            </td>
                            <td className="t-sm p-3">
                              {formatDate(row.createdAt)}
                            </td>
                            <td className="t-sm p-3">
                              {row.completedAt
                                ? formatDate(row.completedAt)
                                : "Not completed"}
                            </td>
                          </tr>
                        ))
                      ) : (
                        <tr>
                          <td
                            className="t-sm p-4"
                            colSpan={6}
                            style={{ color: "var(--ink-3)" }}
                          >
                            No chart rows match the selected filters.
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              ) : (
                <div
                  aria-label={`${insights.selectedChart.title} chart`}
                  className="card"
                  role="img"
                  style={{
                    padding: 18,
                    background: "var(--surface-2)",
                    minHeight: 300,
                  }}
                >
                  <div
                    style={{
                      display: "grid",
                      gridTemplateColumns: `repeat(${Math.max(
                        1,
                        insights.series[0]?.points.length ?? 1,
                      )}, minmax(14px, 1fr))`,
                      gap: 8,
                      alignItems: "end",
                      minHeight: 220,
                    }}
                  >
                    {(insights.series[0]?.points ?? []).map(
                      (point, pointIndex) => (
                        <div
                          key={point.date}
                          style={{
                            display: "flex",
                            alignItems: "end",
                            gap: 3,
                            height: 220,
                          }}
                        >
                          {insights.series.map((series, seriesIndex) => {
                            const seriesPoint =
                              series.points[pointIndex] ?? point;
                            return (
                              <span
                                key={series.id}
                                title={`${series.name}: ${seriesPoint.value}`}
                                style={{
                                  background: seriesColor(seriesIndex),
                                  borderRadius:
                                    "var(--radius) var(--radius) 0 0",
                                  display: "block",
                                  flex: 1,
                                  minHeight: 2,
                                  height: `${Math.max(
                                    2,
                                    (seriesPoint.value / maxValue) * 100,
                                  )}%`,
                                }}
                              />
                            );
                          })}
                        </div>
                      ),
                    )}
                  </div>
                  <div className="mt-4 flex flex-wrap gap-3">
                    {insights.series.map((series, index) => (
                      <span className="t-xs" key={series.id}>
                        <span
                          aria-hidden="true"
                          style={{
                            background: seriesColor(index),
                            borderRadius: "var(--radius-pill)",
                            display: "inline-block",
                            height: 8,
                            marginRight: 6,
                            width: 8,
                          }}
                        />
                        {series.name}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className="mt-4 flex flex-wrap items-center gap-2">
                <Link
                  className="btn sm"
                  href={projectInsightsHref(scope, owner, projectNumber, {
                    ...activeQuery(insights),
                    table: !showingTable,
                  })}
                >
                  {showingTable ? "View as chart" : "View as data table"}
                </Link>
                <span className="t-xs">
                  Cache computed {formatDateTime(insights.cache.computedAt)}
                  {insights.cache.stale ? " · stale" : ""}
                </span>
              </div>
            </div>
          </div>

          {status ? (
            <div className="card mt-4 p-4">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <div className="t-label">Latest project status</div>
                  <h3 className="t-h3 mt-1">{status.label}</h3>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                    {status.body ?? "No status message was published."}
                  </p>
                </div>
                <span className="chip soft">
                  {status.label} · {formatDate(status.createdAt)}
                </span>
              </div>
            </div>
          ) : null}

          {insights.unavailableReason ? (
            <div className="chip warn mt-4">{insights.unavailableReason}</div>
          ) : null}
        </section>
      </div>
    </main>
  );
}
