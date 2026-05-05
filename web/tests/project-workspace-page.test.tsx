import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectWorkspacePage } from "@/components/ProjectWorkspacePage";
import type { ProjectWorkspace } from "@/lib/api";

function workspace(
  overrides: Partial<ProjectWorkspace> = {},
): ProjectWorkspace {
  return {
    project: {
      id: "project-1",
      number: 12,
      title: "Editorial table workspace",
      description: "Tracks repository work across the next milestone.",
      state: "open",
      visibility: "private",
      owner: "namuh",
      href: "/orgs/namuh/projects/12",
      workspaceHref: "/orgs/namuh/projects/12/views/1",
      viewerRole: "write",
    },
    selectedView: {
      id: "view-1",
      number: 1,
      name: "Table",
      layout: "table",
      href: "/orgs/namuh/projects/12/views/1",
      configuration: {},
      updatedAt: "2026-05-05T00:00:00Z",
    },
    views: [
      {
        id: "view-1",
        number: 1,
        name: "Table",
        layout: "table",
        href: "/orgs/namuh/projects/12/views/1",
        configuration: {},
        updatedAt: "2026-05-05T00:00:00Z",
      },
      {
        id: "view-2",
        number: 2,
        name: "Bugs",
        layout: "table",
        href: "/orgs/namuh/projects/12/views/2",
        configuration: {},
        updatedAt: "2026-05-05T00:00:00Z",
      },
    ],
    fields: [
      {
        id: "field-status",
        name: "Status",
        fieldType: "single_select",
        position: 1,
        settings: {},
        hidden: false,
        editable: true,
      },
      {
        id: "field-target",
        name: "Target date",
        fieldType: "date",
        position: 3,
        settings: {},
        hidden: false,
        editable: true,
      },
      {
        id: "field-priority",
        name: "Priority",
        fieldType: "single_select",
        position: 4,
        settings: {},
        hidden: false,
        editable: true,
      },
      {
        id: "field-hidden",
        name: "Secret",
        fieldType: "text",
        position: 5,
        settings: {},
        hidden: true,
        editable: false,
      },
    ],
    layoutChoices: [
      {
        layout: "table",
        label: "Table",
        keyboardHint: "t",
        active: true,
        enabled: true,
        unavailableReason: null,
      },
      {
        layout: "board",
        label: "Board",
        keyboardHint: "b",
        active: false,
        enabled: true,
        unavailableReason: null,
      },
      {
        layout: "roadmap",
        label: "Roadmap",
        keyboardHint: "r",
        active: false,
        enabled: true,
        unavailableReason: null,
      },
    ],
    boardConfig: {
      columnField: {
        id: "field-status",
        name: "Status",
        fieldType: "single_select",
      },
      swimlaneField: null,
      eligibleColumnFields: [
        {
          id: "field-status",
          name: "Status",
          fieldType: "single_select",
        },
      ],
      eligibleSwimlaneFields: [],
      columns: [],
      emptyColumnsVisible: true,
      unavailableReason: null,
    },
    roadmapConfig: {
      startDateField: {
        id: "field-target",
        name: "Target date",
        fieldType: "date",
      },
      targetDateField: {
        id: "field-target",
        name: "Target date",
        fieldType: "date",
      },
      markerFields: [],
      eligibleDateFields: [
        {
          id: "field-target",
          name: "Target date",
          fieldType: "date",
        },
      ],
      eligibleMarkerFields: [],
      zoom: "month",
      zoomOptions: ["month", "quarter", "year"],
      unavailableReason: null,
    },
    items: [
      {
        id: "item-1",
        itemType: "issue",
        position: "1",
        title: "Wire the table shell",
        body: null,
        state: "open",
        number: 42,
        href: "/namuh/opengithub/issues/42",
        repository: {
          id: "repo-1",
          owner: "namuh",
          name: "opengithub",
          fullName: "namuh/opengithub",
          href: "/namuh/opengithub",
        },
        fieldValues: [
          {
            fieldId: "field-status",
            value: "In progress",
            displayValue: "In progress",
          },
          {
            fieldId: "field-priority",
            value: "P1",
            displayValue: "P1",
          },
        ],
        labels: [{ id: "label-1", name: "frontend", color: "rust" }],
        assignees: [{ id: "user-1", login: "mona", avatarUrl: null }],
        updatedAt: "2026-05-06T00:00:00Z",
      },
      {
        id: "item-2",
        itemType: "draft_issue",
        position: "2",
        title: "Draft launch notes",
        body: "Write the rollout note.",
        state: null,
        number: null,
        href: null,
        repository: null,
        fieldValues: [
          {
            fieldId: "field-status",
            value: "Backlog",
            displayValue: "Backlog",
          },
          {
            fieldId: "field-priority",
            value: "P2",
            displayValue: "P2",
          },
        ],
        labels: [],
        assignees: [],
        updatedAt: "2026-05-04T00:00:00Z",
      },
    ],
    total: 2,
    page: 1,
    pageSize: 30,
    groups: [
      { key: "In progress", label: "In progress", count: 1 },
      { key: "Backlog", label: "Backlog", count: 1 },
    ],
    slices: [{ key: "In progress", label: "In progress", count: 1 }],
    filters: {
      query: "is:open",
      sort: "manual",
      group: "Status",
      slice: null,
      tokens: ["is:open"],
      page: 1,
      pageSize: 30,
    },
    unsavedView: {
      active: true,
      reasons: ["query"],
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "write",
      canEdit: true,
      canManageViews: true,
      canChangeLayout: true,
      canAddItems: true,
    },
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectWorkspacePage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the Editorial workspace shell with saved views and field columns", () => {
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Editorial table workspace" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Table" })).toHaveClass("active");
    expect(screen.getByRole("link", { name: "Bugs" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/2?q=is%3Aopen&sort=manual&group=Status",
    );
    expect(
      screen.getByRole("columnheader", { name: "Status" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: "Secret" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Unsaved view")).toHaveClass("chip", "warn");
  });

  it("renders issue and draft rows with concrete item links and grouped headers", () => {
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    expect(screen.getAllByText("In progress").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Backlog").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: "Wire the table shell" }),
    ).toHaveAttribute("href", "/namuh/opengithub/issues/42");
    expect(
      screen.getByRole("link", { name: "Draft launch notes" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
    expect(screen.getByText("frontend")).toHaveClass("chip", "soft");
    expect(screen.getByTitle("mona")).toHaveTextContent("M");
  });

  it("keeps filters, slices, and field value chips URL-backed", () => {
    const assign = vi.fn();
    vi.stubGlobal("location", { assign });
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    expect(screen.getByRole("link", { name: /All items 2/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
    expect(screen.getByRole("link", { name: /In progress 1/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status&slice=In+progress",
    );
    expect(screen.getByRole("link", { name: "is:open x" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1?sort=manual&group=Status",
    );

    fireEvent.change(screen.getByLabelText("Filter project items"), {
      target: { value: "label:frontend" },
    });
    fireEvent.submit(screen.getByLabelText("Filter project items"));
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=label%3Afrontend&sort=manual&group=Status",
    );
  });

  it("shows honest disabled states for controls implemented in later phases", () => {
    render(
      <ProjectWorkspacePage
        owner="mona"
        scope="user"
        viewNumber={1}
        workspace={workspace({
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canEdit: false,
            canManageViews: false,
            canChangeLayout: false,
            canAddItems: false,
          },
        })}
      />,
    );

    expect(screen.getByRole("button", { name: "Insights" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Settings" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "+ View" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "View menu" }));
    expect(screen.getByRole("button", { name: /Table/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Board/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Roadmap/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Add item" })).toBeDisabled();
  });

  it("saves view-state changes through the same-origin project view route", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "View menu" }));
    fireEvent.change(screen.getByLabelText("Filter query"), {
      target: { value: "is:open label:frontend" },
    });
    fireEvent.change(screen.getByLabelText("Sort"), {
      target: { value: "updated_desc" },
    });
    fireEvent.change(screen.getByLabelText("Group by"), {
      target: { value: "Status" },
    });
    fireEvent.click(screen.getByLabelText("Secret"));
    fireEvent.submit(screen.getByRole("form", { name: "View configuration" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/views/view-1/state",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          query: "is:open label:frontend",
          sort: "updated_desc",
          group: "Status",
          slice: null,
          hiddenFieldIds: [],
          expectedUpdatedAt: "2026-05-05T00:00:00Z",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith("/orgs/namuh/projects/12/views/1");
  });

  it("persists layout choices through the same-origin project layout route", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "View menu" }));
    expect(screen.getByRole("button", { name: /Table/ })).toHaveTextContent(
      "t",
    );
    expect(screen.getByRole("button", { name: /Board/ })).toHaveTextContent(
      "b",
    );
    expect(screen.getByRole("button", { name: /Roadmap/ })).toHaveTextContent(
      "r",
    );
    expect(screen.getByText("Column by")).toHaveClass("t-label");
    expect(screen.getByText("Swimlanes")).toHaveClass("t-label");
    expect(screen.getByText("Sort by")).toHaveClass("t-label");
    expect(screen.getByText("Field sum")).toHaveClass("t-label");
    expect(screen.getAllByText("Slice by")[0]).toHaveClass("t-label");

    fireEvent.click(screen.getByRole("button", { name: /Board/ }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/views/view-1/layout",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          layout: "board",
          columnFieldId: "field-status",
          swimlaneFieldId: null,
          startFieldId: null,
          targetFieldId: null,
          expectedUpdatedAt: "2026-05-05T00:00:00Z",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
  });

  it("shows server validation errors and can revert URL-backed view state", async () => {
    const assign = vi.fn();
    vi.stubGlobal("location", { assign });
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        json: async () => ({
          error: {
            code: "validation_failed",
            message: "sort must be supported",
          },
          status: 422,
        }),
      }),
    );
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "View menu" }));
    fireEvent.submit(screen.getByRole("form", { name: "View configuration" }));
    expect(await screen.findByText("sort must be supported")).toHaveClass(
      "chip",
      "err",
    );
    fireEvent.click(screen.getByRole("button", { name: "Revert" }));
    expect(assign).toHaveBeenCalledWith("/orgs/namuh/projects/12/views/1");
  });

  it("edits a project field through the same-origin item field route", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const firstRow = screen
      .getByRole("link", {
        name: "Wire the table shell",
      })
      .closest("tr");
    expect(firstRow).not.toBeNull();
    fireEvent.click(
      within(firstRow as HTMLTableRowElement).getByTitle("Edit Status"),
    );
    fireEvent.change(screen.getByLabelText("Status value"), {
      target: { value: "Done" },
    });
    fireEvent.submit(
      screen.getByRole("form", {
        name: "Edit Status for Wire the table shell",
      }),
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/items/item-1/fields/field-status",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          value: "Done",
          expectedUpdatedAt: "2026-05-06T00:00:00Z",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
  });

  it("shows inline field validation errors without local-only edits", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        json: async () => ({
          error: {
            code: "validation_failed",
            message: "Status must be open or closed",
          },
          status: 422,
        }),
      }),
    );
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const firstRow = screen
      .getByRole("link", {
        name: "Wire the table shell",
      })
      .closest("tr");
    expect(firstRow).not.toBeNull();
    fireEvent.click(
      within(firstRow as HTMLTableRowElement).getByTitle("Edit Status"),
    );
    fireEvent.change(screen.getByLabelText("Status value"), {
      target: { value: "Blocked" },
    });
    fireEvent.submit(
      screen.getByRole("form", {
        name: "Edit Status for Wire the table shell",
      }),
    );

    expect(
      await screen.findByText("Status must be open or closed"),
    ).toHaveClass("chip", "err");
    expect(screen.getByDisplayValue("Blocked")).toBeInTheDocument();
  });

  it("adds linked items and draft issues through real project item routes", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add item" }));
    fireEvent.change(screen.getByLabelText("Issue or pull request URL"), {
      target: { value: "/namuh/opengithub/pull/44" },
    });
    fireEvent.submit(
      screen.getByRole("form", { name: "Add linked issue or pull request" }),
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/items",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          itemType: "pull_request",
          url: "/namuh/opengithub/pull/44",
          positionAfterItemId: "item-2",
        }),
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Draft issue" }));
    fireEvent.change(screen.getByLabelText("Draft title"), {
      target: { value: "Triage copied roadmap notes" },
    });
    fireEvent.change(screen.getByLabelText("Draft body"), {
      target: { value: "Keep this as a project-only draft." },
    });
    fireEvent.submit(
      screen.getByRole("form", { name: "Create draft project item" }),
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/items",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          itemType: "draft_issue",
          title: "Triage copied roadmap notes",
          body: "Keep this as a project-only draft.",
          positionAfterItemId: "item-2",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
  });

  it("bulk adds project URLs and persists row reorder and removal", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add item" }));
    fireEvent.click(screen.getByRole("button", { name: "Bulk add" }));
    fireEvent.change(
      screen.getByLabelText("Bulk issue and pull request URLs"),
      {
        target: {
          value: "/namuh/opengithub/issues/45\n/namuh/opengithub/pull/46",
        },
      },
    );
    fireEvent.submit(
      screen.getByRole("form", { name: "Bulk add project items" }),
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/items/bulk",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          items: [
            { itemType: "issue", url: "/namuh/opengithub/issues/45" },
            { itemType: "pull_request", url: "/namuh/opengithub/pull/46" },
          ],
        }),
      }),
    );

    const secondRow = screen
      .getByRole("link", { name: "Draft launch notes" })
      .closest("tr");
    expect(secondRow).not.toBeNull();
    fireEvent.click(
      within(secondRow as HTMLTableRowElement).getByRole("button", {
        name: "Up",
      }),
    );
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/items/item-2/position",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          beforeItemId: "item-1",
          afterItemId: null,
          expectedUpdatedAt: "2026-05-04T00:00:00Z",
        }),
      }),
    );

    fireEvent.click(
      within(secondRow as HTMLTableRowElement).getByRole("button", {
        name: "Remove",
      }),
    );
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(3));
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/projects/project-1/items/item-2",
      { method: "DELETE" },
    );
  });

  it("renders board columns, swimlanes, card metadata, and empty-column toggles", () => {
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace({
          selectedView: {
            ...workspace().selectedView,
            layout: "board",
            name: "Board",
          },
          boardConfig: {
            columnField: {
              id: "field-status",
              name: "Status",
              fieldType: "single_select",
            },
            swimlaneField: {
              id: "field-priority",
              name: "Priority",
              fieldType: "single_select",
            },
            eligibleColumnFields: [
              {
                id: "field-status",
                name: "Status",
                fieldType: "single_select",
              },
            ],
            eligibleSwimlaneFields: [
              {
                id: "field-priority",
                name: "Priority",
                fieldType: "single_select",
              },
            ],
            columns: [
              {
                key: "In progress",
                label: "In progress",
                fieldId: "field-status",
                count: 2,
                itemLimit: 1,
                overLimit: true,
                visible: true,
              },
              {
                key: "Backlog",
                label: "Backlog",
                fieldId: "field-status",
                count: 1,
                itemLimit: null,
                overLimit: false,
                visible: true,
              },
              {
                key: "Done",
                label: "Done",
                fieldId: "field-status",
                count: 0,
                itemLimit: null,
                overLimit: false,
                visible: true,
              },
            ],
            emptyColumnsVisible: true,
            unavailableReason: null,
          },
        })}
      />,
    );

    expect(screen.getByRole("heading", { name: "Board" })).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    expect(
      screen.getAllByRole("region", { name: "In progress board column" })[0],
    ).toBeInTheDocument();
    expect(screen.getAllByText("Over limit")[0]).toHaveClass("chip", "warn");
    expect(screen.getAllByText("P1")[0]).toHaveClass("t-label");
    expect(screen.getAllByText("P2")[0]).toHaveClass("t-label");
    expect(screen.getByText("frontend")).toHaveClass("chip", "soft");
    expect(screen.getByRole("button", { name: "Hide empty columns" }));

    fireEvent.click(screen.getByRole("button", { name: "Hide empty columns" }));
    expect(
      screen.queryByRole("region", { name: "Done board column" }),
    ).not.toBeInTheDocument();
  });

  it("moves board cards by updating the backing column field value", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace({
          selectedView: {
            ...workspace().selectedView,
            layout: "board",
            name: "Board",
          },
          boardConfig: {
            columnField: {
              id: "field-status",
              name: "Status",
              fieldType: "single_select",
            },
            swimlaneField: null,
            eligibleColumnFields: [
              {
                id: "field-status",
                name: "Status",
                fieldType: "single_select",
              },
            ],
            eligibleSwimlaneFields: [],
            columns: [
              {
                key: "In progress",
                label: "In progress",
                fieldId: "field-status",
                count: 1,
                itemLimit: null,
                overLimit: false,
                visible: true,
              },
              {
                key: "Done",
                label: "Done",
                fieldId: "field-status",
                count: 0,
                itemLimit: null,
                overLimit: false,
                visible: true,
              },
            ],
            emptyColumnsVisible: true,
            unavailableReason: null,
          },
        })}
      />,
    );

    fireEvent.change(
      screen.getByLabelText("Move Wire the table shell to column"),
      { target: { value: "Done" } },
    );

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/items/item-1/position",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          beforeItemId: null,
          afterItemId: null,
          groupFieldId: "field-status",
          groupValue: "Done",
          expectedUpdatedAt: "2026-05-06T00:00:00Z",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
  });

  it("uses Editorial primitives instead of banned GitHub visual values", () => {
    const { container } = render(
      <ProjectWorkspacePage
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );
    const html = container.innerHTML;
    for (const banned of [
      "#0969da",
      "#1f883d",
      "#cf222e",
      "@primer/",
      "Octicon",
    ]) {
      expect(html).not.toContain(banned);
    }
    expect(within(container).getByText("Slices")).toHaveClass("t-label");
  });
});
