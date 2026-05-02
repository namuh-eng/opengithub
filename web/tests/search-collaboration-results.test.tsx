import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CollaborationSearchResultsPage } from "@/components/CollaborationSearchResultsPage";
import type {
  CollaborationSearchResponse,
  GlobalSearchResult,
} from "@/lib/api";

function result(
  overrides: Partial<GlobalSearchResult> = {},
): GlobalSearchResult {
  return {
    document: {
      id: overrides.document?.id ?? "doc-issue",
      repository_id: overrides.document?.repository_id ?? "repo-1",
      owner_user_id: overrides.document?.owner_user_id ?? "user-1",
      owner_organization_id: overrides.document?.owner_organization_id ?? null,
      kind: overrides.document?.kind ?? "issue",
      resource_id: overrides.document?.resource_id ?? "repo-1:41",
      title: overrides.document?.title ?? "Search 004 issue shell",
      body: overrides.document?.body ?? "Issue body with search 004 context",
      path: overrides.document?.path ?? null,
      language: overrides.document?.language ?? null,
      branch: overrides.document?.branch ?? null,
      visibility: overrides.document?.visibility ?? "public",
      metadata: overrides.document?.metadata ?? {
        number: 41,
        state: "open",
        labels: [{ name: "urgent" }],
        assignees: [{ login: "mona" }],
        milestone: { title: "M1" },
        authorLogin: "octavia",
        commentCount: 7,
        interactionCount: 13,
      },
      indexed_at: overrides.document?.indexed_at ?? "2026-05-02T00:00:00Z",
      created_at: overrides.document?.created_at ?? "2026-05-02T00:00:00Z",
      updated_at: overrides.document?.updated_at ?? "2026-05-02T00:00:00Z",
    },
    rank: overrides.rank ?? 1,
    type: overrides.type ?? "issues",
    href: overrides.href ?? "/mona/editorial/issues/41",
    title: overrides.title ?? "Search 004 issue shell",
    summary: overrides.summary ?? "Issue body with search 004 context",
    owner_login: overrides.owner_login ?? "mona",
    repository_name: overrides.repository_name ?? "editorial",
    display_name: overrides.display_name ?? null,
    avatar_url: overrides.avatar_url ?? null,
    visibility: overrides.visibility ?? "public",
    updated_at: overrides.updated_at ?? "2026-05-02T00:00:00Z",
    snippet: overrides.snippet ?? null,
    snippets: overrides.snippets ?? [],
    match_count: overrides.match_count ?? 0,
    hidden_match_count: overrides.hidden_match_count ?? 0,
    blob_href: overrides.blob_href ?? null,
    commit: overrides.commit ?? null,
  };
}

function response(items: GlobalSearchResult[]): CollaborationSearchResponse {
  return {
    items,
    total: 61,
    page: 2,
    pageSize: 30,
    typeCounts: [
      { resultType: "issues", label: "Issues", count: 41 },
      { resultType: "pull_requests", label: "Pull requests", count: 20 },
    ],
    facets: {
      states: [{ value: "open", label: "open", count: 18, selected: true }],
      owners: [{ value: "mona", label: "mona", count: 9, selected: false }],
      labels: [{ value: "urgent", label: "urgent", count: 5, selected: false }],
      assignees: [{ value: "mona", label: "mona", count: 3, selected: false }],
      reviewers: [
        { value: "octavia", label: "octavia", count: 2, selected: false },
      ],
      milestones: [{ value: "M1", label: "M1", count: 4, selected: false }],
    },
    activeChips: [
      {
        qualifier: "state",
        value: "open",
        label: "state:open",
        removeQuery: "search004",
      },
    ],
    sortOptions: [
      { value: "best_match", label: "Best match", selected: false },
      { value: "most_commented", label: "Most commented", selected: true },
      { value: "least_commented", label: "Least commented", selected: false },
    ],
    activeSort: "most_commented",
    queryDurationMs: 12,
  };
}

