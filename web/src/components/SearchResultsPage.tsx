import Link from "next/link";
import type { ReactNode } from "react";
import { CodeSearchResultsPage } from "@/components/CodeSearchResultsPage";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  ApiErrorEnvelope,
  CodeSearchResponse,
  GlobalSearchResult,
  ListEnvelope,
} from "@/lib/api";
import {
  activeSearchType,
  SEARCH_TABS,
  searchTypeHref,
} from "@/lib/navigation";

type SearchResultsPageProps = {
  activeType: string;
  query: string;
  results:
    | ListEnvelope<GlobalSearchResult>
    | CodeSearchResponse
    | ApiErrorEnvelope
    | null;
};

const SEARCH_TYPE_LABELS = new Map<string, string>(
  SEARCH_TABS.map((tab) => [tab.value, tab.label]),
);

const SEARCH_TYPE_DESCRIPTIONS = new Map<string, string>(
  SEARCH_TABS.map((tab) => [tab.value, tab.description]),
);

function isErrorEnvelope(
  value: SearchResultsPageProps["results"],
): value is ApiErrorEnvelope {
  return Boolean(value && "error" in value);
}

function searchHref(query: string, type: string, page: number) {
  const params = new URLSearchParams();
  params.set("q", query);
  params.set("type", type);
  if (page > 1) {
    params.set("page", String(page));
  }
  return `/search?${params.toString()}`;
}

function visibilityLabel(visibility: string) {
  return visibility.charAt(0).toUpperCase() + visibility.slice(1);
}

function resultKicker(result: GlobalSearchResult) {
  if (result.type === "repositories") {
    return `${result.owner_login ?? "owner"} / ${result.repository_name ?? result.title}`;
  }
  if (result.type === "users") {
    return result.owner_login ?? result.document.resource_id;
  }
  if (result.type === "organizations") {
    return result.owner_login ?? result.document.resource_id;
  }
  return result.owner_login && result.repository_name
    ? `${result.owner_login} / ${result.repository_name}`
    : result.document.kind;
}

function ResultIcon({ result }: { result: GlobalSearchResult }) {
  const label =
    result.type === "repositories"
      ? "R"
      : result.type === "users"
        ? "U"
        : result.type === "organizations"
          ? "O"
          : result.type === "code"
            ? "{}"
            : result.type === "commits"
              ? "C"
              : result.type === "issues"
                ? "I"
                : result.type === "pull_requests"
                  ? "P"
                  : "S";

  return (
    <span
      aria-hidden="true"
      className="av sm shrink-0"
      style={{
        background: "var(--surface-2)",
        color: "var(--ink-2)",
        fontFamily: "var(--mono)",
      }}
    >
      {label}
    </span>
  );
}

function metadataString(
  metadata: Record<string, unknown>,
  key: string,
): string | null {
  const value = metadata[key];
  return typeof value === "string" && value.trim() ? value : null;
}

