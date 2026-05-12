import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectsListPage } from "@/components/ProjectsListPage";
import type { ProjectList, ProjectRow, ProjectTemplateRow } from "@/lib/api";

const push = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push }),
}));

function project(overrides: Partial<ProjectRow> = {}): ProjectRow {
  return {
    id: "project-1",
    number: 12,
    title: "Roadmap planning",
    description: "Tracks repository work across the next milestone.",
    state: "open",
    visibility: "public",
    href: "/orgs/namuh/projects/12",
    workspaceHref: "/orgs/namuh/projects/12/views/1",
    owner: "namuh",
    isTemplate: false,
    defaultRepository: {
      id: "repo-1",
      owner: "namuh",
      name: "opengithub",
      fullName: "namuh/opengithub",
      href: "/namuh/opengithub",
    },
    linkedRepositoriesCount: 2,
    status: {
      status: "on_track",
      label: "On track",
      body: "Shipping steadily.",
      createdAt: "2026-05-01T00:00:00Z",
    },
    counts: {
      total: 8,
      open: 6,
      closed: 1,
      draft: 1,
    },
    viewerRole: "write",
    viewerCanCopy: true,
    createdAt: "2026-04-20T00:00:00Z",
    updatedAt: "2026-05-03T00:00:00Z",
    closedAt: null,
    ...overrides,
  };
}

function template(
  overrides: Partial<ProjectTemplateRow> = {},
): ProjectTemplateRow {
  return {
    id: "template-1",
    projectId: "project-template-source",
    title: "Team planning template",
    description: "Reusable planning setup.",
    projectTitle: "Platform template",
    projectHref: "/orgs/namuh/projects/4/views/1",
    isPublic: true,
    viewerCanCopy: false,
    createdAt: "2026-05-01T00:00:00Z",
    ...overrides,
  };
}

function projectList(overrides: Partial<ProjectList> = {}): ProjectList {
  return {
    items: [project()],
    total: 1,
    page: 1,
    pageSize: 30,
    scope: {
      kind: "organization",
      login: "namuh",
      repository: null,
      href: "/orgs/namuh/projects",
    },
    filters: {
      query: null,
      state: "open",
      tab: "projects",
      sort: "recently_updated",
      page: 1,
      pageSize: 30,
    },
    counts: {
      open: 1,
      closed: 1,
      templates: 1,
      total: 2,
    },
    templates: {
      items: [template()],
      total: 1,
      page: 1,
      pageSize: 30,
    },
    viewerPermissions: {
      authenticated: true,
      viewerRole: "write",
      canCreate: true,
      canCopy: true,
    },
    unavailableReason: null,
    ...overrides,
  };
}

