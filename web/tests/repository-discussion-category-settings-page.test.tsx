import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDiscussionCategorySettingsPage } from "@/components/RepositoryDiscussionCategorySettingsPage";
import type {
  DiscussionCategorySettingsView,
  RepositoryOverview,
} from "@/lib/api";

function repositoryOverview(): RepositoryOverview {
  return {
    id: "repo-1",
    owner_user_id: "user-1",
    owner_organization_id: null,
    owner_login: "namuh-eng",
    name: "opengithub",
    description: "A rust-first collaboration platform.",
    visibility: "public",
    default_branch: "main",
    is_archived: false,
    created_by_user_id: "user-1",
    created_at: "2026-05-01T00:00:00Z",
    updated_at: "2026-05-01T00:00:00Z",
    viewerPermission: "admin",
    branchCount: 3,
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
      forksCount: 1,
      releasesCount: 0,
      deploymentsCount: 0,
      contributorsCount: 2,
      languages: [],
    },
    viewerState: {
      forkedRepositoryHref: null,
      starred: false,
      watching: false,
    },
    cloneUrls: {
      git: "git@opengithub.namuh.co:namuh-eng/opengithub.git",
      https: "https://opengithub.namuh.co/namuh-eng/opengithub.git",
      zip: "/namuh-eng/opengithub/archive/refs/heads/main.zip",
    },
  };
}

