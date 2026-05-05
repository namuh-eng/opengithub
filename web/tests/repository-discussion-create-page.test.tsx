import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RepositoryDiscussionCategoryChooser } from "@/components/RepositoryDiscussionCategoryChooser";
import { RepositoryDiscussionCreatePage } from "@/components/RepositoryDiscussionCreatePage";
import type { DiscussionCreationView, RepositoryOverview } from "@/lib/api";
import {
  repositoryDiscussionChooseCategoryHref,
  repositoryNewDiscussionHref,
} from "@/lib/navigation";

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
    viewerPermission: "write",
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

function creationView(
  overrides: Partial<DiscussionCreationView> = {},
): DiscussionCreationView {
  const base: DiscussionCreationView = {
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
      permission: "write",
      canRead: true,
      canVote: true,
      canCreate: true,
    },
    enabled: true,
    disabledReason: null,
    categories: [
      {
        id: "cat-1",
        slug: "general",
        name: "General",
        emoji: "💬",
        description: "General project conversation.",
        acceptsAnswers: false,
        isPoll: false,
        count: 8,
        openCount: 5,
        href: "/namuh-eng/opengithub/discussions/categories/general",
        formHref: "/namuh-eng/opengithub/discussions/new?category=general",
      },
      {
        id: "cat-2",
        slug: "q-a",
        name: "Q&A",
        emoji: "🙏",
        description:
          "Ask an answerable question with enough context for maintainers.",
        acceptsAnswers: true,
        isPoll: false,
        count: 3,
        openCount: 2,
        href: "/namuh-eng/opengithub/discussions/categories/q-a",
        formHref: "/namuh-eng/opengithub/discussions/new?category=q-a",
      },
      {
        id: "cat-3",
        slug: "polls",
        name: "Polls",
        emoji: "📊",
        description: "Collect structured feedback from the community.",
        acceptsAnswers: false,
        isPoll: true,
        count: 1,
        openCount: 1,
        href: "/namuh-eng/opengithub/discussions/categories/polls",
        formHref: "/namuh-eng/opengithub/discussions/new?category=polls",
      },
    ],
    selectedCategory: null,
    form: {
      categorySlug: null,
      templatePath: null,
      title: "",
      description: null,
      body: "",
      fields: [],
      valid: true,
      fallback: true,
      parseError: null,
    },
    similarSearch: {
      required: true,
      query: "",
      href: "/namuh-eng/opengithub/discussions?q=",
    },
    communityLinks: [
      {
        id: "link-1",
        label: "Code of conduct",
        href: "/namuh-eng/opengithub/community/code-of-conduct",
        kind: "code_of_conduct",
      },
    ],
  };
  return { ...base, ...overrides };
}

