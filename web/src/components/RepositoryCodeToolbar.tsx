"use client";

import Link from "next/link";
import { useEffect, useMemo, useState, useTransition } from "react";
import type {
  ListEnvelope,
  RepositoryFile,
  RepositoryFileFinderItem,
  RepositoryOverview,
  RepositoryRefSummary,
  RepositorySummary,
} from "@/lib/api";

type RepositoryToolbarTarget = Pick<
  RepositorySummary,
  "owner_login" | "name" | "default_branch"
> & {
  files?: RepositoryFile[];
};

type RepositoryCodeToolbarProps = {
  repository: RepositoryOverview;
};

type RepositoryToolbarTargetProps = {
  repository: RepositoryToolbarTarget;
};

type RepositoryRefControlProps = RepositoryToolbarTargetProps & {
  activeRef?: string;
  currentPath?: string;
};

type RepositoryFileFinderProps = RepositoryRefControlProps;

function formatCount(value: number, label: string) {
  return `${new Intl.NumberFormat("en").format(value)} ${label}`;
}

function basePath(repository: RepositoryToolbarTarget) {
  return `/${repository.owner_login}/${repository.name}`;
}

export function RepositoryBranchSelector({
  repository,
  activeRef,
  currentPath = "",
}: RepositoryRefControlProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedKind, setSelectedKind] = useState<"branch" | "tag">("branch");
  const [refs, setRefs] = useState<RepositoryRefSummary[]>([]);
  const [isPending, startTransition] = useTransition();
  const base = basePath(repository);
  const currentRef = activeRef ?? repository.default_branch;

  useEffect(() => {
    if (!open && refs.length > 0 && query === "") {
      return;
    }
    startTransition(async () => {
      try {
        const params = new URLSearchParams({
          activeRef: currentRef,
          pageSize: "100",
        });
        if (query.trim()) {
          params.set("q", query.trim());
        }
        if (currentPath.trim()) {
          params.set("currentPath", currentPath.trim());
        }
        const response = await fetch(`${base}/refs?${params.toString()}`);
        if (!response.ok) {
          return;
        }
        const envelope =
          (await response.json()) as ListEnvelope<RepositoryRefSummary>;
        setRefs(envelope.items);
      } catch {
        setRefs([]);
      }
    });
  }, [base, currentPath, currentRef, open, query, refs.length]);

  const branches = refs.filter((ref) => ref.kind === "branch");
  const tags = refs.filter((ref) => ref.kind === "tag");
  const visibleRefs = selectedKind === "branch" ? branches : tags;

  return (
    <details
      className="relative"
      onToggle={(event) => setOpen(event.currentTarget.open)}
    >
      <summary
        aria-label={`Switch branches or tags. Current ref ${currentRef}`}
        className="btn sm inline-flex cursor-pointer list-none items-center gap-2"
      >
        <span aria-hidden="true">⑂</span>
        {currentRef}
      </summary>
      <div
        className="absolute left-0 z-20 mt-2 w-80 overflow-hidden rounded-md text-sm max-sm:w-[calc(100vw-3rem)]"
        style={{
          border: "1px solid var(--line)",
          background: "var(--surface)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <div
          className="border-b px-3 py-2 font-semibold"
          style={{ borderColor: "var(--line)", color: "var(--ink-1)" }}
        >
          Switch branches/tags
        </div>
        <label className="sr-only" htmlFor="repository-ref-search">
          Search branches and tags
        </label>
        <input
          aria-label="Search branches and tags"
          className="input h-10 w-full border-b px-3 outline-none"
          style={{ borderColor: "var(--line)" }}
          id="repository-ref-search"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Find a branch or tag"
          value={query}
        />
        <div
          className="grid grid-cols-2 border-b text-sm font-semibold"
          style={{ borderColor: "var(--line)" }}
        >
          <button
            aria-pressed={selectedKind === "branch"}
            className={`px-3 py-2 ${
              selectedKind === "branch"
                ? "border-b-2 border-[var(--accent)]"
                : "hover:bg-[var(--surface-2)]"
            }`}
            style={{
              color:
                selectedKind === "branch" ? "var(--ink-1)" : "var(--ink-3)",
            }}
            onClick={() => setSelectedKind("branch")}
            type="button"
          >
            Branches {branches.length}
          </button>
          <button
            aria-pressed={selectedKind === "tag"}
            className={`px-3 py-2 ${
              selectedKind === "tag"
                ? "border-b-2 border-[var(--accent)]"
                : "hover:bg-[var(--surface-2)]"
            }`}
            style={{
              color: selectedKind === "tag" ? "var(--ink-1)" : "var(--ink-3)",
            }}
            onClick={() => setSelectedKind("tag")}
            type="button"
          >
            Tags {tags.length}
          </button>
        </div>
        <div className="max-h-80 overflow-y-auto">
          <div className="px-3 py-2 t-label" style={{ color: "var(--ink-3)" }}>
            {selectedKind === "branch" ? "Branches" : "Tags"}
          </div>
          {visibleRefs.map((ref) => (
            <Link
              className="flex items-center justify-between gap-3 px-3 py-2 hover:bg-[var(--surface-2)]"
              href={ref.samePathHref ?? ref.href}
              key={ref.name}
              style={{ color: "var(--ink-1)" }}
            >
              <span className="truncate">{ref.shortName}</span>
              {ref.active ? (
                <span className="t-xs" style={{ color: "var(--ok)" }}>
                  Current
                </span>
              ) : ref.targetShortOid ? (
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {ref.targetShortOid}
                </span>
              ) : null}
            </Link>
          ))}
          {visibleRefs.length === 0 ? (
            <p className="px-3 py-3" style={{ color: "var(--ink-3)" }}>
              {isPending ? "Loading refs..." : "No matching refs"}
            </p>
          ) : null}
        </div>
        {isPending && open ? (
          <p
            className="border-t px-3 py-2 t-xs"
            style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
          >
            Loading refs...
          </p>
        ) : null}
      </div>
    </details>
  );
}

export function RepositoryFileFinder({
  repository,
  activeRef,
}: RepositoryFileFinderProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<RepositoryFileFinderItem[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [isPending, startTransition] = useTransition();
  const base = basePath(repository);
  const files = repository.files ?? [];
  const currentRef = activeRef ?? repository.default_branch;

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      const tagName = target?.tagName?.toLowerCase();
      const isTyping =
        tagName === "input" ||
        tagName === "textarea" ||
        target?.isContentEditable;
      if (isTyping || event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }
      if (event.key.toLowerCase() === "t") {
        event.preventDefault();
        window.location.assign(
          `${base}/find/${encodeURIComponent(currentRef)}`,
        );
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [base, currentRef]);

  useEffect(() => {
    if (!open) {
      return;
    }

    const controller = new AbortController();
    const normalizedQuery = query.trim().toLowerCase();
    setItems(
      files
        .filter(
          (file) =>
            !normalizedQuery ||
            file.path.toLowerCase().includes(normalizedQuery),
        )
        .slice(0, 20)
        .map((file) => {
          const name = file.path.split("/").at(-1) ?? file.path;
          return {
            path: file.path,
            name,
            kind: "file",
            href: `${base}/blob/${encodeURIComponent(currentRef)}/${file.path
              .split("/")
              .map(encodeURIComponent)
              .join("/")}`,
            byteSize: file.byteSize,
            language: null,
          };
        }),
    );

    startTransition(async () => {
      try {
        const params = new URLSearchParams({
          ref: currentRef,
          q: query,
        });
        const response = await fetch(`${base}/file-finder?${params}`, {
          signal: controller.signal,
        });
        if (!response.ok) {
          if (!controller.signal.aborted) {
            setItems([]);
          }
          return;
        }
        const envelope =
          (await response.json()) as ListEnvelope<RepositoryFileFinderItem>;
        if (!controller.signal.aborted) {
          setItems(envelope.items);
          setActiveIndex(0);
        }
      } catch {
        if (!controller.signal.aborted) {
          setItems([]);
        }
      }
    });

    return () => controller.abort();
  }, [base, currentRef, files, open, query]);

  function openActiveItem() {
    const item = items[activeIndex];
    if (item) {
      window.location.assign(item.href);
    }
  }

  return (
    <div className="relative">
      <button
        className="btn sm"
        onClick={() => setOpen((value) => !value)}
        type="button"
      >
        Go to file
      </button>
      {open ? (
        <div
          className="absolute right-0 z-20 mt-2 w-96 overflow-hidden rounded-md text-sm max-sm:right-auto max-sm:left-0 max-sm:w-[calc(100vw-3rem)]"
          style={{
            border: "1px solid var(--line)",
            background: "var(--surface)",
            boxShadow: "var(--shadow-md)",
          }}
        >
          <label className="sr-only" htmlFor="repository-file-finder">
            Find a file
          </label>
          <input
            aria-label="Find a file"
            className="input h-10 w-full border-b px-3 outline-none"
            style={{ borderColor: "var(--line)" }}
            id="repository-file-finder"
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "ArrowDown") {
                event.preventDefault();
                setActiveIndex((index) =>
                  Math.min(items.length - 1, index + 1),
                );
              }
              if (event.key === "ArrowUp") {
                event.preventDefault();
                setActiveIndex((index) => Math.max(0, index - 1));
              }
              if (event.key === "Enter") {
                event.preventDefault();
                openActiveItem();
              }
              if (event.key === "Escape") {
                setOpen(false);
              }
            }}
            placeholder="Type to search files"
            value={query}
          />
          <ul
            className="max-h-80 overflow-y-auto py-1"
            id="repository-file-finder-results"
          >
            {items.map((item, index) => (
              <li id={`file-finder-${index}`} key={item.path}>
                <Link
                  aria-current={index === activeIndex ? "true" : undefined}
                  className={`block px-3 py-2 hover:bg-[var(--surface-2)] ${
                    index === activeIndex ? "bg-[var(--surface-2)]" : ""
                  }`}
                  href={item.href}
                >
                  <span
                    className="block truncate font-semibold"
                    style={{ color: "var(--accent)" }}
                  >
                    {item.path}
                  </span>
                  <span className="t-xs" style={{ color: "var(--ink-3)" }}>
                    {item.language ?? "File"} · {item.byteSize} bytes
                  </span>
                </Link>
              </li>
            ))}
          </ul>
          {items.length === 0 ? (
            <p className="px-3 py-3" style={{ color: "var(--ink-3)" }}>
              {isPending ? "Searching files..." : "No matching files"}
            </p>
          ) : null}
          <div
            className="border-t px-3 py-2"
            style={{ borderColor: "var(--line)" }}
          >
            <Link
              className="t-xs hover:underline"
              href={`${base}/find/${encodeURIComponent(currentRef)}`}
              style={{ color: "var(--accent)" }}
            >
              Open full file finder
            </Link>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function AddFileMenu({ repository }: RepositoryCodeToolbarProps) {
  const base = basePath(repository);
  return (
    <details className="relative">
      <summary className="btn sm inline-flex cursor-pointer list-none items-center">
        Add file
      </summary>
      <div
        className="absolute right-0 z-20 mt-2 w-56 overflow-hidden rounded-md py-1 text-sm"
        style={{
          border: "1px solid var(--line)",
          background: "var(--surface)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <Link
          className="block px-3 py-2 hover:bg-[var(--surface-2)]"
          href={`${base}/new/${repository.default_branch}`}
          style={{ color: "var(--ink-1)" }}
        >
          Create new file
        </Link>
        <Link
          className="block px-3 py-2 hover:bg-[var(--surface-2)]"
          href={`${base}/upload/${repository.default_branch}`}
          style={{ color: "var(--ink-1)" }}
        >
          Upload files
        </Link>
      </div>
    </details>
  );
}

function CloneMenu({ repository }: RepositoryCodeToolbarProps) {
  const [copied, setCopied] = useState<string | null>(null);

  async function copy(value: string, label: string) {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(`${label} copied`);
    } catch {
      setCopied("Copy unavailable");
    }
  }

  return (
    <details className="relative">
      <summary className="btn primary inline-flex cursor-pointer list-none items-center">
        Code
      </summary>
      <div
        className="absolute right-0 z-20 mt-2 w-80 rounded-md p-3 text-sm max-sm:w-[calc(100vw-3rem)]"
        style={{
          border: "1px solid var(--line)",
          background: "var(--surface)",
          color: "var(--ink-1)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <p className="font-semibold">Clone</p>
        <div className="mt-3">
          <label
            className="block t-label"
            htmlFor="clone-https"
            style={{ color: "var(--ink-3)" }}
          >
            HTTPS
          </label>
          <div className="mt-1 flex">
            <input
              className="input min-w-0 flex-1 rounded-r-none border-r-0 px-2 t-mono-sm"
              id="clone-https"
              readOnly
              value={repository.cloneUrls.https}
            />
            <button
              className="btn sm rounded-l-none"
              onClick={() => copy(repository.cloneUrls.https, "HTTPS")}
              type="button"
            >
              Copy
            </button>
          </div>
        </div>
        {copied ? (
          <p className="mt-2 t-xs" role="status" style={{ color: "var(--ok)" }}>
            {copied}
          </p>
        ) : null}
        <Link
          className="mt-3 block hover:underline"
          href={repository.cloneUrls.zip}
          style={{ color: "var(--accent)" }}
        >
          Download ZIP
        </Link>
      </div>
    </details>
  );
}

export function RepositoryCodeToolbar({
  repository,
}: RepositoryCodeToolbarProps) {
  const base = useMemo(() => basePath(repository), [repository]);

  return (
    <div className="flex flex-wrap items-center gap-3">
      <span className="t-sm" style={{ color: "var(--ink-3)" }}>
        Default branch
      </span>
      <Link
        className="t-sm font-semibold hover:underline"
        href={`${base}/tree/${repository.default_branch}`}
        style={{ color: "var(--accent)" }}
      >
        {repository.default_branch}
      </Link>
      <RepositoryBranchSelector repository={repository} />
      <Link
        className="t-sm hover:underline"
        href={`${base}/branches`}
        style={{ color: "var(--ink-3)" }}
      >
        {formatCount(repository.branchCount, "Branches")}
      </Link>
      <Link
        className="t-sm hover:underline"
        href={`${base}/tags`}
        style={{ color: "var(--ink-3)" }}
      >
        {formatCount(repository.tagCount, "Tags")}
      </Link>
      <div className="ml-auto flex flex-wrap items-center gap-2 max-md:ml-0">
        <RepositoryFileFinder repository={repository} />
        <AddFileMenu repository={repository} />
        <CloneMenu repository={repository} />
      </div>
    </div>
  );
}
