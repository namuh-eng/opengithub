import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { NextRequest } from "next/server";
import { afterEach, describe, expect, it, vi } from "vitest";
import { POST as dismissHintRoute } from "@/app/dashboard/onboarding/hints/[hintKey]/route";
import { DashboardOnboarding } from "@/components/DashboardOnboarding";
import type {
  DashboardActivityItem,
  DashboardFeedEvent,
  DashboardFeedPreferences,
  DashboardHintDismissal,
  DashboardIssueSummary,
  DashboardReviewRequest,
  DashboardSummary,
  DashboardTopRepository,
  RepositorySummary,
} from "@/lib/api";
import {
  dashboardSummaryPath,
  resetDashboardFeedPreferences,
  saveDashboardFeedPreferences,
} from "@/lib/api";

const user = {
  id: "user-1",
  email: "mona@example.com",
  display_name: "Mona",
  avatar_url: null,
};

function repository(overrides: Partial<RepositorySummary> = {}) {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Repository collaboration workspace",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-30T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    ...overrides,
  } satisfies RepositorySummary;
}

function topRepository(
  overrides: Partial<DashboardTopRepository> = {},
): DashboardTopRepository {
  return {
    ownerLogin: "mona",
    name: "octo-app",
    visibility: "public",
    primaryLanguage: "TypeScript",
    primaryLanguageColor: "#3178c6",
    updatedAt: "2026-04-30T00:00:00Z",
    lastVisitedAt: null,
    href: "/mona/octo-app",
    ...overrides,
  };
}

function dismissedHint(hintKey: string): DashboardHintDismissal {
  return {
    id: `dismissal-${hintKey}`,
    userId: "user-1",
    hintKey,
    dismissedAt: "2026-04-30T00:00:00Z",
  };
}

function feedEvent(
  overrides: Partial<DashboardFeedEvent> = {},
): DashboardFeedEvent {
  return {
    id: "feed-event-1",
    eventType: "push",
    title: "Pushed dashboard feed changes",
    excerpt: "Updated the dashboard feed controls",
    occurredAt: "2026-04-30T12:30:00Z",
    actorLogin: "mona",
    actorAvatarUrl: null,
    repositoryName: "mona/octo-app",
    repositoryHref: "/mona/octo-app",
    targetHref: "/mona/octo-app/commit/abc123",
    actionSummary: "mona pushed to mona/octo-app",
    ...overrides,
  };
}

function assignedIssue(
  overrides: Partial<DashboardIssueSummary> = {},
): DashboardIssueSummary {
  return {
    id: "assigned-issue-1",
    title: "Fix failing setup workflow",
    repositoryName: "mona/octo-app",
    number: 11,
    href: "/mona/octo-app/issues/11",
    updatedAt: "2026-04-30T11:00:00Z",
    ...overrides,
  };
}

function recentActivity(
  overrides: Partial<DashboardActivityItem> = {},
): DashboardActivityItem {
  return {
    id: "activity-1",
    kind: "issue",
    title: "Fix failing setup workflow",
    number: 11,
    state: "open",
    repositoryName: "mona/octo-app",
    repositoryHref: "/mona/octo-app",
    href: "/mona/octo-app/issues/11",
    occurredAt: "2026-04-30T11:00:00Z",
    description: "commented on issue #11",
    actorLogin: "mona",
    actorAvatarUrl: null,
    ...overrides,
  };
}

function reviewRequest(
  overrides: Partial<DashboardReviewRequest> = {},
): DashboardReviewRequest {
  return {
    id: "review-request-1",
    title: "Add dashboard activity feed",
    repositoryName: "mona/octo-app",
    number: 12,
    href: "/mona/octo-app/pull/12",
    updatedAt: "2026-04-30T10:00:00Z",
    ...overrides,
  };
}

