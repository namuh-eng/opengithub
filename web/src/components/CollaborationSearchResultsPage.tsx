import Link from "next/link";
import { QueryTabNavigation } from "@/components/QueryTabNavigation";
import type {
  ApiErrorEnvelope,
  CodeSearchFacetValue,
  CollaborationSearchResponse,
  GlobalSearchResult,
} from "@/lib/api";
import { SEARCH_TABS, searchHref, searchTypeHref } from "@/lib/navigation";

type Props = {
  activeType: "issues" | "pull_requests" | string;
  query: string;
  results: CollaborationSearchResponse | ApiErrorEnvelope | null;
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

function addQualifier(query: string, qualifier: string, value: string) {
  const quoted = /\s/.test(value) ? `"${value.replaceAll('"', '\\"')}"` : value;
  return `${query.trim()} ${qualifier}:${quoted}`.trim();
}

function facetHref(
  activeType: string,
  query: string,
  qualifier: string,
  value: string,
) {
  return searchHref(addQualifier(query, qualifier, value), activeType);
}

function pageHref(
  activeType: string,
  query: string,
  page: number,
  sort?: string,
) {
  return searchHref(query, activeType, {
    ...(page > 1 ? { page: String(page) } : {}),
    ...(sort ? { sort } : {}),
  });
}

function FacetGroup({
  activeType,
  facets,
  query,
  qualifier,
  title,
}: {
  activeType: string;
  facets: CodeSearchFacetValue[];
  query: string;
  qualifier: string;
  title: string;
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
            href={facetHref(activeType, query, qualifier, facet.value)}
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

function ResultRow({
  activeType,
  result,
}: {
  activeType: string;
  result: GlobalSearchResult;
}) {
  const metadata = result.document.metadata;
  const state = metadataString(metadata, "state") ?? "open";
  const number = metadataNumber(metadata, "number");
  const labels = metadataLabels(metadata);
  const assignees = metadataPeople(metadata, "assignees");
  const reviewers = metadataPeople(metadata, "reviewers");
  const author = metadataString(metadata, "authorLogin");
  const headRef = metadataString(metadata, "headRef");
  const baseRef = metadataString(metadata, "baseRef");
  const comments = metadataNumber(metadata, "commentCount");
  const interactions = metadataNumber(metadata, "interactionCount");
  const milestone = milestoneTitle(metadata);

  return (
    <Link className="list-row items-start gap-3 px-0" href={result.href}>
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
          {result.owner_login ?? "owner"} /{" "}
          {result.repository_name ?? "repository"}
        </span>
        <span className="mt-1 block text-[15px] font-semibold text-[color:var(--ink-1)]">
          {result.title}
        </span>
        {result.summary ? (
          <span
            className="t-sm mt-1 line-clamp-2 block"
            style={{ color: "var(--ink-3)" }}
          >
            {result.summary}
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
}: Props) {
  const label = typeLabel(activeType);
  const successful = results && !isErrorEnvelope(results) ? results : null;
  const facets = successful?.facets ?? {
    assignees: [],
    labels: [],
    milestones: [],
    reviewers: [],
    states: [],
  };
  const activeChips = successful?.activeChips ?? [];
  const activeSort = successful?.activeSort ?? "best-match";
  const sortOptions = successful?.sortOptions ?? [
    { label: "Best match", selected: true, value: "best-match" },
  ];
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
              title="State"
            />
            <FacetGroup
              activeType={activeType}
              facets={facets.labels}
              query={query}
              qualifier="label"
              title="Labels"
            />
            <FacetGroup
              activeType={activeType}
              facets={facets.assignees}
              query={query}
              qualifier="assignee"
              title="Assignees"
            />
            {activeType === "pull_requests" ? (
              <FacetGroup
                activeType={activeType}
                facets={facets.reviewers}
                query={query}
                qualifier="reviewer"
                title="Reviewers"
              />
            ) : null}
            <FacetGroup
              activeType={activeType}
              facets={facets.milestones}
              query={query}
              qualifier="milestone"
              title="Milestones"
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
              {activeChips.length ? (
                <div className="mt-2 flex flex-wrap gap-2">
                  {activeChips.map((chip) => (
                    <Link
                      className="chip active"
                      href={searchHref(chip.removeQuery, activeType, {
                        sort: activeSort,
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
              {sortOptions.map((option) => (
                <Link
                  className={option.selected ? "chip active" : "chip soft"}
                  href={searchHref(query, activeType, { sort: option.value })}
                  key={option.value}
                >
                  {option.label}
                </Link>
              ))}
              <Link
                className="btn sm"
                href={`/search?saved=1&q=${encodeURIComponent(query)}&type=${encodeURIComponent(activeType)}`}
              >
                Save
              </Link>
            </div>
          </div>

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
                  key={result.document.id}
                  result={result}
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
