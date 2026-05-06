"use client";

import Link from "next/link";
import { useState } from "react";
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

function stringConfig(value: unknown) {
  return typeof value === "string" ? value : "";
}

function ChartMutationFields({
  chart,
}: {
  chart?: ProjectInsights["selectedChart"];
}) {
  return (
    <>
      <label>
        <span className="t-label mb-1 block">Title</span>
        <input
          className="input w-full"
          defaultValue={chart?.title ?? ""}
          name="title"
          required
        />
      </label>
      <label>
        <span className="t-label mb-1 block">Description</span>
        <input
          className="input w-full"
          defaultValue={chart?.description ?? ""}
          name="description"
        />
      </label>
      <label>
        <span className="t-label mb-1 block">Chart type</span>
        <select
          className="input w-full"
          defaultValue={chart?.chartType ?? "bar"}
          name="chartType"
        >
          <option value="bar">Bar</option>
          <option value="line">Line</option>
          <option value="stacked_area">Stacked area</option>
          <option value="number">Number</option>
          <option value="burn_up">Burn up</option>
        </select>
      </label>
      <label>
        <span className="t-label mb-1 block">Filter</span>
        <input
          className="input w-full"
          defaultValue={stringConfig(chart?.configuration.filter)}
          name="filter"
          placeholder="is:closed type:issue"
        />
      </label>
      <div className="grid grid-cols-3 gap-2">
        <label>
          <span className="t-label mb-1 block">X field</span>
          <input
            className="input w-full"
            defaultValue={stringConfig(chart?.configuration.xFieldId)}
            name="xFieldId"
          />
        </label>
        <label>
          <span className="t-label mb-1 block">Y field</span>
          <input
            className="input w-full"
            defaultValue={stringConfig(chart?.configuration.yFieldId)}
            name="yFieldId"
          />
        </label>
        <label>
          <span className="t-label mb-1 block">Group</span>
          <input
            className="input w-full"
            defaultValue={stringConfig(chart?.configuration.groupFieldId)}
            name="groupFieldId"
          />
        </label>
      </div>
      <label>
        <span className="t-label mb-1 block">Visibility</span>
        <select
          className="input w-full"
          defaultValue={chart?.visibility ?? "private"}
          name="visibility"
        >
          <option value="private">Private to editors</option>
          <option value="project">Visible to project viewers</option>
        </select>
      </label>
    </>
  );
}

