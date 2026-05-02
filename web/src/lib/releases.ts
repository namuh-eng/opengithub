import type {
  AuthSession,
  RepositoryOverview,
  RepositoryVisibility,
} from "@/lib/api";

export type ReleaseAuthor = {
  id: string;
  login: string;
  displayName: string | null;
  avatarUrl: string | null;
};

export type ReleaseAsset = {
  id: string;
  name: string;
  label: string | null;
  sizeBytes: number;
  downloadCount: number;
  contentType: string;
  downloadHref: string;
  createdAt: string;
};

export type ReleaseArchive = {
  kind: "zip" | "tar";
  label: string;
  href: string;
};

export type ReleaseReactionKind = "heart" | "hooray" | "rocket" | "eyes";

export type ReleaseReactionSummary = {
  kind: ReleaseReactionKind;
  label: string;
  count: number;
  viewerReacted: boolean;
};

export type RepositoryRelease = {
  id: string;
  repositoryId: string;
  ownerLogin: string;
  repositoryName: string;
  tagName: string;
  title: string;
  bodyMarkdown: string;
  author: ReleaseAuthor;
  contributors: ReleaseAuthor[];
  targetCommit: {
    oid: string;
    shortOid: string;
    href: string;
    verified: boolean;
  };
  publishedAt: string;
  createdAt: string;
  draft: boolean;
  prerelease: boolean;
  latest: boolean;
  href: string;
  tagHref: string;
  compareHref: string;
  assets: ReleaseAsset[];
  archives: ReleaseArchive[];
  reactions: ReleaseReactionSummary[];
};

export type RepositoryReleaseListView = {
  repository: {
    id: string;
    ownerLogin: string;
    name: string;
    visibility: RepositoryVisibility;
  };
  viewerPermission: string | null;
  viewerCanReact: boolean;
  signInHref: string;
  items: RepositoryRelease[];
  latestRelease: RepositoryRelease | null;
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
  nextHref: string | null;
  previousHref: string | null;
};

export type ReleaseToggleResult = {
  releaseId: string;
  reactions: ReleaseReactionSummary[];
};

const REACTION_LABELS: Record<ReleaseReactionKind, string> = {
  heart: "Love",
  hooray: "Celebrate",
  rocket: "Rocket",
  eyes: "Watching",
};

const SEEDED_VIEWER_REACTIONS = new Map<string, Set<ReleaseReactionKind>>();

function baseHref(owner: string, repo: string) {
  return `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`;
}

function releaseHref(owner: string, repo: string, tagName: string) {
  return `${baseHref(owner, repo)}/releases/tag/${encodeURIComponent(tagName)}`;
}

function archiveHref(
  owner: string,
  repo: string,
  tagName: string,
  kind: "zip" | "tar",
) {
  const suffix = kind === "zip" ? "zip" : "tar.gz";
  return `${baseHref(owner, repo)}/archive/refs/tags/${encodeURIComponent(tagName)}.${suffix}`;
}

function author(id: string, login: string, displayName: string): ReleaseAuthor {
  return { id, login, displayName, avatarUrl: null };
}

