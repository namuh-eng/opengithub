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
        id: "field-hidden",
        name: "Secret",
        fieldType: "text",
        position: 2,
        settings: {},
        hidden: true,
        editable: false,
      },
    ],
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
            canAddItems: false,
          },
        })}
      />,
    );

    expect(screen.getByRole("button", { name: "Insights" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Settings" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "+ View" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "View configuration" }),
    ).toBeDisabled();
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

    fireEvent.click(screen.getByRole("button", { name: "View configuration" }));
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

    fireEvent.click(screen.getByRole("button", { name: "View configuration" }));
    fireEvent.submit(screen.getByRole("form", { name: "View configuration" }));
    expect(await screen.findByText("sort must be supported")).toHaveClass(
      "chip",
      "err",
    );
    fireEvent.click(screen.getByRole("button", { name: "Revert" }));
    expect(assign).toHaveBeenCalledWith("/orgs/namuh/projects/12/views/1");
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
