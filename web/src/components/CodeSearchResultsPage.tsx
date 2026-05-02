"use client";

import Link from "next/link";
import type { FormEvent, ReactNode } from "react";
import { useMemo, useState } from "react";
import type {
  ApiErrorEnvelope,
  CodeSearchChip,
  CodeSearchFacetValue,
  CodeSearchResponse,
  CodeSearchTypeCount,
  GlobalSearchResult,
  ListEnvelope,
} from "@/lib/api";
import {
  addCodeSearchQualifierHref,
  codeSearchHref,
  codeSearchViewHref,
  removeCodeSearchQualifier,
  removeCodeSearchQualifierHref,
  searchTypeHref,
  toggleCodeSearchQualifierHref,
} from "@/lib/navigation";

type CodeSearchResultsPageProps = {
  query: string;
  view?: string;
  saved?: boolean;
  results:
    | CodeSearchResponse
    | ListEnvelope<GlobalSearchResult>
    | ApiErrorEnvelope
    | null;
};

function isErrorEnvelope(
  value: CodeSearchResultsPageProps["results"],
): value is ApiErrorEnvelope {
  return Boolean(value && "error" in value);
}

function isCodeSearchResponse(
  value: CodeSearchResultsPageProps["results"],
): value is CodeSearchResponse {
  return Boolean(value && !("error" in value) && "typeCounts" in value);
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

function ResultTypeCount({
  count,
  query,
}: {
  count: CodeSearchTypeCount;
  query: string;
}) {
  const active = count.resultType === "code";
  return (
    <Link
      aria-current={active ? "page" : undefined}
      className={`list-row items-center justify-between gap-3 px-0 py-2 ${active ? "font-semibold" : ""}`}
      href={searchTypeHref(String(count.resultType), query)}
    >
      <span>{count.label}</span>
      <span className={active ? "chip active" : "chip soft"}>
        {count.count}
      </span>
    </Link>
  );
}

function FacetLink({
  facet,
  query,
  qualifier,
}: {
  facet: CodeSearchFacetValue;
  query: string;
  qualifier: "language" | "path";
}) {
  const href = facet.selected
    ? toggleCodeSearchQualifierHref(query, qualifier, facet.value, true)
    : addCodeSearchQualifierHref(query, qualifier, facet.value);
  return (
    <Link
      aria-current={facet.selected ? "true" : undefined}
      className="list-row items-center justify-between gap-3 px-0 py-2"
      href={href}
    >
      <span className="min-w-0 truncate">{facet.label}</span>
      <span className={facet.selected ? "chip active" : "chip soft"}>
        {facet.count}
      </span>
    </Link>
  );
}

function ActiveChip({ chip }: { chip: CodeSearchChip }) {
  return (
    <Link
      className="chip soft"
      href={removeCodeSearchQualifierHref(chip.removeQuery)}
      title={`Remove ${chip.label}`}
    >
      {chip.label}
      <span aria-hidden="true"> x</span>
    </Link>
  );
}

function lineHref(
  blobHref: string | null,
  fallbackHref: string,
  line: number | null,
) {
  const baseHref = blobHref ?? fallbackHref.split("#")[0] ?? fallbackHref;
  return line && line > 0 ? `${baseHref}#L${line}` : baseHref;
}

function firstQualifierValue(query: string, qualifier: string) {
  const match = query.match(
    new RegExp(`(?:^|\\s)${qualifier}:(?:"([^"]*)"|(\\S+))`, "i"),
  );
  return match?.[1] ?? match?.[2] ?? "";
}

function hasQualifierValue(query: string, qualifier: string, value: string) {
  const normalizedValue = value.toLocaleLowerCase();
  return (
    query
      .match(new RegExp(`(?:^|\\s)${qualifier}:(?:"[^"]*"|\\S+)`, "gi"))
      ?.some((token) => {
        const rawValue = token.trim().slice(token.indexOf(":") + 1);
        return (
          rawValue.replace(/^"|"$/g, "").toLocaleLowerCase() === normalizedValue
        );
      }) ?? false
  );
}

function AdvancedFilters({ query }: { query: string }) {
  const [owner, setOwner] = useState(firstQualifierValue(query, "owner"));
  const [symbol, setSymbol] = useState(firstQualifierValue(query, "symbol"));
  const [excludeArchived, setExcludeArchived] = useState(
    hasQualifierValue(query, "archived", "false") ||
      hasQualifierValue(query, "is", "unarchived"),
  );

  function submitFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    let nextQuery = removeCodeSearchQualifier(query, "owner");
    nextQuery = removeCodeSearchQualifier(nextQuery, "user");
    nextQuery = removeCodeSearchQualifier(nextQuery, "org");
    nextQuery = removeCodeSearchQualifier(nextQuery, "symbol");
    nextQuery = removeCodeSearchQualifier(nextQuery, "archived");
    nextQuery = removeCodeSearchQualifier(nextQuery, "is", "unarchived");
    const parts = [nextQuery];
    if (owner.trim()) {
      parts.push(`owner:${owner.trim()}`);
    }
    if (symbol.trim()) {
      parts.push(`symbol:${symbol.trim()}`);
    }
    if (excludeArchived) {
      parts.push("archived:false");
    }
    window.location.assign(codeSearchHref(parts.join(" ")));
  }

  return (
    <form className="mt-4 space-y-3" onSubmit={submitFilters}>
      <label className="block">
        <span className="t-label" style={{ color: "var(--ink-4)" }}>
          Owner
        </span>
        <input
          className="input mt-1 w-full"
          onChange={(event) => setOwner(event.target.value)}
          placeholder="namuh"
          value={owner}
        />
      </label>
      <label className="block">
        <span className="t-label" style={{ color: "var(--ink-4)" }}>
          Symbol
        </span>
        <input
          className="input mt-1 w-full"
          onChange={(event) => setSymbol(event.target.value)}
          placeholder="router"
          value={symbol}
        />
      </label>
      <label className="flex items-center gap-2 t-sm">
        <input
          checked={excludeArchived}
          onChange={(event) => setExcludeArchived(event.target.checked)}
          type="checkbox"
        />
        Exclude archived repositories
      </label>
      <button className="btn sm" type="submit">
        Apply filters
      </button>
    </form>
  );
}

