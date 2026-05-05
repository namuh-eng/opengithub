import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProjectsListPage } from "@/components/ProjectsListPage";
import type { ProjectList, ProjectRow, ProjectTemplateRow } from "@/lib/api";

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
  it("renders dense Editorial project rows with concrete workspace links", () => {
    render(
      <ProjectsListPage list={projectList()} scopeLabel="namuh projects" />,
    );

    expect(
      screen.getByRole("heading", { name: "namuh projects" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Search all projects")).toHaveValue("");
    expect(screen.getByText("On track")).toBeInTheDocument();
    expect(screen.getByText(/#12/)).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /Roadmap planning/ }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/12/views/1");
    expect(screen.getByRole("link", { name: "Open" })).toHaveAttribute(
      "href",
      "/orgs/namuh/projects/12/views/1",
    );
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
    expect(
      screen.getByRole("link", { name: /Team planning template/ }),
    ).toHaveAttribute("href", "/orgs/namuh/projects/4/views/1");
    expect(screen.getByRole("button", { name: "Copy" })).toBeDisabled();
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