function metadataNumber(
  metadata: Record<string, unknown>,
  key: string,
): number | null {
  const value = metadata[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function metadataLabels(metadata: Record<string, unknown>) {
  const labels = metadata.labels;
  if (!Array.isArray(labels)) {
    return [];
  }

  return labels
    .map((label) => {
      if (!label || typeof label !== "object" || !("name" in label)) {
        return null;
      }
      const name = (label as { name?: unknown }).name;
      return typeof name === "string" && name.trim() ? name : null;
    })
    .filter((label): label is string => Boolean(label));
}

function stateChipClass(type: string, state: string | null) {
  if (state === "merged") {
    return "chip ok";
  }
  if (state === "closed") {
    return "chip err";
  }
  if (type === "pull_requests") {
    return "chip warn";
  }
  return "chip ok";
}

function HighlightedFragment({
  fragment,
  ranges,
}: {
  fragment: string;
  ranges: { start: number; end: number }[];
}) {
  if (ranges.length === 0) {
    return <>{fragment}</>;
  }

  const pieces: ReactNode[] = [];
  let cursor = 0;
  for (const range of ranges) {
    const start = Math.max(cursor, Math.min(fragment.length, range.start));
    const end = Math.max(start, Math.min(fragment.length, range.end));
    if (start > cursor) {
      pieces.push(fragment.slice(cursor, start));
    }
    if (end > start) {
      pieces.push(
        <mark
          key={`${start}-${end}`}
          style={{
            background: "var(--accent-soft)",
            borderRadius: "var(--radius)",
            color: "var(--ink-1)",
            padding: "0 2px",
          }}
        >
          {fragment.slice(start, end)}
        </mark>,
      );
    }
    cursor = end;
  }
  if (cursor < fragment.length) {
    pieces.push(fragment.slice(cursor));
  }
  return <>{pieces}</>;
}

function GenericSearchResultCard({ result }: { result: GlobalSearchResult }) {
  return (
    <Link className="list-row items-start gap-3 px-0" href={result.href}>
      <ResultIcon result={result} />
      <span className="min-w-0 flex-1">
        <span className="t-label block" style={{ color: "var(--ink-3)" }}>
          {resultKicker(result)}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {result.display_name ?? result.title}
        </span>
        {result.summary ? (
          <span className="t-sm mt-1 block" style={{ color: "var(--ink-3)" }}>
            {result.summary}
          </span>
        ) : null}
        <span className="mt-3 flex flex-wrap items-center gap-2">
          <span className="chip soft">
            {visibilityLabel(result.visibility)}
          </span>
          {result.document.language ? (
            <span className="chip soft">{result.document.language}</span>
          ) : null}
          {result.document.path ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {result.document.path}
            </span>
          ) : null}
        </span>
      </span>
    </Link>
  );
}

function CodeSearchResultCard({ result }: { result: GlobalSearchResult }) {
  const snippet = result.snippet;
  if (!snippet) {
    return <GenericSearchResultCard result={result} />;
  }

  return (
    <Link className="list-row items-start gap-3 px-0" href={result.href}>
      <ResultIcon result={result} />
      <span className="min-w-0 flex-1">
        <span className="t-label block" style={{ color: "var(--ink-3)" }}>
          {resultKicker(result)}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {snippet.path}
          {snippet.line_number ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              :{snippet.line_number}
            </span>
          ) : null}
        </span>
        <span
          className="t-mono-sm mt-2 block overflow-x-auto rounded-[var(--radius)] p-3"
          style={{
            background: "var(--surface-2)",
            color: "var(--ink-2)",
          }}
        >
          <HighlightedFragment
            fragment={snippet.fragment}
            ranges={snippet.match_ranges}
          />
        </span>
        <span className="mt-3 flex flex-wrap items-center gap-2">
          <span className="chip soft">{snippet.branch}</span>
          {snippet.language ? (
            <span className="chip soft">{snippet.language}</span>
          ) : null}
          <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {result.document.resource_id}
          </span>
        </span>
      </span>
    </Link>
  );
}

function CommitSearchResultCard({ result }: { result: GlobalSearchResult }) {
  const commit = result.commit;
  if (!commit) {
    return <GenericSearchResultCard result={result} />;
  }

  return (
    <Link className="list-row items-start gap-3 px-0" href={result.href}>
      <ResultIcon result={result} />
      <span className="min-w-0 flex-1">
        <span className="t-label block" style={{ color: "var(--ink-3)" }}>
          {resultKicker(result)}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {commit.message_title}
        </span>
        {commit.message_body ? (
          <span className="t-sm mt-1 block" style={{ color: "var(--ink-3)" }}>
            {commit.message_body}
          </span>
        ) : null}
        <span className="mt-3 flex flex-wrap items-center gap-2">
          <span className="chip soft">Commit</span>
          <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {commit.short_oid}
          </span>
          {commit.author_login ? (
            <span className="t-sm" style={{ color: "var(--ink-3)" }}>
              by {commit.author_login}
            </span>
          ) : null}
        </span>
      </span>
    </Link>
  );
}

