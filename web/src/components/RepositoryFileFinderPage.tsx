"use client";

import Link from "next/link";
import { useEffect, useMemo, useRef, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  RepositoryFileFinderItem,
  RepositoryFileFinderResult,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryFileFinderPageProps = {
  repository: RepositoryOverview;
  finder: RepositoryFileFinderResult;
};

type ScoredFile = RepositoryFileFinderItem & {
  score: number;
  matchIndexes: number[];
};

function fuzzyScore(path: string, query: string): ScoredFile["score"] | null {
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return 1;
  }

  let score = 0;
  let searchFrom = 0;
  let previousIndex = -1;
  const haystack = path.toLowerCase();

  for (const char of needle) {
    const index = haystack.indexOf(char, searchFrom);
    if (index === -1) {
      return null;
    }
    const boundaryBonus =
      index === 0 || ["/", "-", "_", "."].includes(path[index - 1] ?? "")
        ? 8
        : 0;
    const consecutiveBonus = previousIndex + 1 === index ? 12 : 0;
    score += 20 + boundaryBonus + consecutiveBonus - Math.min(index, 40) / 4;
    previousIndex = index;
    searchFrom = index + 1;
  }

  return score - path.length / 100;
}

function matchIndexes(path: string, query: string) {
  const indexes: number[] = [];
  let searchFrom = 0;
  const haystack = path.toLowerCase();

  for (const char of query.trim().toLowerCase()) {
    const index = haystack.indexOf(char, searchFrom);
    if (index === -1) {
      return [];
    }
    indexes.push(index);
    searchFrom = index + 1;
  }

  return indexes;
}

function highlightedPath(path: string, indexes: number[]) {
  const matched = new Set(indexes);
  return path.split("").map((char, index) => (
    <span
      className={matched.has(index) ? "text-[var(--accent)]" : undefined}
      key={`${path.slice(0, index + 1)}-${char}`}
    >
      {char}
    </span>
  ));
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export function RepositoryFileFinderPage({
  repository,
  finder,
}: RepositoryFileFinderPageProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const scoredFiles = useMemo(() => {
    const scored = finder.items
      .map((file) => {
        const score = fuzzyScore(file.path, query);
        if (score === null) {
          return null;
        }
        return {
          ...file,
          score,
          matchIndexes: matchIndexes(file.path, query),
        };
      })
      .filter((file): file is ScoredFile => Boolean(file));

    scored.sort(
      (left, right) =>
        right.score - left.score ||
        left.path.localeCompare(right.path, undefined, { sensitivity: "base" }),
    );

    return scored;
  }, [finder.items, query]);

  const visibleFiles = scoredFiles;
  const activeOptionIndex = Math.min(
    Math.max(activeIndex, 0),
    Math.max(visibleFiles.length - 1, 0),
  );
  const activeFile = visibleFiles[activeOptionIndex];

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}`}
      frameClassName="block"
      repository={repository}
    >
      <div className="mx-auto max-w-4xl px-0 py-2">
        <div
          className="overflow-hidden rounded-md"
          style={{
            border: "1px solid var(--line)",
            background: "var(--surface)",
            boxShadow: "var(--shadow-md)",
          }}
        >
          <div className="border-b p-4" style={{ borderColor: "var(--line)" }}>
            <div className="flex flex-wrap items-center gap-3">
              <div className="min-w-0 flex-1">
                <p className="t-label" style={{ color: "var(--ink-3)" }}>
                  File finder
                </p>
                <h1 className="mt-1 t-h2" style={{ color: "var(--ink-1)" }}>
                  {repository.owner_login}/{repository.name}
                </h1>
              </div>
              <span className="chip soft t-mono-sm">
                {finder.resolvedRef.shortName}
              </span>
              <span className="kbd">t</span>
            </div>
            <label className="sr-only" htmlFor="repo-file-finder-page-input">
              Fuzzy-find a file path
            </label>
            <input
              aria-activedescendant={
                activeFile
                  ? `repo-file-finder-result-${activeOptionIndex}`
                  : undefined
              }
              aria-controls="repo-file-finder-page-results"
              aria-expanded="true"
              aria-label="Fuzzy-find a file path"
              className="input mt-4 h-12 w-full px-4 t-mono-sm"
              id="repo-file-finder-page-input"
              onChange={(event) => {
                setQuery(event.target.value);
                setActiveIndex(0);
              }}
              onKeyDown={(event) => {
                if (event.key === "ArrowDown") {
                  event.preventDefault();
                  setActiveIndex((index) =>
                    Math.min(Math.max(visibleFiles.length - 1, 0), index + 1),
                  );
                }
                if (event.key === "ArrowUp") {
                  event.preventDefault();
                  setActiveIndex((index) => Math.max(0, index - 1));
                }
                if (event.key === "Enter" && activeFile) {
                  event.preventDefault();
                  window.location.assign(activeFile.href);
                }
                if (event.key === "Escape") {
                  event.preventDefault();
                  setQuery("");
                  setActiveIndex(0);
                  inputRef.current?.focus();
                }
              }}
              placeholder="Type a path, symbol, or filename..."
              ref={inputRef}
              role="combobox"
              value={query}
            />
            <div
              className="mt-3 flex flex-wrap items-center gap-3 t-xs"
              style={{ color: "var(--ink-3)" }}
            >
              <span>
                {query.trim()
                  ? `${visibleFiles.length} matching paths`
                  : `${finder.total} cached paths`}
              </span>
              <span>
                <span className="kbd">↑↓</span> navigate
              </span>
              <span>
                <span className="kbd">↵</span> open
              </span>
              <Link
                className="ml-auto hover:underline"
                href={`/${repository.owner_login}/${repository.name}`}
                style={{ color: "var(--accent)" }}
              >
                Back to code
              </Link>
            </div>
          </div>

          <div
            className="max-h-[64vh] overflow-y-auto"
            id="repo-file-finder-page-results"
            role="listbox"
          >
            {visibleFiles.map((file, index) => {
              const active = index === activeOptionIndex;
              return (
                <Link
                  aria-selected={active}
                  className="list-row block px-4 py-3 focus:outline-none"
                  href={file.href}
                  id={`repo-file-finder-result-${index}`}
                  key={file.path}
                  onMouseEnter={() => setActiveIndex(index)}
                  role="option"
                  style={{
                    background: active ? "var(--surface-2)" : undefined,
                    color: "var(--ink-1)",
                  }}
                >
                  <div className="flex min-w-0 items-center gap-3">
                    <span
                      aria-hidden="true"
                      className="t-mono-sm"
                      style={{ color: "var(--ink-4)" }}
                    >
                      {file.path.includes("/") ? "dir" : "root"}
                    </span>
                    <span className="min-w-0 flex-1 truncate t-mono-sm">
                      {highlightedPath(file.path, file.matchIndexes)}
                    </span>
                    <span className="chip soft">{file.language ?? "File"}</span>
                    <span className="t-xs">{formatBytes(file.byteSize)}</span>
                  </div>
                </Link>
              );
            })}
            {visibleFiles.length === 0 ? (
              <div className="px-4 py-10 text-center" role="status">
                <p className="t-h3" style={{ color: "var(--ink-1)" }}>
                  No matching files
                </p>
                <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                  Try a shorter path fragment or clear the query to browse the
                  cached file list.
                </p>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </RepositoryShell>
  );
}