describe("ProjectsListPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    push.mockReset();
  });

  it("renders dense Editorial project rows with concrete workspace links", () => {
    render(
      <ProjectsListPage list={projectList()} scopeLabel="namuh projects" />,
    );

    expect(
      screen.getByRole("heading", { name: "namuh projects" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Search all projects")).toHaveValue("");
    expect(screen.getByLabelText("Sort projects")).toHaveValue(
      "recently_updated",
    );
    expect(screen.getByText("On track")).toBeInTheDocument();
    expect(screen.getByText(/#12/)).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /Roadmap planning/ }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "Open" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1",
    );
    expect(screen.getByRole("link", { name: "Insights" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/insights",
    );
    expect(
      screen.getByRole("button", { name: "More project options" }),
    ).toBeInTheDocument();
  });

  it("renders the templates tab and disables unavailable copy actions", () => {
    render(
      <ProjectsListPage
        list={projectList({
          filters: {
            query: "is:open",
            state: "open",
            tab: "templates",
            sort: "recently_updated",
            page: 1,
            pageSize: 30,
          },
        })}
      />,
    );

    expect(screen.getByLabelText("Search all projects")).toHaveValue("is:open");
    expect(screen.getByRole("link", { name: /Projects 2/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=is%3Aopen",
    );
    expect(screen.getByRole("link", { name: /Templates 1/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=is%3Aopen&tab=templates",
    );
    expect(
      screen.getByRole("link", { name: /Team planning template/ }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/4/views/1");
    fireEvent.click(
      screen.getByRole("button", { name: "More project options" }),
    );
    expect(screen.getByRole("button", { name: "Copy" })).toBeDisabled();
  });

  it("submits the copy dialog through the same-origin proxy and redirects", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          id: "project-copy",
          number: 13,
          title: "[COPY] Roadmap planning",
          href: "/namuh/projects/13",
          workspaceHref: "/namuh/projects/13/views/1",
          owner: "namuh",
          copiedViews: 1,
          copiedFields: 2,
          copiedWorkflows: 1,
          copiedDraftItems: 1,
        }),
        { status: 201, headers: { "content-type": "application/json" } },
      ),
    );
    render(<ProjectsListPage list={projectList()} />);

    fireEvent.click(
      screen.getByRole("button", { name: "More project options" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(
      screen.getByRole("dialog", { name: "Roadmap planning" }),
    ).toBeInTheDocument();
    expect(
      screen.getByDisplayValue("[COPY] Roadmap planning"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Copy project" }));

    await waitFor(() =>
      expect(globalThis.fetch).toHaveBeenCalledWith(
        "/api/projects/project-1/copies",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            title: "[COPY] Roadmap planning",
            includeDraftIssues: true,
          }),
        }),
      ),
    );
    expect(push).toHaveBeenCalledWith("/namuh/projects/13/views/1");
  });

  it("keeps server permission errors inside the copy dialog", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          error: {
            code: "forbidden",
            message: "You do not have access to this project list",
          },
          status: 403,
        }),
        { status: 403, headers: { "content-type": "application/json" } },
      ),
    );
    render(<ProjectsListPage list={projectList()} />);

    fireEvent.click(
      screen.getByRole("button", { name: "More project options" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    fireEvent.click(
      screen.getByRole("checkbox", { name: /Include draft issues/ }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Copy project" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have access to this project list",
    );
    expect(push).not.toHaveBeenCalled();
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/projects/project-1/copies",
      expect.objectContaining({
        body: JSON.stringify({
          title: "[COPY] Roadmap planning",
          includeDraftIssues: false,
        }),
      }),
    );
  });

  it("dismisses the Welcome to Projects banner without leaving the page", () => {
    render(<ProjectsListPage list={projectList()} />);

    expect(screen.getByText("Welcome to Projects")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Dismiss Welcome to Projects" }),
    );

    expect(screen.queryByText("Welcome to Projects")).not.toBeInTheDocument();
  });

  it("builds URL-backed search, state, sort, and pagination controls", () => {
    render(
      <ProjectsListPage
        list={projectList({
          items: [project({ id: "project-2", title: "Design systems" })],
          total: 90,
          page: 2,
          filters: {
            query: "roadmap",
            state: "closed",
            tab: "projects",
            sort: "name_asc",
            page: 2,
            pageSize: 30,
          },
        })}
      />,
    );

    expect(screen.getByLabelText("Search all projects")).toHaveValue("roadmap");
    expect(screen.getByLabelText("Sort projects")).toHaveValue("name_asc");
    expect(screen.getByRole("link", { name: /Open 1/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=roadmap&sort=name_asc",
    );
    expect(screen.getByRole("link", { name: /Closed 1/ })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=roadmap&state=closed&sort=name_asc",
    );
    expect(
      screen.getByRole("link", { name: "Search: roadmap x" }),
    ).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?state=closed&sort=name_asc",
    );
    expect(screen.getByRole("link", { name: "Clear filters" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects",
    );
    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=roadmap&state=closed&sort=name_asc",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects?q=roadmap&state=closed&sort=name_asc&page=3",
    );
  });

  it("preserves the profile projects tab while switching user project templates", () => {
    render(
      <ProjectsListPage
        list={projectList({
          scope: {
            kind: "user",
            login: "mona",
            repository: null,
            href: "/mona?tab=projects",
          },
          filters: {
            query: "planning",
            state: "open",
            tab: "projects",
            sort: "recently_updated",
            page: 1,
            pageSize: 30,
          },
        })}
      />,
    );

    expect(screen.getByRole("link", { name: /Templates 1/ })).toHaveAttribute(
      "href",
      "/mona?tab=projects&q=planning&projectTab=templates",
    );
    expect(
      screen.getByRole("textbox", { name: "Search all projects" }),
    ).toHaveAttribute("name", "q");
  });

  it("shows unavailable and empty states without placeholder links", () => {
    render(
      <ProjectsListPage
        list={projectList({
          items: [],
          total: 0,
          unavailableReason: "Organization policy has disabled Projects.",
          viewerPermissions: {
            authenticated: true,
            viewerRole: "read",
            canCreate: false,
            canCopy: false,
          },
        })}
      />,
    );

    expect(
      screen.getByText("Organization policy has disabled Projects."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "New project" })).toBeDisabled();
    expect(screen.queryByRole("link", { name: "#" })).not.toBeInTheDocument();
  });

  it("uses Editorial primitives instead of banned GitHub visual values", () => {
    const { container } = render(<ProjectsListPage list={projectList()} />);

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
    expect(within(container).getByText("Welcome to Projects")).toHaveClass(
      "t-label",
    );
  });
});
