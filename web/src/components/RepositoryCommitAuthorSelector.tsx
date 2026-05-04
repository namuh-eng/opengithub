"use client";

import Link from "next/link";
import { useMemo, useRef, useState } from "react";
import type { RepositoryCommitAuthorOption } from "@/lib/api";
import { repositoryCommitHistoryHref } from "@/lib/navigation";

type RepositoryCommitAuthorSelectorProps = {
  owner: string;
  repo: string;
  refName: string;
  path: string | null;
  activeAuthor: string | null;
  until: string | null;
  authors: RepositoryCommitAuthorOption[];
};

function initials(login: string) {
  return login
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export function RepositoryCommitAuthorSelector({
  owner,
  repo,
  refName,
  path,
  activeAuthor,
  until,
  authors,
}: RepositoryCommitAuthorSelectorProps) {
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [query, setQuery] = useState("");
  const filteredAuthors = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return authors;
    }
    return authors.filter((author) =>
      author.login.toLowerCase().includes(normalizedQuery),
    );
  }, [authors, query]);

  return (
    <details className="relative" ref={detailsRef}>
      <summary
        aria-label={`Filter commits by author. Current author ${activeAuthor ?? "All users"}`}
        className="btn sm inline-flex cursor-pointer list-none"
      >
        {activeAuthor ?? "All users"}
      </summary>
      <div
        className="absolute left-0 z-20 mt-2 w-80 overflow-hidden rounded-md py-2 max-sm:w-[calc(100vw-3rem)]"
        role="dialog"
        aria-label="Filter commits by author"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--line)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <div
          className="border-b px-3 pb-2"
          style={{ borderColor: "var(--line)" }}
        >
          <p className="font-semibold" style={{ color: "var(--ink-1)" }}>
            Author
          </p>
          <p className="t-xs">
            Choose one contributor for this commit history.
          </p>
        </div>
        <label className="sr-only" htmlFor="commit-author-search">
          Find an author
        </label>
        <input
          aria-label="Find an author"
          className="input h-10 w-full rounded-none border-0 border-b px-3"
          id="commit-author-search"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Find an author"
          value={query}
        />
        <div className="max-h-80 overflow-y-auto py-1">
          <Link
            aria-current={activeAuthor ? undefined : "page"}
            className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
            href={repositoryCommitHistoryHref({
              owner,
              repo,
              refName,
              path,
              until,
            })}
            onClick={() => {
              if (detailsRef.current) {
                detailsRef.current.open = false;
              }
            }}
          >
            <span className="av sm">AU</span>
            <span className="min-w-0 flex-1 truncate">All users</span>
            {!activeAuthor ? (
              <span className="chip active">Selected</span>
            ) : null}
          </Link>
          {filteredAuthors.map((author) => (
            <Link
              aria-current={author.active ? "page" : undefined}
              className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-[var(--surface-2)]"
              href={repositoryCommitHistoryHref({
                owner,
                repo,
                refName,
                path,
                author: author.login,
                until,
              })}
              key={author.login}
              onClick={() => {
                if (detailsRef.current) {
                  detailsRef.current.open = false;
                }
              }}
            >
              <span className="av sm">{initials(author.login)}</span>
              <span className="min-w-0 flex-1 truncate">{author.login}</span>
              <span className="t-xs t-num">{author.count}</span>
              {author.active ? (
                <span className="chip active">Selected</span>
              ) : null}
            </Link>
          ))}
          {filteredAuthors.length === 0 ? (
            <p className="px-3 py-4 t-sm" style={{ color: "var(--ink-3)" }}>
              No authors match that search.
            </p>
          ) : null}
        </div>
      </div>
    </details>
  );
}
