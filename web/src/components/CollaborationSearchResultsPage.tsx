"use client";

import Link from "next/link";
import type { FormEvent } from "react";
import { useState } from "react";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  ApiErrorEnvelope,
  CodeSearchFacetValue,
  CollaborationSearchResponse,
  GlobalSearchResult,
} from "@/lib/api";
import {
  addSearchQualifier,
  SEARCH_TABS,
  searchHref,
  searchTypeHref,
  toggleSearchQualifier,
} from "@/lib/navigation";

type Props = {
  activeType: "issues" | "pull_requests" | string;
  query: string;
  results: CollaborationSearchResponse | ApiErrorEnvelope | null;
  saved?: boolean;
  view?: "comfortable" | "compact";
};

function isErrorEnvelope(value: Props["results"]): value is ApiErrorEnvelope {
  return Boolean(value && "error" in value);
}

function typeLabel(activeType: string) {
  return activeType === "pull_requests" ? "Pull requests" : "Issues";
}

function metadataString(metadata: Record<string, unknown>, key: string) {
  const value = metadata[key];
  return typeof value === "string" && value.trim() ? value : null;
}

function metadataNumber(metadata: Record<string, unknown>, key: string) {
  const value = metadata[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function metadataPeople(metadata: Record<string, unknown>, key: string) {
  const value = metadata[key];
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item) => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as { login?: unknown; name?: unknown };
      const login = record.login ?? record.name;
      return typeof login === "string" && login.trim() ? login : null;
    })
    .filter((login): login is string => Boolean(login));
}

function metadataLabels(metadata: Record<string, unknown>) {
  const labels = metadata.labels;
  if (!Array.isArray(labels)) {
    return [];
  }
  return labels
    .map((item) => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const name = (item as { name?: unknown }).name;
      return typeof name === "string" && name.trim() ? name : null;
    })
    .filter((name): name is string => Boolean(name));
}

function milestoneTitle(metadata: Record<string, unknown>) {
  const milestone = metadata.milestone;
  if (typeof milestone === "string") {
    return milestone;
  }
  if (milestone && typeof milestone === "object") {
    const title = (milestone as { title?: unknown }).title;
    return typeof title === "string" ? title : null;
  }
  return null;
}

function stateChipClass(activeType: string, state: string | null) {
  if (state === "merged") {
    return "chip ok";
  }
  if (state === "closed") {
    return "chip err";
  }
  return activeType === "pull_requests" ? "chip warn" : "chip ok";
}

function facetHref(
  activeType: string,
  query: string,
  qualifier: string,
  value: string,
  sort?: string,
  view?: string,
) {
  return searchHref(
    toggleSearchQualifier(query, qualifier, value),
    activeType,
    {
      sort,
      view,
    },
  );
}

function pageHref(
  activeType: string,
  query: string,
  page: number,
  sort?: string,
  view?: string,
) {
  return searchHref(query, activeType, {
    ...(page > 1 ? { page: String(page) } : {}),
    ...(sort ? { sort } : {}),
    ...(view ? { view } : {}),
  });
}

function FacetGroup({
  activeType,
  facets,
  query,
  qualifier,
  sort,
  title,
  view,
}: {
  activeType: string;
  facets: CodeSearchFacetValue[];
  query: string;
  qualifier: string;
  sort?: string;
  title: string;
  view?: string;
}) {
  if (facets.length === 0) {
    return null;
  }

  return (
    <div>
      <p className="t-label" style={{ color: "var(--ink-4)" }}>
        {title}
      </p>
      <div className="mt-2 space-y-1">
        {facets.map((facet) => (
          <Link
            className={`flex items-center justify-between gap-3 rounded-[var(--radius)] px-2 py-1 t-sm ${facet.selected ? "chip active" : ""}`}
            href={facetHref(
              activeType,
              query,
              qualifier,
              facet.value,
              sort,
              view,
            )}
            key={`${qualifier}-${facet.value}`}
          >
            <span className="truncate">{facet.label}</span>
            <span className="t-num" style={{ color: "var(--ink-3)" }}>
              {facet.count}
            </span>
          </Link>
        ))}
      </div>
    </div>
  );
}

