import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectAccessSettingsPage } from "@/components/ProjectAccessSettingsPage";
import { ProjectDangerZonePage } from "@/components/ProjectDangerZonePage";
import { ProjectSettingsPage } from "@/components/ProjectSettingsPage";
import { ProjectTemplateSettingsPage } from "@/components/ProjectTemplateSettingsPage";
import type { ProjectSettings } from "@/lib/api";

const mockRouterPush = vi.hoisted(() => vi.fn());

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockRouterPush }),
}));

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
    mockRouterPush.mockReset();
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
    expect(screen.getByRole("link", { name: "Access" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/access",
    );
    expect(screen.getByRole("link", { name: "Templates" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/templates",
    );
    expect(screen.getByRole("link", { name: "Danger Zone" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/danger",
    );
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

  it("renders access settings and submits user, team, role, and remove mutations", async () => {
    const base = settings({
      teamGrants: [
        {
          id: "team-grant-1",
          team: {
            id: "team-1",
            slug: "platform",
            name: "Platform",
            href: "/orgs/namuh/teams/platform",
          },
          role: "read",
          memberCount: 4,
          updatedAt: "2026-05-03T00:00:00Z",
        },
      ],
      eligibleUsers: [{ id: "user-3", login: "lee", avatarUrl: null }],
      eligibleTeams: [
        {
          id: "team-2",
          slug: "docs",
          name: "Docs",
          href: "/orgs/namuh/teams/docs",
        },
      ],
    });
    const fetchMock = vi
      .spyOn(global, "fetch")
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          settings({
            ...base,
            accessGrants: [
              ...base.accessGrants,
              {
                id: "grant-2",
                user: { id: "user-3", login: "lee", avatarUrl: null },
                role: "read",
                source: "direct",
                inherited: false,
                updatedAt: "2026-05-06T00:00:00Z",
              },
            ],
          }),
      } as Response)
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          settings({
            ...base,
            accessGrants: [{ ...base.accessGrants[0], role: "admin" }],
          }),
      } as Response)
      .mockResolvedValueOnce({ ok: true, json: async () => base } as Response);

    render(
      <ProjectAccessSettingsPage
        owner="namuh"
        scope="organization"
        settings={base}
      />,
    );

    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings",
    );
    expect(screen.getByText("Platform")).toBeVisible();

    fireEvent.change(screen.getByLabelText("Collaborator or team"), {
      target: { value: "user:user-3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add access" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/access-grants",
        expect.objectContaining({
          body: expect.stringContaining('"targetType":"user"'),
          method: "POST",
        }),
      ),
    );
    expect(await screen.findByText("Project access granted.")).toBeVisible();

    fireEvent.change(screen.getByLabelText("Role for mona"), {
      target: { value: "admin" },
    });
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/access-grants/grant-1",
        expect.objectContaining({
          body: expect.stringContaining('"role":"admin"'),
          method: "PATCH",
        }),
      ),
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/access-grants/grant-1",
        expect.objectContaining({ method: "DELETE" }),
      ),
    );
  });

  it("disables access controls for read-only project viewers", () => {
    render(
      <ProjectAccessSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings({
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

    expect(screen.getByRole("button", { name: "Add access" })).toBeDisabled();
    expect(screen.getByLabelText("Role for mona")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Remove" })).toBeDisabled();
    expect(screen.getByText("Read-only")).toBeVisible();
  });

  it("renders danger zone and closes, reopens, and deletes through lifecycle endpoints", async () => {
    const fetchMock = vi
      .spyOn(global, "fetch")
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          settings({
            project: { ...settings().project, state: "closed" },
            dangerState: {
              ...settings().dangerState,
              state: "closed",
              closedAt: "2026-05-06T00:00:00Z",
              closedBy: { id: "user-1", login: "ashley", avatarUrl: null },
            },
            viewerPermissions: {
              ...settings().viewerPermissions,
              canClose: false,
              canReopen: true,
            },
          }),
      } as Response)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => settings(),
      } as Response)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          deleted: true,
          projectId: "project-1",
          destinationHref: "/orgs/namuh/projects",
        }),
      } as Response);

    render(
      <ProjectDangerZonePage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings",
    );
    expect(screen.getByLabelText("Type Editorial planning")).toHaveValue("");
    expect(
      screen.getByRole("button", { name: "Delete project" }),
    ).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Close project" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/close",
        expect.objectContaining({ method: "POST" }),
      ),
    );
    expect(await screen.findByText("Project closed.")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Reopen project" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/reopen",
        expect.objectContaining({ method: "POST" }),
      ),
    );

    fireEvent.change(screen.getByLabelText("Type Editorial planning"), {
      target: { value: "Editorial planning" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Delete project" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1",
        expect.objectContaining({
          body: expect.stringContaining('"confirmation":"Editorial planning"'),
          method: "DELETE",
        }),
      ),
    );
  });

  it("renders the dedicated templates settings page and saves copy-source metadata", async () => {
    const fetchMock = vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: async () =>
        settings({
          template: {
            isTemplate: true,
            templateId: "template-2",
            title: "QA launch template",
            description: "Copy the verified launch board.",
            isPublic: true,
            createdAt: "2026-05-06T00:00:00Z",
          },
        }),
    } as Response);

    render(
      <ProjectTemplateSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings({
          template: {
            isTemplate: false,
            templateId: null,
            title: null,
            description: null,
            isPublic: false,
            createdAt: null,
          },
        })}
      />,
    );

    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings",
    );
    expect(screen.getByText("Copy-source settings")).toBeVisible();
    fireEvent.click(screen.getByLabelText("Set this project as a template"));
    fireEvent.change(screen.getByLabelText("Template title"), {
      target: { value: "QA launch template" },
    });
    fireEvent.change(screen.getByLabelText("Copy-source information"), {
      target: { value: "Copy the verified launch board." },
    });
    fireEvent.click(screen.getByLabelText("Allow copies from visible users"));
    fireEvent.click(screen.getByRole("button", { name: "Save template" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/template",
        expect.objectContaining({
          body: expect.stringContaining("QA launch template"),
          method: "PATCH",
        }),
      ),
    );
    expect(await screen.findByText("Template settings saved.")).toBeVisible();
    expect(screen.getByText("Template template-2")).toBeVisible();
  });
});
