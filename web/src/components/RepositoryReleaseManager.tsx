"use client";

import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";
import type {
  ReleaseAsset,
  RepositoryOverview,
  RepositoryReleaseDetail,
} from "@/lib/api";

type RepositoryReleaseManagerProps = {
  repository: RepositoryOverview;
  release?: RepositoryReleaseDetail;
};

function basePath(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}`;
}

function canWrite(repository: RepositoryOverview) {
  return ["owner", "admin", "write"].includes(
    repository.viewerPermission ?? "",
  );
}

async function submitReleaseAction(
  repository: RepositoryOverview,
  payload: Record<string, unknown>,
) {
  const response = await fetch(`${basePath(repository)}/releases/actions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw new Error(body?.error?.message ?? "Release action failed.");
  }
  return body;
}

export function RepositoryReleaseManager({
  release,
  repository,
}: RepositoryReleaseManagerProps) {
  if (!canWrite(repository)) {
    return null;
  }
  return (
    <RepositoryReleaseManagerInner release={release} repository={repository} />
  );
}

function RepositoryReleaseManagerInner({
  release,
  repository,
}: RepositoryReleaseManagerProps) {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState(release?.title ?? "");
  const [tagName, setTagName] = useState(release?.tagName ?? "");
  const [target, setTarget] = useState(repository.default_branch);
  const [body, setBody] = useState(release?.body ?? "");
  const [draft, setDraft] = useState(release?.draft ?? false);
  const [prerelease, setPrerelease] = useState(release?.prerelease ?? false);
  const [assetName, setAssetName] = useState("");
  const [assetLabel, setAssetLabel] = useState("");
  const [assetSize, setAssetSize] = useState("0");
  const [message, setMessage] = useState("");
  const [isPending, startTransition] = useTransition();

  const editing = Boolean(release);
  const disabled = isPending || Boolean(release?.immutable);

  function run(action: () => Promise<unknown>, success: string) {
    setMessage("");
    startTransition(async () => {
      try {
        const result = await action();
        setMessage(success);
        if (
          !editing &&
          result &&
          typeof result === "object" &&
          "links" in result
        ) {
          const href = (result as RepositoryReleaseDetail).links.htmlHref;
          router.push(href);
          return;
        }
        router.refresh();
      } catch (error) {
        setMessage(
          error instanceof Error ? error.message : "Release action failed.",
        );
      }
    });
  }

  function submitRelease() {
    run(
      () =>
        submitReleaseAction(repository, {
          action: editing ? "update" : "create",
          releaseId: release?.id,
          release: {
            tagName,
            target,
            title,
            body,
            draft,
            prerelease,
          },
        }),
      editing ? "Release updated." : "Release created.",
    );
  }

  function publishRelease() {
    if (!release) return;
    run(
      () =>
        submitReleaseAction(repository, {
          action: "publish",
          releaseId: release.id,
        }),
      "Draft published.",
    );
  }

  function deleteRelease() {
    if (!release) return;
    const confirmed = window.confirm(
      `Delete release ${release.tagName}? The tag and source files stay in the repository.`,
    );
    if (!confirmed) return;
    run(
      () =>
        submitReleaseAction(repository, {
          action: "delete",
          releaseId: release.id,
        }),
      "Release deleted.",
    );
    router.push(`${basePath(repository)}/releases`);
  }

  function createAsset() {
    if (!release) return;
    run(
      () =>
        submitReleaseAction(repository, {
          action: "createAsset",
          releaseId: release.id,
          asset: {
            name: assetName,
            label: assetLabel,
            byteSize: Number(assetSize) || 0,
            contentType: "application/octet-stream",
          },
        }),
      "Asset added.",
    );
    setAssetName("");
    setAssetLabel("");
    setAssetSize("0");
  }

  function deleteAsset(asset: ReleaseAsset) {
    if (!release) return;
    const confirmed = window.confirm(`Remove asset ${asset.name}?`);
    if (!confirmed) return;
    run(
      () =>
        submitReleaseAction(repository, {
          action: "deleteAsset",
          releaseId: release.id,
          assetId: asset.id,
        }),
      "Asset removed.",
    );
  }

  return (
    <section className="card mb-6 p-5" aria-label="Release management">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="t-label">Release management</p>
          <h2 className="t-h3 mt-2">
            {editing ? "Edit this release" : "Draft or publish a release"}
          </h2>
        </div>
        <button
          aria-expanded={open}
          className="btn sm"
          onClick={() => setOpen((value) => !value)}
          type="button"
        >
          {open ? "Close" : editing ? "Edit" : "New release"}
        </button>
      </div>
      {release?.immutable ? (
        <p className="t-xs mt-3" role="status">
          Immutable releases cannot be changed.
        </p>
      ) : null}
      {open ? (
        <div className="mt-5 grid gap-4">
          <div className="grid grid-cols-2 gap-3 max-sm:grid-cols-1">
            <label className="grid gap-1 t-sm">
              Tag
              <input
                className="input"
                disabled={disabled}
                onChange={(event) => setTagName(event.target.value)}
                placeholder="v1.0.0"
                value={tagName}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Target branch, tag, or SHA
              <input
                className="input"
                disabled={disabled}
                onChange={(event) => setTarget(event.target.value)}
                value={target}
              />
            </label>
          </div>
          <label className="grid gap-1 t-sm">
            Title
            <input
              className="input"
              disabled={disabled}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Release title"
              value={title}
            />
          </label>
          <label className="grid gap-1 t-sm">
            Notes
            <textarea
              className="input min-h-36"
              disabled={disabled}
              onChange={(event) => setBody(event.target.value)}
              placeholder="Markdown release notes"
              value={body}
            />
          </label>
          <div className="flex flex-wrap gap-4">
            <label className="flex items-center gap-2 t-sm">
              <input
                checked={draft}
                disabled={disabled}
                onChange={(event) => setDraft(event.target.checked)}
                type="checkbox"
              />
              Save as draft
            </label>
            <label className="flex items-center gap-2 t-sm">
              <input
                checked={prerelease}
                disabled={disabled}
                onChange={(event) => setPrerelease(event.target.checked)}
                type="checkbox"
              />
              Mark as pre-release
            </label>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              className="btn accent"
              disabled={disabled}
              aria-disabled={disabled}
              onClick={submitRelease}
              type="button"
            >
              {editing ? "Save release" : "Create release"}
            </button>
            {release?.draft ? (
              <button
                className="btn"
                disabled={disabled}
                aria-disabled={disabled}
                onClick={publishRelease}
                type="button"
              >
                Publish draft
              </button>
            ) : null}
            {editing ? (
              <button
                className="btn"
                disabled={disabled}
                aria-disabled={disabled}
                onClick={deleteRelease}
                type="button"
              >
                Delete release
              </button>
            ) : null}
          </div>
          {release ? (
            <div
              className="border-t pt-4"
              style={{ borderColor: "var(--line)" }}
            >
              <h3 className="t-h3">Assets</h3>
              <div className="mt-3 grid grid-cols-[1fr_1fr_120px_auto] gap-2 max-md:grid-cols-1">
                <input
                  aria-label="Asset name"
                  className="input"
                  disabled={disabled}
                  onChange={(event) => setAssetName(event.target.value)}
                  placeholder="asset.tar.gz"
                  value={assetName}
                />
                <input
                  aria-label="Asset label"
                  className="input"
                  disabled={disabled}
                  onChange={(event) => setAssetLabel(event.target.value)}
                  placeholder="Label"
                  value={assetLabel}
                />
                <input
                  aria-label="Asset byte size"
                  className="input"
                  disabled={disabled}
                  min="0"
                  onChange={(event) => setAssetSize(event.target.value)}
                  type="number"
                  value={assetSize}
                />
                <button
                  className="btn"
                  disabled={disabled}
                  aria-disabled={disabled}
                  onClick={createAsset}
                  type="button"
                >
                  Upload asset
                </button>
              </div>
              {release.assets.length > 0 ? (
                <ul className="mt-3 space-y-2">
                  {release.assets.map((asset) => (
                    <li
                      className="list-row flex items-center justify-between gap-3 py-2"
                      key={asset.id}
                    >
                      <span className="t-mono-sm">{asset.name}</span>
                      <button
                        className="btn sm"
                        disabled={disabled}
                        aria-disabled={disabled}
                        onClick={() => deleteAsset(asset)}
                        type="button"
                      >
                        Remove
                      </button>
                    </li>
                  ))}
                </ul>
              ) : null}
            </div>
          ) : null}
        </div>
      ) : null}
      {message ? (
        <p className="t-xs mt-3" role="status">
          {message}
        </p>
      ) : null}
    </section>
  );
}
