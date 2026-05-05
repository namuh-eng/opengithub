import { fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDiscussionCategoryTemplatePage } from "@/components/RepositoryDiscussionCategoryTemplatePage";
import type {
  DiscussionCategoryTemplateView,
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

function templateView(
  overrides: Partial<DiscussionCategoryTemplateView> = {},
): DiscussionCategoryTemplateView {
  const base: DiscussionCategoryTemplateView = {
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
    category: {
      id: "cat-2",
      slug: "q-a",
      name: "Q&A",
      emoji: "❓",
      description: "Ask and answer project questions.",
      format: "question_and_answer",
      acceptsAnswers: true,
      isPoll: false,
      isDefault: false,
      sectionId: "section-1",
      sectionName: "Product work",
      templatePath: ".github/DISCUSSION_TEMPLATE/q-a.yml",
      count: 12,
      openCount: 9,
      position: 2,
      href: "/namuh-eng/opengithub/discussions/categories/q-a",
      editHref:
        "/namuh-eng/opengithub/discussions/categories/edit?category=cat-2",
      templateHref:
        "/namuh-eng/opengithub/discussions/categories/cat-2/template",
      createdAt: "2026-05-01T00:00:00Z",
      updatedAt: "2026-05-01T00:00:00Z",
    },
    path: ".github/DISCUSSION_TEMPLATE/q-a.yml",
    content:
      "name: Q&A\ndescription: Ask a question\nbody:\n  - type: input\n    id: summary\n    attributes:\n      label: Summary\n    validations:\n      required: true\n",
    contentSha: "abc123",
    branch: "main",
    form: {
      categorySlug: "q-a",
      templatePath: ".github/DISCUSSION_TEMPLATE/q-a.yml",
      title: "Q&A",
      description: "Ask a question",
      body: "",
      fields: [
        {
          id: "summary",
          fieldType: "input",
          label: "Summary",
          description: null,
          placeholder: null,
          required: true,
          options: [],
        },
      ],
      valid: true,
      fallback: false,
      parseError: null,
    },
    commitHref: null,
    blobHref:
      "/namuh-eng/opengithub/blob/main/.github/DISCUSSION_TEMPLATE/q-a.yml",
  };
  return { ...base, ...overrides };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("RepositoryDiscussionCategoryTemplatePage", () => {
  it("renders YAML editor, parsed preview, and concrete navigation", () => {
    render(
      <RepositoryDiscussionCategoryTemplatePage
        repository={repositoryOverview()}
        template={templateView()}
      />,
    );

    expect(
      screen.getByRole("heading", { level: 1, name: /Q&A/ }),
    ).toBeVisible();
    expect(
      (screen.getByLabelText("Discussion template YAML") as HTMLTextAreaElement)
        .value,
    ).toContain("name: Q&A");
    expect(screen.getByText("Summary")).toBeVisible();
    expect(screen.getByText("Required")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Back to categories" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/categories/edit",
    );
    expect(screen.getByRole("link", { name: "View file" })).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/blob/main/.github/DISCUSSION_TEMPLATE/q-a.yml",
    );
  });

  it("previews YAML and commits with conflict-safe payloads", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          categorySlug: "q-a",
          templatePath: ".github/DISCUSSION_TEMPLATE/q-a.yml",
          title: "Support",
          description: "New preview",
          body: "",
          fields: [
            {
              id: "steps",
              fieldType: "textarea",
              label: "Steps",
              description: "What happened?",
              placeholder: null,
              required: true,
              options: [],
            },
          ],
          valid: true,
          fallback: false,
          parseError: null,
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          commitHref: "/namuh-eng/opengithub/commits/def456",
          commitOid: "def456",
          proposed: true,
          template: {
            ...templateView(),
            branch: "discussion-template-update",
            content: "name: Support\n",
            contentSha: "def456",
          },
        }),
      });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionCategoryTemplatePage
        repository={repositoryOverview()}
        template={templateView()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Discussion template YAML"), {
      target: { value: "name: Support\n" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Preview" }));

    await screen.findByText("Template preview refreshed.");
    expect(screen.getByText("Steps")).toBeVisible();
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "/api/repos/namuh-eng/opengithub/settings/discussions/categories/cat-2/template/preview",
    );
    expect(JSON.parse(String(fetchMock.mock.calls[0]?.[1]?.body))).toEqual({
      content: "name: Support\n",
    });

    fireEvent.change(screen.getByLabelText("Branch"), {
      target: { value: "discussion-template-update" },
    });
    fireEvent.click(screen.getByLabelText("Propose on a separate branch"));
    fireEvent.click(screen.getByRole("button", { name: "Commit template" }));

    await screen.findByText(
      "Template change was committed on a proposed branch.",
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe(
      "/api/repos/namuh-eng/opengithub/settings/discussions/categories/cat-2/template",
    );
    expect(
      JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body)),
    ).toMatchObject({
      branch: "discussion-template-update",
      content: "name: Support\n",
      expectedContentSha: "abc123",
      proposeChange: true,
    });
  });

  it("shows sanitized API errors and has no dead controls", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        error: {
          code: "validation_failed",
          message: "discussion template YAML cannot be empty",
        },
        status: 422,
      }),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { container } = render(
      <RepositoryDiscussionCategoryTemplatePage
        repository={repositoryOverview()}
        template={templateView()}
      />,
    );

    fireEvent.change(screen.getByLabelText("Discussion template YAML"), {
      target: { value: "" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Preview" }));

    await screen.findByText("discussion template YAML cannot be empty");
    expect(container.querySelector('a[href="#"]')).toBeNull();
    for (const button of within(container).getAllByRole("button")) {
      expect(button).not.toHaveAttribute("onclick", "");
    }
    expect(container.innerHTML).not.toMatch(/#0969da|@primer\/|Octicon/);
  });
});
