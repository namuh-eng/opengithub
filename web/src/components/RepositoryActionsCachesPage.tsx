"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  RepositoryActionsCaches,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryActionsCachesPageProps = {
  repository: RepositoryOverview;
  detail: RepositoryActionsCaches;
  validationError?: ApiErrorEnvelope | null;
};

function bytesLabel(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  const kib = value / 1024;
  if (kib < 1024) {
    return `${kib.toFixed(kib >= 10 ? 0 : 1)} KB`;
  }
  const mib = kib / 1024;
  if (mib < 1024) {
    return `${mib.toFixed(mib >= 10 ? 0 : 1)} MB`;
  }
  const gib = mib / 1024;
  return `${gib.toFixed(gib >= 10 ? 0 : 1)} GB`;
}

function dateLabel(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) {
    return "Not recorded";
  }
  return date.toLocaleString("en", {
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    month: "short",
    year: "numeric",
  });
}

export function RepositoryActionsCachesPage({
  repository,
  detail,
  validationError,
}: RepositoryActionsCachesPageProps) {
  const router = useRouter();
  const basePath = `/${repository.owner_login}/${repository.name}`;
  const [pendingId, setPendingId] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const usage =
    detail.limitBytes > 0
      ? Math.min(
          100,
          Math.round((detail.totalSizeBytes / detail.limitBytes) * 100),
        )
      : 0;

  async function deleteCache(cacheId: string) {
    setPendingId(cacheId);
    setMessage("");
    try {
      const response = await fetch(`${basePath}/actions/caches/${cacheId}`, {
        method: "DELETE",
        cache: "no-store",
      });
      const body = (await response
        .json()
        .catch(() => null)) as ApiErrorEnvelope | null;
      if (!response.ok || (body && "error" in body)) {
        throw new Error(
          body && "error" in body
            ? body.error.message
            : "Cache could not be deleted.",
        );
      }
      setMessage("Cache deleted.");
      router.refresh();
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Cache could not be deleted.",
      );
    } finally {
      setPendingId(null);
    }
  }

  return (
    <RepositoryShell
      activePath={`${basePath}/actions/caches`}
      frameClassName="max-w-7xl"
      repository={repository}
    >
      <div className="space-y-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <Link
                className="t-sm hover:underline"
                href={`${basePath}/actions`}
              >
                Actions
              </Link>
              <span className="t-xs">/</span>
              <span className="t-sm">Caches</span>
            </div>
            <p className="t-label">Repository Insights</p>
            <h1 className="t-h1 mt-1">Dependency caches</h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Workflow jobs can reserve cache entries for dependency folders and
              restore them by key and version. Oldest entries are evicted after
              the repository reaches the 10 GB cache budget.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link className="btn" href={`${basePath}/actions`}>
              All workflows
            </Link>
            <Link className="btn" href="/docs/api#actions-artifacts-caches">
              API docs
            </Link>
          </div>
        </div>

        {validationError ? (
          <div className="card p-4" role="status">
            <p className="t-label" style={{ color: "var(--err)" }}>
              Caches unavailable
            </p>
            <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
              {validationError.error.message}
            </p>
          </div>
        ) : null}

        <section className="grid gap-3 md:grid-cols-3">
          <div className="card p-4">
            <p className="t-label">Stored caches</p>
            <p className="t-h2 mt-2">{detail.caches.total}</p>
          </div>
          <div className="card p-4">
            <p className="t-label">Storage used</p>
            <p className="t-h2 mt-2">{bytesLabel(detail.totalSizeBytes)}</p>
            <div
              aria-label={`${usage}% of cache budget used`}
              className="mt-3 h-2 overflow-hidden rounded-[var(--radius-pill)]"
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={usage}
              style={{ background: "var(--surface-3)" }}
            >
              <div
                className="h-full rounded-[var(--radius-pill)]"
                style={{
                  background: "var(--accent)",
                  width: `${usage}%`,
                }}
              />
            </div>
          </div>
          <div className="card p-4">
            <p className="t-label">Repository limit</p>
            <p className="t-h2 mt-2">{bytesLabel(detail.limitBytes)}</p>
          </div>
        </section>

        {message ? (
          <p className="t-sm" role="status" style={{ color: "var(--ink-2)" }}>
            {message}
          </p>
        ) : null}

        <section className="card overflow-hidden">
          <div
            className="grid grid-cols-[minmax(0,1.4fr)_minmax(140px,.7fr)_120px_190px_96px] gap-3 border-b px-4 py-3 max-lg:hidden"
            style={{ borderColor: "var(--line)" }}
          >
            <p className="t-label">Key</p>
            <p className="t-label">Version</p>
            <p className="t-label">Size</p>
            <p className="t-label">Last used</p>
            <p className="t-label text-right">Action</p>
          </div>
          {detail.caches.items.length ? (
            detail.caches.items.map((cache) => (
              <div
                className="grid grid-cols-[minmax(0,1.4fr)_minmax(140px,.7fr)_120px_190px_96px] items-center gap-3 border-b px-4 py-3 max-lg:grid-cols-1"
                key={cache.id}
                style={{ borderColor: "var(--line-soft)" }}
              >
                <div className="min-w-0">
                  <p className="t-sm truncate font-medium">{cache.key}</p>
                  <p className="t-xs t-mono mt-1 truncate">{cache.scope}</p>
                </div>
                <p className="t-mono-sm truncate">{cache.version}</p>
                <p className="t-sm t-num">{bytesLabel(cache.sizeBytes)}</p>
                <p className="t-sm">{dateLabel(cache.lastUsedAt)}</p>
                <div className="flex justify-end">
                  <button
                    className="btn sm"
                    disabled={!detail.canDelete || pendingId === cache.id}
                    onClick={() => deleteCache(cache.id)}
                    title={
                      detail.canDelete
                        ? undefined
                        : "Write access is required to delete caches."
                    }
                    type="button"
                  >
                    {pendingId === cache.id ? "Deleting" : "Delete"}
                  </button>
                </div>
              </div>
            ))
          ) : (
            <div className="p-8 text-center">
              <p className="t-h3">No dependency caches yet</p>
              <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
                Cache entries will appear after workflow jobs reserve them
                through the Actions cache API.
              </p>
            </div>
          )}
        </section>
      </div>
    </RepositoryShell>
  );
}
