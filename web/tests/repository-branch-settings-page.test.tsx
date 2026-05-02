import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryBranchSettingsPage } from "@/components/RepositoryBranchSettingsPage";
import type {
  BranchPolicyRequirements,
  RepositoryBranchSettings,
  RepositoryBranchSettingsFetchResult,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: null,
    owner_organization_id: "org-1",
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
    ...overrides,
  };
}

function requirements(
  overrides: Partial<BranchPolicyRequirements> = {},
): BranchPolicyRequirements {
  return {
    allowsDeletions: false,
    allowsForcePushes: false,
    locked: false,
    requiredApprovingReviewCount: 1,
    requiredDeploymentEnvironments: [],
    requiredStatusChecks: ["ci", "biome"],
    requiresConversationResolution: true,
    requiresDeployments: false,
    requiresLinearHistory: true,
    requiresMergeQueue: false,
    requiresSignedCommits: false,
    requiresUpToDateBranch: true,
    restrictsPushes: true,
    ...overrides,
  };
}

function branchSettings(
  overrides: Partial<RepositoryBranchSettings> = {},
): RepositoryBranchSettings {
  return {
    id: "repo-1",
    ownerLogin: "namuh-eng",
    name: "opengithub",
    visibility: "private",
    defaultBranch: "main",
    defaultBranchSummary: {
      href: "/namuh-eng/opengithub/tree/main",
      matchingRuleCount: 1,
      matchingRulesetCount: 1,
      name: "main",
      protected: true,
    },
    viewerPermission: "admin",
    canEdit: true,
    refs: [
      {
        name: "main",
        protected: true,
        matchingRuleCount: 1,
        matchingRulesetCount: 1,
        updatedAt: "2026-05-03T00:00:00Z",
      },
      {
        name: "feature/editorial-shell",
        protected: false,
        matchingRuleCount: 0,
        matchingRulesetCount: 0,
        updatedAt: "2026-05-02T00:00:00Z",
      },
    ],
    rules: [
      {
        id: "rule-1",
        pattern: "main",
        description: "Protect the release branch.",
        enforcement: "active",
        matchingBranches: ["main"],
        matchingBranchCount: 1,
        requirements: requirements(),
        bypassActors: [{ actorId: "team-1", actorType: "team", label: "Core" }],
        canEdit: true,
        canDelete: true,
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-03T00:00:00Z",
      },
    ],
    rulesets: [
      {
        id: "ruleset-1",
        name: "Release branches",
        target: "branch",
        enforcement: "evaluate",
        patterns: ["release/*"],
        matchingBranches: [],
        matchingBranchCount: 0,
        requirements: requirements({
          requiredApprovingReviewCount: 2,
          requiredStatusChecks: ["ci"],
          requiresConversationResolution: false,
          requiresSignedCommits: true,
        }),
        bypassActors: [],
        canEdit: true,
        canDelete: true,
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-03T00:00:00Z",
      },
    ],
    statusCheckSuggestions: ["ci", "biome"],
    auditEvents: [],
    ...overrides,
  };
}

function okResult(
  overrides: Partial<RepositoryBranchSettings> = {},
): RepositoryBranchSettingsFetchResult {
  return { ok: true, settings: branchSettings(overrides) };
}

describe("repository branch settings page", () => {
  it("renders default branch, policies, requirements, and Editorial primitives", () => {
    const { container } = render(
      <RepositoryBranchSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "namuh-eng/opengithub" }),
    ).toBeVisible();
    expect(screen.getAllByRole("link", { name: "main" })[0]).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/tree/main",
    );
    expect(screen.getAllByText("Protected").length).toBeGreaterThan(0);
    expect(screen.getByText("Release branches")).toBeVisible();
    expect(screen.getByText("Evaluate")).toBeVisible();
    expect(screen.getAllByText("check: ci").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Linear history").length).toBeGreaterThan(0);
    expect(screen.getByText("Bypass: Core")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "New branch protection rule" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/branches?new=rule#branch-rules",
    );
    expect(
      screen.getAllByRole("link", { name: "New ruleset" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/branches?new=ruleset#branch-rules",
    );

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(3);
    expect(container.querySelector(".chip")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("renders read-only policy explanations for non-admin readers", () => {
    render(
      <RepositoryBranchSettingsPage
        repository={repositoryOverview({ viewerPermission: "read" })}
        settingsResult={okResult({
          canEdit: false,
          viewerPermission: "read",
          rules: [
            {
              ...branchSettings().rules[0],
              canDelete: false,
              canEdit: false,
            },
          ],
          rulesets: [
            {
              ...branchSettings().rulesets[0],
              canDelete: false,
              canEdit: false,
            },
          ],
        })}
      />,
    );

    expect(screen.getByText("Viewer: read")).toBeVisible();
    expect(
      screen.getByText(
        "You can view active and evaluate-only policies, but editing requires admin access.",
      ),
    ).toBeVisible();
    expect(screen.getAllByText("Read-only").length).toBeGreaterThan(1);
    expect(
      screen.queryByRole("link", { name: "New branch protection rule" }),
    ).not.toBeInTheDocument();
  });

  it("renders empty states with concrete creation links", () => {
    render(
      <RepositoryBranchSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult({
          defaultBranchSummary: {
            href: "/namuh-eng/opengithub/tree/main",
            matchingRuleCount: 0,
            matchingRulesetCount: 0,
            name: "main",
            protected: false,
          },
          refs: [],
          rules: [],
          rulesets: [],
          statusCheckSuggestions: [],
        })}
      />,
    );

    expect(screen.getByText("Unprotected")).toBeVisible();
    expect(screen.getByText("No branch rules are configured")).toBeVisible();
    expect(
      screen.getByText(
        "No branch refs have been indexed for this repository yet.",
      ),
    ).toBeVisible();
    expect(screen.getByText("No suggestions yet")).toBeVisible();
    expect(
      screen.getAllByRole("link", { name: "New ruleset" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/branches?new=ruleset#branch-rules",
    );
  });

  it("shows the Phase 3 editor entry state from the new-rule query", () => {
    render(
      <RepositoryBranchSettingsPage
        intent="rule"
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Branch protection rule editor" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Return to policies" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/settings/branches");
  });

  it("renders unavailable states with recovery navigation", () => {
    render(
      <RepositoryBranchSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          code: "forbidden",
          message: "forbidden",
          ok: false,
          status: 403,
        }}
      />,
    );

    const status = screen.getByRole("status");
    expect(within(status).getByText("Read access required")).toBeVisible();
    expect(
      within(status).getByRole("link", { name: "Repository Code" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub");
    expect(
      within(status).getByRole("link", { name: "Dashboard" }),
    ).toHaveAttribute("href", "/dashboard");
  });

  it("does not render inert anchors or unnamed visible buttons", () => {
    const { container } = render(
      <RepositoryBranchSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of container.querySelectorAll("button")) {
      expect(button.textContent?.trim()).not.toBe("");
    }
  });
});