function AdvancedQualifierLinks({
  activeType,
  query,
  sort,
  view,
}: {
  activeType: string;
  query: string;
  sort?: string;
  view?: string;
}) {
  const controls = [
    ["author", "@author"],
    ["commenter", "@commenter"],
    ["involves", "@involved"],
    ["mentions", "@mentioned"],
    ["comments", ">10"],
    ["interactions", ">10"],
    ...(activeType === "issues"
      ? ([
          ["linked", "pr"],
          ["closed", "completed"],
        ] satisfies [string, string][])
      : ([] satisfies [string, string][])),
  ];

  return (
    <div>
      <p className="t-label" style={{ color: "var(--ink-4)" }}>
        Advanced search
      </p>
      <div className="mt-2 flex flex-wrap gap-2">
        {controls.map(([qualifier, value]) => (
          <Link
            className="chip soft"
            href={searchHref(
              addSearchQualifier(query, qualifier, value),
              activeType,
              { sort, view },
            )}
            key={`${qualifier}-${value}`}
          >
            {qualifier}:{value}
          </Link>
        ))}
      </div>
    </div>
  );
}

function ResultRow({
  activeType,
  result,
  view,
}: {
  activeType: string;
  result: GlobalSearchResult | import("@/lib/api").CollaborationSearchResult;
  view: "comfortable" | "compact";
}) {
  const isRich = "repository" in result;
  const metadata = isRich ? {} : result.document.metadata;
  const state = isRich
    ? result.state
    : (metadataString(metadata, "state") ?? "open");
  const number = isRich ? result.number : metadataNumber(metadata, "number");
  const labels = isRich
    ? result.labels.map((label) => label.name)
    : metadataLabels(metadata);
  const assignees = isRich
    ? result.assignees.map((user) => user.login)
    : metadataPeople(metadata, "assignees");
  const reviewers = isRich ? [] : metadataPeople(metadata, "reviewers");
  const author = isRich
    ? result.author?.login
    : metadataString(metadata, "authorLogin");
  const headRef = isRich ? result.headRef : metadataString(metadata, "headRef");
  const baseRef = isRich ? result.baseRef : metadataString(metadata, "baseRef");
  const comments = isRich
    ? result.commentCount
    : metadataNumber(metadata, "commentCount");
  const interactions = isRich
    ? result.interactionCount
    : metadataNumber(metadata, "interactionCount");
  const milestone = isRich ? result.milestone?.title : milestoneTitle(metadata);
  const ownerLogin = isRich
    ? result.repository.ownerLogin
    : (result.owner_login ?? "owner");
  const repositoryName = isRich
    ? result.repository.name
    : (result.repository_name ?? "repository");
  const summary = isRich ? result.snippets[0]?.fragment : result.summary;

  return (
    <Link
      className={`list-row items-start gap-3 px-0 ${view === "compact" ? "py-2" : ""}`}
      href={result.href}
    >
      <span
        aria-hidden="true"
        className="av sm shrink-0"
        style={{
          background: "var(--surface-2)",
          color: "var(--ink-2)",
          fontFamily: "var(--mono)",
        }}
      >
        {activeType === "pull_requests" ? "P" : "I"}
      </span>
      <span className="min-w-0 flex-1">
        <span className="t-label block" style={{ color: "var(--ink-3)" }}>
          {ownerLogin} / {repositoryName}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {result.title}
        </span>
        {summary && view !== "compact" ? (
          <span
            className="t-sm mt-1 line-clamp-2 block"
            style={{ color: "var(--ink-3)" }}
          >
            {summary}
          </span>
        ) : null}
        <span className="mt-3 flex flex-wrap items-center gap-2">
          <span className={stateChipClass(activeType, state)}>{state}</span>
          {number ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              #{number}
            </span>
          ) : null}
          {author ? (
            <span className="t-sm" style={{ color: "var(--ink-3)" }}>
              by {author}
            </span>
          ) : null}
          {labels.map((label) => (
            <span className="chip soft" key={label}>
              {label}
            </span>
          ))}
          {milestone ? (
            <span className="chip soft">Milestone: {milestone}</span>
          ) : null}
          {assignees.map((login) => (
            <span className="chip soft" key={`assignee-${login}`}>
              @{login}
            </span>
          ))}
          {reviewers.map((login) => (
            <span className="chip soft" key={`reviewer-${login}`}>
              review: @{login}
            </span>
          ))}
          {headRef && baseRef ? (
            <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
              {headRef} {"->"} {baseRef}
            </span>
          ) : null}
          {comments !== null ? (
            <span className="chip soft">{comments} comments</span>
          ) : null}
          {interactions !== null ? (
            <span className="chip soft">{interactions} interactions</span>
          ) : null}
        </span>
      </span>
    </Link>
  );
}

