"use client";

import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type { RepositoryOverview } from "@/lib/api";
import type {
  ReleaseReactionKind,
  ReleaseReactionSummary,
  RepositoryRelease,
  RepositoryReleaseListView,
} from "@/lib/releases";

type RepositoryReleasesPageProps = {
  repository: RepositoryOverview;
  releases: RepositoryReleaseListView;
  mode?: "list" | "detail";
};

const REACTION_GLYPHS: Record<ReleaseReactionKind, string> = {
  heart: "♥",
  hooray: "✦",
  rocket: "↗",
  eyes: "◉",
};

function formatDate(value: string) {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  const units = ["KB", "MB", "GB"];
  let size = value / 1024;
  let unit = units[0];
  for (const nextUnit of units.slice(1)) {
    if (size < 1024) {
      break;
    }
    size /= 1024;
    unit = nextUnit;
  }
  return `${size.toFixed(size >= 10 ? 0 : 1)} ${unit}`;
}

function Avatar({ user }: { user: RepositoryRelease["author"] }) {
  return user.avatarUrl ? (
    <Image
      alt=""
      className="av sm"
      height={24}
      src={user.avatarUrl}
      width={24}
    />
  ) : (
    <span aria-hidden="true" className="av sm">
      {user.login.slice(0, 1).toUpperCase()}
    </span>
  );
}

function ReleaseNotes({
  markdown,
  owner,
  repo,
}: {
  markdown: string;
  owner: string;
  repo: string;
}) {
  const lines = markdown.split("\n");
  const nodes: React.ReactNode[] = [];
  let listItems: string[] = [];

  const flushList = () => {
    if (!listItems.length) {
      return;
    }
    nodes.push(
      <ul
        className="mt-3 list-disc space-y-1 pl-5"
        key={`list-${nodes.length}`}
      >
        {listItems.map((item) => (
          <li key={item}>{renderInline(item, owner, repo)}</li>
        ))}
      </ul>,
    );
    listItems = [];
  };

  lines.forEach((line) => {
    if (line.startsWith("## ")) {
      flushList();
      nodes.push(
        <h3 className="t-h3 mt-5" key={`h-${line}`}>
          {line.slice(3)}
        </h3>,
      );
      return;
    }
    if (line.startsWith("- ")) {
      listItems.push(line.slice(2));
      return;
    }
    if (line.trim()) {
      flushList();
      nodes.push(
        <p className="t-sm mt-3 leading-6" key={`p-${nodes.length}`}>
          {renderInline(line, owner, repo)}
        </p>,
      );
    }
  });
  flushList();

  return <div>{nodes}</div>;
}

function renderInline(value: string, owner: string, repo: string) {
  let offset = 0;
  return value.split(/(PR #[0-9]+)/g).map((part) => {
    const key = `${offset}:${part}`;
    offset += part.length;
    const match = /^PR #([0-9]+)$/.exec(part);
    if (!match) {
      return <span key={key}>{part}</span>;
    }
    return (
      <Link
        className="underline decoration-dotted"
        href={`/${owner}/${repo}/pull/${match[1]}`}
        key={key}
      >
        {part}
      </Link>
    );
  });
}

function ReactionBar({
  release,
  owner,
  repo,
  viewerCanReact,
  signInHref,
}: {
  release: RepositoryRelease;
  owner: string;
  repo: string;
  viewerCanReact: boolean;
  signInHref: string;
}) {
  const [reactions, setReactions] = useState(release.reactions);
  const [pendingKind, setPendingKind] = useState<ReleaseReactionKind | null>(
    null,
  );
  async function toggle(kind: ReleaseReactionKind) {
    if (!viewerCanReact || pendingKind) {
      return;
    }
    setPendingKind(kind);
    setReactions((current) =>
      current.map((reaction) =>
        reaction.kind === kind
          ? {
              ...reaction,
              count: reaction.count + (reaction.viewerReacted ? -1 : 1),
              viewerReacted: !reaction.viewerReacted,
            }
          : reaction,
      ),
    );
    const response = await fetch(
      `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/releases/reactions`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ releaseId: release.id, kind }),
      },
    );
    if (response.ok) {
      const body = (await response.json()) as {
        reactions: ReleaseReactionSummary[];
      };
      setReactions(body.reactions);
    }
    setPendingKind(null);
  }

  return (
    <div className="mt-5 flex flex-wrap items-center gap-2">
      {reactions.map((reaction) => (
        <button
          aria-label={`${reaction.label} ${reaction.count}`}
          aria-pressed={reaction.viewerReacted}
          className={`chip ${reaction.viewerReacted ? "accent" : "soft"}`}
          disabled={!viewerCanReact || pendingKind === reaction.kind}
          key={reaction.kind}
          onClick={() => toggle(reaction.kind)}
          title={viewerCanReact ? reaction.label : "Sign in to react"}
          type="button"
        >
          <span aria-hidden="true">{REACTION_GLYPHS[reaction.kind]}</span>
          <span>{reaction.label}</span>
          <span className="t-num">{reaction.count}</span>
        </button>
      ))}
      {!viewerCanReact ? (
        <Link
          className="t-xs underline decoration-dotted"
          href={signInHref}
          style={{ color: "var(--ink-3)" }}
        >
          Sign in to react
        </Link>
      ) : null}
    </div>
  );
}

