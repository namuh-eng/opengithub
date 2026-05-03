import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RepositoryReleaseFormPage } from "@/components/RepositoryReleaseFormPage";
import {
  RepositoryReleaseDetailPage,
  RepositoryReleasesPage,
  RepositoryTagsPage,
} from "@/components/RepositoryReleasesPage";
import type {
  ListEnvelope,
  ReleaseManagementContext,
  ReleaseTagSummary,
  RepositoryOverview,
  RepositoryReleaseDetail,
  RepositoryReleaseSummary,
} from "@/lib/api";

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: vi.fn(),
    refresh: vi.fn(),
  }),
}));

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
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
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "read",
    branchCount: 2,
    tagCount: 2,
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
      releasesCount: 2,
      deploymentsCount: 0,
      contributorsCount: 2,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function release(
  overrides: Partial<RepositoryReleaseSummary> = {},
): RepositoryReleaseSummary {
  return {
    id: "release-1",
    tagName: "v2.0.0",
    title: "Stable Editorial release",
    bodyExcerpt: "<h2>Highlights</h2><p>Safe release notes.</p>",
    draft: false,
    prerelease: false,
    latest: true,
    verified: true,
    targetOid: "abcdef1234567890",
    shortOid: "abcdef1",
    author: {
      id: "user-1",
      login: "mona",
      displayName: "Mona",
      avatarUrl: null,
    },
    publishedAt: "2026-05-03T00:00:00Z",
    createdAt: "2026-05-03T00:00:00Z",
    updatedAt: "2026-05-03T00:00:00Z",
    assets: [
      {
        id: "asset-1",
        name: "opengithub.tar.gz",
        label: "Linux build",
        contentType: "application/gzip",
        byteSize: 2048,
        downloadCount: 42,
        checksumSha256: "abc",
        href: "/api/repos/mona/octo-app/releases/assets/asset-1",
        createdAt: "2026-05-03T00:00:00Z",
      },
    ],
    reactions: {
      totalCount: 3,
      thumbsUp: 1,
      thumbsDown: 0,
      laugh: 0,
      hooray: 0,
      confused: 0,
      heart: 1,
      rocket: 1,
      eyes: 0,
      viewerReaction: null,
    },
    contributors: [
      {
        id: "user-2",
        login: "ashley",
        displayName: "Ashley",
        avatarUrl: null,
      },
    ],
    links: {
      htmlHref: "/mona/octo-app/releases/tag/v2.0.0",
      apiHref: "/api/repos/mona/octo-app/releases/release-1",
      tagHref: "/mona/octo-app/tree/v2.0.0",
      zipballHref: "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
      tarballHref: "/api/repos/mona/octo-app/releases/tarball/v2.0.0",
      compareHref: "/mona/octo-app/compare/v2.0.0",
    },
    ...overrides,
  };
}

function releaseEnvelope(
  items: RepositoryReleaseSummary[],
): ListEnvelope<RepositoryReleaseSummary> {
  return { items, page: 1, pageSize: 30, total: items.length };
}

function tag(overrides: Partial<ReleaseTagSummary> = {}): ReleaseTagSummary {
  return {
    id: "tag-1",
    name: "v2.0.0",
    targetOid: "abcdef1234567890",
    shortOid: "abcdef1",
    commitMessage: "Release v2",
    committedAt: "2026-05-03T00:00:00Z",
    verified: true,
    releaseId: "release-1",
    releaseHref: "/mona/octo-app/releases/tag/v2.0.0",
    zipballHref: "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
    tarballHref: "/api/repos/mona/octo-app/releases/tarball/v2.0.0",
    compareHref: "/mona/octo-app/compare/v2.0.0",
    ...overrides,
  };
}

function tagEnvelope(
  items: ReleaseTagSummary[],
): ListEnvelope<ReleaseTagSummary> {
  return { items, page: 1, pageSize: 30, total: items.length };
}

function releaseRef(
  overrides: Partial<ReleaseManagementContext["availableRefs"][number]> = {},
): ReleaseManagementContext["availableRefs"][number] {
  return {
    name: "main",
    shortName: "main",
    kind: "branch",
    targetOid: "abcdef1234567890",
    shortOid: "abcdef1",
    committedAt: "2026-05-03T00:00:00Z",
    ...overrides,
  };
}

