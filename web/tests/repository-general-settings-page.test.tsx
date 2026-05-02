import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryGeneralSettingsPage } from "@/components/RepositoryGeneralSettingsPage";
import type {
  RepositoryOverview,
  RepositorySettings,
  RepositorySettingsFetchResult,
} from "@/lib/api";

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "namuh",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "public",
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
      contributorsCount: 1,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:namuh/opengithub.git",
      https: "https://opengithub.namuh.co/namuh/opengithub.git",
      zip: "/namuh/opengithub/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function repositorySettings(
  overrides: Partial<RepositorySettings> = {},
): RepositorySettings {
  return {
    id: "repo-1",
    ownerLogin: "namuh",
    name: "opengithub",
    description: "A calmer place for code to live.",
    visibility: "public",
    defaultBranch: "main",
    isTemplate: false,
    allowForking: true,
    webCommitSignoffRequired: false,
    features: {
      issuesEnabled: false,
      projectsEnabled: true,
      wikiEnabled: true,
    },
    merge: {
      allowSquash: true,
      allowMergeCommit: true,
      allowRebase: true,
      defaultMethod: "squash",
    },
    danger: {
      isArchived: false,
      canArchive: true,
      canUnarchive: false,
      deleteSupported: false,
      transferSupported: false,
    },
    branches: ["main", "release/next", "docs"],
    viewerPermission: "admin",
    updatedAt: "2026-05-03T00:00:00Z",
    auditEvents: [
      {
        id: "audit-1",
        eventType: "repository.settings.update",
        changedFields: ["features", "merge"],
        actorUserId: "user-1",
        createdAt: "2026-05-02T00:00:00Z",
      },
    ],
    ...overrides,
  };
}

function okResult(
  overrides: Partial<RepositorySettings> = {},
): RepositorySettingsFetchResult {
  return { ok: true, settings: repositorySettings(overrides) };
}

describe("repository general settings page", () => {
  it("renders the real admin settings state with Editorial primitives", () => {
    const { container } = render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "namuh/opengithub" }),
    ).toBeVisible();
    expect(screen.getByLabelText("Repository name")).toHaveValue("opengithub");
    expect(screen.getByLabelText("Repository description")).toHaveValue(
      "A calmer place for code to live.",
    );
    expect(screen.getByText("public")).toBeVisible();
    expect(screen.getByLabelText("Default branch")).toHaveValue("main");
    expect(screen.getByText("Forking enabled")).toBeVisible();
    expect(screen.getByText("Web signoff optional")).toBeVisible();
    expect(screen.getByText("Default method: Squash")).toBeVisible();
    expect(screen.getByText("repository.settings.update")).toBeVisible();
    expect(screen.getByRole("link", { name: "View branches" })).toHaveAttribute(
      "href",
      "/namuh/opengithub/branches",
    );

    expect(container.querySelectorAll(".card").length).toBeGreaterThan(4);
    expect(container.querySelector(".input")).not.toBeNull();
    expect(container.querySelector(".chip")).not.toBeNull();
    expect(container.innerHTML).toContain("var(--ink-3)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("shows current feature, merge, archive, and branch values without editable local state", () => {
    render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult({
          isTemplate: true,
          webCommitSignoffRequired: true,
          merge: {
            allowSquash: false,
            allowMergeCommit: true,
            allowRebase: false,
            defaultMethod: "merge_commit",
          },
          danger: {
            isArchived: true,
            canArchive: false,
            canUnarchive: true,
            deleteSupported: false,
            transferSupported: false,
          },
        })}
      />,
    );

    expect(screen.getByText("Enabled")).toBeVisible();
    expect(screen.getByText("Archived")).toBeVisible();
    expect(screen.getByText("Web signoff required")).toBeVisible();
    expect(screen.getByText("Default method: Merge commit")).toBeVisible();
    expect(screen.getByLabelText("Allow squash merging")).not.toBeChecked();
    expect(screen.getByLabelText("Allow merge commits")).toBeChecked();
    expect(screen.getByLabelText("Allow rebase merging")).not.toBeChecked();
    expect(
      screen.getByRole("button", { name: "Unarchive repository unavailable" }),
    ).toBeDisabled();
  });

  it("renders forbidden and unavailable states without leaking settings", () => {
    render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: false,
          status: 403,
          code: "forbidden",
          message: "permission denied",
        }}
      />,
    );

    expect(screen.getByText("Admin access required")).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Repository settings are restricted",
      }),
    ).toBeVisible();
    expect(screen.queryByLabelText("Repository name")).not.toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Repository Code" }),
    ).toHaveAttribute("href", "/namuh/opengithub");
  });

  it("keeps danger zone actions disabled and avoids inert links or unnamed buttons", () => {
    const { container } = render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    const dangerZone = screen
      .getByRole("heading", { name: "Destructive actions" })
      .closest("section");
    expect(dangerZone).not.toBeNull();
    for (const name of [
      "Archive repository unavailable",
      "Transfer repository unavailable",
      "Delete repository unavailable",
    ]) {
      expect(
        within(dangerZone as HTMLElement).getByRole("button", { name }),
      ).toBeDisabled();
    }

    expect(
      container.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of Array.from(container.querySelectorAll("button"))) {
      expect(button).toHaveAccessibleName(/.+/);
    }
  });
});