function ReleaseCard({
  release,
  owner,
  repo,
  viewerCanReact,
  signInHref,
}: {
  release: RepositoryRelease;
  owner: string;
  repo: string;
  viewerCanReact: boolean;
  signInHref: string;
}) {
  return (
    <article className="card overflow-hidden" data-testid="release-card">
      <div className="grid grid-cols-[180px_minmax(0,1fr)] gap-0 max-md:grid-cols-1">
        <aside
          className="border-r p-5 max-md:border-r-0 max-md:border-b"
          style={{ borderColor: "var(--line)" }}
        >
          <Link
            className="t-mono-sm hover:underline"
            href={release.tagHref}
            style={{ color: "var(--ink-1)" }}
          >
            {release.tagName}
          </Link>
          <p className="t-xs mt-2" style={{ color: "var(--ink-3)" }}>
            Published {formatDate(release.publishedAt)}
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            {release.latest ? (
              <span className="chip accent">Latest</span>
            ) : null}
            {release.prerelease ? (
              <span className="chip warn">Pre-release</span>
            ) : null}
            {release.targetCommit.verified ? (
              <span className="chip ok">Verified</span>
            ) : null}
          </div>
          <details className="mt-5">
            <summary className="btn sm inline-flex cursor-pointer">
              Compare
            </summary>
            <form
              action={`/${owner}/${repo}/compare`}
              className="mt-3 space-y-2"
            >
              <input name="base" type="hidden" value={release.tagName} />
              <label
                className="t-xs block"
                htmlFor={`compare-head-${release.id}`}
                style={{ color: "var(--ink-3)" }}
              >
                Search branch or tag
              </label>
              <input
                className="input w-full"
                defaultValue="main"
                id={`compare-head-${release.id}`}
                name="head"
                type="search"
              />
              <button className="btn sm primary w-full" type="submit">
                Open compare
              </button>
            </form>
          </details>
        </aside>
        <div className="p-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0">
              <h2 className="t-h2" id={`release-${release.id}`}>
                <Link className="hover:underline" href={release.href}>
                  {release.title}
                </Link>
              </h2>
              <div
                className="mt-2 flex flex-wrap items-center gap-2 t-xs"
                style={{ color: "var(--ink-3)" }}
              >
                <Avatar user={release.author} />
                <span>{release.author.login}</span>
                <span aria-hidden="true">·</span>
                <Link
                  className="t-mono-sm hover:underline"
                  href={release.targetCommit.href}
                >
                  {release.targetCommit.shortOid}
                </Link>
              </div>
            </div>
          </div>
          <div
            className="mt-5 border-t pt-1"
            style={{ borderColor: "var(--line-soft)" }}
          >
            <ReleaseNotes
              markdown={release.bodyMarkdown}
              owner={owner}
              repo={repo}
            />
          </div>
          <div className="mt-5 flex flex-wrap items-center gap-2">
            <span className="t-xs" style={{ color: "var(--ink-3)" }}>
              Contributors
            </span>
            <div className="flex -space-x-2">
              {release.contributors.map((contributor) => (
                <span
                  className="rounded-full border"
                  key={contributor.id}
                  style={{ borderColor: "var(--surface)" }}
                >
                  <Avatar user={contributor} />
                </span>
              ))}
            </div>
          </div>
          <details
            className="mt-5 rounded-lg border p-4"
            style={{
              borderColor: "var(--line)",
              background: "var(--surface-2)",
            }}
          >
            <summary className="cursor-pointer t-h3">
              Assets ({release.assets.length + release.archives.length})
            </summary>
            <div
              className="mt-4 divide-y"
              style={{ borderColor: "var(--line-soft)" }}
            >
              {release.assets.map((asset) => (
                <Link
                  className="flex items-center justify-between gap-4 py-3 t-sm hover:underline"
                  href={asset.downloadHref}
                  key={asset.id}
                >
                  <span className="min-w-0 truncate">{asset.name}</span>
                  <span
                    className="shrink-0 t-xs"
                    style={{ color: "var(--ink-3)" }}
                  >
                    {formatBytes(asset.sizeBytes)} ·{" "}
                    {asset.downloadCount.toLocaleString()} downloads
                  </span>
                </Link>
              ))}
              {release.archives.map((archive) => (
                <Link
                  className="flex items-center justify-between gap-4 py-3 t-sm hover:underline"
                  href={archive.href}
                  key={archive.kind}
                >
                  <span>{archive.label}</span>
                  <span className="t-xs" style={{ color: "var(--ink-3)" }}>
                    archive
                  </span>
                </Link>
              ))}
            </div>
          </details>
          <ReactionBar
            release={release}
            owner={owner}
            repo={repo}
            viewerCanReact={viewerCanReact}
            signInHref={signInHref}
          />
        </div>
      </div>
    </article>
  );
}

