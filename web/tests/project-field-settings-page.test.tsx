import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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
  afterEach(() => {
    vi.restoreAllMocks();
  });

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
    expect(screen.getByLabelText("Backlog option name")).toHaveDisplayValue(
      "Backlog",
    );
    expect(screen.getByLabelText("In progress option name")).toHaveDisplayValue(
      "In progress",
    );
    expect(screen.getByRole("button", { name: "Add option" })).toBeDisabled();
    expect(screen.getByLabelText("Backlog option name")).toHaveValue("Backlog");
    expect(screen.getByLabelText("Backlog option color")).toHaveValue("yellow");
  });

  it("renders iteration schedules and mutation controls for editable fields", () => {
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
    expect(screen.getByRole("button", { name: "Delete" })).toBeEnabled();
  });

  it("creates a field through the same-origin project field API", async () => {
    const nextSettings = settings({
      fields: [
        ...settings().fields,
        {
          id: "field-priority",
          name: "Priority",
          fieldType: "single_select",
          position: 4,
          settings: {},
          builtIn: false,
          editable: true,
          deletable: true,
          usageCount: 0,
          options: [],
          iterations: [],
          breaks: [],
          cacheVersion: 1,
          updatedAt: "2026-05-05T00:00:00Z",
        },
      ],
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(nextSettings), {
        status: 201,
        headers: { "content-type": "application/json" },
      }),
    );

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
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "Priority" },
    });
    fireEvent.change(within(dialog).getByLabelText("Type"), {
      target: { value: "single_select" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Create field" }),
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/fields",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            name: "Priority",
            fieldType: "single_select",
          }),
        }),
      ),
    );
    expect(await screen.findByText("Field created.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Priority/ })).toBeInTheDocument();
  });

  it("renames and deletes custom fields with stale timestamp payloads", async () => {
    const renamedSettings = settings({
      fields: settings().fields.map((field) =>
        field.id === "field-status" ? { ...field, name: "Stage" } : field,
      ),
    });
    const deletedSettings = settings({
      fields: settings().fields.filter((field) => field.id !== "field-status"),
    });
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(JSON.stringify(renamedSettings), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(deletedSettings), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        selectedFieldId="field-status"
        settings={settings()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Stage" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save changes" }));
    await screen.findByText("Field renamed.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/api/projects/project-1/fields/field-status",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          name: "Stage",
          expectedUpdatedAt: "2026-05-03T00:00:00Z",
        }),
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    const dialog = screen.getByRole("dialog");
    expect(
      within(dialog).getByText(/Linked issues and pull requests/),
    ).toBeInTheDocument();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Delete field" }),
    );
    await screen.findByText(
      "Field deleted. Existing item values were removed.",
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/api/projects/project-1/fields/field-status",
      expect.objectContaining({
        method: "DELETE",
        body: JSON.stringify({
          expectedUpdatedAt: "2026-05-03T00:00:00Z",
        }),
      }),
    );
  });

  it("adds, updates, reorders, and deletes single-select options", async () => {
    const addedSettings = settings({
      fields: settings().fields.map((field) =>
        field.id === "field-status"
          ? {
              ...field,
              options: [
                ...field.options,
                {
                  id: "option-3",
                  name: "Ready",
                  color: "green",
                  position: 3,
                  description: "Ready to ship",
                },
              ],
            }
          : field,
      ),
    });
    const updatedSettings = settings({
      fields: settings().fields.map((field) =>
        field.id === "field-status"
          ? {
              ...field,
              options: field.options.map((option) =>
                option.id === "option-1"
                  ? {
                      ...option,
                      name: "Queued",
                      color: "blue",
                      description: "Queued work",
                    }
                  : option,
              ),
            }
          : field,
      ),
    });
    const reorderedSettings = settings({
      fields: settings().fields.map((field) =>
        field.id === "field-status"
          ? {
              ...field,
              options: [
                { ...field.options[1], position: 1 },
                { ...field.options[0], position: 2 },
              ],
            }
          : field,
      ),
    });
    const deletedSettings = settings({
      fields: settings().fields.map((field) =>
        field.id === "field-status"
          ? { ...field, options: field.options.slice(1), usageCount: 4 }
          : field,
      ),
    });
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(JSON.stringify(addedSettings), {
          status: 201,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(updatedSettings), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(reorderedSettings), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(deletedSettings), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    render(
      <ProjectFieldSettingsPage
        owner="namuh"
        scope="organization"
        selectedFieldId="field-status"
        settings={settings()}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("Ready"), {
      target: { value: "Ready" },
    });
    fireEvent.change(screen.getAllByLabelText("Color")[0], {
      target: { value: "green" },
    });
    fireEvent.change(screen.getByPlaceholderText("Optional"), {
      target: { value: "Ready to ship" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add option" }));
    await screen.findByText("Option added.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/api/projects/project-1/fields/field-status/options",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          name: "Ready",
          color: "green",
          description: "Ready to ship",
        }),
      }),
    );

    fireEvent.change(screen.getByLabelText("Backlog option name"), {
      target: { value: "Queued" },
    });
    fireEvent.change(screen.getByLabelText("Backlog option color"), {
      target: { value: "blue" },
    });
    fireEvent.change(screen.getByLabelText("Backlog option description"), {
      target: { value: "Queued work" },
    });
    fireEvent.click(screen.getAllByRole("button", { name: "Save option" })[0]);
    await screen.findByText("Option saved.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/api/projects/project-1/fields/field-status/options/option-1",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          name: "Queued",
          color: "blue",
          description: "Queued work",
        }),
      }),
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Move Queued option down" }),
    );
    await screen.findByText("Options reordered.");
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      "/api/projects/project-1/fields/field-status/options/reorder",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({ optionIds: ["option-2", "option-1"] }),
      }),
    );

    fireEvent.click(
      screen.getAllByRole("button", { name: "Delete option" })[0],
    );
    await screen.findByText(
      "Option deleted. Matching item values were removed.",
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      4,
      "/api/projects/project-1/fields/field-status/options/option-2",
      expect.objectContaining({ method: "DELETE" }),
    );
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