function seededReleases(
  owner: string,
  repo: string,
  repositoryId: string,
): RepositoryRelease[] {
  const b = baseHref(owner, repo);
  const mona = author("user-1", "mona", "Mona");
  const hubot = author("user-2", "hubot", "Hubot");
  const orbit = author("user-3", "orbit", "Orbit Maintainer");

  const releases = [
    {
      id: `${repositoryId}:rel-3`,
      tagName: "v2.1.0",
      title: "Quiet launch controls",
      bodyMarkdown:
        "## Highlights\n\n- Adds repository launch checklists for maintainers.\n- Ships the asset integrity manifest from PR #128.\n\n## Upgrade notes\n\nUse the signed archive when mirroring production packages.",
      author: mona,
      contributors: [mona, hubot, orbit],
      oid: "8f14e45fceea167a5a36dedd4bea2543a1f2c7d8",
      publishedAt: "2026-04-28T14:30:00Z",
      prerelease: false,
      verified: true,
      assets: [
        [
          "asset-1",
          "opengithub-2.1.0-darwin-arm64.tar.gz",
          "macOS arm64 bundle",
          18_240_512,
          412,
          "application/gzip",
        ],
        [
          "asset-2",
          "opengithub-2.1.0-linux-x64.tar.gz",
          "Linux x64 bundle",
          21_884_928,
          389,
          "application/gzip",
        ],
        [
          "asset-3",
          "checksums.txt",
          "SHA256 manifest",
          2_048,
          612,
          "text/plain",
        ],
      ] as const,
      reactions: { heart: 17, hooray: 9, rocket: 12, eyes: 4 },
    },
    {
      id: `${repositoryId}:rel-2`,
      tagName: "v2.1.0-rc.1",
      title: "Release candidate for launch controls",
      bodyMarkdown:
        "## Candidate scope\n\n- Exercises the release history paginator.\n- Validates asset download telemetry before the stable cut.\n\nSee PR #121 for reviewer notes.",
      author: orbit,
      contributors: [orbit, mona],
      oid: "4d967e3f8bbd91be9f2a8c5a7d46c9e2f1b0a345",
      publishedAt: "2026-04-24T10:00:00Z",
      prerelease: true,
      verified: true,
      assets: [
        [
          "asset-4",
          "opengithub-2.1.0-rc.1.tar.gz",
          "Candidate bundle",
          17_112_064,
          91,
          "application/gzip",
        ],
      ] as const,
      reactions: { heart: 5, hooray: 2, rocket: 7, eyes: 11 },
    },
    {
      id: `${repositoryId}:rel-1`,
      tagName: "v2.0.0",
      title: "Repository workspace foundation",
      bodyMarkdown:
        "## Foundation\n\n- Introduces the repository workspace shell.\n- Connects branch and tag archives.\n- Documents follow-up work in PR #98.",
      author: hubot,
      contributors: [hubot, mona],
      oid: "9a0364b9e99bb480dd25e1f0284c8555c4bda7aa",
      publishedAt: "2026-04-12T09:45:00Z",
      prerelease: false,
      verified: false,
      assets: [] as const,
      reactions: { heart: 22, hooray: 14, rocket: 18, eyes: 6 },
    },
  ];

  const latestStableId =
    releases.find((release) => !release.prerelease)?.id ?? null;

  return releases.map((release) => {
    const viewerKinds =
      SEEDED_VIEWER_REACTIONS.get(release.id) ?? new Set<ReleaseReactionKind>();
    return {
      id: release.id,
      repositoryId,
      ownerLogin: owner,
      repositoryName: repo,
      tagName: release.tagName,
      title: release.title,
      bodyMarkdown: release.bodyMarkdown,
      author: release.author,
      contributors: release.contributors,
      targetCommit: {
        oid: release.oid,
        shortOid: release.oid.slice(0, 7),
        href: `${b}/commit/${release.oid}`,
        verified: release.verified,
      },
      publishedAt: release.publishedAt,
      createdAt: release.publishedAt,
      draft: false,
      prerelease: release.prerelease,
      latest: release.id === latestStableId,
      href: releaseHref(owner, repo, release.tagName),
      tagHref: `${b}/tree/${encodeURIComponent(release.tagName)}`,
      compareHref: `${b}/compare?base=${encodeURIComponent(release.tagName)}&head=${encodeURIComponent("main")}`,
      assets: release.assets.map(
        ([id, name, label, sizeBytes, downloadCount, contentType]) => ({
          id: `${release.id}:${id}`,
          name,
          label,
          sizeBytes,
          downloadCount,
          contentType,
          downloadHref: `${b}/releases/download/${encodeURIComponent(release.tagName)}/${encodeURIComponent(name)}`,
          createdAt: release.publishedAt,
        }),
      ),
      archives: [
        {
          kind: "zip",
          label: "Source code (zip)",
          href: archiveHref(owner, repo, release.tagName, "zip"),
        },
        {
          kind: "tar",
          label: "Source code (tar.gz)",
          href: archiveHref(owner, repo, release.tagName, "tar"),
        },
      ],
      reactions: (Object.keys(REACTION_LABELS) as ReleaseReactionKind[]).map(
        (kind) => ({
          kind,
          label: REACTION_LABELS[kind],
          count: release.reactions[kind] + (viewerKinds.has(kind) ? 1 : 0),
          viewerReacted: viewerKinds.has(kind),
        }),
      ),
    } satisfies RepositoryRelease;
  });
}