export function RepositoryReleasesPage({
  repository,
  releases,
  mode = "list",
}: RepositoryReleasesPageProps) {
  const owner = repository.owner_login;
  const repo = repository.name;
  const releaseBase = `/${owner}/${repo}/releases`;
  const latestHref = `${releaseBase}/latest`;

  return (
    <RepositoryShell
      activePath={releaseBase}
      frameClassName="max-w-6xl"
      repository={repository}
    >
      <div className="space-y-6">
        <header className="card p-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Release history
              </p>
              <h1 className="t-h1 mt-2">Releases</h1>
              <p
                className="t-sm mt-2 max-w-2xl leading-6"
                style={{ color: "var(--ink-3)" }}
              >
                Published builds, signed source archives, release notes, assets,
                and community reactions for {owner}/{repo}.
              </p>
            </div>
            {releases.latestRelease ? (
              <Link className="btn primary" href={latestHref}>
                Latest release
              </Link>
            ) : null}
          </div>
          <nav
            aria-label="Release sections"
            className="tabs mt-5 flex gap-2 border-b"
            style={{ borderColor: "var(--line)" }}
          >
            <Link
              className={`tab border-b-2 px-3 py-3 ${mode === "list" ? "active" : ""}`}
              href={releaseBase}
              style={{
                borderColor: mode === "list" ? "var(--accent)" : "transparent",
              }}
            >
              Releases
            </Link>
            <Link
              className="tab border-b-2 px-3 py-3"
              href={`/${owner}/${repo}/tags`}
              style={{ borderColor: "transparent" }}
            >
              Tags
            </Link>
          </nav>
        </header>

        {releases.items.length ? (
          <div className="space-y-5">
            {releases.items.map((release) => (
              <ReleaseCard
                key={release.id}
                release={release}
                owner={owner}
                repo={repo}
                viewerCanReact={releases.viewerCanReact}
                signInHref={releases.signInHref}
              />
            ))}
          </div>
        ) : (
          <section className="card p-6 text-center">
            <h2 className="t-h2">No releases published yet</h2>
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Tags can still be browsed from the repository selector.
            </p>
          </section>
        )}

        {mode === "list" && releases.totalPages > 1 ? (
          <nav
            aria-label="Release pagination"
            className="flex items-center justify-between gap-3"
          >
            {releases.previousHref ? (
              <Link className="btn" href={releases.previousHref}>
                Previous
              </Link>
            ) : (
              <span />
            )}
            <span className="t-sm" style={{ color: "var(--ink-3)" }}>
              Page {releases.page} of {releases.totalPages}
            </span>
            {releases.nextHref ? (
              <Link className="btn" href={releases.nextHref}>
                Next
              </Link>
            ) : (
              <span />
            )}
          </nav>
        ) : null}
      </div>
    </RepositoryShell>
  );
}
