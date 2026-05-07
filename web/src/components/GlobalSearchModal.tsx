"use client";

import Link from "next/link";
import {
  type KeyboardEvent,
  useEffect,
  useLayoutEffect,
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

type FlatSuggestion = {
  action: SearchModalAction;
  groupTitle: string;
  href: string;
  id: string;
  title: string;
  description: string | null;
  kind: string;
  savedSearchId?: string;
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
      savedSearchId: item.id,
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
  const [refreshKey, setRefreshKey] = useState(0);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [savedName, setSavedName] = useState("");
  const [savedQuery, setSavedQuery] = useState(initialQuery);
  const [savedFeedback, setSavedFeedback] = useState<string | null>(null);
  const [savedError, setSavedError] = useState<string | null>(null);
  const [savingSearch, setSavingSearch] = useState(false);
  const [deletingSavedSearchId, setDeletingSavedSearchId] = useState<
    string | null
  >(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const savedNameRef = useRef<HTMLInputElement | null>(null);
  const flatSuggestions = useMemo(
    () => flattenSuggestions(dashboard),
    [dashboard],
  );
  const groups = useMemo(
    () => groupedFlatSuggestions(flatSuggestions),
    [flatSuggestions],
  );
  const selectedSuggestion =
    selectedIndex === null ? undefined : flatSuggestions[selectedIndex];
  const modalRef = useRef<HTMLDivElement | null>(null);
  const queryBuilderChips = [
    { label: "language:rust", value: "language:rust" },
    { label: "repo:owner/name", value: "repo:" },
    { label: "org:name", value: "org:" },
    { label: "path:src/", value: "path:src/" },
    { label: "is:open", value: "is:open" },
  ];

  useLayoutEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setQuery(initialQuery);
    setSavedQuery(initialQuery);
  }, [initialQuery]);

  useEffect(() => {
    void refreshKey;
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
        setSelectedIndex(null);
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
  }, [query, refreshKey]);

  useEffect(() => {
    if (selectedIndex !== null && selectedIndex >= flatSuggestions.length) {
      setSelectedIndex(null);
    }
  }, [flatSuggestions.length, selectedIndex]);

  function applyAction(action: SearchModalAction, replacementTitle?: string) {
    if (action.kind === "replace_token") {
      const replacement =
        replacementTitle?.includes(":") &&
        currentTokenPrefix(query) === replacementTitle.split(":", 1)[0]
          ? replacementTitle
          : action.nextQuery;
      setQuery(
        replacement === action.nextQuery
          ? action.nextQuery.endsWith(" ")
            ? action.nextQuery
            : `${action.nextQuery} `
          : replaceCurrentSearchToken(query, replacement),
      );
      window.requestAnimationFrame(() => inputRef.current?.focus());
      return;
    }
    if (action.kind === "open_saved_search_dialog") {
      openCreateDialog();
      return;
    }
    window.location.assign(searchModalActionHref(action, query));
  }

  function openCreateDialog() {
    setSavedName("");
    setSavedQuery(query.trim());
    setSavedError(null);
    setSavedFeedback(null);
    setCreateDialogOpen(true);
    window.requestAnimationFrame(() => savedNameRef.current?.focus());
  }

  function closeCreateDialog() {
    setCreateDialogOpen(false);
    setSavedError(null);
    setSavingSearch(false);
    window.requestAnimationFrame(() => inputRef.current?.focus());
  }

  async function createSavedSearch() {
    const normalizedName = savedName.trim();
    const normalizedQuery = savedQuery.trim();
    setSavedFeedback(null);
    if (!normalizedName) {
      setSavedError("Name is required.");
      return;
    }
    if (!normalizedQuery) {
      setSavedError("Query is required.");
      return;
    }

    setSavingSearch(true);
    setSavedError(null);
    try {
      const response = await fetch("/search/saved-searches", {
        body: JSON.stringify({
          name: normalizedName,
          query: normalizedQuery,
          scope: "repositories",
        }),
        headers: { "content-type": "application/json" },
        method: "POST",
      });
      const body = await response.json().catch(() => null);
      if (!response.ok || body?.error) {
        throw new Error(
          body?.error?.message ?? "Saved search could not be created.",
        );
      }
      setSavedFeedback(`Saved "${body.name}".`);
      setQuery(normalizedQuery);
      setRefreshKey((value) => value + 1);
      setCreateDialogOpen(false);
      window.requestAnimationFrame(() => inputRef.current?.focus());
    } catch (createError) {
      setSavedError(
        createError instanceof Error
          ? createError.message
          : "Saved search could not be created.",
      );
    } finally {
      setSavingSearch(false);
    }
  }

  async function deleteSavedSearch(id: string, title: string) {
    setDeletingSavedSearchId(id);
    setSavedFeedback(null);
    setSavedError(null);
    try {
      const response = await fetch(`/search/saved-searches/${id}`, {
        method: "DELETE",
      });
      if (!response.ok) {
        const body = await response.json().catch(() => null);
        throw new Error(
          body?.error?.message ?? "Saved search could not be deleted.",
        );
      }
      setSavedFeedback(`Deleted "${title}".`);
      setRefreshKey((value) => value + 1);
    } catch (deleteError) {
      setSavedError(
        deleteError instanceof Error
          ? deleteError.message
          : "Saved search could not be deleted.",
      );
    } finally {
      setDeletingSavedSearchId(null);
    }
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

  function replaceCurrentSearchToken(
    currentQuery: string,
    replacement: string,
  ) {
    const trimmedEnd = currentQuery.trimEnd();
    const replaceFrom = [...trimmedEnd]
      .map((character, index) => ({ character, index }))
      .reverse()
      .find(({ character }) => /\s/.test(character))?.index;
    const start = replaceFrom === undefined ? 0 : replaceFrom + 1;
    const nextQuery = `${currentQuery.slice(0, start)}${replacement}`;
    return nextQuery.endsWith(" ") ? nextQuery : `${nextQuery} `;
  }

  function currentTokenPrefix(currentQuery: string) {
    const token = currentQuery.trimEnd().split(/\s+/).at(-1) ?? "";
    return token.includes(":") ? token.split(":", 1)[0] : null;
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (createDialogOpen) {
        closeCreateDialog();
        return;
      }
      onClose();
      return;
    }
    if (event.key === "ArrowDown" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex((current) =>
        current === null ? 0 : (current + 1) % flatSuggestions.length,
      );
      return;
    }
    if (event.key === "ArrowUp" && flatSuggestions.length > 0) {
      event.preventDefault();
      setSelectedIndex((current) =>
        current === null
          ? flatSuggestions.length - 1
          : (current - 1 + flatSuggestions.length) % flatSuggestions.length,
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
      applyAction(selectedSuggestion.action, selectedSuggestion.title);
    }
  }

  function onDialogKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.defaultPrevented) {
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      if (createDialogOpen) {
        closeCreateDialog();
        return;
      }
      onClose();
      return;
    }

    if (event.key !== "Tab") {
      return;
    }

    const focusable = Array.from(
      modalRef.current?.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ) ?? [],
    ).filter((element) => {
      const style = window.getComputedStyle(element);
      return (
        !element.hasAttribute("disabled") &&
        element.getAttribute("aria-hidden") !== "true" &&
        style.display !== "none" &&
        style.visibility !== "hidden"
      );
    });

    if (focusable.length === 0) {
      return;
    }

    const first = focusable[0];
    const last = focusable.at(-1) ?? first;
    const active = document.activeElement;

    if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
      return;
    }

    if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
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
        onKeyDown={onDialogKeyDown}
        ref={modalRef}
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
              <button
                className="btn sm"
                onClick={openCreateDialog}
                type="button"
              >
                Create saved search
              </button>
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
          {savedFeedback ? (
            <p
              className="px-3 py-2 t-xs"
              role="status"
              style={{ color: "var(--ok)" }}
            >
              {savedFeedback}
            </p>
          ) : null}
          {savedError && !createDialogOpen ? (
            <p
              className="px-3 py-2 t-xs"
              role="alert"
              style={{ color: "var(--err)" }}
            >
              {savedError}
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
                      onClick={() => applyAction(item.action, item.title)}
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
                if (item.kind === "saved_search") {
                  return (
                    <div
                      aria-selected={index === selectedIndex}
                      className={`palette-item ${
                        index === selectedIndex ? "selected" : ""
                      }`}
                      id={`global-search-${item.id}`}
                      key={item.id}
                      onMouseEnter={() => setSelectedIndex(index)}
                      role="option"
                      tabIndex={-1}
                    >
                      <span className="ico">
                        <SearchIcon />
                      </span>
                      <Link
                        className="min-w-0 flex-1 truncate"
                        href={item.href}
                        onClick={onClose}
                      >
                        {item.title}
                      </Link>
                      <span className="desc">{item.description}</span>
                      {item.savedSearchId ? (
                        <button
                          className="btn sm ghost"
                          disabled={
                            deletingSavedSearchId === item.savedSearchId
                          }
                          onClick={() =>
                            deleteSavedSearch(
                              item.savedSearchId ?? "",
                              item.title,
                            )
                          }
                          type="button"
                        >
                          Delete
                        </button>
                      ) : null}
                    </div>
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
        {createDialogOpen ? (
          <div
            aria-label="Create saved search"
            aria-modal="true"
            className="absolute inset-x-4 top-20 z-[2] card p-4 shadow-lg"
            role="dialog"
            style={{ background: "var(--surface)" }}
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <h3 className="t-h3">Create saved search</h3>
                <Link className="t-xs" href="/docs/api#search-saved-create">
                  Saved search documentation
                </Link>
              </div>
              <button
                aria-label="Cancel saved search creation"
                className="btn sm ghost"
                onClick={closeCreateDialog}
                type="button"
              >
                Cancel
              </button>
            </div>
            <div className="mt-4 grid gap-3">
              <label className="grid gap-1">
                <span className="t-label">Name</span>
                <input
                  aria-invalid={
                    savedError?.includes("Name") ? "true" : undefined
                  }
                  className="input"
                  onChange={(event) => setSavedName(event.target.value)}
                  ref={savedNameRef}
                  required
                  value={savedName}
                />
              </label>
              <label className="grid gap-1">
                <span className="t-label">Query</span>
                <input
                  aria-invalid={
                    savedError?.includes("Query") ? "true" : undefined
                  }
                  className="input"
                  onChange={(event) => setSavedQuery(event.target.value)}
                  required
                  value={savedQuery}
                />
              </label>
              {savedError ? (
                <p
                  className="t-xs"
                  role="alert"
                  style={{ color: "var(--err)" }}
                >
                  {savedError}
                </p>
              ) : null}
              <div className="flex justify-end gap-2">
                <button
                  className="btn sm ghost"
                  onClick={closeCreateDialog}
                  type="button"
                >
                  Cancel
                </button>
                <button
                  className="btn sm accent"
                  disabled={savingSearch}
                  onClick={createSavedSearch}
                  type="button"
                >
                  {savingSearch ? "Creating..." : "Create saved search"}
                </button>
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
