"use client";

import Link from "next/link";
import { useEffect, useMemo, useState, useTransition } from "react";
import type {
  ListEnvelope,
  ReleaseAsset,
  ReleaseReactionContent,
  ReleaseReactionSummary,
  RepositoryOverview,
  RepositoryRefSummary,
  RepositoryReleaseSummary,
} from "@/lib/api";
import { repositoryCompareRangeHref } from "@/lib/navigation";

type RepositoryReleaseInteractionsProps = {
  authenticated: boolean;
  release: RepositoryReleaseSummary;
  repository: RepositoryOverview;
};

const reactionOptions: {
  content: ReleaseReactionContent;
  label: string;
  value: keyof Omit<ReleaseReactionSummary, "totalCount" | "viewerReaction">;
}[] = [
  { content: "thumbs_up", label: "+1", value: "thumbsUp" },
  { content: "heart", label: "heart", value: "heart" },
  { content: "rocket", label: "rocket", value: "rocket" },
  { content: "eyes", label: "eyes", value: "eyes" },
];

function basePath(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}`;
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  const units = ["KB", "MB", "GB"];
  let size = value / 1024;
  let unit = units[0];
  for (const candidate of units) {
    unit = candidate;
    if (size < 1024 || candidate === units.at(-1)) break;
    size /= 1024;
  }
  return `${size >= 10 ? size.toFixed(0) : size.toFixed(1)} ${unit}`;
}

function sourceArchiveLinks(release: RepositoryReleaseSummary) {
  return [
    ["Source code (zip)", release.links.zipballHref],
    ["Source code (tar.gz)", release.links.tarballHref],
  ] as const;
}

function AssetsDisclosure({
  assets,
  repository,
  release,
}: {
  assets: ReleaseAsset[];
  repository: RepositoryOverview;
  release: RepositoryReleaseSummary;
}) {
  return (
    <details className="mt-4 rounded-[var(--radius)] border border-[var(--line)] bg-[var(--surface)]">
      <summary className="cursor-pointer px-3 py-2 t-sm font-medium">
        Assets <span className="t-num">{assets.length + 2}</span>
      </summary>
      <div
        className="border-t px-3 py-2"
        style={{ borderColor: "var(--line)" }}
      >
        <ul className="space-y-2">
          {assets.map((asset) => (
            <li className="flex flex-wrap items-center gap-2" key={asset.id}>
              <Link
                className="t-mono-sm hover:underline"
                href={`${basePath(repository)}/releases/assets/${asset.id}`}
              >
                {asset.name}
              </Link>
              <span className="t-xs">{formatBytes(asset.byteSize)}</span>
              <span className="t-xs t-num">
                {asset.downloadCount} downloads
              </span>
            </li>
          ))}
          {assets.length === 0 ? (
            <li className="t-xs">No uploaded release assets yet.</li>
          ) : null}
          {sourceArchiveLinks(release).map(([label, href]) => (
            <li className="flex flex-wrap items-center gap-2" key={href}>
              <Link className="t-mono-sm hover:underline" href={href}>
                {label}
              </Link>
              <span className="t-xs">Generated from {release.tagName}</span>
            </li>
          ))}
        </ul>
      </div>
    </details>
  );
}

function ReactionBar({
  authenticated,
  release,
  repository,
}: RepositoryReleaseInteractionsProps) {
  const [reactions, setReactions] = useState(release.reactions);
  const [message, setMessage] = useState(
    authenticated ? "" : "Sign in to react to this release.",
  );
  const [pendingReaction, setPendingReaction] =
    useState<ReleaseReactionContent | null>(null);

  async function toggleReaction(content: ReleaseReactionContent) {
    if (!authenticated) {
      setMessage("Sign in to react to this release.");
      return;
    }
    const previous = reactions;
    setPendingReaction(content);
    setMessage("");
    try {
      const response = await fetch(
        `${basePath(repository)}/releases/reactions`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ content, releaseId: release.id }),
        },
      );
      const payload = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(
          payload?.error?.message ?? "Release reaction could not be updated.",
        );
      }
      setReactions(payload as ReleaseReactionSummary);
      setMessage("Reaction updated.");
    } catch (error) {
      setReactions(previous);
      setMessage(
        error instanceof Error
          ? error.message
          : "Release reaction could not be updated.",
      );
    } finally {
      setPendingReaction(null);
    }
  }

  return (
    <div className="mt-4">
      <fieldset aria-label="Release reactions" className="flex flex-wrap gap-2">
        {reactionOptions.map(({ content, label, value }) => {
          const active = reactions.viewerReaction === content;
          return (
            <button
              aria-pressed={active}
              className={`chip ${active ? "active" : "soft"}`}
              disabled={pendingReaction !== null}
              key={content}
              onClick={() => void toggleReaction(content)}
              type="button"
            >
              {label} <span className="t-num">{reactions[value]}</span>
            </button>
          );
        })}
      </fieldset>
      {message ? (
        <p className="t-xs mt-2" role="status">
          {message}
        </p>
      ) : null}
    </div>
  );
}

function CompareSelector({
  release,
  repository,
}: {
  release: RepositoryReleaseSummary;
  repository: RepositoryOverview;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [refs, setRefs] = useState<RepositoryRefSummary[]>([]);
  const [isPending, startTransition] = useTransition();
  const base = basePath(repository);

  useEffect(() => {
    if (!open) {
      return;
    }
    startTransition(async () => {
      try {
        const params = new URLSearchParams({
          activeRef: repository.default_branch,
          pageSize: "100",
        });
        if (query.trim()) {
          params.set("q", query.trim());
        }
        const response = await fetch(`${base}/refs?${params.toString()}`);
        if (!response.ok) return;
        const envelope =
          (await response.json()) as ListEnvelope<RepositoryRefSummary>;
        setRefs(envelope.items);
      } catch {
        setRefs([]);
      }
    });
  }, [base, open, query, repository.default_branch]);

  const visibleRefs = useMemo(
    () =>
      refs
        .filter((ref) => ref.kind === "branch" || ref.kind === "tag")
        .filter((ref) =>
          query.trim()
            ? ref.shortName.toLowerCase().includes(query.trim().toLowerCase())
            : true,
        )
        .slice(0, 12),
    [query, refs],
  );

  return (
    <div className="relative">
      <button
        aria-expanded={open}
        className="btn sm"
        onClick={() => setOpen((current) => !current)}
        type="button"
      >
        Compare
      </button>
      {open ? (
        <div
          className="absolute left-0 z-20 mt-2 w-80 overflow-hidden rounded-md text-sm max-sm:w-[calc(100vw-3rem)]"
          style={{
            border: "1px solid var(--line)",
            background: "var(--surface)",
            boxShadow: "var(--shadow-md)",
          }}
        >
          <label className="sr-only" htmlFor={`release-compare-${release.id}`}>
            Search branches and tags to compare
          </label>
          <input
            aria-label="Search branches and tags to compare"
            className="input h-10 w-full border-b px-3 outline-none"
            id={`release-compare-${release.id}`}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Find a branch or tag"
            style={{ borderColor: "var(--line)" }}
            value={query}
          />
          <div className="max-h-72 overflow-y-auto">
            {visibleRefs.map((ref) => (
              <Link
                className="flex items-center justify-between gap-3 px-3 py-2 hover:bg-[var(--surface-2)]"
                href={repositoryCompareRangeHref(
                  repository.owner_login,
                  repository.name,
                  release.tagName,
                  ref.shortName,
                )}
                key={`${ref.kind}-${ref.name}`}
              >
                <span className="truncate t-mono-sm">{ref.shortName}</span>
                <span className="t-xs capitalize">{ref.kind}</span>
              </Link>
            ))}
            {visibleRefs.length === 0 ? (
              <p className="px-3 py-3 t-xs">
                {isPending ? "Loading refs..." : "No matching refs"}
              </p>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}

export function RepositoryReleaseInteractions(
  props: RepositoryReleaseInteractionsProps,
) {
  return (
    <>
      <AssetsDisclosure
        assets={props.release.assets}
        release={props.release}
        repository={props.repository}
      />
      <div className="mt-4 flex flex-wrap gap-2">
        {sourceArchiveLinks(props.release).map(([label, href]) => (
          <Link className="btn sm" href={href} key={href}>
            {label}
          </Link>
        ))}
        <CompareSelector
          release={props.release}
          repository={props.repository}
        />
      </div>
      <ReactionBar {...props} />
    </>
  );
}
