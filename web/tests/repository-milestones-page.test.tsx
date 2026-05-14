import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositoryMilestonesPage } from "@/components/RepositoryMilestonesPage";
import type {
  RepositoryMilestoneSummary,
  RepositoryMilestonesView,
  RepositoryOverview,
} from "@/lib/api";
import { repositoryMilestonesHref } from "@/lib/navigation";

const refreshMock = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: refreshMock }),
}));

beforeEach(() => {
  refreshMock.mockReset();
  vi.stubGlobal(
    "confirm",
    vi.fn(() => true),
  );
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => ({
      ok: true,
      json: async () => milestoneSummary({ id: "milestone-new" }),
    })),
  );
});

function repositoryOverview(
  overrides: Partial<RepositoryOverview> = {},
): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "mona",
    name: "octo-app",
    description: "Milestone test repository",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "write",
    branchCount: 2,
    tagCount: 1,
    defaultBranchRef: null,
    latestCommit: null,
    rootEntries: [],
    files: [],
    readme: null,
    sidebar: {
      about: null,
      websiteUrl: null,
      topics: [],
      starsCount: 0,
      watchersCount: 0,
      forksCount: 0,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 0,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:mona/octo-app.git",
      https: "https://opengithub.namuh.co/mona/octo-app.git",
      zip: "/mona/octo-app/archive/refs/heads/main.zip",
    },
    ...overrides,
  };
}

function milestoneSummary(
  overrides: Partial<RepositoryMilestoneSummary> = {},
): RepositoryMilestoneSummary {
  return {
    id: "milestone-1",
    title: "Launch readiness",
    description: "Track blockers before launch.",
    state: "open",
    dueOn: "2026-05-20T00:00:00Z",
    closedAt: null,
    createdAt: "2026-05-01T00:00:00Z",
    updatedAt: "2026-05-06T00:00:00Z",
    progress: {
      openCount: 3,
      closedCount: 2,
      totalCount: 5,
      percentComplete: 40,
    },
    openIssuesHref:
      "/mona/octo-app/issues?q=milestone%3A%22Launch+readiness%22+state%3Aopen&state=open&milestone=Launch+readiness",
    closedIssuesHref:
      "/mona/octo-app/issues?q=milestone%3A%22Launch+readiness%22+state%3Aclosed&state=closed&milestone=Launch+readiness",
    href: "/mona/octo-app/milestones/milestone-1",
    ...overrides,
  };
}

function milestonesView(
  overrides: Partial<RepositoryMilestonesView> = {},
): RepositoryMilestonesView {
  const items = overrides.items ?? [
    milestoneSummary(),
    milestoneSummary({
      id: "milestone-2",
      title: "Documentation sweep",
      description: null,
      dueOn: null,
      progress: {
        openCount: 0,
        closedCount: 4,
        totalCount: 4,
        percentComplete: 100,
      },
      openIssuesHref:
        "/mona/octo-app/issues?q=milestone%3A%22Documentation+sweep%22+state%3Aopen&state=open&milestone=Documentation+sweep",
      closedIssuesHref:
        "/mona/octo-app/issues?q=milestone%3A%22Documentation+sweep%22+state%3Aclosed&state=closed&milestone=Documentation+sweep",
      href: "/mona/octo-app/milestones/milestone-2",
    }),
  ];
  return {
    items,
    total: overrides.total ?? items.length,
    page: 1,
    pageSize: 100,
    openCount: overrides.openCount ?? 2,
    closedCount: overrides.closedCount ?? 1,
    filters: overrides.filters ?? {
      state: "open",
      sort: "updated-desc",
      q: null,
    },
    viewer: overrides.viewer ?? {
      permission: "write",
      canEditMilestones: true,
    },
    repository: {
      id: "repo-1",
      ownerLogin: "mona",
      name: "octo-app",
      visibility: "public",
      isArchived: false,
    },
    ...overrides,
  };
}

function renderMilestones(
  view: RepositoryMilestonesView = milestonesView(),
  repository: RepositoryOverview = repositoryOverview(),
) {
  return render(
    <RepositoryMilestonesPage
      milestones={view}
      query={{
        state: view.filters.state,
        sort: view.filters.sort,
        q: view.filters.q,
      }}
      repository={repository}
    />,
  );
}

function expectNoDeadControls(container: HTMLElement) {
  expect(container.querySelectorAll('a[href="#"], a:not([href])')).toHaveLength(
    0,
  );
  for (const button of Array.from(container.querySelectorAll("button"))) {
    expect(button.textContent?.trim()).not.toEqual("");
    if (button.hasAttribute("disabled")) {
      expect(button).toHaveAttribute("aria-disabled", "true");
    }
  }
}

