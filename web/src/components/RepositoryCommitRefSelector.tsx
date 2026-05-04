"use client";

import Link from "next/link";
import { useEffect, useMemo, useState, useTransition } from "react";
import type { ListEnvelope, RepositoryRefSummary } from "@/lib/api";
import { repositoryCommitHistoryHref } from "@/lib/navigation";

type RepositoryCommitRefSelectorProps = {
  owner: string;
  repo: string;
  activeRef: string;
  defaultBranch: string;
  path: string | null;
  author: string | null;
  until: string | null;
};

function refKindLabel(kind: "branch" | "tag") {
  return kind === "branch" ? "Branches" : "Tags";
}

function refCommitHref({
  owner,
  repo,
  ref,
  path,
  author,
  until,
}: {
  owner: string;
  repo: string;
  ref: RepositoryRefSummary;
  path: string | null;
  author: string | null;
  until: string | null;
}) {
  return repositoryCommitHistoryHref({
    owner,
    repo,
    refName: ref.shortName,
    path,
    author,
    until,
  });
}

export function RepositoryCommitRefSelector({
  owner,
  repo,
  activeRef,
  defaultBranch,
  path,
  author,
  until,
}: RepositoryCommitRefSelectorProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedKind, setSelectedKind] = useState<"branch" | "tag">("branch");
  const [refs, setRefs] = useState<RepositoryRefSummary[]>([]);
  const [isPending, startTransition] = useTransition();
  const base = `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`;

  useEffect(() => {
    if (!open && refs.length > 0 && query === "") {
      return;
    }

    const controller = new AbortController();
    startTransition(async () => {
      try {
        const params = new URLSearchParams({
          activeRef,
          pageSize: "100",
        });
        if (query.trim()) {
          params.set("q", query.trim());
        }
        if (path?.trim()) {
          params.set("currentPath", path.trim());
        }
        const response = await fetch(`${base}/refs?${params.toString()}`, {
          signal: controller.signal,
        });
        if (!response.ok) {
          return;
        }
        const envelope =
          (await response.json()) as ListEnvelope<RepositoryRefSummary>;
        setRefs(envelope.items);
      } catch {
        if (!controller.signal.aborted) {
          setRefs([]);
        }
      }
    });

    return () => controller.abort();
  }, [activeRef, base, open, path, query, refs.length]);

  const branches = useMemo(
    () => refs.filter((ref) => ref.kind === "branch"),
    [refs],
  );
  const tags = useMemo(() => refs.filter((ref) => ref.kind === "tag"), [refs]);
  const visibleRefs = selectedKind === "branch" ? branches : tags;
  const selectedRefs = visibleRefs.filter((ref) => ref.active);

  return (
    <details
      className="relative"
      onToggle={(event) => setOpen(event.currentTarget.open)}
    >
      <summary
        aria-label={`Switch branches or tags. Current ref ${activeRef}`}
        className="btn sm inline-flex cursor-pointer list-none items-center gap-2"
      >
        <span aria-hidden="true">Ref</span>
        <span className="max-w-40 truncate">{activeRef}</span>
      </summary>
      <div
        className="absolute left-0 z-30 mt-2 w-96 overflow-hidden rounded-md text-sm max-sm:w-[calc(100vw-3rem)]"
        role="dialog"
        aria-label="Switch branches or tags"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--line)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <div
          className="border-b px-3 py-2"
          style={{ borderColor: "var(--line)" }}
        >
          <p className="font-semibold" style={{ color: "var(--ink-1)" }}>
            Switch branches/tags
          </p>
          <p className="t-xs">Commit history reloads for the selected ref.</p>
        </div>
        <label className="sr-only" htmlFor="commit-ref-search">
          Find a branch or tag
        </label>
        <input
          aria-label="Find a branch or tag"
          className="input h-10 w-full rounded-none border-0 border-b px-3"
          id="commit-ref-search"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Find a branch or tag"
          value={query}
        />
        <div
          className="grid grid-cols-2 border-b"
          style={{ borderColor: "var(--line)" }}
        >
          {(["branch", "tag"] as const).map((kind) => {
            const active = selectedKind === kind;
            const count = kind === "branch" ? branches.length : tags.length;
            return (
              <button
                aria-pressed={active}
                className="px-3 py-2 text-left font-semibold"
                key={kind}
                onClick={() => setSelectedKind(kind)}
                style={{
                  borderBottom: active
                    ? "2px solid var(--accent)"
                    : "2px solid transparent",
                  color: active ? "var(--ink-1)" : "var(--ink-3)",
                }}
                type="button"
              >
                {refKindLabel(kind)}{" "}
                <span className="t-num" style={{ color: "var(--ink-4)" }}>
                  {count}
                </span>
              </button>
            );
          })}
        </div>
        <div className="max-h-80 overflow-y-auto py-1">
          {visibleRefs.map((ref) => {
            const isDefault =
              ref.kind === "branch" && ref.shortName === defaultBranch;
            return (
              <Link
                aria-current={ref.active ? "page" : undefined}
                className="flex items-center gap-3 px-3 py-2 hover:bg-[var(--surface-2)]"
                href={refCommitHref({
                  owner,
                  repo,
                  ref,
                  path,
                  author,
                  until,
                })}
                key={ref.name}
                role="menuitemradio"
                aria-checked={ref.active}
                style={{ color: "var(--ink-1)" }}
              >
                <span aria-hidden="true">{ref.active ? "*" : ""}</span>
                <span className="min-w-0 flex-1 truncate">{ref.shortName}</span>
                {isDefault ? <span className="chip soft">Default</span> : null}
                {ref.active ? (
                  <span className="chip active">Selected</span>
                ) : null}
                {ref.targetShortOid ? (
                  <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                    {ref.targetShortOid}
                  </span>
                ) : null}
              </Link>
            );
          })}
          {visibleRefs.length === 0 ? (
            <p className="px-3 py-4 t-sm" style={{ color: "var(--ink-3)" }}>
              {isPending
                ? "Loading refs..."
                : `No matching ${refKindLabel(selectedKind).toLowerCase()}.`}
            </p>
          ) : null}
        </div>
        <div
          className="flex items-center justify-between gap-3 border-t px-3 py-2"
          style={{ borderColor: "var(--line)" }}
        >
          <Link className="t-sm hover:underline" href={`${base}/branches`}>
            View all branches
          </Link>
          <span className="t-xs">
            {selectedRefs.length > 0 ? "Current ref selected" : "Choose a ref"}
          </span>
        </div>
      </div>
    </details>
  );
}