function dashboardSummary({
  repositories = [],
  topRepositories,
  recentActivity = [],
  assignedIssues = [],
  reviewRequests = [],
  dismissedHints = [],
  feedEvents = [],
  feedPreferences = { feedTab: "following", eventTypes: [] },
}: {
  repositories?: RepositorySummary[];
  topRepositories?: DashboardTopRepository[];
  recentActivity?: DashboardSummary["recentActivity"];
  assignedIssues?: DashboardIssueSummary[];
  reviewRequests?: DashboardReviewRequest[];
  dismissedHints?: DashboardHintDismissal[];
  feedEvents?: DashboardFeedEvent[];
  feedPreferences?: DashboardFeedPreferences;
} = {}): DashboardSummary {
  const sidebarRepositories =
    topRepositories ??
    repositories.map((item) =>
      topRepository({
        ownerLogin: item.owner_login,
        name: item.name,
        visibility: item.visibility,
        primaryLanguage: null,
        primaryLanguageColor: null,
        updatedAt: item.updated_at,
        href: `/${item.owner_login}/${item.name}`,
      }),
    );

  return {
    user,
    repositories: {
      items: repositories,
      total: repositories.length,
      page: 1,
      pageSize: 30,
    },
    topRepositories: {
      items: sidebarRepositories,
      total: sidebarRepositories.length,
      page: 1,
      pageSize: 30,
    },
    hasRepositories: repositories.length > 0,
    recentActivity,
    feedEvents,
    feedPreferences,
    supportedFeedEventTypes: [
      "star",
      "follow",
      "repository_create",
      "help_wanted_issue",
      "help_wanted_pull_request",
      "push",
      "fork",
      "release",
    ],
    assignedIssues,
    reviewRequests,
    dismissedHints,
  };
}

describe("dashboard API types", () => {
  it("keeps the sidebar top repository contract camel-cased", () => {
    const summary = dashboardSummary({
      repositories: [repository()],
      topRepositories: [
        topRepository({ lastVisitedAt: "2026-04-30T12:00:00Z" }),
      ],
    });

    expect(summary.topRepositories.items[0]).toEqual({
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      primaryLanguage: "TypeScript",
      primaryLanguageColor: "#3178c6",
      updatedAt: "2026-04-30T00:00:00Z",
      lastVisitedAt: "2026-04-30T12:00:00Z",
      href: "/mona/octo-app",
    });
  });

  it("keeps feed preferences and event contracts camel-cased", () => {
    const summary = dashboardSummary({
      feedEvents: [feedEvent({ eventType: "release" })],
      feedPreferences: { feedTab: "for_you", eventTypes: ["release", "fork"] },
    });

    expect(summary.feedPreferences).toEqual({
      feedTab: "for_you",
      eventTypes: ["release", "fork"],
    });
    expect(summary.feedEvents[0]).toMatchObject({
      eventType: "release",
      actorLogin: "mona",
      repositoryHref: "/mona/octo-app",
      targetHref: "/mona/octo-app/commit/abc123",
      actionSummary: "mona pushed to mona/octo-app",
    });
    expect(summary.supportedFeedEventTypes).toContain("help_wanted_issue");
  });

  it("builds dashboard summary URLs with shareable feed filters", () => {
    expect(
      dashboardSummaryPath({
        feedTab: "for_you",
        eventTypes: ["release", "fork"],
        repositoryFilter: " octo ",
      }),
    ).toBe(
      "/api/dashboard?feedTab=for_you&eventType=release&eventType=fork&repositoryFilter=octo",
    );
    expect(dashboardSummaryPath()).toBe("/api/dashboard");
  });

  it("writes and resets dashboard feed preferences through the Rust API", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ feedTab: "for_you", eventTypes: ["release"] }),
          { status: 200 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            feedPreferences: { feedTab: "following", eventTypes: [] },
          }),
          { status: 200 },
        ),
      );
    vi.stubGlobal("fetch", fetchMock);
    vi.stubEnv("API_URL", "http://api.local");

    await expect(
      saveDashboardFeedPreferences("__Host-session=signed", {
        feedTab: "for_you",
        eventTypes: ["release"],
      }),
    ).resolves.toEqual({ feedTab: "for_you", eventTypes: ["release"] });
    await expect(
      resetDashboardFeedPreferences("__Host-session=signed"),
    ).resolves.toEqual({ feedTab: "following", eventTypes: [] });

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "http://api.local/api/dashboard/feed-preferences",
      {
        method: "PUT",
        headers: {
          "content-type": "application/json",
          cookie: "__Host-session=signed",
        },
        body: JSON.stringify({
          feedTab: "for_you",
          eventTypes: ["release"],
        }),
        cache: "no-store",
      },
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "http://api.local/api/dashboard/feed-preferences",
      {
        method: "DELETE",
        headers: { cookie: "__Host-session=signed" },
        cache: "no-store",
      },
    );
  });
});

