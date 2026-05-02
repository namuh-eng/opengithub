import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SearchResultsPage } from "@/components/SearchResultsPage";
import {
  type GlobalSearchResult,
  globalSearchPath,
  type ListEnvelope,
} from "@/lib/api";

function result(
  overrides: Partial<GlobalSearchResult> = {},
): GlobalSearchResult {
  return {
    document: {
      id: overrides.document?.id ?? "doc-1",
      repository_id: overrides.document?.repository_id ?? null,
      owner_user_id: overrides.document?.owner_user_id ?? "user-1",
      owner_organization_id: overrides.document?.owner_organization_id ?? null,
      kind: overrides.document?.kind ?? "repository",
      resource_id: overrides.document?.resource_id ?? "repo-1",
      title: overrides.document?.title ?? "editorial-search",
      body: overrides.document?.body ?? "A calm repository search surface",
      path: overrides.document?.path ?? null,
      language: overrides.document?.language ?? null,
      branch: overrides.document?.branch ?? null,
      visibility: overrides.document?.visibility ?? "public",
      metadata: overrides.document?.metadata ?? {},
      indexed_at: overrides.document?.indexed_at ?? "2026-05-01T00:00:00Z",
      created_at: overrides.document?.created_at ?? "2026-05-01T00:00:00Z",
      updated_at: overrides.document?.updated_at ?? "2026-05-01T00:00:00Z",
    },
    rank: overrides.rank ?? 1,
    type: overrides.type ?? "repositories",
    href: overrides.href ?? "/mona/editorial-search",
    title: overrides.title ?? "editorial-search",
    summary: overrides.summary ?? "A calm repository search surface",
    owner_login: overrides.owner_login ?? "mona",
    repository_name: overrides.repository_name ?? "editorial-search",
    display_name: overrides.display_name ?? null,
    avatar_url: overrides.avatar_url ?? null,
    visibility: overrides.visibility ?? "public",
    updated_at: overrides.updated_at ?? "2026-05-01T00:00:00Z",
    snippet: overrides.snippet ?? null,
    snippets: overrides.snippets ?? [],
    match_count: overrides.match_count ?? (overrides.snippet ? 1 : 0),
    hidden_match_count: overrides.hidden_match_count ?? 0,
    blob_href: overrides.blob_href ?? null,
    commit: overrides.commit ?? null,
  };
}

function envelope(
  items: GlobalSearchResult[],
): ListEnvelope<GlobalSearchResult> {
  return {
    items,
    total: items.length,
    page: 1,
    pageSize: 30,
  };
}