function managementContext(
  overrides: Partial<ReleaseManagementContext> = {},
): ReleaseManagementContext {
  const tagRef = releaseRef({
    kind: "tag",
    name: "v2.0.0",
    shortName: "v2.0.0",
  });
  return {
    repositoryId: "repo-1",
    ownerLogin: "mona",
    name: "octo-app",
    canWrite: true,
    archived: false,
    release: null,
    availableTags: [tagRef],
    availableRefs: [releaseRef(), tagRef],
    defaultTarget: "main",
    previousTagCandidates: [tagRef],
    latestPolicyOptions: [
      {
        value: "automatic",
        label: "Automatic",
        description: "Use the newest stable published release.",
      },
      {
        value: "legacy",
        label: "Legacy",
        description: "Keep the existing latest marker unchanged.",
      },
    ],
    uploadLimits: {
      maxAssetBytes: 2_147_483_648,
      maxAssetCount: 100,
      allowedStorageKinds: ["local", "s3"],
      expiresInSeconds: 900,
    },
    ...overrides,
  };
}

function expectNoDeadControls(container: HTMLElement) {
  expect(container.querySelectorAll('a[href="#"], a:not([href])')).toHaveLength(
    0,
  );
  for (const button of Array.from(container.querySelectorAll("button"))) {
    expect(button.textContent?.trim()).not.toEqual("");
    if (button.hasAttribute("disabled")) {
      expect(button).toHaveAttribute("aria-disabled", "true");
    }
  }
}

describe("RepositoryReleasesPage", () => {
  it("renders Editorial release cards with metadata, assets, reactions, and concrete links", () => {
    const { container } = render(
      <RepositoryReleasesPage
        authenticated={true}
        releases={releaseEnvelope([
          release(),
          release({
            id: "release-2",
            latest: false,
            prerelease: true,
            tagName: "v2.1.0-beta",
            title: "Beta train",
          }),
        ])}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Releases" }),
    ).toBeVisible();
    expect(screen.getByText("Latest")).toBeVisible();
    expect(screen.getAllByText("Verified").length).toBeGreaterThan(0);
    expect(screen.getByText("Pre-release")).toBeVisible();
    expect(screen.getAllByText("Highlights").length).toBeGreaterThan(0);
    expect(screen.getAllByText("opengithub.tar.gz").length).toBeGreaterThan(0);
    expect(screen.getAllByText("rocket").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "Latest release" }),
    ).toHaveAttribute("href", "/mona/octo-app/releases/latest");
    expect(screen.getAllByRole("button", { name: "Compare" })[0]).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "Source code (zip)" })[0],
    ).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
    );
    expectNoDeadControls(container);
  });

  it("renders empty, forbidden, and unavailable states without placeholder links", () => {
    const repository = repositoryOverview();
    const { container, rerender } = render(
      <RepositoryReleasesPage
        authenticated={false}
        releases={releaseEnvelope([])}
        repository={repository}
      />,
    );
    expect(screen.getByText("No published releases yet")).toBeVisible();

    rerender(
      <RepositoryReleasesPage
        authenticated={false}
        releases={{
          error: { code: "permission_denied", message: "forbidden" },
          status: 403,
        }}
        repository={repository}
      />,
    );
    expect(screen.getByText("Releases could not load")).toBeVisible();
    expect(screen.getByText("Access restricted")).toBeVisible();
    expectNoDeadControls(container);
  });

  it("shows release creation controls only for write-capable viewers", () => {
    const { rerender } = render(
      <RepositoryReleasesPage
        authenticated={true}
        releases={releaseEnvelope([])}
        repository={repositoryOverview({ viewerPermission: "read" })}
      />,
    );
    expect(
      screen.queryByText("Draft or publish a release"),
    ).not.toBeInTheDocument();

    rerender(
      <RepositoryReleasesPage
        authenticated={true}
        releases={releaseEnvelope([])}
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );
    expect(screen.getByRole("link", { name: "New release" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases/new",
    );
  });
});

