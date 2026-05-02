"use client";

import Link from "next/link";
import {
  type KeyboardEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type {
  RecentSearchSuggestion,
  SavedSearchSuggestion,
  SearchSuggestionDashboard,
  SearchSuggestionGroup,
  SearchSuggestionItem,
} from "@/lib/api";
import {
  replaceSearchQueryToken,
  type SearchModalAction,
  searchHref,
  searchModalActionHref,
} from "@/lib/navigation";

type GlobalSearchModalProps = {
  initialQuery?: string;
  onClose: () => void;
};

type FlatSuggestion =
  | {
      action: SearchModalAction;
      groupTitle: string;
      href: string;
      id: string;
      title: string;
      description: string | null;
      kind: string;
    }
  | {
      action: SearchModalAction;
      groupTitle: string;
      href: string;
      id: string;
      title: string;
      description: string | null;
      kind: "saved_search" | "recent_search";
    };

function SearchIcon() {
  return (
    <svg aria-hidden="true" height="16" viewBox="0 0 16 16" width="16">
      <path
        d="m11.2 11.2 2.3 2.3M7.1 12.2a5.1 5.1 0 1 1 0-10.2 5.1 5.1 0 0 1 0 10.2Z"
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.5"
      />
    </svg>
  );
}

function itemHref(item: SearchSuggestionItem) {
  if (item.action === "replace_token") {
    return searchHref(item.nextQuery ?? "");
  }
  return item.href ?? searchHref(item.nextQuery ?? "");
}

function itemAction(item: SearchSuggestionItem): SearchModalAction {
  if (item.action === "replace_token") {
    return {
      kind: "replace_token",
      nextQuery: item.nextQuery ?? "",
    };
  }
  if (item.action === "submit_search") {
    return {
      href: item.href ?? searchHref(item.nextQuery ?? ""),
      kind: "submit_search",
    };
  }
  if (item.action === "open_saved_search_dialog") {
    return { kind: "open_saved_search_dialog" };
  }
  return {
    href: item.href ?? searchHref(item.nextQuery ?? ""),
    kind: "navigate",
  };
}

function flattenSuggestions(
  dashboard: SearchSuggestionDashboard | null,
): FlatSuggestion[] {
  if (!dashboard) {
    return [];
  }

  const apiItems = dashboard.groups.flatMap((group: SearchSuggestionGroup) =>
    group.items
      .filter((item) => item.href || item.nextQuery)
      .map((item) => ({
        action: itemAction(item),
        description: item.description,
        groupTitle: group.title,
        href: itemHref(item),
        id: `${group.id}:${item.id}`,
        kind: item.kind,
        title: item.title,
      })),
  );
  const savedItems = dashboard.savedSearches.map(
    (item: SavedSearchSuggestion) => ({
      description: item.query,
      action: {
        href: item.href,
        kind: "submit_search" as const,
      },
      groupTitle: "Saved searches",
      href: item.href,
      id: `saved:${item.id}`,
      kind: "saved_search" as const,
      title: item.name,
    }),
  );
  const recentItems = dashboard.recentSearches.map(
    (item: RecentSearchSuggestion) => ({
      description: item.resultType ?? item.scope,
      action: {
        href: item.href,
        kind: "submit_search" as const,
      },
      groupTitle: "Recent searches",
      href: item.href,
      id: `recent:${item.id}`,
      kind: "recent_search" as const,
      title: item.query,
    }),
  );

  return [...apiItems, ...savedItems, ...recentItems];
}

function groupedFlatSuggestions(items: FlatSuggestion[]) {
  const groups = new Map<string, FlatSuggestion[]>();
  for (const item of items) {
    groups.set(item.groupTitle, [...(groups.get(item.groupTitle) ?? []), item]);
  }
  return Array.from(groups, ([title, groupItems]) => ({
    items: groupItems,
    title,
  }));
}

function submitHref(query: string) {
  return searchHref(query);
}

export function GlobalSearchModal({
  initialQuery = "",
  onClose,
}: GlobalSearchModalProps) {
  const [query, setQuery] = useState(initialQuery);
  const [dashboard, setDashboard] = useState<SearchSuggestionDashboard | null>(
    null,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const flatSuggestions = useMemo(
    () => flattenSuggestions(dashboard),
    [dashboard],
  );
  const groups = useMemo(
    () => groupedFlatSuggestions(flatSuggestions),
    [flatSuggestions],
  );
  const selectedSuggestion = flatSuggestions[selectedIndex];
  const queryBuilderChips = [
    { label: "language:rust", value: "language:rust" },
    { label: "repo:owner/name", value: "repo:" },
    { label: "org:name", value: "org:" },
    { label: "path:src/", value: "path:src/" },
    { label: "is:open", value: "is:open" },
  ];

  useEffect(() => {
    const frame = window.requestAnimationFrame(() => inputRef.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, []);

  useEffect(() => {
    setQuery(initialQuery);
  }, [initialQuery]);

  useEffect(() => {
    const controller = new AbortController();
    const timer = window.setTimeout(async () => {
      setLoading(true);
      setError(null);
      try {
        const params = new URLSearchParams();
        if (query.trim()) {
          params.set("q", query.trim());
        }
        params.set("scope", "all");
        params.set("limit", "8");
        const response = await fetch(`/search/suggestions?${params}`, {
          signal: controller.signal,
        });
        const body = await response.json();
        if (!response.ok || "error" in body) {
          throw new Error(body?.error?.message ?? "Search suggestions failed.");
        }
        setDashboard(body as SearchSuggestionDashboard);
        setSelectedIndex(0);
      } catch (fetchError) {
        if (!controller.signal.aborted) {
          setDashboard(null);
          setError(
            fetchError instanceof Error
              ? fetchError.message
              : "Search suggestions failed.",
          );
        }
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
        }
      }
    }, 120);

    return () => {
      controller.abort();
      window.clearTimeout(timer);
    };
  }, [query]);

  useEffect(() => {
    if (selectedIndex >= flatSuggestions.length) {
      setSelectedIndex(0);
    }
  }, [flatSuggestions.length, selectedIndex]);

  function applyAction(action: SearchModalAction) {
    if (action.kind === "replace_token") {
      setQuery(
        action.nextQuery.endsWith(" ")
          ? action.nextQuery
          : `${action.nextQuery} `,
      );
      window.requestAnimationFrame(() => inputRef.current?.focus());
      return;
    }
    window.location.assign(searchModalActionHref(action, query));
  }

  function addQualifier(value: string) {
    const token = dashboard?.token;
    const tokenStillMatches =
      token &&
      query.slice(token.replaceFrom, token.replaceTo) === token.value &&
      (token.replaceTo === query.length ||
        /\s/.test(query.charAt(token.replaceTo)));
    const nextQuery = tokenStillMatches
      ? replaceSearchQueryToken(
          query,
          token.replaceFrom,
          token.replaceTo,
          value,
        )
      : `${query.trim()}${query.trim() ? " " : ""}${value} `;
    setQuery(nextQuery);
    window.requestAnimationFrame(() => inputRef.current?.focus());
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key === "ArrowDown" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex((current) => (current + 1) % flatSuggestions.length);
      return;
    }
    if (event.key === "ArrowUp" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex(
        (current) =>
          (current - 1 + flatSuggestions.length) % flatSuggestions.length,
      );
      return;
    }
    if (event.key === "Home" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex(0);
      return;
    }
    if (event.key === "End" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex(flatSuggestions.length - 1);
      return;
    }
    if (event.key === "Enter" && selectedSuggestion) {
      event.preventDefault();
      applyAction(selectedSuggestion.action);
    }
  }

  return (
    <div className="palette-backdrop">
      <button
        aria-label="Close search"
        className="absolute inset-0 cursor-default"
        onClick={onClose}
        style={{ background: "transparent", border: 0 }}
        type="button"
      />
      <div
        aria-label="Search"
        aria-modal="true"
        className="palette relative z-[1]"
        role="dialog"
      >
        <div
          className="border-b px-4 py-3"
          style={{ borderColor: "var(--line)" }}
        >
          <div className="flex items-center justify-between gap-3">
            <h2 className="t-h3">Search</h2>
            <div className="flex items-center gap-2">
              <Link className="btn sm ghost" href="/docs/api#search">
                Syntax tips
              </Link>
              <Link
                className="btn sm ghost"
                href="/issues/new?title=Search%20feedback"
              >
                Feedback
              </Link>
            </div>
          </div>
        </div>
        <form action="/search" className="palette-input">
          <SearchIcon />
          <input
            aria-activedescendant={
              selectedSuggestion
                ? `global-search-${selectedSuggestion.id}`
                : undefined
            }
            aria-controls="global-search-suggestions"
            aria-expanded="true"
            aria-label="Search opengithub"
            autoComplete="off"
            name="q"
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onKeyDown}
            placeholder="Search anywhere: repos, files, issues, people..."
            ref={inputRef}
            role="combobox"
            value={query}
          />
          <input name="type" type="hidden" value="repositories" />
          <Link className="btn sm" href={submitHref(query)}>
            Search
          </Link>
          <span className="kbd">esc</span>
        </form>
        <div
          className="flex flex-wrap items-center gap-2 border-b px-3 py-2"
          style={{ borderColor: "var(--line)" }}
        >
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Add qualifier
          </span>
          {queryBuilderChips.map((chip) => (
            <button
              className="chip soft"
              key={chip.label}
              onClick={() => addQualifier(chip.value)}
              type="button"
            >
              {chip.label}
            </button>
          ))}
        </div>

        <div
          className="palette-list"
          id="global-search-suggestions"
          role="listbox"
        >
          {loading ? (
            <p className="px-3 py-4 t-xs" style={{ color: "var(--ink-3)" }}>
              Loading suggestions...
            </p>
          ) : null}
          {error ? (
            <p className="px-3 py-4 t-xs" style={{ color: "var(--err)" }}>
              {error}
            </p>
          ) : null}
          {!loading && !error && groups.length === 0 ? (
            <p className="px-3 py-4 t-xs" style={{ color: "var(--ink-3)" }}>
              No suggestions match this query.
            </p>
          ) : null}
          {groups.map((group) => (
            <div key={group.title}>
              <div className="palette-section">{group.title}</div>
              {group.items.map((item) => {
                const index = flatSuggestions.findIndex(
                  (candidate) => candidate.id === item.id,
                );
                if (item.action.kind === "replace_token") {
                  return (
                    <button
                      aria-selected={index === selectedIndex}
                      className={`palette-item w-full ${
                        index === selectedIndex ? "selected" : ""
                      }`}
                      id={`global-search-${item.id}`}
                      key={item.id}
                      onClick={() => applyAction(item.action)}
                      onMouseEnter={() => setSelectedIndex(index)}
                      role="option"
                      type="button"
                    >
                      <span className="ico">
                        <SearchIcon />
                      </span>
                      <span className="min-w-0 truncate">{item.title}</span>
                      <span className="desc">{item.description}</span>
                    </button>
                  );
                }
                return (
                  <Link
                    aria-selected={index === selectedIndex}
                    className={`palette-item ${
                      index === selectedIndex ? "selected" : ""
                    }`}
                    href={item.href}
                    id={`global-search-${item.id}`}
                    key={item.id}
                    onClick={onClose}
                    onMouseEnter={() => setSelectedIndex(index)}
                    role="option"
                  >
                    <span className="ico">
                      <SearchIcon />
                    </span>
                    <span className="min-w-0 truncate">{item.title}</span>
                    <span className="desc">{item.description}</span>
                  </Link>
                );
              })}
            </div>
          ))}
        </div>
        <div
          className="flex flex-wrap gap-3 border-t px-3 py-2 t-xs"
          style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
        >
          <span>
            <span className="kbd">↑↓</span> navigate
          </span>
          <span>
            <span className="kbd">↵</span> open
          </span>
          <span>
            <span className="kbd">/</span> open search
          </span>
          <Link className="ml-auto" href="/search?saved=1" onClick={onClose}>
            Manage saved searches
          </Link>
        </div>
      </div>
    </div>
  );
}