function creationFormView(
  overrides: Partial<DiscussionCreationView> = {},
): DiscussionCreationView {
  const view = creationView(overrides);
  return {
    ...view,
    selectedCategory: view.categories[0],
    form: {
      ...view.form,
      categorySlug: "general",
      title: "",
      body: "",
      fallback: true,
    },
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RepositoryDiscussionCategoryChooser", () => {
  it("renders Editorial category cards with working Get started links", () => {
    const { container } = render(
      <RepositoryDiscussionCategoryChooser
        creation={creationView()}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByRole("link", { name: "Discussions" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", { name: "Choose a category" }),
    ).toBeVisible();

    const categories = screen.getByRole("region", {
      name: "Discussion categories",
    });
    expect(
      within(categories).getByRole("heading", { name: "General" }),
    ).toBeVisible();
    expect(within(categories).getByText("Answers enabled")).toBeVisible();
    expect(within(categories).getByText("Poll")).toBeVisible();
    expect(within(categories).getByText("5")).toBeVisible();
    expect(within(categories).getAllByText("open")[0]).toBeVisible();

    expect(
      within(categories).getAllByRole("link", { name: "Get started" })[0],
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/discussions/new?category=general",
    );
    expect(
      screen.getByRole("link", { name: "Code of conduct" }),
    ).toHaveAttribute(
      "href",
      "/namuh-eng/opengithub/community/code-of-conduct",
    );
    expect(container.querySelector('[href="#"]')).toBeNull();
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.innerHTML).not.toContain("@primer/");
  });

  it("shows disabled and empty states without inert links", () => {
    const { container, rerender } = render(
      <RepositoryDiscussionCategoryChooser
        creation={creationView({
          enabled: false,
          disabledReason: "Discussions are disabled by organization policy.",
        })}
        repository={repositoryOverview()}
      />,
    );

    expect(screen.getByText("Discussions disabled")).toBeVisible();
    expect(
      screen.getByText("Discussions are disabled by organization policy."),
    ).toBeVisible();
    expect(screen.queryByRole("link", { name: "Get started" })).toBeNull();
    expect(screen.getAllByText("Get started")[0]).toHaveAttribute(
      "aria-disabled",
      "true",
    );

    rerender(
      <RepositoryDiscussionCategoryChooser
        creation={creationView({ categories: [] })}
        repository={repositoryOverview()}
      />,
    );

    expect(
      screen.getByText("No discussion categories are available."),
    ).toBeVisible();
    expect(container.querySelector('[href="#"]')).toBeNull();
  });

  it("builds chooser and preselected category hrefs with safe query state", () => {
    expect(
      repositoryDiscussionChooseCategoryHref("namuh-eng", "opengithub"),
    ).toBe("/namuh-eng/opengithub/discussions/new/choose");
    expect(
      repositoryNewDiscussionHref("namuh-eng", "opengithub", {
        category: "q-a",
        q: "long title",
        next: "/namuh-eng/opengithub/discussions",
      }),
    ).toBe(
      "/namuh-eng/opengithub/discussions/new?category=q-a&q=long+title&next=%2Fnamuh-eng%2Fopengithub%2Fdiscussions",
    );
    expect(
      repositoryNewDiscussionHref("namuh-eng", "opengithub", {
        category: "general",
        next: "https://example.com/unsafe",
      }),
    ).toBe("/namuh-eng/opengithub/discussions/new?category=general");
  });
});

describe("RepositoryDiscussionCreatePage", () => {
  it("renders the generic Markdown composer, similar-search acknowledgement, and safe links", () => {
    const { container } = render(
      <RepositoryDiscussionCreatePage
        creation={creationFormView()}
        owner="namuh-eng"
        repo="opengithub"
      />,
    );

    expect(screen.getByRole("heading", { name: /General/ })).toBeVisible();
    expect(screen.getByLabelText("Discussion body")).toBeVisible();
    expect(
      screen.getByRole("toolbar", { name: "Markdown formatting toolbar" }),
    ).toBeVisible();
    expect(
      screen.getByRole("checkbox", {
        name: /I have done a search for similar discussions/i,
      }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Choose a different category" }),
    ).toHaveAttribute("href", "/namuh-eng/opengithub/discussions/new/choose");
    expect(container.querySelector('[href="#"]')).toBeNull();
    expect(container.innerHTML).not.toContain("#0969da");
    expect(container.innerHTML).not.toContain("@primer/");
  });

  it("renders sanitized Markdown preview without creating a discussion", async () => {
    const fetchMock = vi.fn(async (url: string | URL | Request) => {
      expect(String(url)).toBe("/markdown/preview");
      return new Response(
        JSON.stringify({
          contentSha: "preview-sha",
          html: "<p><strong>Preview</strong></p>",
          cached: false,
        }),
        { status: 200 },
      );
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionCreatePage
        creation={creationFormView()}
        owner="namuh-eng"
        repo="opengithub"
      />,
    );

    fireEvent.change(screen.getByLabelText("Discussion body"), {
      target: { value: "**Preview**<script>bad()</script>" },
    });
    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));

    expect(await screen.findByText("Preview")).toBeVisible();
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("requires title and similar-search acknowledgement before submit", async () => {
    render(
      <RepositoryDiscussionCreatePage
        creation={creationFormView()}
        owner="namuh-eng"
        repo="opengithub"
      />,
    );

    fireEvent.blur(screen.getByRole("textbox", { name: /Title/i }));
    fireEvent.blur(
      screen.getByRole("checkbox", {
        name: /I have done a search for similar discussions/i,
      }),
    );

    expect(screen.getByText("Title is required.")).toBeVisible();
    expect(
      screen.getByText("Similar-search acknowledgement is required."),
    ).toBeVisible();
  });

  it("submits generic discussions with attachment metadata and redirects", async () => {
    const assign = vi.fn();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { assign },
    });
    const fetchMock = vi.fn(
      async (url: string | URL | Request, init?: RequestInit) => {
        expect(String(url)).toBe(
          "/namuh-eng/opengithub/discussions/new/create",
        );
        const body = JSON.parse(String(init?.body));
        expect(body).toMatchObject({
          categorySlug: "general",
          title: "Search syntax ideas",
          body: "Support saved discussion searches.",
          similarSearchAcknowledged: true,
        });
        expect(body.attachmentDrafts[0]).toMatchObject({
          fileName: "sketch.png",
          contentType: "image/png",
          byteSize: 128,
        });
        expect(body.attachmentDrafts[0].storageKey).toContain(
          "discussion-drafts/",
        );
        return new Response(
          JSON.stringify({
            discussionId: "discussion-1",
            discussionNumber: 42,
            href: "/namuh-eng/opengithub/discussions/42",
            title: "Search syntax ideas",
            category: creationFormView().selectedCategory,
          }),
          { status: 201 },
        );
      },
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionCreatePage
        creation={creationFormView()}
        owner="namuh-eng"
        repo="opengithub"
      />,
    );

    fireEvent.change(screen.getByLabelText("Title *"), {
      target: { value: "Search syntax ideas" },
    });
    fireEvent.change(screen.getByLabelText("Discussion body"), {
      target: { value: "Support saved discussion searches." },
    });
    const file = new File(["x".repeat(128)], "sketch.png", {
      type: "image/png",
    });
    fireEvent.change(screen.getByLabelText("Attachments"), {
      target: { files: [file] },
    });
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /I have done a search for similar discussions/i,
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Start discussion" }));

    await waitFor(() =>
      expect(assign).toHaveBeenCalledWith(
        "/namuh-eng/opengithub/discussions/42",
      ),
    );
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("blocks oversized attachments before contacting the create route", async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    render(
      <RepositoryDiscussionCreatePage
        creation={creationFormView()}
        owner="namuh-eng"
        repo="opengithub"
      />,
    );

    fireEvent.change(screen.getByLabelText("Title *"), {
      target: { value: "Large file" },
    });
    const file = new File(["x"], "large.bin", {
      type: "application/octet-stream",
    });
    Object.defineProperty(file, "size", { value: 25 * 1024 * 1024 + 1 });
    fireEvent.change(screen.getByLabelText("Attachments"), {
      target: { files: [file] },
    });

    expect(
      await screen.findByText("large.bin is larger than 25 MiB."),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Start discussion" }),
    ).toBeDisabled();
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
