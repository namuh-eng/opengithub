import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RepositoryActionsSecretsPage } from "@/components/RepositoryActionsSecretsPage";
import type {
  ActionsSecretSummary,
  ActionsVariableSummary,
  RepositoryActionsSecretsSettings,
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

function actor() {
  return {
    displayName: "Ashley Test",
    id: "user-1",
    login: "ashley",
  };
}

function secret(overrides: Partial<ActionsSecretSummary> = {}) {
  return {
    createdAt: "2026-05-01T00:00:00Z",
    id: "secret-1",
    name: "DEPLOY_KEY",
    scope: { kind: "repository", name: null },
    secretConfigured: true,
    storageKind: "local_envelope",
    updatedAt: "2026-05-03T00:00:00Z",
    updatedBy: actor(),
    visibilityPolicy: "repository",
    ...overrides,
  };
}

function variable(overrides: Partial<ActionsVariableSummary> = {}) {
  return {
    createdAt: "2026-05-01T00:00:00Z",
    id: "variable-1",
    name: "PUBLIC_BASE_URL",
    scope: { kind: "repository", name: null },
    updatedAt: "2026-05-03T00:00:00Z",
    updatedBy: actor(),
    value: "https://opengithub.namuh.co",
    visibilityPolicy: "repository",
    ...overrides,
  };
}

function settings(
  overrides: Partial<RepositoryActionsSecretsSettings> = {},
): RepositoryActionsSecretsSettings {
  return {
    canEdit: true,
    inheritedSecrets: [],
    inheritedVariables: [],
    name: "opengithub",
    ownerLogin: "namuh-eng",
    repositoryId: "repo-1",
    secrets: [secret()],
    variables: [variable()],
    viewerPermission: "admin",
    visibility: "private",
    ...overrides,
  };
}

describe("RepositoryActionsSecretsPage", () => {
  it("renders secret metadata without leaking values or encrypted material", () => {
    const { container } = render(
      <RepositoryActionsSecretsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    expect(screen.getByText("DEPLOY_KEY")).toBeVisible();
    expect(screen.getByText("Write-only values")).toBeVisible();
    expect(screen.getByText("Configured")).toBeVisible();
    expect(
      screen.getByText(/Updated May 3, 2026 by Ashley Test/),
    ).toBeVisible();
    expect(screen.getByRole("link", { name: /Variables/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/secrets?tab=variables",
    );
    expect(container.textContent).not.toContain("super-secret");
    expect(container.textContent).not.toContain("ciphertext");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#1a7f37|#cf222e|@primer\/|Octicon/i,
    );
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(2);
    expect(container.innerHTML).toContain("var(--ink-2)");
  });

  it("renders variables tab with permitted values and inherited metadata", () => {
    render(
      <RepositoryActionsSecretsPage
        activeTab="variables"
        repository={repositoryOverview()}
        settingsResult={{
          ok: true,
          settings: settings({
            inheritedVariables: [
              {
                name: "ORG_CHANNEL",
                scope: { kind: "organization", name: "namuh-eng" },
                updatedAt: "2026-05-02T00:00:00Z",
                value: "release",
                visibilityPolicy: "selected_repositories",
              },
            ],
          }),
        }}
      />,
    );

    expect(screen.getByText("PUBLIC_BASE_URL")).toBeVisible();
    expect(screen.getByText("https://opengithub.namuh.co")).toBeVisible();
    expect(screen.getByText("Inherited variables")).toBeVisible();
    expect(screen.getByText("ORG_CHANNEL")).toBeVisible();
    expect(screen.getByText(/organization: namuh-eng/)).toBeVisible();
    expect(screen.getByRole("link", { name: /Secrets/ })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/secrets?tab=secrets",
    );
  });

  it("renders empty states with concrete links", () => {
    render(
      <RepositoryActionsSecretsPage
        repository={repositoryOverview()}
        settingsResult={{
          ok: true,
          settings: settings({ inheritedSecrets: [], secrets: [] }),
        }}
      />,
    );

    expect(screen.getByText("No repository secrets")).toBeVisible();
    expect(screen.getByRole("link", { name: "Add secret" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/settings/secrets?tab=secrets#repository-secrets",
    );
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs",
    );
  });

  it("renders forbidden state without leaking setting names", () => {
    const { container } = render(
      <RepositoryActionsSecretsPage
        repository={repositoryOverview({ viewerPermission: "read" })}
        settingsResult={{
          code: "forbidden",
          message: "Forbidden",
          ok: false,
          status: 403,
        }}
      />,
    );

    expect(screen.getByText("Actions secrets are restricted")).toBeVisible();
    expect(screen.getByText("Admin access required")).toBeVisible();
    expect(container.textContent).not.toContain("DEPLOY_KEY");
    expect(
      screen.getByRole("link", { name: "Repository Code" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub");
  });

  it("uses disabled controls for mutation placeholders instead of dead buttons", () => {
    render(
      <RepositoryActionsSecretsPage
        repository={repositoryOverview()}
        settingsResult={{ ok: true, settings: settings() }}
      />,
    );

    for (const button of screen.getAllByRole("button")) {
      expect(button).toBeDisabled();
      expect(button).toHaveAttribute(
        "title",
        "Mutation forms are implemented in the next settings phase.",
      );
    }
  });
});
