import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProjectSettingsPage } from "@/components/ProjectSettingsPage";
import type { ProjectSettings } from "@/lib/api";

function settings(overrides: Partial<ProjectSettings> = {}): ProjectSettings {
  return {
    project: {
      id: "project-1",
      number: 12,
      title: "Editorial planning",
      description: "Tracks the launch plan.",
      state: "open",
      visibility: "private",
      owner: "namuh",
      href: "/orgs/namuh/projects/12",
      workspaceHref: "/orgs/namuh/projects/12/views/1",
      viewerRole: "admin",
    },
    general: {
      title: "Editorial planning",
      description: "Tracks the launch plan.",
      readme: "## Launch\nKeep the release calm and legible.",
      visibility: "private",
      defaultRepositoryId: "repo-1",
      createdBy: { id: "user-1", login: "ashley", avatarUrl: null },
      createdAt: "2026-05-01T00:00:00Z",
      updatedAt: "2026-05-05T00:00:00Z",
      readmeRevisionCount: 3,
    },
    policy: {
      ownerKind: "organization",
      organizationId: "org-1",
      projectsEnabled: true,
      basePermission: "read",
      visibilityChangesAllowed: true,
      visibilityLockedReason: null,
    },
    repositories: [
      {
        id: "link-1",
        repositoryId: "repo-1",
        owner: "namuh",
        name: "opengithub",
        fullName: "namuh/opengithub",
        href: "/namuh/opengithub",
        visibility: "private",
        linkType: "primary",
        isDefault: true,
        viewerPermission: "write",
        linkedBy: { id: "user-1", login: "ashley", avatarUrl: null },
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-02T00:00:00Z",
      },
      {
        id: "link-2",
        repositoryId: "repo-2",
        owner: "namuh",
        name: "docs",
        fullName: "namuh/docs",
        href: "/namuh/docs",
        visibility: "public",
        linkType: "secondary",
        isDefault: false,
        viewerPermission: "read",
        linkedBy: null,
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-02T00:00:00Z",
      },
    ],
    accessGrants: [
      {
        id: "grant-1",
        user: { id: "user-2", login: "mona", avatarUrl: null },
        role: "write",
        source: "direct",
        inherited: false,
        updatedAt: "2026-05-03T00:00:00Z",
      },
    ],
    teamGrants: [],
    eligibleUsers: [],
    eligibleTeams: [],
    statusUpdates: [
      {
        id: "status-1",
        status: "at_risk",
        label: "At risk",
        body: "API docs need another review before rollout.",
        startDate: "2026-05-01",
        targetDate: "2026-05-30",
        author: { id: "user-1", login: "ashley", avatarUrl: null },
        createdAt: "2026-05-06T00:00:00Z",
      },
    ],
    template: {
      isTemplate: true,
      templateId: "template-1",
      title: "Launch template",
      description: "Reusable launch board.",
      isPublic: false,
      createdAt: "2026-05-04T00:00:00Z",
    },
    dangerState: {
      state: "open",
      closedAt: null,
      closedBy: null,
      deletedAt: null,
      deletedBy: null,
      deleteConfirmation: "Editorial planning",
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "admin",
      canEditGeneral: true,
      canChangeVisibility: true,
      canLinkRepositories: true,
      canPublishStatus: true,
      canManageTemplate: true,
      canManageAccess: true,
      canClose: true,
      canReopen: false,
      canDelete: true,
    },
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectSettingsPage", () => {
  it("renders organization general settings with concrete navigation", () => {
    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Editorial planning" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to project" }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings",
    );
    expect(screen.getByRole("link", { name: "Fields" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/fields",
    );
    expect(screen.getByRole("link", { name: "Workflows" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/workflows",
    );
    expect(screen.getByRole("button", { name: "Access" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Templates" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Danger Zone" })).toBeDisabled();
  });

  it("renders metadata, README, visibility, and repository defaults", () => {
    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(screen.getByLabelText("Title")).toHaveValue("Editorial planning");
    expect(screen.getByLabelText("Short description")).toHaveValue(
      "Tracks the launch plan.",
    );
    expect(screen.getByLabelText("README Markdown")).toHaveValue(
      "## Launch\nKeep the release calm and legible.",
    );
    expect(screen.getByLabelText("Visibility")).toHaveValue("private");
    expect(screen.getByLabelText("Default repository")).toHaveValue("repo-1");
    expect(
      within(
        screen.getByRole("link", { name: /namuh\/opengithub/i }),
      ).getByText("Default"),
    ).toBeVisible();
  });

  it("shows policy-disabled visibility and read-only metadata controls", () => {
    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings({
          policy: {
            ownerKind: "organization",
            organizationId: "org-1",
            projectsEnabled: true,
            basePermission: "read",
            visibilityChangesAllowed: false,
            visibilityLockedReason:
              "Only organization owners can publish projects.",
          },
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canEditGeneral: false,
            canChangeVisibility: false,
            canLinkRepositories: false,
            canPublishStatus: false,
            canManageTemplate: false,
            canManageAccess: false,
            canClose: false,
            canReopen: false,
            canDelete: false,
          },
        })}
      />,
    );

    expect(screen.getByLabelText("Title")).toBeDisabled();
    expect(screen.getByLabelText("Visibility")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save changes" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Publish update" }),
    ).toBeDisabled();
    expect(
      screen.getAllByText("Only organization owners can publish projects.")[0],
    ).toBeVisible();
    expect(
      screen.getByText(
        "You can inspect settings, but this project role cannot change metadata.",
      ),
    ).toBeVisible();
  });

  it("renders status update controls and latest status summary", () => {
    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(screen.getByLabelText("State")).toHaveValue("at_risk");
    expect(screen.getByLabelText("Start date")).toHaveValue("2026-05-01");
    expect(screen.getByLabelText("Target date")).toHaveValue("2026-05-30");
    expect(screen.getByLabelText("Message")).toHaveValue(
      "API docs need another review before rollout.",
    );
    expect(screen.getAllByText("At risk").length).toBeGreaterThan(0);
    expect(screen.getByText(/ashley/)).toBeVisible();
  });

  it("renders user project routes without organization prefixes", () => {
    render(
      <ProjectSettingsPage owner="ashley" scope="user" settings={settings()} />,
    );

    expect(
      screen.getByRole("link", { name: "Back to project" }),
    ).toHaveAttribute("href", "/ashley/projects/12/views/1");
    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/ashley/projects/12/settings",
    );
    expect(screen.getByRole("link", { name: "Fields" })).toHaveAttribute(
      "href",
      "/ashley/projects/12/settings/fields",
    );
  });

  it("does not render placeholder links, inert inline handlers, or banned visual tokens", () => {
    const { container } = render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(container.querySelector('[href="#"]')).toBeNull();
    expect(container.innerHTML).not.toContain("onClick={() => {}}");
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.innerHTML).not.toContain("#1f883d");
    expect(container.innerHTML).not.toContain("#cf222e");
    expect(container.innerHTML).not.toContain("@primer/");
    expect(container.innerHTML).not.toContain("Octicon");
  });
});