function categorySettings(
  overrides: Partial<DiscussionCategorySettingsView> = {},
): DiscussionCategorySettingsView {
  const base: DiscussionCategorySettingsView = {
    repository: {
      id: "repo-1",
      owner: "namuh-eng",
      name: "opengithub",
      visibility: "public",
      isArchived: false,
      href: "/namuh-eng/opengithub",
      discussionsHref: "/namuh-eng/opengithub/discussions",
    },
    viewer: {
      authenticated: true,
      permission: "admin",
      canRead: true,
      canManage: true,
    },
    enabled: true,
    disabledReason: null,
    categoryLimit: 25,
    remainingCategories: 22,
    sections: [
      {
        id: "section-1",
        name: "Product work",
        position: 1,
        categoryCount: 1,
      },
    ],
    categories: [
      {
        id: "cat-1",
        slug: "general",
        name: "General",
        emoji: "💬",
        description: "General project conversation.",
        format: "open_ended",
        acceptsAnswers: false,
        isPoll: false,
        isDefault: true,
        sectionId: null,
        sectionName: null,
        templatePath: null,
        count: 8,
        openCount: 5,
        position: 1,
        href: "/namuh-eng/opengithub/discussions/categories/general",
        editHref:
          "/namuh-eng/opengithub/discussions/categories/edit?category=cat-1",
        templateHref:
          "/namuh-eng/opengithub/discussions/categories/cat-1/template",
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-01T00:00:00Z",
      },
      {
        id: "cat-2",
        slug: "q-a",
        name: "Q&A",
        emoji: "🙏",
        description: "Ask focused questions.",
        format: "question_and_answer",
        acceptsAnswers: true,
        isPoll: false,
        isDefault: false,
        sectionId: "section-1",
        sectionName: "Product work",
        templatePath: ".github/DISCUSSION_TEMPLATE/q-a.yml",
        count: 3,
        openCount: 2,
        position: 2,
        href: "/namuh-eng/opengithub/discussions/categories/q-a",
        editHref:
          "/namuh-eng/opengithub/discussions/categories/edit?category=cat-2",
        templateHref:
          "/namuh-eng/opengithub/discussions/categories/cat-2/template",
        createdAt: "2026-05-01T00:00:00Z",
        updatedAt: "2026-05-01T00:00:00Z",
      },
    ],
  };
  return { ...base, ...overrides };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RepositoryDiscussionCategorySettingsPage", () => {
  it("renders Editorial category settings grouped by sections", () => {
    render(
      <RepositoryDiscussionCategorySettingsPage
        repository={repositoryOverview()}
        settings={categorySettings()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Discussion categories" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "General" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/general",
    );
    expect(
      screen.getByRole("heading", { name: "Product work" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Question and Answer")).toBeInTheDocument();
    expect(screen.getByText("Can manage")).toBeInTheDocument();
    expect(screen.getByText("22")).toBeInTheDocument();
    expect(screen.queryByText("#0969da")).not.toBeInTheDocument();
  });

  it("creates a category through the same-origin proxy", async () => {
    const updated = categorySettings({
      remainingCategories: 21,
      categories: [
        ...categorySettings().categories,
        {
          ...categorySettings().categories[0],
          id: "cat-3",
          slug: "announcements",
          name: "Announcements",
          emoji: "📣",
          format: "announcement",
          acceptsAnswers: false,
          isDefault: false,
          position: 3,
        },
      ],
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(updated), {
        headers: { "content-type": "application/json" },
        status: 200,
      }),
    );

    render(
      <RepositoryDiscussionCategorySettingsPage
        repository={repositoryOverview()}
        settings={categorySettings()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "New category" }));
    fireEvent.change(screen.getByLabelText("Category emoji"), {
      target: { value: "📣" },
    });
    fireEvent.change(screen.getByLabelText("Category name"), {
      target: { value: "Announcements" },
    });
    fireEvent.change(screen.getByLabelText("Category description"), {
      target: { value: "Release notes and maintainer updates." },
    });
    fireEvent.change(screen.getByLabelText("Category format"), {
      target: { value: "announcement" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create category" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/settings/discussions/categories",
        expect.objectContaining({
          method: "POST",
        }),
      );
    });
    expect(
      JSON.parse(String(fetchMock.mock.calls[0]?.[1]?.body)),
    ).toMatchObject({
      description: "Release notes and maintainer updates.",
      emoji: "📣",
      format: "announcement",
      name: "Announcements",
      sectionId: null,
    });
    expect(
      await screen.findByRole("link", { name: "Announcements" }),
    ).toBeInTheDocument();
  });

  it("edits a category and shows server validation errors", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          error: {
            code: "validation_failed",
            message:
              "discussion category names must be unique within the repository",
          },
          status: 422,
        }),
        { headers: { "content-type": "application/json" }, status: 422 },
      ),
    );

    render(
      <RepositoryDiscussionCategorySettingsPage
        repository={repositoryOverview()}
        settings={categorySettings()}
      />,
    );

    const qaRow = screen
      .getByRole("link", { name: "Q&A" })
      .closest(".list-row");
    expect(qaRow).not.toBeNull();
    fireEvent.click(
      within(qaRow as HTMLElement).getByRole("button", { name: "Edit" }),
    );
    fireEvent.change(screen.getByLabelText("Category name"), {
      target: { value: "General" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save category" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/settings/discussions/categories/cat-2",
        expect.objectContaining({ method: "PATCH" }),
      );
    });
    expect(
      await screen.findByText(
        "discussion category names must be unique within the repository",
      ),
    ).toBeInTheDocument();
  });

  it("creates sections and moves categories through concrete controls", async () => {
    const updated = categorySettings({
      sections: [
        ...categorySettings().sections,
        {
          id: "section-2",
          name: "Maintainer notes",
          position: 2,
          categoryCount: 0,
        },
      ],
    });
    const moved = categorySettings({
      categories: categorySettings().categories.map((category) =>
        category.id === "cat-1"
          ? {
              ...category,
              sectionId: "section-1",
              sectionName: "Product work",
            }
          : category,
      ),
    });
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(JSON.stringify(updated), {
          headers: { "content-type": "application/json" },
          status: 200,
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify(moved), {
          headers: { "content-type": "application/json" },
          status: 200,
        }),
      );

    render(
      <RepositoryDiscussionCategorySettingsPage
        repository={repositoryOverview()}
        settings={categorySettings()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "New section" }));
    fireEvent.change(screen.getByLabelText("Section name"), {
      target: { value: "Maintainer notes" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create section" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/settings/discussions/sections",
        expect.objectContaining({ method: "POST" }),
      );
    });
    expect(
      await screen.findByRole("heading", { name: "Maintainer notes" }),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Move General to section"), {
      target: { value: "section-1" },
    });
    await waitFor(() => {
      expect(fetchMock).toHaveBeenLastCalledWith(
        "/api/repos/namuh-eng/opengithub/settings/discussions/categories/order",
        expect.objectContaining({ method: "PUT" }),
      );
    });
    expect(
      JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body)).items,
    ).toContainEqual(
      expect.objectContaining({ id: "cat-1", sectionId: "section-1" }),
    );
  });

  it("deletes categories only after choosing a move destination", async () => {
    const updated = categorySettings({
      categories: [categorySettings().categories[1]],
      remainingCategories: 23,
    });
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(updated), {
        headers: { "content-type": "application/json" },
        status: 200,
      }),
    );

    render(
      <RepositoryDiscussionCategorySettingsPage
        repository={repositoryOverview()}
        settings={categorySettings()}
      />,
    );

    const generalRow = screen
      .getByRole("link", { name: "General" })
      .closest(".list-row");
    expect(generalRow).not.toBeNull();
    fireEvent.click(
      within(generalRow as HTMLElement).getByRole("button", {
        name: "Delete",
      }),
    );
    expect(
      screen.getByRole("heading", { name: "Move discussions before deleting" }),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Destination category"), {
      target: { value: "cat-2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Delete and move" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/repos/namuh-eng/opengithub/settings/discussions/categories/cat-1",
        expect.objectContaining({ method: "DELETE" }),
      );
    });
    expect(JSON.parse(String(fetchMock.mock.calls[0]?.[1]?.body))).toEqual({
      moveToCategoryId: "cat-2",
    });
    await waitFor(() => {
      expect(
        screen.queryByRole("link", { name: "General" }),
      ).not.toBeInTheDocument();
    });
  });
});
