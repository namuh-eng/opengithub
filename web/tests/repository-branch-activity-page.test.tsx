import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryBranchActivityPage } from "@/components/RepositoryBranchActivityPage";
import type {
  RepositoryBranchActivityView,
  RepositoryBranchDirectoryRow,
  RepositoryCommitListItem,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "private",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "admin",
    branchCount: 3,
    tagCount: 1,
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
      releasesCount: 0,
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
      git: "git@opengithub.namuh.co:namuh-eng/opengithub.git",
      https: "https://opengithub.namuh.co/namuh-eng/opengithub.git",
      zip: "/namuh-eng/opengithub/archive/refs/heads/main.zip",
    },
  };
}

function branch(): RepositoryBranchDirectoryRow {
  return {
    name: "feature/tree-nav",
    qualifiedName: "refs/heads/feature/tree-nav",
    classification: "active",
    isDefault: false,
    href: "/namuh-eng/opengithub/tree/feature%2Ftree-nav",
    commitsHref: "/namuh-eng/opengithub/commits/feature%2Ftree-nav",
    activityHref: "/namuh-eng/opengithub/branches/feature%2Ftree-nav",
    latestCommit: null,
    checks: {
      status: "completed",
      conclusion: "success",
      totalCount: 2,
      completedCount: 2,
      failedCount: 0,
      href: "/namuh-eng/opengithub/actions?commit=abc1234",
    },
    protection: {
      protected: true,
      matchingRuleCount: 0,
      matchingRulesetCount: 1,
      requiredStatusChecks: ["security/review"],
      href: "/namuh-eng/opengithub/settings/branches?branch=feature%2Ftree-nav",
    },
    ahead: 2,
    behind: 1,
    pullRequest: null,
    capabilities: {
      canCopy: true,
      canViewActivity: true,
      canViewRules: true,
      canDelete: false,
      deleteDisabledReason:
        "Protected branches require policy changes before deletion.",
      canRestore: false,
      restoreDisabledReason:
        "Branch restore is handled by a later mutation phase.",
    },
    updatedAt: "2026-05-04T10:00:00Z",
  };
}

function commit(): RepositoryCommitListItem {
  return {
    oid: "abc123456789",
    shortOid: "abc1234",
    message: "Render branch activity",
    subject: "Render branch activity",
    body: null,
    href: "/namuh-eng/opengithub/commit/abc123456789",
    browseHref: "/namuh-eng/opengithub/tree/abc123456789",
    committedAt: "2026-05-04T10:00:00Z",
    authorLogin: "mona",
    authorAvatarUrl: null,
    pullRequests: [],
    status: {
      status: "completed",
      conclusion: "success",
      totalCount: 2,
      completedCount: 2,
      failedCount: 0,
      href: "/namuh-eng/opengithub/actions?commit=abc123456789",
    },
    verification: {
      verified: true,
      signatureState: "verified",
      signatureSummary: "Verified",
    },
  };
}

function activityView(): RepositoryBranchActivityView {
  return {
    repository: {
      ownerLogin: "namuh-eng",
      name: "opengithub",
      defaultBranch: "main",
      visibility: "private",
      viewerPermission: "admin",
    },
    branch: branch(),
    recentCommits: [commit()],
    recentPullRequests: [
      {
        number: 42,
        title: "Open branch directory",
        state: "open",
        draft: false,
        href: "/namuh-eng/opengithub/pull/42",
      },
    ],
    protectionEvents: [
      {
        sourceType: "ruleset",
        name: "Feature branches",
        enforcement: "active",
        href: "/namuh-eng/opengithub/settings/branches?branch=feature%2Ftree-nav",
        requiredStatusChecks: ["security/review"],
        updatedAt: "2026-05-04T11:00:00Z",
      },
    ],
    links: {
      branchesHref: "/namuh-eng/opengithub/branches",
      treeHref: "/namuh-eng/opengithub/tree/feature%2Ftree-nav",
      commitsHref: "/namuh-eng/opengithub/commits/feature%2Ftree-nav",
      compareHref: "/namuh-eng/opengithub/compare/main...feature%2Ftree-nav",
      rulesHref:
        "/namuh-eng/opengithub/settings/branches?branch=feature%2Ftree-nav",
    },
  };
}

describe("repository branch activity page", () => {
  it("renders branch activity, useful destinations, and Editorial primitives", () => {
    const { container } = render(
      <RepositoryBranchActivityPage
        activityResult={{ ok: true, activity: activityView() }}
        branchName="feature/tree-nav"
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "feature/tree-nav" }),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: "Branches" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/branches",
    );
    expect(screen.getByRole("link", { name: "Open tree" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/feature%2Ftree-nav",
    );
    expect(
      screen.getByRole("link", { name: "Commit history" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/commits/feature%2Ftree-nav",
    );
    expect(
      screen.getByRole("link", { name: "Render branch activity" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/commit/abc123456789");
    expect(
      screen.getByRole("link", { name: "#42 Open branch directory" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/pull/42");
    expect(
      screen.getByRole("link", { name: /Feature branches/ }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/branches?branch=feature%2Ftree-nav",
    );
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(2);
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("renders branch activity recovery without dead controls", () => {
    const { container } = render(
      <RepositoryBranchActivityPage
        activityResult={{
          ok: false,
          status: 404,
          code: "ref_not_found",
          message: "repository ref `missing` was not found",
        }}
        branchName="missing"
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("heading", { name: "missing" })).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to Branches" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/branches");
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
  });
});