describe("dashboard onboarding", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.unstubAllGlobals();
  });

  it("renders the zero-repository empty state with working CTAs", () => {
    render(<DashboardOnboarding summary={dashboardSummary()} />);

    expect(
      screen.getByRole("heading", { name: "Start building on opengithub" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Find a repository")).toHaveAttribute(
      "type",
      "search",
    );
    expect(
      screen.getByText("You do not have any repositories yet."),
    ).toBeInTheDocument();
    for (const link of screen.getAllByRole("link", {
      name: "Create repository",
    })) {
      expect(link).toHaveAttribute("href", "/new");
    }
    for (const link of screen.getAllByRole("link", {
      name: "Import repository",
    })) {
      expect(link).toHaveAttribute("href", "/new/import");
    }
    for (const link of screen.getAllByRole("link", {
      name: "Read setup guide",
    })) {
      expect(link).toHaveAttribute("href", "/docs/get-started");
    }
    expect(screen.getByRole("link", { name: "New" })).toHaveAttribute(
      "href",
      "/new",
    );
    expect(screen.getAllByRole("button", { name: "Dismiss" })).toHaveLength(3);
  });

  it("does not leave inert links or unnamed buttons in the empty state", () => {
    const { container } = render(
      <DashboardOnboarding summary={dashboardSummary()} />,
    );

    const hrefs = [...container.querySelectorAll("a")].map((link) =>
      link.getAttribute("href"),
    );
    expect(hrefs).not.toContain("#");
    for (const button of container.querySelectorAll("button")) {
      expect(button).toHaveTextContent(/dismiss/i);
    }
  });

  it("persists dismissed hints through the dashboard proxy route", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response("{}", { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);

    render(<DashboardOnboarding summary={dashboardSummary()} />);

    fireEvent.click(
      screen.getAllByRole("button", { name: "Dismiss" })[0] as HTMLElement,
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/dashboard/onboarding/hints/create-repository",
        { method: "POST" },
      ),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Hint dismissed.");
    expect(
      screen.queryByRole("heading", { name: "Create your first repository" }),
    ).not.toBeInTheDocument();
  });

  it("shows an error and keeps the hint when dismissal fails", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response("{}", { status: 500 })),
    );

    render(<DashboardOnboarding summary={dashboardSummary()} />);

    fireEvent.click(
      screen.getAllByRole("button", { name: "Dismiss" })[0] as HTMLElement,
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "This hint could not be dismissed. Try again.",
      ),
    );
    expect(
      screen.getByRole("heading", { name: "Create your first repository" }),
    ).toBeInTheDocument();
  });

  it("forwards hint dismissal through the Next.js route with the session cookie", async () => {
    vi.stubEnv("API_URL", "http://api.example.test");
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ hintKey: "read-guide" }), {
        status: 200,
        headers: { "content-type": "application/json" },
      }),
    );
    vi.stubGlobal("fetch", fetchMock);

    const response = await dismissHintRoute(
      {
        headers: new Headers({ cookie: "__Host-session=signed" }),
      } as NextRequest,
      {
        params: Promise.resolve({ hintKey: "read-guide" }),
      },
    );

    expect(response.status).toBe(200);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://api.example.test/api/dashboard/onboarding/hints/read-guide",
      {
        method: "POST",
        headers: { cookie: "__Host-session=signed" },
        cache: "no-store",
      },
    );
    await expect(response.json()).resolves.toEqual({ hintKey: "read-guide" });
  });

  it("honors previously dismissed hints from the dashboard API", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          dismissedHints: [dismissedHint("create-repository")],
        })}
      />,
    );

    expect(
      screen.queryByRole("heading", { name: "Create your first repository" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Import an existing project" }),
    ).toBeInTheDocument();
  });

  it("renders repository rows and removes first-run welcome copy when repositories exist", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
        })}
      />,
    );

    expect(
      screen.queryByRole("heading", { name: "Start building on opengithub" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /mona\/octo-app.*public/i }),
    ).toHaveAttribute("href", "/mona/octo-app");
    expect(screen.getByText("TypeScript")).toBeInTheDocument();
    expect(screen.getByText("Updated Apr 30")).toBeInTheDocument();
    expect(screen.getByText("public")).toBeInTheDocument();
    expect(screen.getByText("Dashboard feed")).toBeInTheDocument();
    expect(screen.getByText("Assigned issues")).toBeInTheDocument();
    expect(screen.getByText("Review requests")).toBeInTheDocument();
  });

  it("renders feed tabs, filter controls, and empty-state actions", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
        })}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Dashboard feed" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Following" })).toHaveAttribute(
      "href",
      "/dashboard?feedTab=following",
    );
    expect(screen.getByRole("tab", { name: "Following" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "For you" })).toHaveAttribute(
      "href",
      "/dashboard?feedTab=for_you",
    );
    fireEvent.click(screen.getByText("Filter"));
    expect(screen.getByLabelText("Pushes")).toHaveAttribute("value", "push");
    expect(screen.getByLabelText("Releases")).toHaveAttribute(
      "value",
      "release",
    );
    expect(screen.getByRole("button", { name: "Apply" })).toHaveAttribute(
      "type",
      "submit",
    );
    for (const link of screen.getAllByRole("link", {
      name: "Clear filters",
    })) {
      expect(link).toHaveAttribute("href", "/dashboard?feedTab=following");
    }
    expect(
      screen.getByText("No dashboard feed events match the current filters."),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("link", { name: "Create repository" })[0],
    ).toHaveAttribute("href", "/new");
    expect(
      screen.getAllByRole("link", { name: "Explore repositories" })[0],
    ).toHaveAttribute("href", "/explore");
  });

  it("renders selected feed filters and event cards from dashboard API rows", () => {
    render(
      <DashboardOnboarding
        activeEventTypes={["release", "fork"]}
        activeFeedTab="for_you"
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
          feedEvents: [
            feedEvent({
              id: "event-1",
              eventType: "release",
              title: "Published dashboard feed preview",
              targetHref: "/mona/octo-app/releases/tag/v0.1.0",
              actionSummary: "mona released mona/octo-app",
            }),
            feedEvent({
              id: "event-2",
              eventType: "fork",
              title: "Forked dashboard controls",
              targetHref: "/mona/octo-app/network/members",
              actionSummary: "mona forked mona/octo-app",
            }),
          ],
          assignedIssues: [assignedIssue()],
          reviewRequests: [reviewRequest()],
        })}
      />,
    );

    expect(
      screen.queryByRole("heading", { name: "Start building on opengithub" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "For you" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "Following" })).toHaveAttribute(
      "href",
      "/dashboard?feedTab=following&eventType=release&eventType=fork",
    );
    fireEvent.click(screen.getByText("Filter"));
    expect(screen.getByLabelText("Releases")).toBeChecked();
    expect(screen.getByLabelText("Forks")).toBeChecked();
    expect(screen.getByLabelText("Pushes")).not.toBeChecked();
    expect(
      screen.getByRole("link", { name: "Published dashboard feed preview" }),
    ).toHaveAttribute("href", "/mona/octo-app/releases/tag/v0.1.0");
    expect(screen.getByText("mona released mona/octo-app")).toBeInTheDocument();
    expect(screen.getAllByText("Releases").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "Forked dashboard controls" }),
    ).toHaveAttribute("href", "/mona/octo-app/network/members");
    expect(screen.getAllByText("Forks").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "Fix failing setup workflow" }),
    ).toHaveAttribute("href", "/mona/octo-app/issues/11");
    expect(
      screen.getByText(/mona\/octo-app #11 .* Assigned Apr 30/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Add dashboard activity feed" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/12");
    expect(
      screen.getByText(/mona\/octo-app #12 .* Review requested Apr 30/i),
    ).toBeInTheDocument();
  });

  it("renders recent issue and pull request activity above the dashboard feed", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
          recentActivity: [
            recentActivity(),
            recentActivity({
              id: "activity-2",
              kind: "pull_request",
              title: "Add dashboard activity feed",
              number: 12,
              state: "merged",
              href: "/mona/octo-app/pull/12",
              description: "merged pull request #12",
              actorLogin: "octocat",
            }),
          ],
        })}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Recent activity" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Fix failing setup workflow" }),
    ).toHaveAttribute("href", "/mona/octo-app/issues/11");
    expect(
      screen.getByRole("link", { name: "Add dashboard activity feed" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/12");
    expect(screen.getByText("#11")).toBeInTheDocument();
    expect(screen.getByText("#12")).toBeInTheDocument();
    expect(screen.getByText("open")).toBeInTheDocument();
    expect(screen.getByText("merged")).toBeInTheDocument();
    expect(screen.getByText("mona")).toBeInTheDocument();
    expect(screen.getByText("octocat")).toBeInTheDocument();
  });

  it("renders recent activity empty-state actions", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
          recentActivity: [],
        })}
      />,
    );

    expect(
      screen.getByText("There is no recent activity involving you yet."),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("link", { name: "Create repository" })[0],
    ).toHaveAttribute("href", "/new");
    expect(
      screen.getAllByRole("link", { name: "Explore repositories" })[0],
    ).toHaveAttribute("href", "/explore");
  });

  it("filters top repositories client-side without changing the New destination", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [
            topRepository({
              ownerLogin: "mona",
              name: "octo-app",
              href: "/mona/octo-app",
              primaryLanguage: "TypeScript",
              primaryLanguageColor: "#3178c6",
            }),
            topRepository({
              ownerLogin: "octo-org",
              name: "infra",
              href: "/octo-org/infra",
              visibility: "private",
              primaryLanguage: "Rust",
              primaryLanguageColor: "#dea584",
            }),
          ],
        })}
      />,
    );

    const filter = screen.getByLabelText("Find a repository");
    fireEvent.change(filter, { target: { value: "infra" } });

    expect(
      screen.queryByRole("link", { name: /mona\/octo-app.*public/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /octo-org\/infra.*private/i }),
    ).toHaveAttribute("href", "/octo-org/infra");
    expect(screen.getByText("Rust")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "New" })).toHaveAttribute(
      "href",
      "/new",
    );
  });

  it("shows an empty filter result without hiding the sidebar controls", () => {
    render(
      <DashboardOnboarding
        summary={dashboardSummary({
          repositories: [repository()],
          topRepositories: [topRepository()],
        })}
      />,
    );

    fireEvent.change(screen.getByLabelText("Find a repository"), {
      target: { value: "does-not-exist" },
    });

    expect(
      screen.getByText("No repositories match your filter."),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "New" })).toHaveAttribute(
      "href",
      "/new",
    );
  });
});
