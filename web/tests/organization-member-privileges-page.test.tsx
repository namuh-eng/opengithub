import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OrganizationMemberPrivilegesPage } from "@/components/OrganizationMemberPrivilegesPage";
import type { OrganizationMemberPrivilegesSettings } from "@/lib/api";

function memberPrivilegeSettings(
  overrides: Partial<OrganizationMemberPrivilegesSettings> = {},
): OrganizationMemberPrivilegesSettings {
  const base: OrganizationMemberPrivilegesSettings = {
    capabilities: {
      canUpdate: true,
      locks: [],
      requiresConfirmationFields: [
        "baseRepositoryPermission",
        "projectsBasePermission",
      ],
    },
    organization: {
      href: "/orgs/acme-labs",
      id: "org-1",
      name: "Acme Labs",
      settingsHref: "/organizations/acme-labs/settings/member_privileges",
      slug: "acme-labs",
    },
    policies: {
      appAccessRequestPolicy: "owners_and_members",
      baseRepositoryPermission: "read",
      membersCanChangeRepositoryVisibility: false,
      membersCanCreateInternalRepositories: false,
      membersCanCreatePrivateRepositories: true,
      membersCanCreatePublicRepositories: true,
      membersCanCreateTeams: true,
      membersCanDeleteIssues: false,
      membersCanDeleteRepositories: false,
      membersCanForkPrivateRepositories: true,
      membersCanTransferRepositories: false,
      pagesPrivatePublishing: true,
      pagesPublicPublishing: true,
      projectsBasePermission: "write",
      repositoryDiscussionsEnabled: true,
    },
    viewerState: {
      canArchive: false,
      canDelete: false,
      canEditProfile: true,
      canRename: false,
      role: "owner",
    },
  };
  return { ...base, ...overrides };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("OrganizationMemberPrivilegesPage", () => {
  it("renders every policy card with Editorial controls and concrete links", () => {
    const { container } = render(
      <OrganizationMemberPrivilegesPage settings={memberPrivilegeSettings()} />,
    );

    expect(screen.getByRole("heading", { name: "Base permissions" }));
    expect(screen.getByRole("heading", { name: "Creation visibility" }));
    expect(screen.getByRole("heading", { name: "Forking and discussions" }));
    expect(screen.getByRole("heading", { name: "Projects base permission" }));
    expect(screen.getByRole("heading", { name: "Publishing policy" }));
    expect(screen.getByRole("heading", { name: "App access requests" }));
    expect(
      screen.getByRole("heading", {
        name: "Visibility, delete, and transfer",
      }),
    );
    expect(screen.getByRole("heading", { name: "Team creation" }));
    expect(screen.getByRole("link", { name: "API docs" })).toHaveAttribute(
      "href",
      "/docs/api#organization-member-privileges",
    );
    expect(screen.getByLabelText("Public repositories")).toBeChecked();
    expect(screen.getByLabelText("Internal repositories")).not.toBeChecked();
    expect(screen.getByLabelText("Owners and members")).toBeChecked();
    expect(container.querySelectorAll(".card").length).toBeGreaterThan(7);
    expect(container.querySelector('a[href="#"], a:not([href])')).toBeNull();
    for (const button of screen.getAllByRole("button")) {
      expect(button).toHaveAccessibleName(/.+/);
    }
    expect(container.innerHTML).toContain("var(--surface-2)");
    expect(container.innerHTML).not.toMatch(
      /#0969da|#1f883d|#cf222e|@primer\/|Octicon/i,
    );
  });

  it("saves repository creation, team creation, and app access cards with focused feedback", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    fetchMock
      .mockResolvedValueOnce({
        json: async () =>
          memberPrivilegeSettings({
            policies: {
              ...memberPrivilegeSettings().policies,
              membersCanCreatePublicRepositories: false,
            },
          }),
        ok: true,
      } as Response)
      .mockResolvedValueOnce({
        json: async () =>
          memberPrivilegeSettings({
            policies: {
              ...memberPrivilegeSettings().policies,
              membersCanCreatePublicRepositories: false,
              membersCanCreateTeams: false,
            },
          }),
        ok: true,
      } as Response)
      .mockResolvedValueOnce({
        json: async () =>
          memberPrivilegeSettings({
            policies: {
              ...memberPrivilegeSettings().policies,
              appAccessRequestPolicy: "owners_only",
              membersCanCreatePublicRepositories: false,
              membersCanCreateTeams: false,
            },
          }),
        ok: true,
      } as Response);

    render(
      <OrganizationMemberPrivilegesPage settings={memberPrivilegeSettings()} />,
    );

    expect(
      screen.getByRole("button", { name: "Save repository creation" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByLabelText("Public repositories"));
    fireEvent.click(
      screen.getByRole("button", { name: "Save repository creation" }),
    );
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/organizations/acme-labs/settings/member_privileges/actions",
        expect.objectContaining({
          body: JSON.stringify({
            membersCanCreatePublicRepositories: false,
          }),
          method: "PATCH",
        }),
      );
    });
    expect(
      await screen.findByText("Repository creation policy updated"),
    ).toBeVisible();

    fireEvent.click(screen.getByLabelText("Members can create teams"));
    fireEvent.click(screen.getByRole("button", { name: "Save team creation" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/organizations/acme-labs/settings/member_privileges/actions",
        expect.objectContaining({
          body: JSON.stringify({ membersCanCreateTeams: false }),
          method: "PATCH",
        }),
      );
    });

    fireEvent.click(screen.getByLabelText("Owners only"));
    fireEvent.click(screen.getByRole("button", { name: "Save app access" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/organizations/acme-labs/settings/member_privileges/actions",
        expect.objectContaining({
          body: JSON.stringify({ appAccessRequestPolicy: "owners_only" }),
          method: "PATCH",
        }),
      );
    });
  });

  it("requires a confirmation dialog for base and Projects permission changes", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    fetchMock.mockResolvedValue({
      json: async () =>
        memberPrivilegeSettings({
          policies: {
            ...memberPrivilegeSettings().policies,
            baseRepositoryPermission: "admin",
          },
        }),
      ok: true,
    } as Response);

    render(
      <OrganizationMemberPrivilegesPage settings={memberPrivilegeSettings()} />,
    );

    fireEvent.click(screen.getAllByLabelText("Admin")[0]);
    fireEvent.click(
      screen.getByRole("button", { name: "Save base permission" }),
    );
    expect(
      screen.getByRole("dialog", {
        name: "Confirm organization policy change",
      }),
    ).toBeVisible();
    expect(fetchMock).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Confirm and save" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/organizations/acme-labs/settings/member_privileges/actions",
        expect.objectContaining({
          body: JSON.stringify({
            baseRepositoryPermission: "admin",
            confirmation: "confirm",
          }),
          method: "PATCH",
        }),
      );
    });
    expect(await screen.findByText("Base repository permission updated"));
  });

  it("focuses server confirmation errors and renders policy locks as disabled controls", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch");
    fetchMock.mockResolvedValue({
      json: async () => ({
        details: {
          confirmation: "confirm",
          fields: ["projectsBasePermission"],
        },
        error: {
          code: "confirmation_required",
          message: "Confirm this organization policy change before saving.",
        },
        status: 409,
      }),
      ok: false,
    } as Response);
    const settings = memberPrivilegeSettings({
      capabilities: {
        canUpdate: true,
        locks: [
          {
            enforcedBy: "enterprise",
            field: "membersCanCreatePublicRepositories",
            href: "/docs/api#organization-member-privileges",
            reason: "Enterprise policy requires owner approval.",
          },
        ],
        requiresConfirmationFields: [
          "baseRepositoryPermission",
          "projectsBasePermission",
        ],
      },
    });

    render(<OrganizationMemberPrivilegesPage settings={settings} />);

    expect(screen.getByLabelText("Public repositories")).toBeDisabled();
    expect(screen.getByText("Enterprise policy requires owner approval."));
    expect(screen.getByRole("link", { name: "Why" })).toHaveAttribute(
      "href",
      "/docs/api#organization-member-privileges",
    );

    fireEvent.click(screen.getByLabelText("Internal repositories"));
    fireEvent.click(
      screen.getByRole("button", { name: "Save repository creation" }),
    );
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/organizations/acme-labs/settings/member_privileges/actions",
        expect.objectContaining({
          body: JSON.stringify({
            membersCanCreateInternalRepositories: true,
          }),
          method: "PATCH",
        }),
      );
    });
    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveFocus();
    });
    expect(
      screen.getByRole("dialog", {
        name: "Confirm organization policy change",
      }),
    ).toBeVisible();
  });
});
