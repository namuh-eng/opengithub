import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryReleaseFormPage } from "@/components/RepositoryReleaseFormPage";
import {
  RepositoryReleaseDetailPage,
  RepositoryReleasesPage,
  RepositoryTagsPage,
} from "@/components/RepositoryReleasesPage";
import type {
  GeneratedReleaseNotesPreview,
  ListEnvelope,
  ReleaseManagementContext,
  ReleaseTagSummary,
  RepositoryOverview,
  RepositoryReleaseDetail,
  RepositoryReleaseSummary,
} from "@/lib/api";

const pushMock = vi.fn();
const refreshMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: pushMock,
    refresh: refreshMock,
  }),
}));

beforeEach(() => {
  pushMock.mockReset();
  refreshMock.mockReset();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

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
    signatureSummary: "Verified tag signature from Ashley's public GPG key.",
    releaseId: "release-1",
    releaseHref: "/mona/octo-app/releases/tag/v2.0.0",
    zipballHref: "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
    tarballHref: "/api/repos/mona/octo-app/releases/tarball/v2.0.0",
    compareHref: "/mona/octo-app/compare/v2.0.0...main",
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

  it("renders generated AI changelog output with an editable generation action", () => {
    const detail: RepositoryReleaseDetail = {
      ...release({
        tagName: "release/2026",
        links: {
          ...release().links,
          htmlHref: "/mona/octo-app/releases/tag/release%2F2026",
        },
      }),
      body: "## Full notes",
      bodyHtml: "<h2>Full notes</h2>",
      immutable: false,
      tagSignatureSummary: null,
    };
    const { container } = render(
      <RepositoryReleaseDetailPage
        aiChangelog={{
          enabled: true,
          reason: null,
          previousTag: "v1.9.0",
          targetTag: "v2.0.0",
          output: {
            id: "ai-release-1",
            kind: "changelog",
            scopeType: "release",
            scopeId: "release-1",
            contentHash: "hash",
            promptVersion: "ai-001-v1",
            model: "gpt-4o",
            output: "### Added\n- Editorial AI changelog generation.",
            generatedAt: "2026-05-07T00:00:00Z",
            regeneratedCount: 0,
            cached: true,
          },
        }}
        authenticated={true}
        release={detail}
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    expect(screen.getByLabelText("AI changelog")).toHaveTextContent(
      "Editorial AI changelog",
    );
    expect(
      screen.getByRole("button", { name: "Generate changelog with AI" }),
    ).toBeEnabled();
    expect(
      container.querySelector(
        'form[action="/mona/octo-app/releases/release%2F2026/ai/changelog"]',
      ),
    ).toHaveAttribute("method", "post");
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
  it("renders the dedicated new release form with selectors, policy, and server-confirmed actions", () => {
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
    expect(screen.getByRole("tab", { name: "Preview" })).toBeVisible();
    expect(screen.getByLabelText("Release asset files")).toBeEnabled();
    expect(screen.getByLabelText("Release asset files")).toHaveAttribute(
      "type",
      "file",
    );
    expect(
      screen.getByRole("button", { name: "Publish release" }),
    ).toBeEnabled();
    expect(screen.getByRole("button", { name: "Save draft" })).toBeEnabled();
    expectNoDeadControls(container);
  });

  it("inserts generated notes only after the server returns them", async () => {
    const preview: GeneratedReleaseNotesPreview = {
      title: "Managed release",
      body: "## Managed release\n\n- abc1234 Server generated change",
      target: releaseRef(),
      previousTag: releaseRef({ name: "v2.0.0", shortName: "v2.0.0" }),
      commitCount: 1,
      mergedPullRequestCount: 0,
      contributors: [],
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => preview,
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryReleaseFormPage
        context={managementContext()}
        mode="new"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Managed release" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Generate release notes" }),
    );

    await waitFor(() =>
      expect(screen.getByLabelText("Markdown source")).toHaveValue(
        preview.body,
      ),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "/mona/octo-app/releases/actions",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining('"action":"generatedNotes"'),
      }),
    );
    expect(
      screen.getByText(
        "Generated notes inserted. Review them before publishing.",
      ),
    ).toBeVisible();
  });

  it("publishes, saves drafts, updates, and redirects only from returned API state", async () => {
    const created = {
      ...release({ title: "Managed release", tagName: "v3.0.0" }),
      body: "Notes",
      bodyHtml: "<p>Notes</p>",
      immutable: false,
      tagSignatureSummary: null,
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => created,
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryReleaseFormPage
        context={managementContext()}
        mode="new"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    fireEvent.click(screen.getByLabelText("New tag"));
    fireEvent.change(screen.getByLabelText("New tag name"), {
      target: { value: "v3.0.0" },
    });
    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Managed release" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          method: "POST",
          body: expect.stringContaining('"draft":true'),
        }),
      ),
    );
    await waitFor(() =>
      expect(pushMock).toHaveBeenCalledWith(
        "/mona/octo-app/releases/tag/v2.0.0",
      ),
    );
  });

  it("preserves form data and shows server errors when a mutation is rejected", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        error: {
          code: "validation_failed",
          message: "release tag already exists",
        },
        status: 422,
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryReleaseFormPage
        context={managementContext()}
        mode="new"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Duplicate release" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Publish release" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "release tag already exists",
      ),
    );
    expect(screen.getByLabelText("Title")).toHaveValue("Duplicate release");
    expect(pushMock).not.toHaveBeenCalled();
  });

  it("publishes an edited draft and deletes only after typed confirmation", async () => {
    const detail: RepositoryReleaseDetail = {
      ...release({ draft: true, latest: false }),
      body: "Draft notes",
      bodyHtml: "<p>Draft notes</p>",
      immutable: false,
      tagSignatureSummary: null,
    };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        ...detail,
        draft: false,
        links: {
          ...detail.links,
          htmlHref: "/mona/octo-app/releases/tag/v2.0.0",
        },
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryReleaseFormPage
        context={managementContext({ release: detail })}
        mode="edit"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Delete release" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Publish draft" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          body: expect.stringContaining('"draft":false'),
        }),
      ),
    );

    fetchMock.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ ok: true }),
    });
    fireEvent.click(screen.getByLabelText("Also delete the git tag"));
    fireEvent.change(screen.getByLabelText("Type tag name to confirm"), {
      target: { value: "v2.0.0" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Delete release" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          body: expect.stringContaining('"deleteTag":true'),
        }),
      ),
    );
    expect(pushMock).toHaveBeenCalledWith("/mona/octo-app/releases");
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

  it("uploads, completes, and removes release assets through server-confirmed actions", async () => {
    const detail: RepositoryReleaseDetail = {
      ...release({ draft: true, latest: false }),
      body: "Draft notes",
      bodyHtml: "<p>Draft notes</p>",
      immutable: false,
      tagSignatureSummary: null,
    };
    const intent = {
      id: "intent-1",
      assetName: "manual.zip",
      contentType: "application/zip",
      byteSize: 11,
      checksumSha256: null,
      storageKind: "local",
      uploadUrl:
        "/api/repos/mona/octo-app/releases/manage/upload-intents/intent-1/local-upload",
      handoffToken: "local-upload-intent-1",
      status: "pending",
      expiresAt: "2026-05-03T00:15:00Z",
    };
    const updated: RepositoryReleaseDetail = {
      ...detail,
      assets: [
        ...detail.assets,
        {
          id: "asset-2",
          name: "manual.zip",
          label: null,
          contentType: "application/zip",
          byteSize: 11,
          downloadCount: 0,
          checksumSha256: null,
          href: "/api/repos/mona/octo-app/releases/assets/asset-2",
          createdAt: "2026-05-03T00:10:00Z",
        },
      ],
    };
    const afterDelete: RepositoryReleaseDetail = { ...detail, assets: [] };
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({ ok: true, json: async () => intent })
      .mockResolvedValueOnce({ ok: true, json: async () => updated })
      .mockResolvedValueOnce({ ok: true, json: async () => afterDelete });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryReleaseFormPage
        context={managementContext({ release: detail })}
        mode="edit"
        repository={repositoryOverview({ viewerPermission: "write" })}
      />,
    );

    const input = screen.getByLabelText("Release asset files");
    const file = new File(["hello asset"], "manual.zip", {
      type: "application/zip",
    });
    fireEvent.change(input, { target: { files: [file] } });

    await waitFor(() =>
      expect(fetchMock).toHaveBeenNthCalledWith(
        1,
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          body: expect.stringContaining('"action":"createUploadIntent"'),
        }),
      ),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenNthCalledWith(
        2,
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          body: expect.stringContaining('"action":"completeUploadIntent"'),
        }),
      ),
    );
    expect(await screen.findByText("Attached to release.")).toBeVisible();
    expect(screen.getAllByText("manual.zip").length).toBeGreaterThan(0);

    const uploadedAssetRow = screen.getAllByText("manual.zip")[1].closest("li");
    expect(uploadedAssetRow).not.toBeNull();
    fireEvent.click(
      within(uploadedAssetRow as HTMLElement).getByRole("button", {
        name: "Remove",
      }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/mona/octo-app/releases/actions",
        expect.objectContaining({
          body: expect.stringContaining('"action":"deleteAsset"'),
        }),
      ),
    );
    expect(await screen.findByText("Release asset removed.")).toBeVisible();
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
    fireEvent.click(scoped.getByText("Verified"));
    expect(scoped.getByText("Verified")).toBeVisible();
    expect(
      scoped.getByText("Verified tag signature from Ashley's public GPG key."),
    ).toBeVisible();
    expect(scoped.getByRole("link", { name: "Notes" })).toHaveAttribute(
      "href",
      "/mona/octo-app/releases/tag/v2.0.0",
    );
    expect(scoped.getByRole("link", { name: "Zip" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/releases/zipball/v2.0.0",
    );
    expect(scoped.getByRole("link", { name: "tar.gz" })).toHaveAttribute(
      "href",
      "/api/repos/mona/octo-app/releases/tarball/v2.0.0",
    );
    expect(scoped.getByRole("link", { name: "Compare" })).toHaveAttribute(
      "href",
      "/mona/octo-app/compare/v2.0.0...main",
    );
    expectNoDeadControls(container);
  });

  it("does not style unverified signature metadata as verified", () => {
    render(
      <RepositoryTagsPage
        repository={repositoryOverview()}
        tags={tagEnvelope([
          tag({
            id: "tag-unverified",
            name: "v1.9.0",
            verified: false,
            signatureSummary: "Unsigned or untrusted tag metadata.",
          }),
        ])}
      />,
    );

    const row = screen.getByText("v1.9.0").closest(".list-row");
    expect(row).not.toBeNull();
    const scoped = within(row as HTMLElement);
    const summary = scoped.getByText("Unverified");
    expect(summary).toHaveClass("chip", "warn");
    expect(summary).not.toHaveClass("ok");
    fireEvent.click(summary);
    expect(
      scoped.getByText("Unsigned or untrusted tag metadata."),
    ).toBeVisible();
  });

  it("renders tag pagination links from the current envelope", () => {
    render(
      <RepositoryTagsPage
        repository={repositoryOverview()}
        tags={{
          ...tagEnvelope([tag({ id: "tag-2", name: "v1.0.0" })]),
          page: 2,
          pageSize: 1,
          total: 3,
        }}
      />,
    );

    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tags?page=1",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/mona/octo-app/tags?page=3",
    );
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