function SaveSearchDialog({ query, open }: { query: string; open: boolean }) {
  const [expanded, setExpanded] = useState(open);
  const [name, setName] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function saveSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedName = name.trim();
    if (!normalizedName) {
      setError("Name is required.");
      setFeedback(null);
      return;
    }
    setSaving(true);
    setError(null);
    setFeedback(null);
    try {
      const response = await fetch("/search/saved-searches", {
        body: JSON.stringify({
          name: normalizedName,
          query,
          scope: "code",
        }),
        headers: { "content-type": "application/json" },
        method: "POST",
      });
      const body = await response.json().catch(() => null);
      if (!response.ok || (body && "error" in body)) {
        setError(body?.error?.message ?? "Saved search could not be created.");
        return;
      }
      setFeedback(`Saved "${body.name ?? normalizedName}".`);
      setName("");
    } catch {
      setError("Saved search could not be created.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="card mb-4 p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Saved search
          </p>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            Store this code query in your search dashboard.
          </p>
        </div>
        <button
          aria-expanded={expanded}
          className="btn sm"
          onClick={() => setExpanded((value) => !value)}
          type="button"
        >
          {expanded ? "Close" : "Save search"}
        </button>
      </div>
      {expanded ? (
        <form className="mt-4 flex flex-wrap gap-2" onSubmit={saveSearch}>
          <input
            aria-label="Saved search name"
            className="input min-w-[220px] flex-1"
            onChange={(event) => setName(event.target.value)}
            placeholder="Router references"
            value={name}
          />
          <button className="btn primary" disabled={saving} type="submit">
            {saving ? "Saving..." : "Create saved search"}
          </button>
        </form>
      ) : null}
      {feedback ? (
        <p className="t-sm mt-3" role="status" style={{ color: "var(--ok)" }}>
          {feedback}
        </p>
      ) : null}
      {error ? (
        <p className="t-sm mt-3" role="alert" style={{ color: "var(--err)" }}>
          {error}
        </p>
      ) : null}
    </div>
  );
}

function CodeResultCard({ result }: { result: GlobalSearchResult }) {
  const [collapsed, setCollapsed] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const snippet = result.snippet;
  const snippets = useMemo(() => {
    if (result.snippets.length > 0) {
      return result.snippets;
    }
    return snippet ? [snippet] : [];
  }, [result.snippets, snippet]);
  const repoLabel =
    result.owner_login && result.repository_name
      ? `${result.owner_login}/${result.repository_name}`
      : "repository";
  const path = snippet?.path ?? result.document.path ?? result.title;
  const language = snippet?.language ?? result.document.language;
  const visibleSnippets = expanded ? snippets : snippets.slice(0, 3);
  const hiddenCount = Math.max(
    result.hidden_match_count,
    snippets.length - visibleSnippets.length,
  );
  const matchCount = Math.max(result.match_count, snippets.length);

  return (
    <article className="card p-0">
      <div className="flex flex-wrap items-start justify-between gap-3 border-b border-[color:var(--line-soft)] p-4">
        <div className="min-w-0">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            {repoLabel}
          </p>
          <Link
            className="mt-1 block truncate font-semibold"
            href={lineHref(
              result.blob_href,
              result.href,
              snippet?.line_number ?? null,
            )}
          >
            {path}
            {snippet?.line_number ? (
              <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                :{snippet.line_number}
              </span>
            ) : null}
          </Link>
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-2">
          <span className="chip soft">
            {matchCount} {matchCount === 1 ? "match" : "matches"}
          </span>
          {language ? <span className="chip soft">{language}</span> : null}
          {result.commit ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {result.commit.short_oid}
            </span>
          ) : null}
          <span className="chip soft">{result.visibility}</span>
          <button
            aria-expanded={!collapsed}
            className="btn sm"
            onClick={() => setCollapsed((value) => !value)}
            type="button"
          >
            {collapsed ? "Expand" : "Collapse"}
          </button>
        </div>
      </div>
      {!collapsed ? (
        <div className="overflow-x-auto p-4">
          {visibleSnippets.length > 0 ? (
            <div className="min-w-[520px] overflow-hidden rounded-[var(--radius)] border border-[color:var(--line-soft)]">
              {visibleSnippets.map((match, index) => {
                const href = lineHref(
                  result.blob_href,
                  result.href,
                  match.line_number,
                );
                return (
                  <div
                    className="grid grid-cols-[72px_minmax(0,1fr)] border-b border-[color:var(--line-soft)] last:border-b-0"
                    key={`${match.line_number ?? index}:${match.fragment}`}
                  >
                    <Link
                      className="t-mono-sm px-3 py-2 text-right"
                      href={href}
                      style={{ color: "var(--ink-4)" }}
                    >
                      {match.line_number ?? index + 1}
                    </Link>
                    <pre
                      className="t-mono-sm m-0 px-3 py-2"
                      style={{
                        color: "var(--ink-2)",
                        whiteSpace: "pre-wrap",
                      }}
                    >
                      <HighlightedFragment
                        fragment={match.fragment}
                        ranges={match.match_ranges}
                      />
                    </pre>
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              Indexed file match is available. Open the file to inspect the
              current default-branch content.
            </p>
          )}
          {!expanded && hiddenCount > 0 ? (
            <button
              className="btn sm mt-3"
              onClick={() => setExpanded(true)}
              type="button"
            >
              Show {hiddenCount} more {hiddenCount === 1 ? "match" : "matches"}
            </button>
          ) : null}
          {expanded && snippets.length > 3 ? (
            <button
              className="btn sm mt-3"
              onClick={() => setExpanded(false)}
              type="button"
            >
              Show fewer matches
            </button>
          ) : null}
        </div>
      ) : (
        <div className="p-4">
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            Snippets hidden. Expand this file to review matching lines without
            changing the current search URL.
          </p>
        </div>
      )}
    </article>
  );
}

function CodeSearchEmpty({ query }: { query: string }) {
  return (
    <div className="card p-8">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        No code results
      </p>
      <h2 className="t-h2 mt-2">
        Nothing in indexed files matched {query ? `"${query}"` : "yet"}.
      </h2>
      <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
        Try a shorter symbol, remove a scope chip, or search with{" "}
        <span className="kbd">language:</span>,{" "}
        <span className="kbd">path:</span>, and{" "}
        <span className="kbd">repo:</span> qualifiers.
      </p>
    </div>
  );
}

function CodeSearchError({ error }: { error: ApiErrorEnvelope }) {
  return (
    <div className="card p-6">
      <p className="t-label" style={{ color: "var(--err)" }}>
        Code search unavailable
      </p>
      <h2 className="t-h2 mt-2">{error.error.message}</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
        Remove unsupported qualifiers or shorten the query. Your typed query is
        preserved above.
      </p>
    </div>
  );
}

function Pagination({
  query,
  results,
}: {
  query: string;
  results: CodeSearchResponse | ListEnvelope<GlobalSearchResult>;
}) {
  const totalPages = Math.max(
    1,
    Math.ceil(results.total / (results.pageSize || 30)),
  );
  if (totalPages <= 1) {
    return null;
  }

  return (
    <nav
      aria-label="Search results pages"
      className="mt-6 flex items-center justify-between gap-3"
    >
      <Link
        aria-disabled={results.page <= 1}
        className={`btn sm ${results.page <= 1 ? "pointer-events-none opacity-50" : ""}`}
        href={codeSearchHref(query, {
          page: results.page > 2 ? String(results.page - 1) : null,
        })}
      >
        Previous
      </Link>
      <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
        Page {results.page} of {totalPages}
      </span>
      <Link
        aria-disabled={results.page >= totalPages}
        className={`btn sm ${results.page >= totalPages ? "pointer-events-none opacity-50" : ""}`}
        href={codeSearchHref(query, { page: String(results.page + 1) })}
      >
        Next
      </Link>
    </nav>
  );
}

export function CodeSearchResultsPage({
  query,
  view = "comfortable",
  saved = false,
  results,
}: CodeSearchResultsPageProps) {
  const hasQuery = query.trim().length > 0;
  const successfulResults =
    results && !isErrorEnvelope(results) ? results : null;
  const codeResults = isCodeSearchResponse(results) ? results : null;

  return (
    <section className="mx-auto max-w-[1240px] px-6 py-8">
      <div className="mb-6 grid gap-4 md:grid-cols-[minmax(0,1fr)_360px] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Code search
          </p>
          <h1 className="t-h1 mt-1">Search indexed code</h1>
        </div>
        <search>
          <form action="/search" className="flex gap-2">
            <input
              aria-label="Search query"
              className="input min-w-0 flex-1"
              defaultValue={query}
              name="q"
              placeholder="Search code, symbols, and paths..."
              type="search"
            />
            <input name="type" type="hidden" value="code" />
            <button className="btn primary" type="submit">
              Search
            </button>
          </form>
        </search>
      </div>

      <div className="grid gap-6 lg:grid-cols-[276px_minmax(0,1fr)]">
        <aside className="self-start lg:sticky lg:top-[calc(var(--header-h)+24px)]">
          <div className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Result types
            </p>
            <nav aria-label="Search result types" className="mt-3">
              {(codeResults?.typeCounts ?? []).map((count) => (
                <ResultTypeCount
                  count={count}
                  key={count.resultType}
                  query={query}
                />
              ))}
              {!successfulResults ? (
                <Link
                  aria-current="page"
                  className="list-row items-center justify-between gap-3 px-0 py-2 font-semibold"
                  href={searchTypeHref("code", query)}
                >
                  <span>Code</span>
                  <span className="chip active">0</span>
                </Link>
              ) : null}
            </nav>
          </div>

          <div className="card mt-4 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Languages
            </p>
            <div className="mt-3">
              {(codeResults?.facets.languages ?? []).map((facet) => (
                <FacetLink
                  facet={facet}
                  key={facet.value}
                  query={query}
                  qualifier="language"
                />
              ))}
              {codeResults?.facets.languages.length === 0 ? (
                <p className="t-xs">No language facets for this query.</p>
              ) : null}
            </div>
          </div>

          <div className="card mt-4 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Paths
            </p>
            <div className="mt-3">
              {(codeResults?.facets.paths ?? []).map((facet) => (
                <FacetLink
                  facet={facet}
                  key={facet.value}
                  query={query}
                  qualifier="path"
                />
              ))}
              {codeResults?.facets.paths.length === 0 ? (
                <p className="t-xs">No path facets for this query.</p>
              ) : null}
            </div>
          </div>

          <details className="card mt-4 p-4">
            <summary className="cursor-pointer t-label text-[color:var(--ink-3)]">
              Advanced
            </summary>
            <AdvancedFilters query={query} />
          </details>
        </aside>

        <div className="min-w-0">
          <div className="card mb-4 p-4">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {successfulResults
                    ? `${successfulResults.total} code results${
                        codeResults ? ` · ${codeResults.queryDurationMs}ms` : ""
                      }`
                    : hasQuery
                      ? "Code results"
                      : "Start with a code query"}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  {codeResults?.activeChips.map((chip) => (
                    <ActiveChip
                      chip={chip}
                      key={`${chip.qualifier}:${chip.value}`}
                    />
                  ))}
                  {query ? <span className="kbd">{query}</span> : null}
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                <Link
                  className="btn sm"
                  href={codeSearchHref(query, { saved: "1" })}
                >
                  Save
                </Link>
                <Link
                  aria-current={view === "comfortable" ? "true" : undefined}
                  className={`btn sm ${view === "comfortable" ? "primary" : ""}`}
                  href={codeSearchViewHref(query, "comfortable")}
                >
                  Comfortable
                </Link>
                <Link
                  aria-current={view === "compact" ? "true" : undefined}
                  className={`btn sm ${view === "compact" ? "primary" : ""}`}
                  href={codeSearchViewHref(query, "compact")}
                >
                  Compact
                </Link>
              </div>
            </div>
          </div>

          {saved ? <SaveSearchDialog open={saved} query={query} /> : null}

          {isErrorEnvelope(results) ? (
            <CodeSearchError error={results} />
          ) : null}

          {!hasQuery ? <CodeSearchEmpty query="" /> : null}

          {codeResults?.diagnostics.length ? (
            <div className="card mb-4 p-4">
              <p className="t-label" style={{ color: "var(--warn)" }}>
                Query notes
              </p>
              <ul className="t-sm mt-2 space-y-1">
                {codeResults.diagnostics.map((diagnostic) => (
                  <li key={`${diagnostic.code}:${diagnostic.qualifier ?? ""}`}>
                    {diagnostic.message}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          {successfulResults && successfulResults.items.length > 0 ? (
            <div className={view === "compact" ? "space-y-2" : "space-y-3"}>
              {successfulResults.items.map((result) => (
                <CodeResultCard key={result.document.id} result={result} />
              ))}
            </div>
          ) : successfulResults && hasQuery ? (
            <CodeSearchEmpty query={query} />
          ) : null}

          {successfulResults ? (
            <Pagination query={query} results={successfulResults} />
          ) : null}
        </div>
      </div>
    </section>
  );
}
