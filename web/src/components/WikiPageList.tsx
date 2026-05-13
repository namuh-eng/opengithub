"use client";

import Link from "next/link";
import { useId, useState } from "react";
import type {
  RepositoryWikiHeading,
  RepositoryWikiPageSummary,
} from "@/lib/api";

type WikiPageListProps = {
  owner: string;
  repo: string;
  pages: RepositoryWikiPageSummary[];
  currentOutline: RepositoryWikiHeading[];
};

type TocState =
  | { status: "idle"; outline: RepositoryWikiHeading[] | null; message: null }
  | {
      status: "loading";
      outline: RepositoryWikiHeading[] | null;
      message: null;
    }
  | { status: "ready"; outline: RepositoryWikiHeading[]; message: null }
  | { status: "error"; outline: null; message: string };

type TocResponse = {
  outline?: RepositoryWikiHeading[];
  error?: { message?: string };
};

function wikiPageHref(
  owner: string,
  repo: string,
  page: RepositoryWikiPageSummary,
) {
  const repositoryWikiPrefix = `/${owner}/${repo}/wiki`;
  if (page.href.startsWith(repositoryWikiPrefix)) {
    return page.href;
  }
  const encodedSlug = page.slug
    .split("/")
    .filter(Boolean)
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  return `${repositoryWikiPrefix}/${encodedSlug}`;
}

function pageTocEndpoint(owner: string, repo: string, slug: string) {
  const encodedSlug = slug
    .split("/")
    .filter(Boolean)
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki-toc/${encodedSlug}`;
}

function Chevron({ expanded }: { expanded: boolean }) {
  return (
    <svg
      aria-hidden="true"
      fill="none"
      height="14"
      viewBox="0 0 14 14"
      width="14"
    >
      <path
        d={expanded ? "M3.5 5.25 7 8.75l3.5-3.5" : "M5.25 3.5 8.75 7l-3.5 3.5"}
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.5"
      />
    </svg>
  );
}

function OutlineLinks({
  headings,
  labelledBy,
}: {
  headings: RepositoryWikiHeading[];
  labelledBy: string;
}) {
  if (headings.length === 0) {
    return (
      <p className="t-xs px-2 pb-2" style={{ color: "var(--ink-4)" }}>
        No headings on this page.
      </p>
    );
  }

  return (
    <nav aria-labelledby={labelledBy} className="grid gap-1 px-2 pb-2">
      {headings.map((heading) => (
        <Link
          className="t-xs rounded-md py-1 hover:underline"
          href={heading.href}
          key={heading.id}
          style={{
            color: "var(--ink-3)",
            paddingLeft: `${Math.max(heading.level - 1, 0) * 10}px`,
          }}
        >
          {heading.text}
        </Link>
      ))}
    </nav>
  );
}

export function WikiPageList({
  owner,
  repo,
  pages,
  currentOutline,
}: WikiPageListProps) {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [showAllPages, setShowAllPages] = useState(false);
  const [tocBySlug, setTocBySlug] = useState<Record<string, TocState>>({});
  const labelPrefix = useId();
  const visiblePages = showAllPages ? pages : pages.slice(0, 8);
  const hiddenPageCount = Math.max(pages.length - visiblePages.length, 0);

  async function togglePage(page: RepositoryWikiPageSummary) {
    const willExpand = !expanded[page.slug];
    setExpanded((current) => ({ ...current, [page.slug]: willExpand }));
    if (
      !willExpand ||
      page.active ||
      tocBySlug[page.slug]?.status === "ready"
    ) {
      return;
    }

    setTocBySlug((current) => ({
      ...current,
      [page.slug]: { status: "loading", outline: null, message: null },
    }));

    try {
      const response = await fetch(pageTocEndpoint(owner, repo, page.slug), {
        headers: { accept: "application/json" },
      });
      const body = (await response.json()) as TocResponse;
      if (!response.ok) {
        throw new Error(
          body.error?.message ?? "Wiki page outline could not load.",
        );
      }
      setTocBySlug((current) => ({
        ...current,
        [page.slug]: {
          status: "ready",
          outline: body.outline ?? [],
          message: null,
        },
      }));
    } catch (error) {
      setTocBySlug((current) => ({
        ...current,
        [page.slug]: {
          status: "error",
          outline: null,
          message:
            error instanceof Error
              ? error.message
              : "Wiki page outline could not load.",
        },
      }));
    }
  }

  return (
    <nav aria-label="Wiki pages" className="mt-3 grid gap-1">
      {visiblePages.map((page) => {
        const panelId = `${labelPrefix}-${page.id}-toc`;
        const labelId = `${labelPrefix}-${page.id}-label`;
        const isExpanded = Boolean(expanded[page.slug]);
        const tocState = page.active
          ? { status: "ready" as const, outline: currentOutline, message: null }
          : (tocBySlug[page.slug] ?? {
              status: "idle" as const,
              outline: null,
              message: null,
            });

        return (
          <div className="grid gap-1" key={page.id}>
            <div
              className="grid items-center rounded-md"
              style={{
                background: page.active ? "var(--accent-soft)" : "transparent",
                gridTemplateColumns: "28px minmax(0,1fr)",
              }}
            >
              <button
                aria-controls={panelId}
                aria-expanded={isExpanded}
                aria-label={`${isExpanded ? "Collapse" : "Expand"} ${page.title} table of contents`}
                className="btn sm ghost"
                disabled={!page.hasOutline && !page.active}
                onClick={() => void togglePage(page)}
                style={{ minWidth: 28, paddingInline: 0 }}
                type="button"
              >
                <Chevron expanded={isExpanded} />
              </button>
              <a
                aria-current={page.active ? "page" : undefined}
                className={`min-w-0 rounded-md px-2 py-2 t-sm hover:underline ${
                  page.active ? "font-semibold" : ""
                }`}
                href={wikiPageHref(owner, repo, page)}
                id={labelId}
                style={{
                  color: page.active ? "var(--ink-1)" : "var(--ink-3)",
                }}
              >
                {page.title}
              </a>
            </div>
            {isExpanded ? (
              <div id={panelId}>
                {tocState.status === "loading" ? (
                  <p
                    className="t-xs px-2 pb-2"
                    style={{ color: "var(--ink-4)" }}
                  >
                    Loading headings...
                  </p>
                ) : tocState.status === "error" ? (
                  <p className="t-xs px-2 pb-2" style={{ color: "var(--err)" }}>
                    {tocState.message}
                  </p>
                ) : tocState.status === "ready" ? (
                  <OutlineLinks
                    headings={tocState.outline}
                    labelledBy={labelId}
                  />
                ) : null}
              </div>
            ) : null}
          </div>
        );
      })}
      {hiddenPageCount > 0 ? (
        <button
          className="btn sm ghost mt-2 justify-self-start"
          onClick={() => setShowAllPages(true)}
          type="button"
        >
          Show {hiddenPageCount} more pages
        </button>
      ) : null}
    </nav>
  );
}
