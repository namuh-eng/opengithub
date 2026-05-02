import { fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryReleasesPage } from "@/components/RepositoryReleasesPage";
import type { AuthSession, RepositoryOverview } from "@/lib/api";
import {
  getRepositoryLatestRelease,
  getRepositoryReleasesView,
  resetReleaseReactionStateForTests,
  toggleReleaseReaction,
} from "@/lib/releases";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  const base: RepositoryOverview = {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Release test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-04-01T00:00:00Z",
    updated_at: "2026-04-30T00:00:00Z",
    viewerPermission: "owner",
    branchCount: 1,
    tagCount: 3,
    defaultBranchRef: null,
    latestCommit: null,
    rootEntries: [],
    files: [],
    readme: null,
    sidebar: {
      about: null,
      websiteUrl: null,
      topics: [],
      starsCount: 0,
      watchersCount: 0,
      forksCount: 0,
      releasesCount: 3,
      deploymentsCount: 0,
      contributorsCount: 3,
      languages: [],
    },
    viewerState: {
      starred: false,
      watching: false,
      forkedRepositoryHref: null,
    },
    cloneUrls: {
      https: "https://opengithub.test/mona/octo-app.git",
      git: "git@opengithub.test:mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
  };
  return { ...base, ...overrides };
}

const publicSession: AuthSession = { authenticated: false, user: null };
const authSession: AuthSession = {
  authenticated: true,
  user: {
    id: "user-1",
    email: "mona@example.test",
    display_name: "Mona",
    avatar_url: null,
  },
};

afterEach(() => {
  vi.restoreAllMocks();
  resetReleaseReactionStateForTests();
});

describe("release data contract", () => {
  it("orders releases newest first and paginates release history", () => {
    const view = getRepositoryReleasesView(
      repositoryOverview(),
      publicSession,
      { page: 1, pageSize: 2 },
    );

    expect(view.total).toBe(3);
    expect(view.totalPages).toBe(2);
    expect(view.items.map((release) => release.tagName)).toEqual([
      "v2.1.0",
      "v2.1.0-rc.1",
    ]);
    expect(view.nextHref).toBe("/mona/octo-app/releases?page=2");
  });

  it("resolves latest to the newest non-prerelease release", () => {
    const latest = getRepositoryLatestRelease(
      repositoryOverview(),
      publicSession,
    );

    expect(latest?.items).toHaveLength(1);
    expect(latest?.items[0]?.tagName).toBe("v2.1.0");
    expect(latest?.items[0]?.prerelease).toBe(false);
  });

  it("requires authentication before toggling release reactions", () => {
    const repository = repositoryOverview();
    const release = getRepositoryReleasesView(repository, publicSession)
      .items[0];

    expect(
      toggleReleaseReaction(repository, publicSession, release.id, "rocket"),
    ).toBeNull();

    const result = toggleReleaseReaction(
      repository,
      authSession,
      release.id,
      "rocket",
    );
    expect(
      result?.reactions.find((reaction) => reaction.kind === "rocket")
        ?.viewerReacted,
    ).toBe(true);
  });
});

describe("RepositoryReleasesPage", () => {
  it("renders release cards with labels, markdown notes, assets, reactions, and pagination", () => {
    const repository = repositoryOverview();
    const releases = getRepositoryReleasesView(repository, publicSession, {
      page: 1,
      pageSize: 2,
    });

    render(
      <RepositoryReleasesPage releases={releases} repository={repository} />,
    );

    expect(
      screen.getByRole("heading", { name: "Releases", level: 1 }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Latest release" }),
    ).toHaveAttribute("href", "/mona/octo-app/releases/latest");
    expect(screen.getByRole("link", { name: "Releases" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases",
    );
    expect(screen.getByRole("link", { name: "Tags" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tags",
    );

    const firstCard = screen.getAllByTestId("release-card")[0];
    expect(
      within(firstCard).getByRole("heading", { name: "Quiet launch controls" }),
    ).toBeInTheDocument();
    expect(within(firstCard).getByText("Latest")).toBeInTheDocument();
    expect(within(firstCard).getByText("Verified")).toBeInTheDocument();
    expect(within(firstCard).getByText("Highlights")).toBeInTheDocument();
    expect(
      within(firstCard).getByRole("link", { name: "PR #128" }),
    ).toHaveAttribute("href", "/mona/octo-app/pull/128");
    expect(within(firstCard).getByText("Assets (5)")).toBeInTheDocument();
    expect(within(firstCard).getByText("Compare")).toBeInTheDocument();
    expect(
      within(firstCard).getByLabelText("Search branch or tag"),
    ).toHaveAttribute("name", "head");
    expect(
      within(firstCard).getByRole("button", { name: /Rocket/ }),
    ).toBeDisabled();
    expect(screen.getByText("Page 1 of 2")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases?page=2",
    );
  });

  it("posts reaction toggles for authenticated viewers", async () => {
    const repository = repositoryOverview();
    const releases = getRepositoryReleasesView(repository, authSession, {
      page: 1,
      pageSize: 1,
    });
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        Response.json({
          releaseId: releases.items[0].id,
          reactions: releases.items[0].reactions.map((reaction) =>
            reaction.kind === "rocket"
              ? { ...reaction, count: reaction.count + 1, viewerReacted: true }
              : reaction,
          ),
        }),
      ),
    );

    render(
      <RepositoryReleasesPage releases={releases} repository={repository} />,
    );
    fireEvent.click(screen.getByRole("button", { name: /Rocket/ }));

    expect(
      await screen.findByRole("button", { name: /Rocket 13/ }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(fetch).toHaveBeenCalledWith(
      "/api/repos/mona/octo-app/releases/reactions",
      expect.objectContaining({ method: "POST" }),
    );
  });
});