function CollaborationSearchResultCard({
  result,
}: {
  result: GlobalSearchResult;
}) {
  const metadata = result.document.metadata;
  const number = metadataNumber(metadata, "number");
  const state = metadataString(metadata, "state") ?? "open";
  const labels = metadataLabels(metadata);
  const author = metadataString(metadata, "authorLogin");
  const headRef = metadataString(metadata, "headRef");
  const baseRef = metadataString(metadata, "baseRef");
  const typeLabel = result.type === "pull_requests" ? "Pull request" : "Issue";

  return (
    <Link className="list-row items-start gap-3 px-0" href={result.href}>
      <ResultIcon result={result} />
      <span className="min-w-0 flex-1">
        <span className="t-label block" style={{ color: "var(--ink-3)" }}>
          {resultKicker(result)}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {result.title}
        </span>
        {result.summary ? (
          <span className="t-sm mt-1 block" style={{ color: "var(--ink-3)" }}>
            {result.summary}
          </span>
        ) : null}
        <span className="mt-3 flex flex-wrap items-center gap-2">
          <span className={stateChipClass(result.type, state)}>{state}</span>
          {number ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              #{number}
            </span>
          ) : null}
          <span className="chip soft">{typeLabel}</span>
          {labels.map((label) => (
            <span className="chip soft" key={label}>
              {label}
            </span>
          ))}
          {headRef && baseRef ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {headRef} {"->"} {baseRef}
            </span>
          ) : null}
          {author ? (
            <span className="t-sm" style={{ color: "var(--ink-3)" }}>
              by {author}
            </span>
          ) : null}
        </span>
      </span>
    </Link>
  );
}

function SearchResultCard({ result }: { result: GlobalSearchResult }) {
  if (result.type === "code") {
    return <CodeSearchResultCard result={result} />;
  }
  if (result.type === "commits") {
    return <CommitSearchResultCard result={result} />;
  }
  if (result.type === "issues" || result.type === "pull_requests") {
    return <CollaborationSearchResultCard result={result} />;
  }

  return <GenericSearchResultCard result={result} />;
}

function EmptySearchState({ query }: { query: string }) {
  return (
    <div className="card p-8">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        No results
      </p>
      <h2 className="t-h2 mt-2">
        Nothing matched {query ? `"${query}"` : "yet"}.
      </h2>
      <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
        Try a repository name, owner login, organization slug, or a shorter
        phrase. Search supports focused syntax such as{" "}
        <span className="kbd">owner:namuh</span>,{" "}
        <span className="kbd">language:rust</span>, and{" "}
        <span className="kbd">path:src</span> as indexing expands.
      </p>
    </div>
  );
}

function DiscussionsEmptyState({ query }: { query: string }) {
  return (
    <div className="card p-8">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Discussions
      </p>
      <h2 className="t-h2 mt-2">Discussion search is ready for indexing.</h2>
      <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
        {query
          ? `No discussions are indexed for "${query}" yet.`
          : "Run a search once repository discussions are connected."}{" "}
        The tab stays available so saved search URLs and keyboard navigation do
        not fall through to a placeholder.
      </p>
    </div>
  );
}

function SearchErrorState({ error }: { error: ApiErrorEnvelope }) {
  return (
    <div className="card p-6">
      <p className="t-label" style={{ color: "var(--err)" }}>
        Search unavailable
      </p>
      <h2 className="t-h2 mt-2">{error.error.message}</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
        Refine the query and try again. Short searches need at least two visible
        characters.
      </p>
    </div>
  );
}

function Pagination({
  activeType,
  page,
  query,
  results,
}: {
  activeType: string;
  page: number;
  query: string;
  results: ListEnvelope<GlobalSearchResult>;
}) {
  const pageSize = results.pageSize || 30;
  const totalPages = Math.max(1, Math.ceil(results.total / pageSize));

  if (totalPages <= 1) {
    return null;
  }

  return (
    <nav
      aria-label="Search results pages"
      className="mt-6 flex items-center justify-between gap-3"
    >
      <Link
        aria-disabled={page <= 1}
        className={`btn sm ${page <= 1 ? "pointer-events-none opacity-50" : ""}`}
        href={searchHref(query, activeType, Math.max(1, page - 1))}
      >
        Previous
      </Link>
      <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
        Page {page} of {totalPages}
      </span>
      <Link
        aria-disabled={page >= totalPages}
        className={`btn sm ${page >= totalPages ? "pointer-events-none opacity-50" : ""}`}
        href={searchHref(query, activeType, Math.min(totalPages, page + 1))}
      >
        Next
      </Link>
    </nav>
  );
}