function SaveSearchDialog({
  activeType,
  open,
  query,
}: {
  activeType: string;
  open: boolean;
  query: string;
}) {
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
          scope: activeType,
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
            Store this issue and pull request query in your search dashboard.
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
            placeholder="Open regressions"
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

function ErrorState({ error }: { error: ApiErrorEnvelope }) {
  return (
    <div className="card p-6">
      <p className="t-label" style={{ color: "var(--err)" }}>
        Search unavailable
      </p>
      <h2 className="t-h2 mt-2">{error.error.message}</h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
        Refine the issue or pull request query and try again.
      </p>
    </div>
  );
}

export function CollaborationSearchResultsPage({
  activeType,
  query,
  results,
  saved = false,
  view = "comfortable",
}: Props) {
  const label = typeLabel(activeType);
  const successful = results && !isErrorEnvelope(results) ? results : null;
  const facets = successful?.facets ?? {
    assignees: [],
    labels: [],
    milestones: [],
    owners: [],
    reviewers: [],
    states: [],
  };
  const activeChips = successful?.activeChips ?? [];
  const activeSort =
    successful?.sort?.selected ?? successful?.activeSort ?? "best_match";
  const sortOptions = successful?.sort?.options ??
    successful?.sortOptions ?? [
      { label: "Best match", selected: true, value: "best_match" },
    ];
  const activeSortLabel =
    successful?.sort?.label ??
    sortOptions.find((option) => option.selected)?.label ??
    "Best match";
  const queryDurationMs = successful?.queryDurationMs ?? 0;
  const pageSize = successful?.pageSize ?? 30;
  const totalPages = Math.max(
    1,
    Math.ceil((successful?.total ?? 0) / pageSize),
  );

  return (
    <section className="mx-auto max-w-[1240px] px-6 py-8">
      <div className="mb-6 grid gap-4 md:grid-cols-[minmax(0,1fr)_320px] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Search
          </p>
          <h1 className="t-h1 mt-1">{label} search</h1>
        </div>
        <search>
          <form action="/search" className="flex gap-2">
            <input
              aria-label="Search query"
              className="input min-w-0 flex-1"
              defaultValue={query}
              name="q"
              placeholder={`Search ${label.toLowerCase()}...`}
              type="search"
            />
            <input name="type" type="hidden" value={activeType} />
            <input name="sort" type="hidden" value={activeSort} />
            <input name="view" type="hidden" value={view} />
            <button className="btn primary" type="submit">
              Search
            </button>
          </form>
        </search>
      </div>

      <QueryTabNavigation
        activeValue={activeType}
        ariaLabel="Search result types"
        hrefForTab={(value) => searchTypeHref(value, query)}
        tabs={SEARCH_TABS}
      />

      <div className="mt-6 grid gap-6 md:grid-cols-[260px_minmax(0,1fr)]">
        <aside className="card self-start p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Advanced facets
          </p>
          <div className="mt-4 space-y-5">
            <FacetGroup
              activeType={activeType}
              facets={facets.states}
              query={query}
              qualifier="state"
              sort={activeSort}
              title="State"
              view={view}
            />
            <FacetGroup
              activeType={activeType}
              facets={facets.owners ?? []}
              query={query}
              qualifier="owner"
              sort={activeSort}
              title="Owner"
              view={view}
            />
            <FacetGroup
              activeType={activeType}
              facets={facets.labels}
              query={query}
              qualifier="label"
              sort={activeSort}
              title="Labels"
              view={view}
            />
            <FacetGroup
              activeType={activeType}
              facets={facets.assignees}
              query={query}
              qualifier="assignee"
              sort={activeSort}
              title="Assignees"
              view={view}
            />
            {activeType === "pull_requests" ? (
              <FacetGroup
                activeType={activeType}
                facets={facets.reviewers ?? []}
                query={query}
                qualifier="reviewer"
                sort={activeSort}
                title="Reviewers"
                view={view}
              />
            ) : null}
            <FacetGroup
              activeType={activeType}
              facets={facets.milestones}
              query={query}
              qualifier="milestone"
              sort={activeSort}
              title="Milestones"
              view={view}
            />
            <AdvancedQualifierLinks
              activeType={activeType}
              query={query}
              sort={activeSort}
              view={view}
            />
            <div>
              <p className="t-label" style={{ color: "var(--ink-4)" }}>
                Syntax
              </p>
              <div className="mt-2 flex flex-wrap gap-2">
                <span className="kbd">state:</span>
                <span className="kbd">label:</span>
                <span className="kbd">author:</span>
                <span className="kbd">assignee:</span>
                <span className="kbd">milestone:</span>
              </div>
            </div>
          </div>
        </aside>

        <div>
          <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                {successful
                  ? `${successful.total} ${label.toLowerCase()} results in ${queryDurationMs}ms`
                  : `${label} results`}
              </p>
              {successful?.diagnostics?.length ? (
                <div className="mt-2 space-y-1">
                  {successful.diagnostics.map((diagnostic) => (
                    <p
                      className="t-xs"
                      key={`${diagnostic.code}-${diagnostic.message}`}
                      style={{ color: "var(--warn)" }}
                    >
                      {diagnostic.message}
                    </p>
                  ))}
                </div>
              ) : null}
              {activeChips.length ? (
                <div className="mt-2 flex flex-wrap gap-2">
                  {activeChips.map((chip) => (
                    <Link
                      className="chip active"
                      href={searchHref(chip.removeQuery, activeType, {
                        sort: activeSort,
                        view,
                      })}
                      key={`${chip.qualifier}-${chip.value}`}
                    >
                      {chip.label} ×
                    </Link>
                  ))}
                </div>
              ) : null}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <details className="relative">
                <summary className="btn sm cursor-pointer list-none">
                  Sort by: {activeSortLabel}
                </summary>
                <div className="card absolute right-0 z-20 mt-2 min-w-[220px] p-2 shadow-md">
                  {sortOptions.map((option) => (
                    <Link
                      aria-current={option.selected ? "true" : undefined}
                      className={`block rounded-[var(--radius)] px-3 py-2 t-sm ${option.selected ? "chip active" : ""}`}
                      href={searchHref(query, activeType, {
                        sort: option.value,
                        view,
                      })}
                      key={option.value}
                    >
                      {option.label}
                    </Link>
                  ))}
                </div>
              </details>
              <Link
                className="btn sm"
                href={searchHref(query, activeType, {
                  saved: "1",
                  sort: activeSort,
                  view,
                })}
              >
                Save
              </Link>
              <Link
                aria-current={view === "comfortable" ? "true" : undefined}
                className={`btn sm ${view === "comfortable" ? "primary" : ""}`}
                href={searchHref(query, activeType, {
                  sort: activeSort,
                  view: "comfortable",
                })}
              >
                Comfortable
              </Link>
              <Link
                aria-current={view === "compact" ? "true" : undefined}
                className={`btn sm ${view === "compact" ? "primary" : ""}`}
                href={searchHref(query, activeType, {
                  sort: activeSort,
                  view: "compact",
                })}
              >
                Compact
              </Link>
            </div>
          </div>

          {saved ? (
            <SaveSearchDialog
              activeType={activeType}
              open={saved}
              query={query}
            />
          ) : null}

          {!query.trim() ? (
            <div className="card p-8">
              <h2 className="t-h2">Start with a query.</h2>
              <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
                Search titles, body text, labels, participants, or add advanced
                qualifiers.
              </p>
            </div>
          ) : isErrorEnvelope(results) ? (
            <ErrorState error={results} />
          ) : successful && successful.items.length > 0 ? (
            <div className="card p-4">
              {successful.items.map((result) => (
                <ResultRow
                  activeType={activeType}
                  key={"repository" in result ? result.id : result.document.id}
                  result={result}
                  view={view}
                />
              ))}
            </div>
          ) : (
            <div className="card p-8">
              <h2 className="t-h2">
                No {label.toLowerCase()} matched “{query}”.
              </h2>
              <p className="t-body mt-3" style={{ color: "var(--ink-3)" }}>
                Remove a facet or try a broader phrase.
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                {activeChips[0] ? (
                  <Link
                    className="btn sm"
                    href={searchHref(activeChips[0].removeQuery, activeType, {
                      sort: activeSort,
                      view,
                    })}
                  >
                    Remove {activeChips[0].label}
                  </Link>
                ) : null}
                <Link className="btn sm" href={searchHref("", activeType)}>
                  Clear search
                </Link>
              </div>
            </div>
          )}

          {successful && totalPages > 1 ? (
            <nav
              aria-label="Search results pages"
              className="mt-6 flex items-center justify-between gap-3"
            >
              <Link
                aria-disabled={successful.page <= 1}
                className={`btn sm ${successful.page <= 1 ? "pointer-events-none opacity-50" : ""}`}
                href={pageHref(
                  activeType,
                  query,
                  Math.max(1, successful.page - 1),
                  activeSort,
                  view,
                )}
              >
                Previous
              </Link>
              <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                Page {successful.page} of {totalPages}
              </span>
              <Link
                aria-disabled={successful.page >= totalPages}
                className={`btn sm ${successful.page >= totalPages ? "pointer-events-none opacity-50" : ""}`}
                href={pageHref(
                  activeType,
                  query,
                  Math.min(totalPages, successful.page + 1),
                  activeSort,
                  view,
                )}
              >
                Next
              </Link>
            </nav>
          ) : null}
        </div>
      </div>
    </section>
  );
}