export function ProjectInsightsPage({
  insights,
  scope,
  owner,
}: ProjectInsightsPageProps) {
  const [currentInsights, setCurrentInsights] = useState(insights);
  const [mutationState, setMutationState] = useState<{
    status: "idle" | "saving" | "success" | "error";
    message: string | null;
  }>({ status: "idle", message: null });
  const projectNumber = currentInsights.project.number;
  const canCreate = currentInsights.viewerPermissions.canCreateCharts;
  const canEditSelected =
    !currentInsights.selectedChart.isDefault &&
    currentInsights.viewerPermissions.canEditCharts;
  const canDeleteSelected =
    !currentInsights.selectedChart.isDefault &&
    currentInsights.viewerPermissions.canDeleteCharts;
  const maxValue = maxSeriesValue(currentInsights);
  const status = currentInsights.latestStatus;
  const currentChartId = currentInsights.selectedChart.id;
  const showingTable =
    currentInsights.selectedChart.configuration.table === true;
  const hasMatches = currentInsights.matchingItemCount > 0;

  async function submitChartMutation(
    action: "create" | "edit" | "delete",
    formData: FormData,
  ) {
    const chartId = String(formData.get("chartId") ?? "");
    const endpoint =
      action === "create"
        ? `/api/projects/${encodeURIComponent(currentInsights.project.id)}/charts`
        : `/api/projects/${encodeURIComponent(currentInsights.project.id)}/charts/${encodeURIComponent(chartId)}`;
    const body =
      action === "delete"
        ? { expectedUpdatedAt: String(formData.get("expectedUpdatedAt") ?? "") }
        : {
            title: String(formData.get("title") ?? ""),
            description: String(formData.get("description") ?? ""),
            chartType: String(formData.get("chartType") ?? "bar"),
            filter: String(formData.get("filter") ?? ""),
            xFieldId: String(formData.get("xFieldId") ?? ""),
            yFieldId: String(formData.get("yFieldId") ?? ""),
            groupFieldId: String(formData.get("groupFieldId") ?? ""),
            visibility: String(formData.get("visibility") ?? "private"),
            expectedUpdatedAt: String(formData.get("expectedUpdatedAt") ?? ""),
          };
    setMutationState({ status: "saving", message: "Saving chart..." });
    const response = await fetch(endpoint, {
      method:
        action === "create" ? "POST" : action === "edit" ? "PATCH" : "DELETE",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      setMutationState({
        status: "error",
        message:
          payload?.error?.message ??
          (action === "delete"
            ? "Project chart could not be deleted."
            : "Project chart could not be saved."),
      });
      return;
    }
    setCurrentInsights(payload as ProjectInsights);
    setMutationState({
      status: "success",
      message:
        action === "delete"
          ? "Chart deleted."
          : action === "create"
            ? "Chart created."
            : "Chart saved.",
    });
  }

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
          <h1 className="t-h1 mt-1">{currentInsights.project.title}</h1>
          {currentInsights.project.description ? (
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {currentInsights.project.description}
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
            {currentInsights.defaultCharts.map((chart) => (
              <Link
                aria-current={chart.id === currentChartId ? "page" : undefined}
                className="list-row"
                href={projectInsightsHref(
                  scope,
                  owner,
                  projectNumber,
                  chartQuery(currentInsights, chart),
                )}
                key={chart.id}
                style={{
                  borderRadius: "var(--radius)",
                  borderBottom: 0,
                  padding: "10px 8px",
                  backgroundColor:
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
              <details>
                <summary
                  aria-disabled={!canCreate}
                  className="btn sm cursor-pointer list-none"
                  title={
                    canCreate
                      ? "Create a custom chart"
                      : "Your project role cannot create charts."
                  }
                >
                  New
                </summary>
                {canCreate ? (
                  <form
                    action={(formData) =>
                      void submitChartMutation("create", formData)
                    }
                    className="mt-3 grid gap-2"
                  >
                    <ChartMutationFields />
                    <button className="btn sm primary" type="submit">
                      Create chart
                    </button>
                  </form>
                ) : null}
              </details>
            </div>
            {currentInsights.customCharts.length ? (
              currentInsights.customCharts.map((chart) => (
                <Link
                  aria-current={
                    chart.id === currentChartId ? "page" : undefined
                  }
                  className="list-row"
                  href={projectInsightsHref(
                    scope,
                    owner,
                    projectNumber,
                    chartQuery(currentInsights, chart),
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
                <div className="t-label">
                  {currentInsights.selectedChart.isDefault
                    ? "Burn up chart"
                    : "Custom chart"}
                </div>
                <h2 className="t-h2 mt-1">
                  {currentInsights.selectedChart.title}
                </h2>
                {currentInsights.selectedChart.description ? (
                  <p
                    className="t-sm mt-2 max-w-2xl"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {currentInsights.selectedChart.description}
                  </p>
                ) : null}
              </div>
              <div className="flex flex-wrap gap-2">
                <details>
                  <summary
                    aria-disabled={!canEditSelected}
                    className="btn sm cursor-pointer list-none"
                    title={
                      canEditSelected
                        ? "Edit chart"
                        : "Only custom charts can be edited by project writers."
                    }
                  >
                    Edit
                  </summary>
                  {canEditSelected ? (
                    <form
                      action={(formData) =>
                        void submitChartMutation("edit", formData)
                      }
                      className="card absolute right-0 z-10 mt-2 grid w-[300px] gap-2 p-3"
                    >
                      <input
                        name="chartId"
                        type="hidden"
                        value={currentChartId}
                      />
                      <input
                        name="expectedUpdatedAt"
                        type="hidden"
                        value={currentInsights.selectedChart.updatedAt}
                      />
                      <ChartMutationFields
                        chart={currentInsights.selectedChart}
                      />
                      <button className="btn sm primary" type="submit">
                        Save chart
                      </button>
                    </form>
                  ) : null}
                </details>
                <button
                  className="btn sm"
                  disabled={!currentInsights.viewerPermissions.canShareCharts}
                  title="Chart sharing is finalized in the next phase."
                  type="button"
                >
                  Share
                </button>
                <form
                  action={(formData) =>
                    void submitChartMutation("delete", formData)
                  }
                >
                  <input name="chartId" type="hidden" value={currentChartId} />
                  <input
                    name="expectedUpdatedAt"
                    type="hidden"
                    value={currentInsights.selectedChart.updatedAt}
                  />
                  <button
                    className="btn sm"
                    disabled={!canDeleteSelected}
                    type="submit"
                  >
                    Delete
                  </button>
                </form>
              </div>
            </div>
            {mutationState.message ? (
              <div
                className={`chip ${mutationState.status === "error" ? "err" : mutationState.status === "success" ? "ok" : "soft"} m-4`}
                role="status"
              >
                {mutationState.message}
              </div>
            ) : null}

            <div style={{ padding: 18 }}>
              <form
                action={projectInsightsHref(scope, owner, projectNumber)}
                className="mb-4 flex flex-wrap items-end gap-2"
                method="get"
              >
                <input name="chart" type="hidden" value={currentChartId} />
                <input
                  name="range"
                  type="hidden"
                  value={currentInsights.range.key}
                />
                {currentInsights.range.key === "custom" ? (
                  <>
                    <input
                      name="start"
                      type="hidden"
                      value={currentInsights.range.start}
                    />
                    <input
                      name="end"
                      type="hidden"
                      value={currentInsights.range.end}
                    />
                  </>
                ) : null}
                {showingTable ? (
                  <input name="table" type="hidden" value="true" />
                ) : null}
                <label className="min-w-[240px] flex-1">
                  <span className="t-label mb-1 block">Chart filter</span>
                  <input
                    className="input w-full"
                    defaultValue={currentInsights.filter.query ?? ""}
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
                {currentInsights.range.options.map((option) => (
                  <Link
                    aria-current={option.active ? "page" : undefined}
                    className={`chip ${option.active ? "active" : "soft"}`}
                    href={projectInsightsHref(
                      scope,
                      owner,
                      projectNumber,
                      rangeQuery(currentInsights, option.key),
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
                    {currentInsights.filter.query ? (
                      <input
                        name="filter"
                        type="hidden"
                        value={currentInsights.filter.query}
                      />
                    ) : null}
                    {showingTable ? (
                      <input name="table" type="hidden" value="true" />
                    ) : null}
                    <label className="block">
                      <span className="t-label mb-1 block">Start date</span>
                      <input
                        className="input w-full"
                        defaultValue={currentInsights.range.start}
                        name="start"
                        required
                        type="date"
                      />
                    </label>
                    <label className="mt-3 block">
                      <span className="t-label mb-1 block">End date</span>
                      <input
                        className="input w-full"
                        defaultValue={currentInsights.range.end}
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
                  {currentInsights.matchingItemCount} matching items ·{" "}
                  {formatDate(currentInsights.range.start)} to{" "}
                  {formatDate(currentInsights.range.end)}
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
                      {currentInsights.selectedChart.title} chart data table
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
                      {currentInsights.dataRows.length ? (
                        currentInsights.dataRows.map((row) => (
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
                  aria-label={`${currentInsights.selectedChart.title} chart`}
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
                        currentInsights.series[0]?.points.length ?? 1,
                      )}, minmax(14px, 1fr))`,
                      gap: 8,
                      alignItems: "end",
                      minHeight: 220,
                    }}
                  >
                    {(currentInsights.series[0]?.points ?? []).map(
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
                          {currentInsights.series.map((series, seriesIndex) => {
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
                    {currentInsights.series.map((series, index) => (
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
                    ...activeQuery(currentInsights),
                    table: !showingTable,
                  })}
                >
                  {showingTable ? "View as chart" : "View as data table"}
                </Link>
                <span className="t-xs">
                  Cache computed{" "}
                  {formatDateTime(currentInsights.cache.computedAt)}
                  {currentInsights.cache.stale ? " · stale" : ""}
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

          {currentInsights.unavailableReason ? (
            <div className="chip warn mt-4">
              {currentInsights.unavailableReason}
            </div>
          ) : null}
        </section>
      </div>
    </main>
  );
}
