import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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
    policyLocks: [],
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

function mockFetch(response: unknown, ok = true) {
  return vi.fn().mockResolvedValue({
    json: async () => response,
    ok,
  }) as unknown as typeof fetch;
}

afterEach(() => {
  vi.restoreAllMocks();
});

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
    expect(screen.getByLabelText("Repository visibility")).toHaveValue(
      "public",
    );
    expect(screen.getByLabelText("Default branch")).toHaveValue("main");
    expect(screen.getByLabelText("Allow forking")).toBeChecked();
    expect(
      screen.getByLabelText("Require web commit signoff"),
    ).not.toBeChecked();
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

  it("shows current feature, merge, archive, and branch values as editable controls", () => {
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

    expect(screen.getByText("Archived")).toBeVisible();
    expect(screen.getByLabelText("Template repository")).toBeChecked();
    expect(screen.getByLabelText("Require web commit signoff")).toBeChecked();
    expect(screen.getByText("Default method: Merge commit")).toBeVisible();
    expect(screen.getByLabelText("Allow squash merging")).not.toBeChecked();
    expect(screen.getByLabelText("Allow merge commits")).toBeChecked();
    expect(screen.getByLabelText("Allow rebase merging")).not.toBeChecked();
    expect(
      screen.getByRole("button", { name: "Unarchive repository" }),
    ).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Archive confirmation"), {
      target: { value: "namuh/opengithub" },
    });
    expect(
      screen.getByRole("button", { name: "Unarchive repository" }),
    ).toBeEnabled();
  });

  it("renders organization policy locks for visibility, forking, and destructive actions", () => {
    render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview({
          owner_organization_id: "org-1",
          owner_user_id: null,
          owner_login: "namuh-eng",
        })}
        settingsResult={okResult({
          ownerLogin: "namuh-eng",
          visibility: "private",
          policyLocks: [
            {
              field: "visibility",
              reason:
                "Organization policy prevents members from changing repository visibility.",
              settingsHref:
                "/organizations/namuh-eng/settings/member_privileges",
            },
            {
              field: "allowForking",
              reason:
                "Organization policy prevents private repository forking.",
              settingsHref:
                "/organizations/namuh-eng/settings/member_privileges",
            },
            {
              field: "transferRepository",
              reason:
                "Organization policy prevents members from transferring repositories.",
              settingsHref:
                "/organizations/namuh-eng/settings/member_privileges",
            },
            {
              field: "deleteRepository",
              reason:
                "Organization policy prevents members from deleting repositories.",
              settingsHref:
                "/organizations/namuh-eng/settings/member_privileges",
            },
          ],
        })}
      />,
    );

    expect(screen.getByLabelText("Repository visibility")).toBeDisabled();
    expect(screen.getByLabelText("Allow forking")).toBeDisabled();
    expect(
      screen.getByRole("link", {
        name: /prevents members from changing repository visibility/i,
      }),
    ).toHaveAttribute(
      "href",
      "/organizations/namuh-eng/settings/member_privileges",
    );
    expect(
      screen.getByRole("button", {
        name: "Transfer repository unavailable",
      }),
    ).toHaveTextContent("Transfer locked by organization policy");
    expect(
      screen.getByRole("button", { name: "Delete repository unavailable" }),
    ).toHaveTextContent("Delete locked by organization policy");
  });

  it("submits profile, feature, behavior, and archive writes through the same-origin route", async () => {
    global.fetch = vi
      .fn()
      .mockResolvedValueOnce({
        json: async () =>
          repositorySettings({
            description: "Updated through the settings form.",
            name: "opengithub-next",
          }),
        ok: true,
      })
      .mockResolvedValueOnce({
        json: async () =>
          repositorySettings({
            features: {
              issuesEnabled: true,
              projectsEnabled: true,
              wikiEnabled: false,
            },
          }),
        ok: true,
      })
      .mockResolvedValueOnce({
        json: async () =>
          repositorySettings({
            allowForking: false,
            webCommitSignoffRequired: true,
          }),
        ok: true,
      })
      .mockResolvedValueOnce({
        json: async () =>
          repositorySettings({
            danger: {
              isArchived: true,
              canArchive: false,
              canUnarchive: true,
              deleteSupported: false,
              transferSupported: false,
            },
          }),
        ok: true,
      }) as unknown as typeof fetch;

    render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Repository name"), {
      target: { value: "opengithub-next" },
    });
    fireEvent.change(screen.getByLabelText("Repository description"), {
      target: { value: "Updated through the settings form." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save profile" }));

    await waitFor(() =>
      expect(global.fetch).toHaveBeenCalledWith(
        "/namuh/opengithub/settings/update",
        {
          body: JSON.stringify({
            description: "Updated through the settings form.",
            name: "opengithub-next",
          }),
          headers: { "content-type": "application/json" },
          method: "PATCH",
        },
      ),
    );
    expect(screen.getByText("Repository profile saved.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Save features" }));

    await waitFor(() =>
      expect(global.fetch).toHaveBeenLastCalledWith(
        "/namuh/opengithub/settings/update",
        {
          body: JSON.stringify({
            features: {
              issuesEnabled: false,
              projectsEnabled: true,
              wikiEnabled: true,
            },
          }),
          headers: { "content-type": "application/json" },
          method: "PATCH",
        },
      ),
    );

    fireEvent.click(screen.getByRole("button", { name: "Save behavior" }));

    await waitFor(() =>
      expect(global.fetch).toHaveBeenLastCalledWith(
        "/namuh/opengithub/settings/update",
        {
          body: JSON.stringify({
            allowForking: true,
            webCommitSignoffRequired: false,
          }),
          headers: { "content-type": "application/json" },
          method: "PATCH",
        },
      ),
    );
    await waitFor(() =>
      expect(screen.getByText("Repository behavior saved.")).toBeVisible(),
    );

    fireEvent.change(screen.getByLabelText("Archive confirmation"), {
      target: { value: "namuh/opengithub" },
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Archive repository" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Archive repository" }));

    await waitFor(() =>
      expect(global.fetch).toHaveBeenLastCalledWith(
        "/namuh/opengithub/settings/update",
        {
          body: JSON.stringify({ isArchived: true }),
          headers: { "content-type": "application/json" },
          method: "PATCH",
        },
      ),
    );
    expect(screen.getByText("Repository archived.")).toBeVisible();
  });

  it("keeps server state after failed writes and blocks invalid merge submissions", async () => {
    global.fetch = mockFetch(
      {
        error: {
          code: "validation_failed",
          message: "repository default branch `ghost` was not found",
        },
        status: 422,
      },
      false,
    );

    render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview()}
        settingsResult={okResult()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Repository visibility"), {
      target: { value: "private" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save state" }));

    await waitFor(() =>
      expect(
        screen.getByText("repository default branch `ghost` was not found"),
      ).toBeVisible(),
    );
    expect(screen.getByLabelText("Repository visibility")).toHaveValue(
      "public",
    );

    fireEvent.click(screen.getByLabelText("Allow squash merging"));
    fireEvent.click(screen.getByLabelText("Allow merge commits"));
    fireEvent.click(screen.getByLabelText("Allow rebase merging"));
    expect(
      screen.getByRole("button", { name: "Save merge methods" }),
    ).toBeDisabled();
    expect(
      screen.getByText("At least one merge method must remain enabled."),
    ).toBeVisible();
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
    const archiveButton = within(dangerZone as HTMLElement).getByRole(
      "button",
      {
        name: "Archive repository",
      },
    );
    expect(archiveButton).toBeDisabled();
    expect(
      within(dangerZone as HTMLElement).getByLabelText("Archive confirmation"),
    ).toHaveValue("");
    fireEvent.change(
      within(dangerZone as HTMLElement).getByLabelText("Archive confirmation"),
      { target: { value: "namuh/opengithub" } },
    );
    expect(archiveButton).toBeEnabled();
    expect(
      within(dangerZone as HTMLElement).getByText(
        /Transfer and delete confirmation flows stay disabled/,
      ),
    ).toBeVisible();
    for (const name of [
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

  it("wraps long repository metadata and preserves guarded archive controls", () => {
    const longName = "opengithub-settings-guardrail-with-long-name";
    const longDescription =
      "Long repository descriptions stay readable inside the Editorial settings cards without forcing horizontal overflow on narrow screens.";

    const { container } = render(
      <RepositoryGeneralSettingsPage
        repository={repositoryOverview({ name: longName })}
        settingsResult={okResult({
          description: longDescription,
          name: longName,
        })}
      />,
    );

    expect(
      screen.getByRole("heading", { name: `namuh/${longName}` }),
    ).toBeVisible();
    expect(screen.getByLabelText("Repository description")).toHaveValue(
      longDescription,
    );
    expect(screen.getByText(`Type namuh/${longName} to confirm`)).toBeVisible();
    expect(container.innerHTML).toContain("min-w-0");
    expect(container.innerHTML).not.toMatch(/href="#"/);
  });
});