describe("CollaborationSearchResultsPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders issue search rows with facets, sort links, chips, and pagination", () => {
    render(
      <CollaborationSearchResultsPage
        activeType="issues"
        query="search004 state:open"
        results={response([result()])}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Issues search" }),
    ).toBeVisible();
    expect(screen.getByText("61 issues results in 12ms")).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Search 004 issue shell/ }),
    ).toHaveAttribute("href", "/mona/editorial/issues/41");
    expect(screen.getAllByText("open").length).toBeGreaterThan(0);
    expect(screen.getAllByText("urgent").length).toBeGreaterThan(0);
    expect(screen.getByText("@mona")).toBeVisible();
    expect(screen.getByText("Milestone: M1")).toBeVisible();
    expect(screen.getByText("7 comments")).toBeVisible();
    expect(screen.getByRole("link", { name: "mona9" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen+owner%3Amona&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "urgent5" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen+label%3Aurgent&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "open18" })).toHaveAttribute(
      "href",
      "/search?q=search004&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "comments:>10" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen+comments%3A%3E10&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "state:open ×" })).toHaveAttribute(
      "href",
      "/search?q=search004&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByText("Sort by: Most commented")).toBeVisible();
    expect(
      screen.getByRole("link", { name: "Most commented" }),
    ).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen&type=issues&sort=most_commented&view=comfortable",
    );
    expect(
      screen.getByRole("link", { name: "Least commented" }),
    ).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen&type=issues&sort=least_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "Compact" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen&type=issues&sort=most_commented&view=compact",
    );
    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen&type=issues&sort=most_commented&view=comfortable",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/search?q=search004+state%3Aopen&type=issues&page=3&sort=most_commented&view=comfortable",
    );
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders pull request reviewer and branch metadata without dead controls", () => {
    render(
      <CollaborationSearchResultsPage
        activeType="pull_requests"
        query="search004 reviewer:octavia"
        results={response([
          result({
            document: {
              ...result().document,
              id: "doc-pr",
              kind: "pull_request",
              resource_id: "repo-1:42",
              metadata: {
                number: 42,
                state: "merged",
                labels: [{ name: "review" }],
                assignees: [{ login: "mona" }],
                reviewers: [{ login: "octavia" }],
                milestone: { title: "M2" },
                authorLogin: "mona",
                headRef: "feature/search-004",
                baseRef: "main",
                commentCount: 9,
                interactionCount: 17,
              },
            },
            type: "pull_requests",
            href: "/mona/editorial/pull/42",
            title: "Search 004 pull shell",
          }),
        ])}
        view="compact"
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Pull requests search" }),
    ).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Search 004 pull shell/ }),
    ).toHaveAttribute("href", "/mona/editorial/pull/42");
    expect(screen.getByText("merged")).toBeVisible();
    expect(screen.getByText("review: @octavia")).toBeVisible();
    expect(screen.getByText("feature/search-004 -> main")).toBeVisible();
    expect(screen.getByRole("link", { name: "Save" })).toHaveAttribute(
      "href",
      "/search?q=search004+reviewer%3Aoctavia&type=pull_requests&saved=1&sort=most_commented&view=compact",
    );
    expect(screen.getByRole("link", { name: "Compact" })).toHaveAttribute(
      "aria-current",
      "true",
    );
    expect(
      document.querySelectorAll('button:not([type]), button[type="button"]'),
    ).toHaveLength(0);
  });

  it("creates saved searches with inline success and error feedback", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ name: "Regression triage" }), {
        status: 201,
        headers: { "content-type": "application/json" },
      }),
    );

    render(
      <CollaborationSearchResultsPage
        activeType="issues"
        query="search004 state:open"
        results={response([result()])}
        saved
      />,
    );

    fireEvent.change(screen.getByLabelText("Saved search name"), {
      target: { value: "Regression triage" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create saved search" }),
    );

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "/search/saved-searches",
        expect.objectContaining({
          body: JSON.stringify({
            name: "Regression triage",
            query: "search004 state:open",
            scope: "issues",
          }),
          method: "POST",
        }),
      ),
    );
    expect(await screen.findByText('Saved "Regression triage".')).toBeVisible();
  });
});
