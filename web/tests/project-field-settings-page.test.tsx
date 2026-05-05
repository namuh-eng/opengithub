import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProjectFieldSettingsPage } from "@/components/ProjectFieldSettingsPage";
import type { ProjectFieldSettings } from "@/lib/api";

function settings(
  overrides: Partial<ProjectFieldSettings> = {},
): ProjectFieldSettings {
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
      viewerRole: "write",
    },
    fields: [
      {
        id: "field-title",
        name: "Title",
        fieldType: "title",
        position: 1,
        settings: {},
        builtIn: true,
        editable: false,
        deletable: false,
        usageCount: 8,
        options: [],
        iterations: [],
        breaks: [],
        cacheVersion: 1,
        updatedAt: "2026-05-01T00:00:00Z",
      },
      {
        id: "field-status",
        name: "Status",
        fieldType: "single_select",
        position: 2,
        settings: {},
        builtIn: false,
        editable: true,
        deletable: true,
        usageCount: 7,
        options: [
          {
            id: "option-1",
            name: "Backlog",
            color: "yellow",
            position: 1,
            description: "Not started",
          },
          {
            id: "option-2",
            name: "In progress",
            color: "rust",
            position: 2,
            description: null,
          },
        ],
        iterations: [],
        breaks: [],
        cacheVersion: 2,
        updatedAt: "2026-05-03T00:00:00Z",
      },
      {
        id: "field-cycle",
        name: "Cycle",
        fieldType: "iteration",
        position: 3,
        settings: { durationUnit: "weeks" },
        builtIn: false,
        editable: true,
        deletable: true,
        usageCount: 3,
        options: [],
        iterations: [
          {
            id: "iteration-1",
            name: "Sprint 1",
            startDate: "2026-05-04",
            durationDays: 14,
            position: 1,
          },
        ],
        breaks: [
          {
            id: "break-1",
            name: "Planning break",
            startDate: "2026-05-18",
            durationDays: 2,
          },
        ],
        cacheVersion: 1,
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    limits: {
      maxFields: 50,
      usedFields: 3,
      remainingFields: 47,
      maxOptionsPerField: 25,
      maxIterationsPerField: 20,
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "write",
      canCreateFields: true,
      canRenameFields: true,
      canDeleteFields: true,
      canManageOptions: true,
      canManageIterations: true,
    },
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectFieldSettingsPage", () => {
  it("renders the Editorial field settings shell with concrete navigation", () => {
    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Editorial planning" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Back to project" }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "Fields" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/fields",
    );
    expect(screen.getByRole("link", { name: /Status/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/settings/fields?field=field-status",
    );
    expect(screen.getByText("3 of 50 used · 47 remaining")).toBeInTheDocument();
  });

  it("selects a field from query state and renders single-select options", () => {
    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        selectedFieldId="field-status"
        settings={settings()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Status" })).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toHaveValue("Status");
    expect(screen.getByLabelText("Type")).toHaveValue("single_select");
    expect(screen.getByText("2 of 25 options.")).toBeInTheDocument();
    expect(screen.getByText("Backlog")).toBeInTheDocument();
    expect(screen.getByText("In progress")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add option" })).toBeDisabled();
  });

  it("renders iteration schedules and read-only mutation controls", () => {
    render(
      <ProjectFieldSettingsPage
        owner="mona"
        scope="user"
        selectedFieldId="field-cycle"
        settings={settings()}
      />,
    );

    expect(
      screen.getByRole("link", { name: "Back to project" }),
    ).toHaveAttribute("href", "/mona/projects/12/views/1");
    expect(screen.getByRole("heading", { name: "Cycle" })).toBeInTheDocument();
    expect(screen.getByText("Sprint 1")).toBeInTheDocument();
    expect(screen.getByText("Planning break")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add iteration" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save changes" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Rename" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Delete" })).toBeDisabled();
  });

  it("opens the New field dialog while keeping creation disabled for the later phase", () => {
    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "New field" }));
    const dialog = screen.getByRole("dialog");
    expect(
      within(dialog).getByRole("heading", { name: "New field" }),
    ).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Name")).toHaveAttribute(
      "placeholder",
      "Priority",
    );
    expect(
      within(dialog).getByRole("button", { name: "Create field" }),
    ).toBeDisabled();
  });

  it("shows permission-disabled controls and no placeholder links", () => {
    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        settings={settings({
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canCreateFields: false,
            canRenameFields: false,
            canDeleteFields: false,
            canManageOptions: false,
            canManageIterations: false,
          },
        })}
      />,
    );

    expect(screen.getByRole("button", { name: "New field" })).toBeDisabled();
    expect(
      screen.getByText(
        "You can inspect fields, but this project role cannot change them.",
      ),
    ).toBeInTheDocument();
    for (const link of screen.getAllByRole("link")) {
      expect(link).not.toHaveAttribute("href", "#");
    }
  });
});
