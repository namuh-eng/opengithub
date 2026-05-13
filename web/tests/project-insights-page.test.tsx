import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectInsightsPage } from "@/components/ProjectInsightsPage";
import type { ProjectInsights } from "@/lib/api";

function insights(overrides: Partial<ProjectInsights> = {}): ProjectInsights {
  return {
    project: {
      id: "project-1",
      number: 12,
      title: "Editorial planning",
      description: "Tracks the launch plan.",
      state: "open",
      visibility: "private",
      owner: "namuh",
      href: "/orgs/namuh/projects/12",
      workspaceHref: "/orgs/namuh/projects/12/views/1",
      viewerRole: "admin",
    },
    navigation: {
      returnHref: "/orgs/namuh/projects/12/views/1",
      insightsHref: "/orgs/namuh/projects/12/insights",
      selectedItem: "insights",
    },
    selectedChart: {
      id: "burn-up",
      title: "Burn up",
      description: "Completed items against total scope.",
      chartType: "burn_up",
      href: "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m",
      shareHref: "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m",
      visibility: "project",
      sharedWithViewers: true,
      updatedAt: "2026-05-05T00:00:00Z",
      isDefault: true,
      configuration: { y: "count" },
    },
    defaultCharts: [
      {
        id: "burn-up",
        title: "Burn up",
        description: "Completed items against total scope.",
        chartType: "burn_up",
        href: "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m",
        shareHref: "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m",
        visibility: "project",
        sharedWithViewers: true,
        updatedAt: "2026-05-05T00:00:00Z",
      },
    ],
    customCharts: [
      {
        id: "chart-1",
        title: "Open bugs by team",
        description: "Tracks bug load by assignee team.",
        chartType: "bar",
        href: "/orgs/namuh/projects/12/insights?chart=chart-1&range=1m",
        shareHref: "/orgs/namuh/projects/12/insights?chart=chart-1&range=1m",
        visibility: "project",
        sharedWithViewers: true,
        updatedAt: "2026-05-05T00:00:00Z",
      },
    ],
    range: {
      key: "1m",
      label: "1 month",
      start: "2026-04-06",
      end: "2026-05-06",
      options: [
        {
          key: "2w",
          label: "2 weeks",
          href: "/orgs/namuh/projects/12/insights?range=2w",
          active: false,
        },
        {
          key: "1m",
          label: "1 month",
          href: "/orgs/namuh/projects/12/insights?range=1m",
          active: true,
        },
        {
          key: "3m",
          label: "3 months",
          href: "/orgs/namuh/projects/12/insights?range=3m",
          active: false,
        },
        {
          key: "max",
          label: "Max",
          href: "/orgs/namuh/projects/12/insights?range=max",
          active: false,
        },
      ],
    },
    filter: {
      query: "is:open",
      tokens: ["is:open"],
      unsupportedTokens: [],
    },
    matchingItemCount: 3,
    series: [
      {
        id: "completed",
        name: "Completed",
        color: "accent",
        points: [
          { date: "2026-04-06", value: 1 },
          { date: "2026-05-06", value: 2 },
        ],
      },
      {
        id: "total",
        name: "Total",
        color: "ink",
        points: [
          { date: "2026-04-06", value: 2 },
          { date: "2026-05-06", value: 4 },
        ],
      },
    ],
    dataRows: [
      {
        itemId: "item-1",
        itemType: "issue",
        title: "Ship Insights shell",
        state: "open",
        repository: {
          id: "repo-1",
          owner: "namuh",
          name: "opengithub",
          fullName: "namuh/opengithub",
          href: "/namuh/opengithub",
        },
        createdAt: "2026-04-06T00:00:00Z",
        completedAt: null,
      },
    ],
    latestStatus: {
      status: "at_risk",
      label: "At risk",
      body: "Charts need browser coverage.",
      createdAt: "2026-05-06T00:00:00Z",
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "admin",
      canViewInsights: true,
      canCreateCharts: true,
      canEditCharts: false,
      canDeleteCharts: false,
      canShareCharts: false,
      canViewStatus: true,
    },
    cache: {
      cacheKey: "project-1:burn-up:1m:is-open",
      computedAt: "2026-05-06T00:00:00Z",
      stale: false,
      version: 2,
    },
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectInsightsPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders organization Insights with selected project navigation and chart sidebar", () => {
    render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(
      screen.getByRole("link", { name: "Return to project view" }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "View" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1",
    );
    expect(screen.getByRole("link", { name: "Insights" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings",
    );
    expect(
      screen.getByRole("heading", { name: "Editorial planning" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Open bugs by team/i }),
    ).toBeVisible();
  });

  it("renders user project routes without organization prefixes", () => {
    render(
      <ProjectInsightsPage insights={insights()} owner="ashley" scope="user" />,
    );

    expect(
      screen.getByRole("link", { name: "Return to project view" }),
    ).toHaveAttribute("href", "/ashley/projects/12/views/1");
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/ashley/projects/12/settings",
    );
  });

  it("renders chart semantics, range links, filter form, status, and table link", () => {
    render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(screen.getByRole("img", { name: "Burn up chart" })).toBeVisible();
    expect(
      screen.getByPlaceholderText("is:open label:bug assignee:@me"),
    ).toHaveValue("is:open");
    expect(screen.getByRole("link", { name: "2 weeks" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/insights?chart=burn-up&range=2w&filter=is%3Aopen",
    );
    expect(screen.getByText(/3\s+matching items/)).toBeVisible();
    expect(screen.getByText("Charts need browser coverage.")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "View as data table" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m&filter=is%3Aopen&table=true",
    );
  });

  it("renders the accessible data table and preserves table state in exploration links", () => {
    render(
      <ProjectInsightsPage
        insights={insights({
          selectedChart: {
            ...insights().selectedChart,
            configuration: { table: true },
          },
        })}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(
      screen.getByRole("table", { name: "Burn up chart data table" }),
    ).toBeVisible();
    expect(
      screen.getByRole("cell", { name: "Ship Insights shell" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "2 weeks" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/insights?chart=burn-up&range=2w&filter=is%3Aopen&table=true",
    );
    expect(screen.getByRole("link", { name: "View as chart" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/insights?chart=burn-up&range=1m&filter=is%3Aopen",
    );
  });

  it("syncs route-backed table exploration when server props change", async () => {
    const { rerender } = render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(screen.getByRole("img", { name: "Burn up chart" })).toBeVisible();

    rerender(
      <ProjectInsightsPage
        insights={insights({
          selectedChart: {
            ...insights().selectedChart,
            configuration: { table: true },
          },
        })}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(
      await screen.findByRole("table", { name: "Burn up chart data table" }),
    ).toBeVisible();
  });

  it("renders custom date selection as a real route-backed form", () => {
    render(
      <ProjectInsightsPage
        insights={insights({
          range: {
            ...insights().range,
            key: "custom",
            label: "Custom",
            start: "2026-04-01",
            end: "2026-04-15",
            options: insights().range.options.map((option) => ({
              ...option,
              active: false,
            })),
          },
        })}
        owner="namuh"
        scope="organization"
      />,
    );

    fireEvent.click(screen.getByText("Custom range"));
    expect(screen.getByLabelText("Start date")).toHaveValue("2026-04-01");
    expect(screen.getByLabelText("End date")).toHaveValue("2026-04-15");
    expect(screen.getByRole("button", { name: "Apply dates" })).toHaveAttribute(
      "type",
      "submit",
    );
  });

  it("renders a no-match state without dropping the chart controls", () => {
    render(
      <ProjectInsightsPage
        insights={insights({
          matchingItemCount: 0,
          dataRows: [],
        })}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(screen.getByText("No matching items")).toBeVisible();
    expect(screen.getByRole("img", { name: "Burn up chart" })).toBeVisible();
  });

  it("keeps reader mutation controls disabled with visible capability state", () => {
    render(
      <ProjectInsightsPage
        insights={insights({
          customCharts: [],
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canViewInsights: true,
            canCreateCharts: false,
            canEditCharts: false,
            canDeleteCharts: false,
            canShareCharts: false,
            canViewStatus: true,
          },
        })}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(screen.getByText("New")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText("Edit")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByRole("button", { name: "Share" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Delete" })).toBeDisabled();
    expect(screen.getByText("No custom charts yet.")).toBeVisible();
  });

  it("copies a viewer-safe share link that preserves chart exploration state", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });

    render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    expect(screen.getByText("Shared with project viewers")).toBeVisible();
    expect(screen.getByText("Cache v2")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Share" }));
    await screen.findByText("Share link copied.");
    expect(writeText).toHaveBeenCalledWith(
      "http://localhost:3000/orgs/namuh/projects/12/insights?chart=burn-up&range=1m&filter=is%3Aopen",
    );
  });

  it("posts create, edit, and delete chart mutations through real API routes", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    fetchMock
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify(
            insights({
              viewerPermissions: {
                ...insights().viewerPermissions,
                canEditCharts: true,
                canDeleteCharts: true,
                canShareCharts: true,
              },
              selectedChart: {
                ...insights().selectedChart,
                id: "chart-created",
                title: "Closed issue trend",
                chartType: "line",
                isDefault: false,
                updatedAt: "2026-05-06T01:00:00Z",
              },
            }),
          ),
          { status: 201 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify(
            insights({
              viewerPermissions: {
                ...insights().viewerPermissions,
                canEditCharts: true,
                canDeleteCharts: true,
                canShareCharts: true,
              },
              selectedChart: {
                ...insights().selectedChart,
                id: "chart-created",
                title: "Closed issue trend by type",
                chartType: "bar",
                isDefault: false,
                updatedAt: "2026-05-06T02:00:00Z",
              },
            }),
          ),
          { status: 200 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(insights()), { status: 200 }),
      );

    render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    fireEvent.click(screen.getByText("New"));
    fireEvent.change(screen.getAllByLabelText("Title")[0], {
      target: { value: "Closed issue trend" },
    });
    fireEvent.change(screen.getAllByLabelText("Chart type")[0], {
      target: { value: "line" },
    });
    fireEvent.change(screen.getAllByLabelText("Visibility")[0], {
      target: { value: "project" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create chart" }));
    await screen.findByText("Chart created.");
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/charts",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining("Closed issue trend"),
      }),
    );

    fireEvent.click(screen.getByText("Edit"));
    const editTitleInputs = screen.getAllByLabelText("Title");
    fireEvent.change(editTitleInputs[editTitleInputs.length - 1], {
      target: { value: "Closed issue trend by type" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save chart" }));
    await screen.findByText("Chart saved.");
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/charts/chart-created",
      expect.objectContaining({
        method: "PATCH",
        body: expect.stringContaining("Closed issue trend by type"),
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await screen.findByText("Chart deleted.");
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/charts/chart-created",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  it("uses real filter form controls and avoids placeholder links or banned visual tokens", () => {
    const { container } = render(
      <ProjectInsightsPage
        insights={insights()}
        owner="namuh"
        scope="organization"
      />,
    );

    fireEvent.change(
      screen.getByPlaceholderText("is:open label:bug assignee:@me"),
      {
        target: { value: "is:closed" },
      },
    );
    expect(
      screen.getByRole("button", { name: "Apply filter" }),
    ).toHaveAttribute("type", "submit");
    expect(screen.getByText("Custom range")).toBeVisible();
    expect(container.querySelector('[href="#"]')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.innerHTML).not.toContain("#1f883d");
    expect(container.innerHTML).not.toContain("#cf222e");
    expect(container.innerHTML).not.toContain("@primer/");
    expect(container.innerHTML).not.toContain("Octicon");
  });
});