function normalizePage(value: number | null | undefined) {
  if (!value || !Number.isFinite(value) || value < 1) {
    return 1;
  }
  return Math.floor(value);
}

export function getRepositoryReleasesView(
  repository: RepositoryOverview,
  session: AuthSession,
  query: { page?: number | null; pageSize?: number | null } = {},
): RepositoryReleaseListView {
  const pageSize = Math.min(Math.max(query.pageSize ?? 2, 1), 30);
  const allReleases = seededReleases(
    repository.owner_login,
    repository.name,
    repository.id,
  );
  const total = allReleases.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const page = Math.min(normalizePage(query.page), totalPages);
  const start = (page - 1) * pageSize;
  const items = allReleases.slice(start, start + pageSize);
  const b = `${baseHref(repository.owner_login, repository.name)}/releases`;
  const pageHref = (targetPage: number) => `${b}?page=${targetPage}`;

  return {
    repository: {
      id: repository.id,
      ownerLogin: repository.owner_login,
      name: repository.name,
      visibility: repository.visibility,
    },
    viewerPermission: repository.viewerPermission,
    viewerCanReact: session.authenticated,
    signInHref: `/login?next=${encodeURIComponent(b)}`,
    items,
    latestRelease: allReleases.find((release) => release.latest) ?? null,
    total,
    page,
    pageSize,
    totalPages,
    hasNextPage: page < totalPages,
    hasPreviousPage: page > 1,
    nextHref: page < totalPages ? pageHref(page + 1) : null,
    previousHref: page > 1 ? pageHref(page - 1) : null,
  };
}

export function getRepositoryReleaseByTag(
  repository: RepositoryOverview,
  session: AuthSession,
  tagName: string,
): RepositoryReleaseListView | null {
  const all = getRepositoryReleasesView(repository, session, {
    page: 1,
    pageSize: 30,
  });
  const release = all.items.find((item) => item.tagName === tagName);
  if (!release) {
    return null;
  }
  return {
    ...all,
    items: [release],
    total: 1,
    page: 1,
    pageSize: 1,
    totalPages: 1,
    hasNextPage: false,
    hasPreviousPage: false,
    nextHref: null,
    previousHref: null,
  };
}

export function getRepositoryLatestRelease(
  repository: RepositoryOverview,
  session: AuthSession,
): RepositoryReleaseListView | null {
  const all = getRepositoryReleasesView(repository, session, {
    page: 1,
    pageSize: 30,
  });
  const latest = all.latestRelease;
  if (!latest) {
    return null;
  }
  return {
    ...all,
    items: [latest],
    total: 1,
    page: 1,
    pageSize: 1,
    totalPages: 1,
    hasNextPage: false,
    hasPreviousPage: false,
    nextHref: null,
    previousHref: null,
  };
}

export function toggleReleaseReaction(
  repository: RepositoryOverview,
  session: AuthSession,
  releaseId: string,
  kind: ReleaseReactionKind,
): ReleaseToggleResult | null {
  if (!session.authenticated) {
    return null;
  }
  if (!(kind in REACTION_LABELS)) {
    return null;
  }
  const allReleases = seededReleases(
    repository.owner_login,
    repository.name,
    repository.id,
  );
  const release = allReleases.find((item) => item.id === releaseId);
  if (!release) {
    return null;
  }
  const current =
    SEEDED_VIEWER_REACTIONS.get(releaseId) ?? new Set<ReleaseReactionKind>();
  if (current.has(kind)) {
    current.delete(kind);
  } else {
    current.add(kind);
  }
  SEEDED_VIEWER_REACTIONS.set(releaseId, current);
  const updatedRelease = seededReleases(
    repository.owner_login,
    repository.name,
    repository.id,
  ).find((item) => item.id === releaseId);
  return updatedRelease
    ? { releaseId, reactions: updatedRelease.reactions }
    : null;
}

export function resetReleaseReactionStateForTests() {
  SEEDED_VIEWER_REACTIONS.clear();
}