describe("SearchResultsPage", () => {
  it("renders repository, user, and organization results with real links", () => {
    render(
      <SearchResultsPage
        activeType="repositories"
        query="editorial"
        results={envelope([
          result(),
          result({
            document: {
              ...result().document,
              id: "doc-2",
              kind: "user",
              resource_id: "mona",
              title: "Mona Lisa",
            },
            type: "users",
            href: "/mona",
            title: "Mona Lisa",
            display_name: "Mona Lisa",
            owner_login: "mona",
            repository_name: null,
            summary: "Maintainer focused on review tools",
          }),
          result({
            document: {
              ...result().document,
              id: "doc-3",
              kind: "organization",
              resource_id: "namuh",
              title: "Namuh Labs",
            },
            type: "organizations",
            href: "/orgs/namuh",
            title: "Namuh Labs",
            display_name: "Namuh Labs",
            owner_login: "namuh",
            repository_name: null,
            summary: "Research and product engineering",
          }),
        ])}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Search opengithub" }),
    ).toBeVisible();
    expect(screen.getByText("3 repositories results")).toBeVisible();
    expect(
      screen.getByRole("link", { name: /editorial-search/ }),
    ).toHaveAttribute("href", "/mona/editorial-search");
    expect(screen.getByRole("link", { name: /Mona Lisa/ })).toHaveAttribute(
      "href",
      "/mona",
    );
    expect(screen.getByRole("link", { name: /Namuh Labs/ })).toHaveAttribute(
      "href",
      "/orgs/namuh",
    );
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("renders no-results syntax tips and preserves query in tabs", () => {
    render(
      <SearchResultsPage
        activeType="users"
        query="missing-person"
        results={{ items: [], total: 0, page: 1, pageSize: 30 }}
      />,
    );

    expect(screen.getByText('Nothing matched "missing-person".')).toBeVisible();
    expect(screen.getByText("owner:")).toBeVisible();
    expect(screen.getByRole("link", { name: "Repositories" })).toHaveAttribute(
      "href",
      "/search?q=missing-person&type=repositories",
    );
  });

  it("renders validation errors without leaking internals", () => {
    render(
      <SearchResultsPage
        activeType="repositories"
        query="x"
        results={{
          error: {
            code: "validation_failed",
            message:
              "search query must contain at least two non-whitespace characters",
          },
          status: 422,
        }}
      />,
    );

    expect(screen.getByText("Search unavailable")).toBeVisible();
    expect(screen.getByText(/Short searches need/)).toBeVisible();
    expect(screen.queryByText(/DATABASE_URL|stack trace|panic/i)).toBeNull();
  });

  it("renders code snippets and commit results with direct links", () => {
    render(
      <SearchResultsPage
        activeType="code"
        query="uniquePhase3"
        results={envelope([
          result({
            document: {
              ...result().document,
              id: "doc-code",
              kind: "code",
              resource_id: "repo-1:main:src/search_phase_three.rs",
              title: "src/search_phase_three.rs",
              body: "pub fn uniquePhase3() {}",
              path: "src/search_phase_three.rs",
              language: "Rust",
              branch: "main",
            },
            type: "code",
            href: "/mona/editorial-search/blob/main/src/search_phase_three.rs#L7",
            title: "src/search_phase_three.rs",
            summary: null,
            snippet: {
              path: "src/search_phase_three.rs",
              branch: "main",
              line_number: 7,
              fragment: "pub fn uniquePhase3() {}",
              language: "Rust",
              match_ranges: [{ start: 7, end: 19 }],
            },
          }),
          result({
            document: {
              ...result().document,
              id: "doc-commit",
              kind: "commit",
              resource_id: "abcdef1234567890",
              title: "Add uniquePhase3 search fixture",
              body: "Add uniquePhase3 search fixture\n\nIndex code snippets.",
            },
            type: "commits",
            href: "/mona/editorial-search/commit/abcdef1234567890",
            title: "Add uniquePhase3 search fixture",
            summary: null,
            commit: {
              oid: "abcdef1234567890",
              short_oid: "abcdef123456",
              message_title: "Add uniquePhase3 search fixture",
              message_body: "Index code snippets.",
              author_login: "mona",
              committed_at: "2026-05-01T00:00:00Z",
            },
          }),
        ])}
      />,
    );

    expect(
      screen.getByRole("link", { name: /src\/search_phase_three.rs/ }),
    ).toHaveAttribute(
      "href",
      "/mona/editorial-search/blob/main/src/search_phase_three.rs#L7",
    );
    expect(screen.getByText(/pub fn/)).toBeVisible();
    expect(screen.getAllByText("uniquePhase3").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("link", { name: /Add uniquePhase3 search fixture/ }),
    ).toHaveAttribute("href", "/mona/editorial-search/commit/abcdef1234567890");
    expect(screen.getByText("abcdef123456")).toBeVisible();
  });

  it("renders issue and pull request results with state, labels, and detail links", () => {
    render(
      <SearchResultsPage
        activeType="issues"
        query="phase4"
        results={envelope([
          result({
            document: {
              ...result().document,
              id: "doc-issue",
              kind: "issue",
              resource_id: "repo-1:7",
              title: "Investigate phase4 issue search",
              body: "Issue body carries phase4.",
              metadata: {
                number: 7,
                state: "closed",
                labels: [{ name: "bug", color: "d73a4a" }],
                authorLogin: "mona",
              },
            },
            type: "issues",
            href: "/mona/editorial-search/issues/7",
            title: "Investigate phase4 issue search",
            summary: "Issue body carries phase4.",
          }),
          result({
            document: {
              ...result().document,
              id: "doc-pull",
              kind: "pull_request",
              resource_id: "repo-1:8",
              title: "Review phase4 pull search",
              body: "Pull body carries phase4.",
              branch: "feature/phase4",
              metadata: {
                number: 8,
                state: "merged",
                labels: [],
                authorLogin: "mona",
                headRef: "feature/phase4",
                baseRef: "main",
              },
            },
            type: "pull_requests",
            href: "/mona/editorial-search/pull/8",
            title: "Review phase4 pull search",
            summary: "Pull body carries phase4.",
          }),
        ])}
      />,
    );

    expect(
      screen.getByRole("link", { name: /Investigate phase4 issue search/ }),
    ).toHaveAttribute("href", "/mona/editorial-search/issues/7");
    expect(screen.getByText("#7")).toBeVisible();
    expect(screen.getByText("closed")).toBeVisible();
    expect(screen.getByText("bug")).toBeVisible();
    expect(
      screen.getByRole("link", { name: /Review phase4 pull search/ }),
    ).toHaveAttribute("href", "/mona/editorial-search/pull/8");
    expect(screen.getByText("merged")).toBeVisible();
    expect(screen.getByText("feature/phase4 -> main")).toBeVisible();
  });

  it("keeps the discussions tab concrete with an explicit empty state", () => {
    render(
      <SearchResultsPage
        activeType="discussions"
        query="phase4"
        results={{ items: [], total: 0, page: 1, pageSize: 30 }}
      />,
    );

    expect(screen.getByRole("link", { name: "Discussions" })).toHaveAttribute(
      "href",
      "/search?q=phase4&type=discussions",
    );
    expect(
      screen.getByText("Discussion search is ready for indexing."),
    ).toBeVisible();
    expect(
      screen.getByText(/No discussions are indexed for "phase4" yet/),
    ).toBeVisible();
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
  });

  it("preserves search context through pagination and exposes accessible controls", () => {
    render(
      <SearchResultsPage
        activeType="code"
        query="router guards"
        results={{
          items: [result({ type: "code" })],
          total: 61,
          page: 2,
          pageSize: 30,
        }}
      />,
    );

    expect(
      screen.getByRole("navigation", { name: "Search result types" }),
    ).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Search results pages" }),
    ).toBeVisible();
    expect(
      screen.getByRole("searchbox", { name: "Search query" }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Search" })).toHaveAttribute(
      "type",
      "submit",
    );
    expect(screen.getByRole("link", { name: "Previous" })).toHaveAttribute(
      "href",
      "/search?q=router+guards&type=code",
    );
    expect(screen.getByRole("link", { name: "Next" })).toHaveAttribute(
      "href",
      "/search?q=router+guards&type=code&page=3",
    );
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of Array.from(document.querySelectorAll("button"))) {
      expect(
        button.textContent?.trim() || button.getAttribute("aria-label"),
      ).toBeTruthy();
    }
  });

  it("keeps every advertised search tab linked to a concrete result state", () => {
    render(
      <SearchResultsPage
        activeType="repositories"
        query="phase5"
        results={{ items: [], total: 0, page: 1, pageSize: 30 }}
      />,
    );

    for (const tab of [
      "Repositories",
      "Code",
      "Issues",
      "Pull requests",
      "Commits",
      "Users",
      "Organizations",
      "Discussions",
    ]) {
      const link = screen.getByRole("link", { name: tab });
      expect(link).toHaveAttribute("href");
      expect(link.getAttribute("href")).toContain("q=phase5");
      expect(link.getAttribute("href")).toContain("type=");
      expect(link.getAttribute("href")).not.toBe("#");
    }
  });

  it("builds the API search path with UI type names", () => {
    expect(
      globalSearchPath({
        query: "router guards",
        type: "repositories",
        page: 2,
        pageSize: 30,
      }),
    ).toBe("/api/search?q=router+guards&type=repositories&page=2&pageSize=30");
  });
});