describe("RepositoryReleaseDetailPage", () => {
  it("renders detail markdown and immutable release metadata", () => {
    const detail: RepositoryReleaseDetail = {
      ...release(),
      body: "## Full notes",
      bodyHtml: "<h2>Full notes</h2><p>No scripts.</p>",
      immutable: true,
      tagSignatureSummary: "Signed by release key",
    };
    const { container } = render(
      <RepositoryReleaseDetailPage
        authenticated={true}
        release={detail}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByText("Full notes")).toBeVisible();
    expect(screen.getByText("Immutable")).toBeVisible();
    expect(screen.getByText("Signed by release key")).toBeVisible();
    expect(screen.queryByText("<script>")).not.toBeInTheDocument();
    expectNoDeadControls(container);
  });

  it("links write viewers to the dedicated edit and publish surface", () => {
    const detail: RepositoryReleaseDetail = {
      ...release({ draft: true, latest: false }),
      body: "Draft notes",
      bodyHtml: "<p>Draft notes</p>",
      immutable: false,
      tagSignatureSummary: null,
    };
    render(
      <RepositoryReleaseDetailPage
        authenticated={true}
        release={detail}
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    expect(screen.getByRole("link", { name: "Edit release" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases/edit/release-1",
    );
    expect(screen.getByRole("link", { name: "Publish draft" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases/edit/release-1",
    );
  });
});

describe("RepositoryReleaseFormPage", () => {
  it("renders the dedicated new release form with selectors, preview, policy, and disabled submit actions", () => {
    const { container } = render(
      <RepositoryReleaseFormPage
        context={managementContext()}
        mode="new"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    expect(screen.getByRole("heading", { name: "New release" })).toBeVisible();
    expect(screen.getByLabelText("Existing tag")).toHaveValue("v2.0.0");
    expect(screen.getByLabelText("Target branch, tag, or SHA")).toHaveValue(
      "main",
    );
    expect(screen.getByLabelText("Previous tag")).toHaveValue("v2.0.0");
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Release <script>" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Preview generated notes" }),
    );
    expect(screen.getByText("Release <script>")).toBeVisible();
    expect(screen.queryByText("<script>")).not.toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Preview" })).toBeVisible();
    expect(screen.getByLabelText("Release asset files")).toBeEnabled();
    expect(screen.getByLabelText("Release asset files")).toHaveAttribute(
      "type",
      "file",
    );
    expect(
      screen.getByRole("button", { name: "Publish release" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save draft" })).toBeDisabled();
    expectNoDeadControls(container);
  });

  it("renders edit, immutable, danger, and existing asset states", () => {
    const detail: RepositoryReleaseDetail = {
      ...release({ draft: true, latest: false }),
      body: "Draft notes",
      bodyHtml: "<p>Draft notes</p>",
      immutable: true,
      tagSignatureSummary: null,
    };
    render(
      <RepositoryReleaseFormPage
        context={managementContext({ release: detail })}
        mode="edit"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    expect(screen.getByRole("heading", { name: "Edit release" })).toBeVisible();
    expect(screen.getByText("Immutable release")).toBeVisible();
    expect(screen.getByDisplayValue("Stable Editorial release")).toBeDisabled();
    expect(screen.getByText("opengithub.tar.gz")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Delete release" }),
    ).toBeDisabled();
    expect(screen.getByLabelText("Also delete the git tag")).toBeDisabled();
  });

  it("renders release management forbidden and unavailable states", () => {
    render(
      <RepositoryReleaseFormPage
        context={{
          error: { code: "permission_denied", message: "forbidden" },
          status: 403,
        }}
        mode="new"
        repository={repositoryOverview({ viewerPermission: "read" })}
      />,
    );

    expect(screen.getByText("Write access required")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to releases" }),
    ).toHaveAttribute("href", "/mona/octo-app/releases");
  });
});

describe("RepositoryTagsPage", () => {
  it("renders tag rows with release, commit, archive, and compare links", () => {
    const { container } = render(
      <RepositoryTagsPage
        repository={repositoryOverview()}
        tags={tagEnvelope([tag()])}
      />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Tags" }),
    ).toBeVisible();
    const row = screen.getByText("v2.0.0").closest(".list-row");
    expect(row).not.toBeNull();
    const scoped = within(row as HTMLElement);
    expect(scoped.getByText("Verified")).toBeVisible();
    expect(scoped.getByRole("link", { name: "Release" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases/tag/v2.0.0",
    );
    expect(scoped.getByRole("link", { name: "Zip" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
    );
    expect(scoped.getByRole("link", { name: "Compare" })).toHaveAttribute(
      "href",
      "/mona/octo-app/compare/v2.0.0...main",
    );
    expectNoDeadControls(container);
  });

  it("renders tag empty and unavailable states", () => {
    const repository = repositoryOverview();
    const { rerender } = render(
      <RepositoryTagsPage repository={repository} tags={tagEnvelope([])} />,
    );
    expect(screen.getByText("No repository tags yet")).toBeVisible();

    rerender(
      <RepositoryTagsPage
        repository={repository}
        tags={{
          error: { code: "network_error", message: "Tags are offline." },
          status: 503,
        }}
      />,
    );
    expect(screen.getByText("Tags could not load")).toBeVisible();
    expect(screen.getByText("Tags are offline.")).toBeVisible();
  });
});
