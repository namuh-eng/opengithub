import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { UserProfileOverview } from "@/components/UserProfileOverview";
import type { AuthSession, PublicProfileView } from "@/lib/api";
import { PROFILE_TABS, profileTabHref } from "@/lib/navigation";

const session: AuthSession = {
  authenticated: true,
  user: {
    id: "viewer-1",
    email: "viewer@example.com",
    display_name: "Viewer",
    avatar_url: null,
  },
};

function profile(
  overrides: Partial<PublicProfileView> = {},
): PublicProfileView {
  return {
    identity: {
      id: "user-1",
      login: "ashley",
      displayName: "Ashley Ha",
      avatarUrl: null,
      bio: "Building calm developer tools.",
      company: "Namuh",
      location: "Seoul",
      websiteUrl: "https://namuh.co",
      privateProfile: false,
      followerCount: 42,
      followingCount: 18,
      createdAt: "2026-01-01T00:00:00Z",
    },
    viewer: {
      authenticated: true,
      isSelf: false,
      following: false,
      blocked: false,
      canFollow: true,
      canBlock: true,
      canReport: true,
    },
    tabs: { repositories: 24, projects: 0, packages: 2, stars: 8 },
    readme: {
      body: "# Hello\nI work on OpenGitHub.",
      renderedBody: null,
      updatedAt: "2026-05-01T00:00:00Z",
    },
    pinnedItems: [
      {
        id: "repo-1",
        kind: "repository",
        title: "opengithub",
        description: "A calm forge for code.",
        href: "/ashley/opengithub",
        language: "TypeScript",
        starsCount: 12,
        forksCount: 3,
        updatedAt: "2026-05-01T00:00:00Z",
      },
    ],
    achievements: [
      {
        slug: "first-repository",
        name: "First repository",
        description: "Published a first public repository.",
        awardedAt: "2026-02-01T00:00:00Z",
      },
    ],
    contributions: {
      year: 2026,
      total: 3,
      days: [
        { date: "2026-01-01", count: 0, intensity: 0 },
        { date: "2026-01-02", count: 3, intensity: 2 },
      ],
      recentEvents: [],
    },
    ...overrides,
  };
}

function renderProfile(view = profile()) {
  render(
    <UserProfileOverview
      activeTab="overview"
      hrefForTab={(value) => profileTabHref("ashley", value)}
      profile={view}
      session={session}
      tabs={PROFILE_TABS}
    />,
  );
}

describe("UserProfileOverview", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ following: true, followerCount: 43 })),
    );
  });

  it("renders identity, tabs, pinned items, achievements, and accessible contributions", () => {
    renderProfile();

    expect(
      screen.getByRole("heading", { name: "Ashley Ha" }),
    ).toBeInTheDocument();
    expect(screen.getByText("@ashley")).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /Repositories 24/ }),
    ).toHaveAttribute("href", "/ashley?tab=repositories");
    expect(screen.getByRole("link", { name: "opengithub" })).toHaveAttribute(
      "href",
      "/ashley/opengithub",
    );
    expect(screen.getByText("First repository")).toBeInTheDocument();
    expect(
      screen.getByText("3 contributions on Jan 2, 2026"),
    ).toBeInTheDocument();
  });

  it("optimistically follows with rollback-ready same-origin route", async () => {
    renderProfile();

    fireEvent.click(screen.getByRole("button", { name: "Follow" }));

    expect(
      screen.getByRole("button", { name: "Following" }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(fetch).toHaveBeenCalledWith(
        "/ashley/actions/follow",
        expect.objectContaining({ method: "PUT" }),
      ),
    );
    expect(screen.getByText("43 followers")).toBeInTheDocument();
  });

  it("hides activity surfaces for private profiles", () => {
    renderProfile(
      profile({
        identity: {
          ...profile().identity,
          privateProfile: true,
          followerCount: 0,
          followingCount: 0,
        },
        pinnedItems: [],
        achievements: [],
        contributions: { year: 2026, total: 0, days: [], recentEvents: [] },
      }),
    );

    expect(
      screen.getByText("ashley keeps activity private"),
    ).toBeInTheDocument();
    expect(screen.queryByText("First repository")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "opengithub" }),
    ).not.toBeInTheDocument();
  });

  it("opens login-gated report controls with real submit path", async () => {
    renderProfile();

    fireEvent.click(screen.getByRole("button", { name: "Report" }));
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("combobox"), {
      target: { value: "harassment" },
    });
    vi.mocked(fetch).mockResolvedValueOnce(
      Response.json({ id: "report-1", status: "received" }, { status: 201 }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Submit report" }));

    await waitFor(() =>
      expect(fetch).toHaveBeenCalledWith(
        "/ashley/actions/report",
        expect.objectContaining({ method: "POST" }),
      ),
    );
  });
});
