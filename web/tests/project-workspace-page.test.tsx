import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectWorkspacePage } from "@/components/ProjectWorkspacePage";
import type { ProjectItemDetail, ProjectWorkspace } from "@/lib/api";

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
        id: "field-start",
        name: "Start date",
        fieldType: "date",
        position: 2,
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
        id: "field-start",
        name: "Start date",
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
          id: "field-start",
          name: "Start date",
          fieldType: "date",
        },
        {
          id: "field-target",
          name: "Target date",
          fieldType: "date",
        },
      ],
      eligibleMarkerFields: [
        {
          id: "field-target",
          name: "Target date",
          fieldType: "date",
        },
      ],
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
            fieldId: "field-start",
            value: "2026-05-01",
            displayValue: "2026-05-01",
          },
          {
            fieldId: "field-target",
            value: "2026-05-20",
            displayValue: "2026-05-20",
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
            fieldId: "field-start",
            value: "2026-06-01",
            displayValue: "2026-06-01",
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

function itemDetail(
  overrides: Partial<ProjectItemDetail> = {},
): ProjectItemDetail {
  const baseWorkspace = workspace();
  const item = baseWorkspace.items[1];
  return {
    project: baseWorkspace.project,
    item,
    source: null,
    activity: [
      {
        id: "event-1",
        eventType: "draft_created",
        actor: { id: "user-1", login: "mona", avatarUrl: null },
        metadata: {},
        createdAt: "2026-05-04T00:00:00Z",
      },
    ],
    comments: [
      {
        id: "comment-1",
        author: { id: "user-1", login: "mona", avatarUrl: null },
        body: "Keep this scoped to the launch board.",
        isDeleted: false,
        createdAt: "2026-05-04T01:00:00Z",
        updatedAt: "2026-05-04T01:00:00Z",
      },
    ],
    archive: {
      archived: false,
      archivedAt: null,
      archivedBy: null,
      restoredAt: null,
      restoredBy: null,
    },
    draft: {
      editable: true,
      editVersion: "2026-05-04T00:00:00Z",
      repositoryNotificationsEnabled: false,
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "write",
      canEdit: true,
      canComment: true,
      canConvert: true,
      canArchive: true,
      canRestore: false,
      canRemove: true,
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
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/items/item-1?view=1&q=is%3Aopen&sort=manual&group=Status",
    );
    expect(
      screen.getByRole("link", { name: "Draft launch notes" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/items/item-2?view=1&q=is%3Aopen&sort=manual&group=Status",
    );
    expect(screen.getByText("frontend")).toHaveClass("chip", "soft");
    expect(screen.getByTitle("mona")).toHaveTextContent("M");
  });

  it("renders the route-backed item side panel with draft metadata and actions", async () => {
    const assign = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => workspace(),
    });
    vi.stubGlobal("location", { assign });
    vi.stubGlobal("fetch", fetchMock);
    render(
      <ProjectWorkspacePage
        initialItemDetail={itemDetail()}
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const panel = screen.getByRole("complementary", {
      name: "Project item detail",
    });
    expect(
      within(panel).getByRole("heading", { name: "Draft launch notes" }),
    ).toBeInTheDocument();
    expect(within(panel).getByText("Project-only draft")).toBeInTheDocument();
    expect(
      within(panel).getByDisplayValue("Write the rollout note."),
    ).toBeInTheDocument();
    expect(
      within(panel).getByText("Keep this scoped to the launch board."),
    ).toBeInTheDocument();
    expect(within(panel).getByText("draft created")).toBeInTheDocument();
    expect(within(panel).getByRole("link", { name: "Close" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
    expect(
      within(panel).getByRole("button", { name: "Convert to issue" }),
    ).toBeEnabled();
    expect(
      within(panel).getByRole("button", { name: "Archive" }),
    ).toBeDisabled();

    fireEvent.click(within(panel).getByRole("button", { name: "Remove" }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/items/item-2",
      { method: "DELETE" },
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?q=is%3Aopen&sort=manual&group=Status",
    );
  });

  it("converts a draft project item through real conversion target and mutation routes", async () => {
    const convertedDetail = itemDetail({
      item: {
        ...itemDetail().item,
        itemType: "issue",
        number: 44,
        href: "/namuh/opengithub/issues/44",
        repository: {
          id: "repo-1",
          owner: "namuh",
          name: "opengithub",
          fullName: "namuh/opengithub",
          href: "/namuh/opengithub",
        },
        title: "Draft launch notes",
        body: "Write the rollout note.",
      },
      source: {
        sourceType: "issue",
        id: "issue-44",
        number: 44,
        title: "Draft launch notes",
        state: "open",
        href: "/namuh/opengithub/issues/44",
        repository: {
          id: "repo-1",
          owner: "namuh",
          name: "opengithub",
          fullName: "namuh/opengithub",
          href: "/namuh/opengithub",
        },
        updatedAt: "2026-05-04T04:00:00Z",
        syncedAt: "2026-05-04T04:00:00Z",
        syncVersion: 1,
      },
      draft: null,
      viewerPermissions: {
        ...itemDetail().viewerPermissions,
        canConvert: false,
      },
    });
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          project: workspace().project,
          repositories: [
            {
              id: "repo-1",
              owner: "namuh",
              name: "opengithub",
              fullName: "namuh/opengithub",
              href: "/namuh/opengithub",
              labels: [{ id: "label-1", name: "frontend", color: "rust" }],
              assignees: [{ id: "user-1", login: "mona", avatarUrl: null }],
              milestones: [{ id: "mile-1", title: "M1", state: "open" }],
            },
          ],
          viewerPermissions: {
            authenticated: true,
            viewerRole: "write",
            canConvert: true,
          },
        }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => convertedDetail });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <ProjectWorkspacePage
        initialItemDetail={itemDetail()}
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const panel = screen.getByRole("complementary", {
      name: "Project item detail",
    });
    fireEvent.click(
      within(panel).getByRole("button", { name: "Convert to issue" }),
    );
    await screen.findByRole("form", { name: "Convert draft to issue" });
    fireEvent.click(within(panel).getByLabelText("frontend"));
    fireEvent.click(within(panel).getByLabelText("mona"));
    fireEvent.change(within(panel).getByLabelText("Milestone"), {
      target: { value: "mile-1" },
    });
    fireEvent.submit(
      within(panel).getByRole("form", { name: "Convert draft to issue" }),
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/projects/project-1/items/item-2/convert-to-issue",
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            repositoryId: "repo-1",
            labelIds: ["label-1"],
            assigneeUserIds: ["user-1"],
            milestoneId: "mile-1",
            expectedUpdatedAt: "2026-05-04T00:00:00Z",
          }),
        },
      ),
    );
    expect(
      within(panel).getByText("Draft converted to issue"),
    ).toBeInTheDocument();
    expect(
      within(panel).getByRole("link", { name: "namuh/opengithub #44" }),
    ).toHaveAttribute("href", "/namuh/opengithub/issues/44");
    expect(
      within(panel).getByRole("button", { name: "Convert to issue" }),
    ).toBeDisabled();
  });

  it("edits draft body and project-only comments through real item routes", async () => {
    const updatedDraft = itemDetail({
      item: {
        ...itemDetail().item,
        title: "Edited draft title",
        body: "Edited draft body",
        updatedAt: "2026-05-04T02:00:00Z",
      },
      activity: [
        ...itemDetail().activity,
        {
          id: "event-2",
          eventType: "project.draft.update",
          actor: { id: "user-1", login: "mona", avatarUrl: null },
          metadata: {},
          createdAt: "2026-05-04T02:00:00Z",
        },
      ],
    });
    const addedComment = itemDetail({
      ...updatedDraft,
      comments: [
        ...updatedDraft.comments,
        {
          id: "comment-2",
          author: { id: "user-1", login: "mona", avatarUrl: null },
          body: "Fresh project-only comment",
          isDeleted: false,
          createdAt: "2026-05-04T03:00:00Z",
          updatedAt: "2026-05-04T03:00:00Z",
        },
      ],
    });
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({ ok: true, json: async () => updatedDraft })
      .mockResolvedValueOnce({ ok: true, json: async () => addedComment })
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          itemDetail({
            ...addedComment,
            comments: addedComment.comments.map((comment) =>
              comment.id === "comment-2"
                ? { ...comment, body: "Edited project-only comment" }
                : comment,
            ),
          }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () =>
          itemDetail({
            ...addedComment,
            comments: addedComment.comments.map((comment) =>
              comment.id === "comment-2"
                ? { ...comment, isDeleted: true, body: "[deleted]" }
                : comment,
            ),
          }),
      });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <ProjectWorkspacePage
        initialItemDetail={itemDetail()}
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const panel = screen.getByRole("complementary", {
      name: "Project item detail",
    });
    fireEvent.change(within(panel).getByLabelText("Title"), {
      target: { value: "Edited draft title" },
    });
    fireEvent.change(within(panel).getByLabelText("Body"), {
      target: { value: "Edited draft body" },
    });
    fireEvent.submit(
      within(panel).getByRole("form", { name: "Edit draft project item" }),
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/projects/project-1/items/item-2/draft",
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            title: "Edited draft title",
            body: "Edited draft body",
            expectedUpdatedAt: "2026-05-04T00:00:00Z",
          }),
        },
      ),
    );
    expect(within(panel).getByText("Draft saved")).toBeInTheDocument();

    fireEvent.change(
      within(panel).getByPlaceholderText("Add a project-only comment"),
      { target: { value: "Fresh project-only comment" } },
    );
    fireEvent.submit(
      within(panel).getByRole("form", { name: "Add project item comment" }),
    );
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/projects/project-1/items/item-2/comments",
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            body: "Fresh project-only comment",
            expectedUpdatedAt: "2026-05-04T02:00:00Z",
          }),
        },
      ),
    );
    expect(
      await within(panel).findByText("Fresh project-only comment"),
    ).toBeInTheDocument();

    const editButtons = within(panel).getAllByRole("button", { name: "Edit" });
    const lastEditButton = editButtons[editButtons.length - 1];
    expect(lastEditButton).toBeDefined();
    fireEvent.click(lastEditButton);
    fireEvent.change(within(panel).getByLabelText("Edit comment by mona"), {
      target: { value: "Edited project-only comment" },
    });
    fireEvent.click(within(panel).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/projects/project-1/items/item-2/comments/comment-2",
        {
          method: "PATCH",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            body: "Edited project-only comment",
            expectedUpdatedAt: "2026-05-04T02:00:00Z",
          }),
        },
      ),
    );

    const deleteButtons = within(panel).getAllByRole("button", {
      name: "Delete",
    });
    const lastDeleteButton = deleteButtons[deleteButtons.length - 1];
    expect(lastDeleteButton).toBeDefined();
    fireEvent.click(lastDeleteButton);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/projects/project-1/items/item-2/comments/comment-2",
        { method: "DELETE" },
      ),
    );
    expect(within(panel).getAllByText("Comment deleted")).toHaveLength(2);
  });

  it("shows a side panel error when direct item detail loading fails", () => {
    render(
      <ProjectWorkspacePage
        initialItemError="Private linked item is hidden."
        owner="namuh"
        scope="organization"
        viewNumber={1}
        workspace={workspace()}
      />,
    );

    const panel = screen.getByRole("complementary", {
      name: "Project item detail",
    });
    expect(within(panel).getByText("This item cannot be opened.")).toHaveClass(
      "t-h2",
    );
    expect(
      within(panel).getByText("Private linked item is hidden."),
    ).toHaveClass("t-sm");
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

  it("renders roadmap rows and persists date, marker, and zoom controls", async () => {
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
            layout: "roadmap",
            name: "Roadmap",
          },
          roadmapConfig: {
            ...(workspace().roadmapConfig ?? {
              startDateField: null,
              targetDateField: null,
              markerFields: [],
              eligibleDateFields: [],
              eligibleMarkerFields: [],
              zoom: "month",
              zoomOptions: ["month", "quarter", "year"],
              unavailableReason: null,
            }),
            zoom: "month",
          },
          filters: {
            ...workspace().filters,
            query: null,
            tokens: [],
          },
        })}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Roadmap" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    expect(screen.getByText("May 1 - May 20")).toBeInTheDocument();
    expect(screen.getAllByText("Missing dates")[0]).toHaveClass("chip", "warn");
    expect(screen.getByText("Jan")).toHaveClass("t-label");

    fireEvent.change(screen.getByLabelText("Roadmap target date field"), {
      target: { value: "field-start" },
    });
    fireEvent.click(screen.getByRole("button", { name: "quarter" }));
    fireEvent.click(screen.getAllByLabelText("Target date")[1]);
    fireEvent.submit(screen.getByRole("form", { name: "Roadmap settings" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/projects/project-1/views/view-1/roadmap-settings",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          startFieldId: "field-start",
          targetFieldId: "field-start",
          markerFieldIds: ["field-target"],
          zoom: "quarter",
          expectedUpdatedAt: "2026-05-05T00:00:00Z",
        }),
      }),
    );
    expect(assign).toHaveBeenCalledWith(
      "/orgs/namuh/projects/12/views/1?sort=manual&group=Status",
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