describe("RepositoryMilestonesPage", () => {
  it("renders tabs, progress, due dates, and concrete issue links", () => {
    const { container } = renderMilestones();

    expect(screen.getByRole("heading", { name: "Milestones" })).toBeVisible();
    expect(screen.getByRole("link", { name: /Open 2/ })).toHaveAttribute(
      "href",
      repositoryMilestonesHref("mona", "octo-app", {
        state: "open",
        sort: "updated-desc",
      }),
    );
    expect(screen.getByRole("link", { name: /Closed 1/ })).toHaveAttribute(
      "href",
      expect.stringContaining("state=closed"),
    );
    expect(screen.getByText("Launch readiness")).toBeVisible();
    expect(screen.getByRole("img", { name: "40% complete" })).toBeVisible();
    expect(screen.getByText("May 20, 2026")).toBeVisible();
    expect(screen.getByRole("link", { name: /3 open issues/ })).toHaveAttribute(
      "href",
      expect.stringContaining("/mona/octo-app/issues"),
    );
    expect(
      screen.getByRole("link", { name: /2 closed issues/ }),
    ).toHaveAttribute("href", expect.stringContaining("state=closed"));
    expectNoDeadControls(container);
  });

  it("preserves tab state in sort links", () => {
    renderMilestones(
      milestonesView({
        filters: {
          state: "closed",
          sort: "due-asc",
          q: "docs",
        },
      }),
    );

    fireEvent.click(screen.getByText("Sort"));
    expect(
      screen.getByRole("menuitemradio", { name: /Most issues/ }),
    ).toHaveAttribute(
      "href",
      repositoryMilestonesHref("mona", "octo-app", {
        state: "closed",
        sort: "issues-desc",
        q: "docs",
      }),
    );
  });

  it("renders milestone search and preserves search in tab links", () => {
    renderMilestones(
      milestonesView({
        filters: {
          state: "open",
          sort: "alpha-asc",
          q: "launch",
        },
      }),
    );

    expect(screen.getByLabelText("Search milestones")).toHaveValue("launch");
    expect(screen.getByRole("link", { name: /Closed 1/ })).toHaveAttribute(
      "href",
      repositoryMilestonesHref("mona", "octo-app", {
        state: "closed",
        sort: "alpha-asc",
        q: "launch",
      }),
    );
    expect(screen.getByRole("link", { name: "Clear" })).toHaveAttribute(
      "href",
      repositoryMilestonesHref("mona", "octo-app", {
        state: "open",
        sort: "alpha-asc",
      }),
    );
  });

  it("creates milestones through the same-origin proxy and refreshes", async () => {
    renderMilestones();

    fireEvent.click(screen.getByRole("button", { name: "New milestone" }));
    fireEvent.change(screen.getByLabelText("Milestone title"), {
      target: { value: "Contributor beta" },
    });
    fireEvent.change(screen.getByLabelText("Milestone due date"), {
      target: { value: "2026-06-01" },
    });
    fireEvent.change(screen.getByLabelText("Milestone description"), {
      target: { value: "Invite external maintainers." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save milestone" }));

    await waitFor(() => expect(refreshMock).toHaveBeenCalled());
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions",
      expect.objectContaining({
        method: "POST",
        headers: { "content-type": "application/json" },
        body: expect.any(String),
      }),
    );
    const [, request] = vi.mocked(fetch).mock.calls[0];
    expect(JSON.parse(String((request as RequestInit).body))).toEqual({
      title: "Contributor beta",
      description: "Invite external maintainers.",
      dueOn: "2026-06-01",
    });
  });

  it("edits, closes, and deletes permissioned milestones", async () => {
    renderMilestones();

    const firstRow = screen.getByText("Launch readiness").closest("article");
    expect(firstRow).toBeTruthy();
    fireEvent.click(within(firstRow as HTMLElement).getByText("Edit"));
    fireEvent.change(screen.getByLabelText("Milestone title"), {
      target: { value: "Launch readiness v2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save milestone" }));

    await waitFor(() => expect(refreshMock).toHaveBeenCalledTimes(1));
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions/milestone-1",
      expect.objectContaining({ method: "PATCH" }),
    );

    fireEvent.click(within(firstRow as HTMLElement).getByText("Close"));
    await waitFor(() => expect(refreshMock).toHaveBeenCalledTimes(2));
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions/milestone-1",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ action: "close" }),
      }),
    );

    fireEvent.click(within(firstRow as HTMLElement).getByText("Delete"));
    await waitFor(() => expect(refreshMock).toHaveBeenCalledTimes(3));
    expect(fetch).toHaveBeenCalledWith(
      "/mona/octo-app/milestones/actions/milestone-1",
      { method: "DELETE" },
    );
  });

  it("hides writer controls for readers and shows a working empty state CTA only to writers", () => {
    const { rerender } = renderMilestones(
      milestonesView({
        items: [],
        total: 0,
        viewer: {
          permission: "read",
          canEditMilestones: false,
        },
      }),
    );

    expect(screen.queryByRole("button", { name: "New milestone" })).toBeNull();
    expect(screen.getByText("No open milestones")).toBeVisible();

    rerender(
      <RepositoryMilestonesPage
        milestones={milestonesView({ items: [], total: 0 })}
        query={{ state: "open", sort: "updated-desc" }}
        repository={repositoryOverview()}
      />,
    );
    expect(
      screen.getAllByRole("button", { name: "New milestone" }),
    ).toHaveLength(2);
  });
});