export function SearchResultsPage({
  activeType,
  query,
  results,
}: SearchResultsPageProps) {
  const normalizedType = activeSearchType(activeType);
  if (normalizedType === "code") {
    return <CodeSearchResultsPage query={query} results={results} />;
  }

  const activeTypeLabel =
    SEARCH_TYPE_LABELS.get(normalizedType) ?? "Repositories";
  const description = SEARCH_TYPE_DESCRIPTIONS.get(normalizedType);
  const hasQuery = query.trim().length > 0;
  const successfulResults =
    results && !isErrorEnvelope(results) ? results : null;

  return (
    <section className="mx-auto max-w-[1240px] px-6 py-8">
      <div className="mb-6 grid gap-4 md:grid-cols-[minmax(0,1fr)_320px] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Search
          </p>
          <h1 className="t-h1 mt-1">Search opengithub</h1>
        </div>
        <search>
          <form action="/search" className="flex gap-2">
            <input
              aria-label="Search query"
              className="input min-w-0 flex-1"
              defaultValue={query}
              name="q"
              placeholder="Search repositories, people, organizations..."
              type="search"
            />
            <input name="type" type="hidden" value={normalizedType} />
            <button className="btn primary" type="submit">
              Search
            </button>
          </form>
        </search>
      </div>

      <QueryTabNavigation
        activeValue={normalizedType}
        ariaLabel="Search result types"
        hrefForTab={(value) => searchTypeHref(value, query)}
        tabs={SEARCH_TABS}
      />

      <div className="mt-6 grid gap-6 md:grid-cols-[240px_minmax(0,1fr)]">
        <aside className="card self-start p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Refine
          </p>
          <div className="mt-4 space-y-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-4)" }}>
                Type
              </p>
              <p className="t-sm mt-1 font-semibold">{activeTypeLabel}</p>
              {description ? <p className="t-xs mt-1">{description}</p> : null}
            </div>
            <div>
              <p className="t-label" style={{ color: "var(--ink-4)" }}>
                Query
              </p>
              <p className="t-mono-sm mt-1 break-words">
                {query || "No query entered"}
              </p>
            </div>
            <div>
              <p className="t-label" style={{ color: "var(--ink-4)" }}>
                Syntax
              </p>
              <div className="mt-2 flex flex-wrap gap-2">
                <span className="kbd">owner:</span>
                <span className="kbd">language:</span>
                <span className="kbd">path:</span>
              </div>
            </div>
          </div>
        </aside>

        <div>
          <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
            <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {successfulResults
                ? `${successfulResults.total} ${activeTypeLabel.toLowerCase()} results`
                : hasQuery
                  ? `${activeTypeLabel} results`
                  : "Start with a query"}
            </p>
            {successfulResults ? (
              <span className="chip soft">Page {successfulResults.page}</span>
            ) : null}
          </div>

          {!hasQuery ? (
            normalizedType === "discussions" ? (
              <DiscussionsEmptyState query="" />
            ) : (
              <EmptySearchState query="" />
            )
          ) : isErrorEnvelope(results) ? (
            <SearchErrorState error={results} />
          ) : successfulResults && successfulResults.items.length > 0 ? (
            <div className="card p-4">
              {successfulResults.items.map((result) => (
                <SearchResultCard key={result.document.id} result={result} />
              ))}
            </div>
          ) : normalizedType === "discussions" ? (
            <DiscussionsEmptyState query={query} />
          ) : (
            <EmptySearchState query={query} />
          )}

          {successfulResults ? (
            <Pagination
              activeType={normalizedType}
              page={successfulResults.page}
              query={query}
              results={successfulResults}
            />
          ) : null}
        </div>
      </div>
    </section>
  );
}
