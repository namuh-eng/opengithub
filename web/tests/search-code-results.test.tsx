import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CodeSearchResultsPage } from "@/components/CodeSearchResultsPage";
import type { CodeSearchResponse, GlobalSearchResult } from "@/lib/api";

function codeResult(
  overrides: Partial<GlobalSearchResult> = {},
): GlobalSearchResult {
  return {
    document: {
      id: overrides.document?.id ?? "code-doc-1",
      repository_id: overrides.document?.repository_id ?? "repo-1",
      owner_user_id: overrides.document?.owner_user_id ?? "user-1",
      owner_organization_id: overrides.document?.owner_organization_id ?? null,
      kind: overrides.document?.kind ?? "code",
      resource_id:
        overrides.document?.resource_id ??
        "repo-1:main:crates/api/src/search.rs",
      title: overrides.document?.title ?? "crates/api/src/search.rs",
      body: overrides.document?.body ?? "async fn search_code_results() {}",
      path: overrides.document?.path ?? "crates/api/src/search.rs",
      language: overrides.document?.language ?? "Rust",
      branch: overrides.document?.branch ?? "main",
      visibility: overrides.document?.visibility ?? "public",
      metadata: overrides.document?.metadata ?? {},
      indexed_at: overrides.document?.indexed_at ?? "2026-05-01T00:00:00Z",
      created_at: overrides.document?.created_at ?? "2026-05-01T00:00:00Z",
      updated_at: overrides.document?.updated_at ?? "2026-05-01T00:00:00Z",
    },
    rank: overrides.rank ?? 1,
    type: overrides.type ?? "code",
    href:
      overrides.href ??
      "/mona/editorial-search/blob/main/crates/api/src/search.rs#L42",
    title: overrides.title ?? "crates/api/src/search.rs",
    summary: overrides.summary ?? null,
    owner_login: overrides.owner_login ?? "mona",
    repository_name: overrides.repository_name ?? "editorial-search",
    display_name: overrides.display_name ?? null,
    avatar_url: overrides.avatar_url ?? null,
    visibility: overrides.visibility ?? "public",
    updated_at: overrides.updated_at ?? "2026-05-01T00:00:00Z",
    snippet: overrides.snippet ?? {
      path: "crates/api/src/search.rs",
      branch: "main",
      line_number: 42,
      fragment: "async fn search_code_results() {}",
      language: "Rust",
      match_ranges: [{ start: 9, end: 28 }],
    },
    commit: overrides.commit ?? null,
  };
}

function codeResponse(
  overrides: Partial<CodeSearchResponse> = {},
): CodeSearchResponse {
  return {
    items: overrides.items ?? [codeResult()],
    total: overrides.total ?? 1,
    page: overrides.page ?? 1,
    pageSize: overrides.pageSize ?? 30,
    typeCounts: overrides.typeCounts ?? [
      { resultType: "code", label: "Code", count: 12 },
      { resultType: "issues", label: "Issues", count: 2 },
      { resultType: "pull_requests", label: "Pull requests", count: 1 },
    ],
    facets: overrides.facets ?? {
      languages: [
        { value: "Rust", label: "Rust", count: 10, selected: true },
        { value: "TypeScript", label: "TypeScript", count: 2, selected: false },
      ],
      paths: [
        { value: "crates/api", label: "crates/api", count: 8, selected: false },
      ],
    },
    activeChips: overrides.activeChips ?? [
      {
        qualifier: "language",
        value: "Rust",
        label: "language:Rust",
        removeQuery: "search_code_results",
      },
    ],
    queryDurationMs: overrides.queryDurationMs ?? 7,
    diagnostics: overrides.diagnostics ?? [],
  };
}

describe("CodeSearchResultsPage", () => {
  it("renders the dedicated two-pane code search workspace", () => {
    render(
      <CodeSearchResultsPage
        query="search_code_results language:Rust"
        results={codeResponse()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "Search indexed code" }),
    ).toBeVisible();
    expect(screen.getByText("1 code results · 7ms")).toBeVisible();
    expect(screen.getByText("Result types")).toBeVisible();
    expect(screen.getByRole("link", { name: /Code\s*12/ })).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust&type=code",
    );
    expect(screen.getByRole("link", { name: /Issues\s*2/ })).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust&type=issues",
    );
    expect(screen.getByText("Languages")).toBeVisible();
    expect(screen.getByText("Paths")).toBeVisible();
    expect(screen.getByText("Advanced")).toBeVisible();
  });

  it("links facets, removable chips, view controls, and file line anchors", () => {
    render(
      <CodeSearchResultsPage
        query="search_code_results language:Rust"
        results={codeResponse()}
      />,
    );

    expect(screen.getByRole("link", { name: /Rust\s*10/ })).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust&type=code",
    );
    expect(
      screen.getByRole("link", { name: /TypeScript\s*2/ }),
    ).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust+language%3ATypeScript&type=code",
    );
    expect(
      screen.getByRole("link", { name: /crates\/api\s*8/ }),
    ).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust+path%3Acrates%2Fapi&type=code",
    );
    expect(screen.getByRole("link", { name: /language:Rust/ })).toHaveAttribute(
      "href",
      "/search?q=search_code_results&type=code",
    );
    expect(screen.getByRole("link", { name: "Save" })).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust&type=code&saved=1",
    );
    expect(screen.getByRole("link", { name: "Compact" })).toHaveAttribute(
      "href",
      "/search?q=search_code_results+language%3ARust&type=code&view=compact",
    );
    expect(
      screen.getByRole("link", { name: /crates\/api\/src\/search.rs/ }),
    ).toHaveAttribute(
      "href",
      "/mona/editorial-search/blob/main/crates/api/src/search.rs#L42",
    );
    expect(screen.getByText("search_code_results")).toBeVisible();
  });

  it("renders inline API errors and keeps controls concrete", () => {
    render(
      <CodeSearchResultsPage
        query="fork:true"
        results={{
          error: {
            code: "validation_failed",
            message: "Unsupported qualifier: fork",
          },
          status: 422,
        }}
      />,
    );

    expect(screen.getByText("Code search unavailable")).toBeVisible();
    expect(screen.getByText("Unsupported qualifier: fork")).toBeVisible();
    expect(
      screen.getByRole("searchbox", { name: "Search query" }),
    ).toHaveAttribute("value", "fork:true");
    expect(
      document.querySelectorAll('a[href="#"], a:not([href])'),
    ).toHaveLength(0);
    for (const button of Array.from(document.querySelectorAll("button"))) {
      expect(
        button.textContent?.trim() || button.getAttribute("aria-label"),
      ).toBeTruthy();
    }
  });
});
