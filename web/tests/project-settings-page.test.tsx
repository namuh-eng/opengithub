import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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
  afterEach(() => {
    vi.restoreAllMocks();
  });

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
      screen.getByRole("link", { name: /namuh\/opengithub/i }),
    ).toBeVisible();
    expect(screen.getByText("Default")).toBeVisible();
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

  it("submits general settings and refreshes success feedback from the API", async () => {
    const fetchMock = vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: async () =>
        settings({
          general: {
            ...settings().general,
            title: "Updated planning",
            updatedAt: "2026-05-06T00:00:00Z",
          },
        }),
    } as Response);

    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Updated planning" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save changes" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/settings",
        expect.objectContaining({
          body: expect.stringContaining("Updated planning"),
          method: "PATCH",
        }),
      ),
    );
    expect(await screen.findByText("Project settings saved.")).toBeVisible();
    expect(
      screen.getByRole("heading", { name: "Updated planning" }),
    ).toBeVisible();
  });

  it("publishes status updates and saves template settings through real endpoints", async () => {
    const fetchMock = vi
      .spyOn(global, "fetch")
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          settings({
            statusUpdates: [
              {
                ...settings().statusUpdates[0],
                status: "complete",
                label: "Complete",
              },
            ],
          }),
      } as Response)
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          settings({
            template: {
              ...settings().template,
              isTemplate: false,
              title: null,
            },
          }),
      } as Response);

    render(
      <ProjectSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    fireEvent.change(screen.getByLabelText("State"), {
      target: { value: "complete" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Publish update" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/status-updates",
        expect.objectContaining({ method: "POST" }),
      ),
    );

    fireEvent.click(screen.getByLabelText("Set this project as a template"));
    fireEvent.click(screen.getByRole("button", { name: "Save template" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/template",
        expect.objectContaining({
          body: expect.stringContaining('"isTemplate":false'),
          method: "PATCH",
        }),
      ),
    );
  });
});
