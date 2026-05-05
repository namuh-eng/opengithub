import Link from "next/link";
import type { DiscussionFilterState, DiscussionLabelSummary } from "@/lib/api";
import {
  type RepositoryDiscussionHrefQuery,
  repositoryDiscussionsHref,
} from "@/lib/navigation";

type RepositoryDiscussionFiltersProps = {
  owner: string;
  repo: string;
  filters: DiscussionFilterState;
  labels: DiscussionLabelSummary[];
};

const SORT_OPTIONS = [
  ["latest", "Latest activity"],
  ["newest", "Newest"],
  ["top", "Top"],
  ["most_commented", "Most commented"],
] as const;

function queryFor(
  filters: DiscussionFilterState,
  overrides: Partial<RepositoryDiscussionHrefQuery>,
): RepositoryDiscussionHrefQuery {
  return {
    q: filters.query,
    label: filters.label,
    state: filters.state,
    answered: filters.answered,
    locked: filters.locked,
    pinned: filters.pinned,
    sort: filters.sort,
    page: 1,
    pageSize: filters.pageSize,
    ...overrides,
  };
}

function activeLabel(value: string | null | boolean, fallback: string) {
  if (value === null || value === false) return fallback;
  if (value === true) return "Yes";
  return value;
}

export function RepositoryDiscussionFilters({
  owner,
  repo,
  filters,
  labels,
}: RepositoryDiscussionFiltersProps) {
  const activeSort =
    SORT_OPTIONS.find(([value]) => value === filters.sort)?.[1] ??
    "Latest activity";

  return (
    <div className="flex flex-wrap items-center gap-3">
      <form
        action={repositoryDiscussionsHref(owner, repo)}
        className="flex min-w-[260px] flex-1 flex-wrap items-center gap-3"
        method="get"
      >
        <label className="input min-w-[260px] flex-1" htmlFor="discussion-q">
          <span aria-hidden="true">⌕</span>
          <input
            aria-label="discussion-query"
            defaultValue={filters.query || "is:open"}
            id="discussion-q"
            name="q"
            placeholder="is:open label:help-wanted"
          />
        </label>
        {filters.state !== "open" ? (
          <input name="state" type="hidden" value={filters.state} />
        ) : null}
        {filters.label ? (
          <input name="label" type="hidden" value={filters.label} />
        ) : null}
        {filters.sort !== "latest" ? (
          <input name="sort" type="hidden" value={filters.sort} />
        ) : null}
        <button className="btn" type="submit">
          Search
        </button>
      </form>

      <details className="relative">
        <summary className="btn cursor-pointer list-none">
          State: {filters.state}
        </summary>
        <div className="card absolute right-0 z-20 mt-2 grid w-48 gap-1 p-2 shadow-md">
          {["open", "closed", "all"].map((state) => (
            <Link
              aria-current={filters.state === state ? "page" : undefined}
              className="rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryDiscussionsHref(
                owner,
                repo,
                queryFor(filters, { state, page: 1 }),
              )}
              key={state}
            >
              {state}
            </Link>
          ))}
        </div>
      </details>

      <details className="relative">
        <summary className="btn cursor-pointer list-none">
          Label: {activeLabel(filters.label, "Any")}
        </summary>
        <div className="card absolute right-0 z-20 mt-2 grid max-h-80 w-64 gap-1 overflow-y-auto p-2 shadow-md">
          <Link
            className="rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
            href={repositoryDiscussionsHref(
              owner,
              repo,
              queryFor(filters, { label: null, page: 1 }),
            )}
          >
            Any label
          </Link>
          {labels.map((label) => (
            <Link
              aria-current={filters.label === label.name ? "page" : undefined}
              className="flex items-center justify-between gap-3 rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryDiscussionsHref(
                owner,
                repo,
                queryFor(filters, { label: label.name, page: 1 }),
              )}
              key={label.id}
            >
              <span className="min-w-0 truncate">{label.name}</span>
              <span className="t-num t-xs">{label.count}</span>
            </Link>
          ))}
        </div>
      </details>

      <details className="relative">
        <summary className="btn cursor-pointer list-none">
          Filter:{" "}
          {filters.answered
            ? "Answered"
            : filters.answered === false
              ? "Unanswered"
              : filters.locked
                ? "Locked"
                : filters.pinned
                  ? "Pinned"
                  : "Any"}
        </summary>
        <div className="card absolute right-0 z-20 mt-2 grid w-56 gap-1 p-2 shadow-md">
          {[
            ["Any discussion", {}],
            ["Answered", { answered: true }],
            ["Unanswered", { answered: false }],
            ["Locked", { locked: true }],
            ["Pinned", { pinned: true }],
          ].map(([label, overrides]) => (
            <Link
              className="rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryDiscussionsHref(
                owner,
                repo,
                queryFor(filters, {
                  answered: null,
                  locked: null,
                  pinned: null,
                  ...(overrides as Partial<RepositoryDiscussionHrefQuery>),
                }),
              )}
              key={label as string}
            >
              {label as string}
            </Link>
          ))}
        </div>
      </details>

      <details className="relative">
        <summary className="btn cursor-pointer list-none">
          Sort: {activeSort}
        </summary>
        <div className="card absolute right-0 z-20 mt-2 grid w-56 gap-1 p-2 shadow-md">
          {SORT_OPTIONS.map(([value, label]) => (
            <Link
              aria-current={filters.sort === value ? "page" : undefined}
              className="rounded-[var(--radius)] px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryDiscussionsHref(
                owner,
                repo,
                queryFor(filters, { sort: value, page: 1 }),
              )}
              key={value}
            >
              {label}
            </Link>
          ))}
        </div>
      </details>

      <Link className="btn sm" href={repositoryDiscussionsHref(owner, repo)}>
        Clear
      </Link>
    </div>
  );
}
