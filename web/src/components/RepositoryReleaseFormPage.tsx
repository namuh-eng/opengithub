"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";
import { MarkdownEditor } from "@/components/MarkdownEditor";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  GeneratedReleaseNotesPreview,
  ReleaseManagementContext,
  ReleaseMutation,
  ReleaseRefOption,
  ReleaseUploadIntent,
  RepositoryOverview,
  RepositoryReleaseDetail,
} from "@/lib/api";

type RepositoryReleaseFormPageProps = {
  context: ReleaseManagementContext | ApiErrorEnvelope;
  mode: "new" | "edit";
  repository: RepositoryOverview;
};

type AssetUploadRow = {
  id: string;
  name: string;
  byteSize: number;
  contentType: string;
  checksumSha256: string | null;
  status: "queued" | "uploading" | "complete" | "error" | "cancelled";
  message: string;
  intent?: ReleaseUploadIntent;
};

function isApiError(value: unknown): value is ApiErrorEnvelope {
  return Boolean(value && typeof value === "object" && "error" in value);
}

function basePath(repository: RepositoryOverview) {
  return `/${repository.owner_login}/${repository.name}`;
}

function selectLabel(option: ReleaseRefOption) {
  const suffix = option.shortOid ? ` · ${option.shortOid}` : "";
  return `${option.shortName}${suffix}`;
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

function ReleaseFormUnavailable({
  error,
  repository,
}: {
  error: ApiErrorEnvelope;
  repository: RepositoryOverview;
}) {
  const forbidden = error.status === 401 || error.status === 403;
  return (
    <RepositoryShell
      activePath={`${basePath(repository)}/releases`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section className="card p-6" role="status">
        <span className={`chip ${forbidden ? "warn" : "err"}`}>
          {forbidden ? "Write access required" : "Unavailable"}
        </span>
        <h1 className="t-h1 mt-4">Release management</h1>
        <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
          {forbidden
            ? "Only maintainers with write access can draft, publish, or edit repository releases."
            : error.error.message}
        </p>
        <Link className="btn mt-5" href={`${basePath(repository)}/releases`}>
          Back to releases
        </Link>
      </section>
    </RepositoryShell>
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

async function sha256Hex(file: File) {
  if (!globalThis.crypto?.subtle) {
    return null;
  }
  const digest = await globalThis.crypto.subtle.digest(
    "SHA-256",
    await file.arrayBuffer(),
  );
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export function RepositoryReleaseFormPage({
  context,
  mode,
  repository,
}: RepositoryReleaseFormPageProps) {
  if (isApiError(context)) {
    return <ReleaseFormUnavailable error={context} repository={repository} />;
  }

  return (
    <RepositoryReleaseFormContent
      context={context}
      mode={mode}
      repository={repository}
    />
  );
}

function RepositoryReleaseFormContent({
  context,
  mode,
  repository,
}: {
  context: ReleaseManagementContext;
  mode: "new" | "edit";
  repository: RepositoryOverview;
}) {
  const router = useRouter();
  const [currentRelease, setCurrentRelease] = useState(context.release);
  const release = currentRelease;
  const editing = mode === "edit";
  const locked = context.archived || Boolean(release?.immutable);
  const [tagMode, setTagMode] = useState<"existing" | "new">(
    release?.tagName
      ? "existing"
      : context.availableTags.length > 0
        ? "existing"
        : "new",
  );
  const [tagName, setTagName] = useState(
    release?.tagName ?? context.availableTags[0]?.shortName ?? "",
  );
  const [newTagName, setNewTagName] = useState("");
  const [target, setTarget] = useState(
    release?.tagName ?? context.defaultTarget,
  );
  const [previousTag, setPreviousTag] = useState(
    context.previousTagCandidates[0]?.shortName ?? "",
  );
  const [title, setTitle] = useState(release?.title ?? "");
  const [body, setBody] = useState(release?.body ?? "");
  const [draft, setDraft] = useState(release?.draft ?? true);
  const [prerelease, setPrerelease] = useState(release?.prerelease ?? false);
  const [latestPolicy, setLatestPolicy] = useState(
    context.latestPolicyOptions[0]?.value ?? "automatic",
  );
  const [deleteTag, setDeleteTag] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState("");
  const [editorVersion, setEditorVersion] = useState(0);
  const [message, setMessage] = useState<{
    kind: "success" | "error";
    text: string;
  } | null>(null);
  const [notesPreview, setNotesPreview] =
    useState<GeneratedReleaseNotesPreview | null>(null);
  const [assetRows, setAssetRows] = useState<AssetUploadRow[]>([]);
  const [isPending, startTransition] = useTransition();

  const activeTag = tagMode === "new" ? newTagName : tagName;
  const disabled = locked || isPending;
  const deleteReady = !editing || deleteConfirm.trim() === release?.tagName;

  function buildMutation(overrides: Partial<ReleaseMutation> = {}) {
    return {
      tagName: activeTag,
      target,
      title,
      body,
      draft,
      prerelease,
      latestPolicy,
      ...overrides,
    };
  }

  function runAction(action: () => Promise<unknown>, success: string) {
    setMessage(null);
    startTransition(async () => {
      try {
        const result = await action();
        setMessage({ kind: "success", text: success });
        if (result && typeof result === "object" && "links" in result) {
          router.push((result as RepositoryReleaseDetail).links.htmlHref);
          return;
        }
        router.refresh();
      } catch (error) {
        setMessage({
          kind: "error",
          text:
            error instanceof Error ? error.message : "Release action failed.",
        });
      }
    });
  }

  function saveRelease(nextDraft: boolean) {
    const releaseMutation = buildMutation({ draft: nextDraft });
    runAction(
      () =>
        submitReleaseAction(repository, {
          action: editing ? "update" : "create",
          releaseId: release?.id,
          release: releaseMutation,
        }),
      nextDraft
        ? "Draft saved."
        : editing
          ? "Release updated."
          : "Release published.",
    );
  }

  function publishDraft() {
    if (!release) return;
    runAction(
      () =>
        submitReleaseAction(repository, {
          action: "update",
          releaseId: release.id,
          release: buildMutation({ draft: false }),
        }),
      "Draft published.",
    );
  }

  function deleteRelease() {
    if (!release || !deleteReady) return;
    runAction(
      async () => {
        await submitReleaseAction(repository, {
          action: "delete",
          releaseId: release.id,
          deleteTag,
        });
        router.push(`${basePath(repository)}/releases`);
        return null;
      },
      deleteTag ? "Release and tag deleted." : "Release deleted.",
    );
  }

  function generateNotes() {
    runAction(async () => {
      const preview = (await submitReleaseAction(repository, {
        action: "generatedNotes",
        request: {
          target,
          previousTag: previousTag || null,
          title: title || activeTag || null,
        },
      })) as GeneratedReleaseNotesPreview;
      setNotesPreview(preview);
      setBody(preview.body);
      setEditorVersion((value) => value + 1);
      return null;
    }, "Generated notes inserted. Review them before publishing.");
  }

  function upsertAssetRow(next: AssetUploadRow) {
    setAssetRows((rows) => {
      const existing = rows.findIndex((row) => row.id === next.id);
      if (existing === -1) return [...rows, next];
      return rows.map((row) => (row.id === next.id ? next : row));
    });
  }

  async function uploadFile(file: File) {
    const rowId = `${file.name}-${file.size}-${file.lastModified}`;
    const contentType = file.type || "application/octet-stream";
    if (!release) {
      upsertAssetRow({
        id: rowId,
        name: file.name,
        byteSize: file.size,
        contentType,
        checksumSha256: null,
        status: "queued",
        message: "Save a draft before attaching binary assets.",
      });
      return;
    }
    if (file.size > context.uploadLimits.maxAssetBytes) {
      upsertAssetRow({
        id: rowId,
        name: file.name,
        byteSize: file.size,
        contentType,
        checksumSha256: null,
        status: "error",
        message: `File exceeds ${formatBytes(context.uploadLimits.maxAssetBytes)}.`,
      });
      return;
    }
    upsertAssetRow({
      id: rowId,
      name: file.name,
      byteSize: file.size,
      contentType,
      checksumSha256: null,
      status: "uploading",
      message: "Creating upload intent...",
    });
    try {
      const checksumSha256 = await sha256Hex(file);
      const intent = (await submitReleaseAction(repository, {
        action: "createUploadIntent",
        asset: {
          name: file.name,
          contentType,
          byteSize: file.size,
          checksumSha256,
        },
      })) as ReleaseUploadIntent;
      upsertAssetRow({
        id: rowId,
        name: file.name,
        byteSize: file.size,
        contentType,
        checksumSha256,
        status: "uploading",
        message: "Completing upload...",
        intent,
      });
      const updated = (await submitReleaseAction(repository, {
        action: "completeUploadIntent",
        intentId: intent.id,
        completion: {
          releaseId: release.id,
          handoffToken: intent.handoffToken,
          checksumSha256,
        },
      })) as RepositoryReleaseDetail;
      setCurrentRelease(updated);
      upsertAssetRow({
        id: rowId,
        name: file.name,
        byteSize: file.size,
        contentType,
        checksumSha256,
        status: "complete",
        message: "Attached to release.",
        intent,
      });
      setMessage({ kind: "success", text: "Release asset uploaded." });
    } catch (error) {
      upsertAssetRow({
        id: rowId,
        name: file.name,
        byteSize: file.size,
        contentType,
        checksumSha256: null,
        status: "error",
        message:
          error instanceof Error ? error.message : "Asset upload failed.",
      });
      setMessage({
        kind: "error",
        text: error instanceof Error ? error.message : "Asset upload failed.",
      });
    }
  }

  function handleAssetFiles(files: FileList | File[]) {
    if (disabled) return;
    for (const file of Array.from(files)) {
      void uploadFile(file);
    }
  }

  async function cancelAssetUpload(row: AssetUploadRow) {
    if (!row.intent) {
      setAssetRows((rows) =>
        rows.map((item) =>
          item.id === row.id
            ? { ...item, status: "cancelled", message: "Removed from queue." }
            : item,
        ),
      );
      return;
    }
    try {
      const intent = (await submitReleaseAction(repository, {
        action: "cancelUploadIntent",
        intentId: row.intent.id,
        reason: "cancelled in release form",
      })) as ReleaseUploadIntent;
      setAssetRows((rows) =>
        rows.map((item) =>
          item.id === row.id
            ? {
                ...item,
                intent,
                status: "cancelled",
                message: "Upload cancelled.",
              }
            : item,
        ),
      );
    } catch (error) {
      setMessage({
        kind: "error",
        text:
          error instanceof Error
            ? error.message
            : "Asset upload could not be cancelled.",
      });
    }
  }

  async function deleteAsset(assetId: string) {
    if (!release) return;
    setMessage(null);
    try {
      const updated = (await submitReleaseAction(repository, {
        action: "deleteAsset",
        releaseId: release.id,
        assetId,
      })) as RepositoryReleaseDetail;
      setCurrentRelease(updated);
      setMessage({ kind: "success", text: "Release asset removed." });
    } catch (error) {
      setMessage({
        kind: "error",
        text:
          error instanceof Error
            ? error.message
            : "Release asset could not be removed.",
      });
    }
  }

  return (
    <RepositoryShell
      activePath={`${basePath(repository)}/releases`}
      frameClassName="max-w-6xl"
      repository={repository}
    >
      <section>
        <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="t-label">Release management</p>
            <h1 className="t-h1 mt-2">
              {editing ? "Edit release" : "New release"}
            </h1>
            <p
              className="t-body mt-3 max-w-2xl"
              style={{ color: "var(--ink-2)" }}
            >
              Compose notes, choose a tag and target, prepare assets, and set
              publication policy. Every publish, draft, edit, and delete action
              is confirmed by the repository API.
            </p>
          </div>
          <Link className="btn" href={`${basePath(repository)}/releases`}>
            Back to releases
          </Link>
        </div>

        {locked ? (
          <div className="card mb-5 p-4" role="status">
            <span className="chip warn">
              {context.archived ? "Archived repository" : "Immutable release"}
            </span>
            <p className="t-sm mt-3" style={{ color: "var(--ink-2)" }}>
              This release can be inspected, but mutation controls are disabled.
            </p>
          </div>
        ) : null}

        <div className="grid grid-cols-[minmax(0,1fr)_320px] gap-6 max-lg:grid-cols-1">
          <div className="space-y-5">
            <section className="card p-5" aria-labelledby="release-ref-title">
              <h2 className="t-h3" id="release-ref-title">
                Tag and target
              </h2>
              <div className="mt-4 grid grid-cols-2 gap-4 max-md:grid-cols-1">
                <fieldset className="grid gap-2">
                  <legend className="t-sm">Release tag</legend>
                  <div className="flex flex-wrap gap-2">
                    <label className="chip soft">
                      <input
                        checked={tagMode === "existing"}
                        disabled={
                          disabled || context.availableTags.length === 0
                        }
                        onChange={() => setTagMode("existing")}
                        type="radio"
                      />{" "}
                      Existing
                    </label>
                    <label className="chip soft">
                      <input
                        checked={tagMode === "new"}
                        disabled={disabled}
                        onChange={() => setTagMode("new")}
                        type="radio"
                      />{" "}
                      New tag
                    </label>
                  </div>
                  {tagMode === "existing" ? (
                    <select
                      aria-label="Existing tag"
                      className="input"
                      disabled={disabled || context.availableTags.length === 0}
                      onChange={(event) => setTagName(event.target.value)}
                      value={tagName}
                    >
                      {context.availableTags.map((tag) => (
                        <option key={tag.name} value={tag.shortName}>
                          {selectLabel(tag)}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <input
                      aria-label="New tag name"
                      className="input"
                      disabled={disabled}
                      onChange={(event) => setNewTagName(event.target.value)}
                      placeholder="v1.0.0"
                      value={newTagName}
                    />
                  )}
                </fieldset>
                <label className="grid gap-2 t-sm">
                  Target branch, tag, or SHA
                  <select
                    className="input"
                    disabled={disabled}
                    onChange={(event) => setTarget(event.target.value)}
                    value={target}
                  >
                    {context.availableRefs.map((ref) => (
                      <option key={`${ref.kind}:${ref.name}`} value={ref.name}>
                        {ref.kind} · {selectLabel(ref)}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <p className="t-xs mt-3">
                Selected release tag:{" "}
                <span className="t-mono-sm">{activeTag || "not selected"}</span>
              </p>
            </section>

            <section className="card p-5" aria-labelledby="release-title">
              <h2 className="t-h3" id="release-title">
                Release details
              </h2>
              <label className="mt-4 grid gap-2 t-sm">
                Title
                <input
                  className="input"
                  disabled={disabled}
                  onChange={(event) => setTitle(event.target.value)}
                  placeholder="Release title"
                  value={title}
                />
              </label>
            </section>

            <MarkdownEditor
              key={editorVersion}
              initialMarkdown={body}
              initialRendered={{
                cached: false,
                contentSha: release?.id ?? "new-release",
                html: release?.bodyHtml || "<p>Nothing to preview.</p>",
              }}
              onMarkdownChange={setBody}
              owner={repository.owner_login}
              refName={target}
              repo={repository.name}
            />

            <section
              className="card p-5"
              aria-labelledby="release-assets-title"
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <h2 className="t-h3" id="release-assets-title">
                    Assets
                  </h2>
                  <p className="t-xs mt-1">
                    Uploads are limited to{" "}
                    {formatBytes(context.uploadLimits.maxAssetBytes)} per file
                    and {context.uploadLimits.maxAssetCount} assets.
                  </p>
                </div>
                <span className="chip soft">
                  {context.uploadLimits.allowedStorageKinds.join(" / ")}
                </span>
              </div>
              <label
                className="mt-4 rounded-[var(--radius)] border border-dashed p-5 text-center"
                style={{ borderColor: "var(--line)" }}
                onDragOver={(event) => {
                  event.preventDefault();
                }}
                onDrop={(event) => {
                  event.preventDefault();
                  handleAssetFiles(event.dataTransfer.files);
                }}
              >
                <p className="t-sm">Drag files here or choose assets.</p>
                <input
                  aria-label="Release asset files"
                  className="input mt-3"
                  disabled={disabled}
                  multiple
                  onChange={(event) => {
                    if (event.target.files) {
                      handleAssetFiles(event.target.files);
                    }
                    event.currentTarget.value = "";
                  }}
                  type="file"
                />
                <p className="t-xs mt-2">
                  Upload intent creation is server-confirmed before assets are
                  attached to a release.
                </p>
                {!release ? (
                  <p className="t-xs mt-2" style={{ color: "var(--warn)" }}>
                    Save a draft first to attach files to a release record.
                  </p>
                ) : null}
              </label>
              {assetRows.length ? (
                <ul className="mt-4 space-y-2" aria-label="Pending assets">
                  {assetRows.map((row) => (
                    <li
                      className="list-row flex flex-wrap items-center gap-3 py-2"
                      key={row.id}
                    >
                      <span className="t-mono-sm">{row.name}</span>
                      <span className="t-xs">{formatBytes(row.byteSize)}</span>
                      <span
                        className={`chip ${
                          row.status === "complete"
                            ? "ok"
                            : row.status === "error"
                              ? "err"
                              : row.status === "cancelled"
                                ? "soft"
                                : "warn"
                        }`}
                      >
                        {row.status}
                      </span>
                      <span className="t-xs">{row.message}</span>
                      {row.checksumSha256 ? (
                        <span className="t-mono-sm">
                          {row.checksumSha256.slice(0, 12)}
                        </span>
                      ) : null}
                      {row.status === "queued" || row.status === "uploading" ? (
                        <button
                          aria-disabled={disabled}
                          className="btn sm"
                          disabled={disabled}
                          onClick={() => void cancelAssetUpload(row)}
                          type="button"
                        >
                          Cancel
                        </button>
                      ) : null}
                    </li>
                  ))}
                </ul>
              ) : null}
              {release?.assets.length ? (
                <ul className="mt-4 space-y-2">
                  {release.assets.map((asset) => (
                    <li
                      className="list-row flex flex-wrap items-center gap-3 py-2"
                      key={asset.id}
                    >
                      <span className="t-mono-sm">{asset.name}</span>
                      <span className="t-xs">
                        {formatBytes(asset.byteSize)}
                      </span>
                      {asset.checksumSha256 ? (
                        <span className="t-mono-sm">
                          sha256:{asset.checksumSha256.slice(0, 12)}
                        </span>
                      ) : null}
                      <button
                        aria-disabled={disabled}
                        className="btn sm"
                        disabled={disabled}
                        onClick={() => void deleteAsset(asset.id)}
                        type="button"
                      >
                        Remove
                      </button>
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
          </div>

          <aside className="space-y-5">
            <section className="card p-5" aria-labelledby="notes-title">
              <h2 className="t-h3" id="notes-title">
                Generated notes
              </h2>
              <label className="mt-4 grid gap-2 t-sm">
                Previous tag
                <select
                  className="input"
                  disabled={disabled}
                  onChange={(event) => setPreviousTag(event.target.value)}
                  value={previousTag}
                >
                  <option value="">Automatic</option>
                  {context.previousTagCandidates.map((tag) => (
                    <option key={tag.name} value={tag.shortName}>
                      {selectLabel(tag)}
                    </option>
                  ))}
                </select>
              </label>
              <button
                className="btn mt-4 w-full"
                disabled={disabled}
                aria-disabled={disabled}
                onClick={generateNotes}
                type="button"
              >
                {isPending ? "Generating..." : "Generate release notes"}
              </button>
              {notesPreview ? (
                <div
                  className="mt-4 rounded-[var(--radius)] border p-3"
                  style={{ borderColor: "var(--line)" }}
                >
                  <p className="t-xs">
                    {notesPreview.commitCount} commits and{" "}
                    {notesPreview.mergedPullRequestCount} merged pull requests
                    inserted into the editor.
                  </p>
                </div>
              ) : null}
            </section>

            <section className="card p-5" aria-labelledby="publish-title">
              <h2 className="t-h3" id="publish-title">
                Publication
              </h2>
              <div className="mt-4 space-y-3">
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
              <fieldset className="mt-5 grid gap-3">
                <legend className="t-sm">Latest release policy</legend>
                {context.latestPolicyOptions.map((option) => (
                  <label className="grid gap-1 t-sm" key={option.value}>
                    <span className="flex items-center gap-2">
                      <input
                        checked={latestPolicy === option.value}
                        disabled={disabled}
                        onChange={() => setLatestPolicy(option.value)}
                        type="radio"
                      />
                      {option.label}
                    </span>
                    <span className="t-xs">{option.description}</span>
                  </label>
                ))}
              </fieldset>
              <div className="mt-5 grid gap-2">
                <button
                  aria-disabled={disabled}
                  className="btn accent"
                  disabled={disabled}
                  onClick={() =>
                    editing ? saveRelease(draft) : saveRelease(false)
                  }
                  type="button"
                >
                  {editing ? "Update release" : "Publish release"}
                </button>
                {editing && release?.draft ? (
                  <button
                    aria-disabled={disabled}
                    className="btn accent"
                    disabled={disabled}
                    onClick={publishDraft}
                    type="button"
                  >
                    Publish draft
                  </button>
                ) : null}
                <button
                  aria-disabled={disabled}
                  className="btn"
                  disabled={disabled}
                  onClick={() => saveRelease(true)}
                  type="button"
                >
                  Save draft
                </button>
              </div>
            </section>

            {editing ? (
              <section className="card p-5" aria-labelledby="danger-title">
                <h2 className="t-h3" id="danger-title">
                  Danger zone
                </h2>
                <label className="mt-4 flex items-center gap-2 t-sm">
                  <input
                    checked={deleteTag}
                    disabled={disabled}
                    onChange={(event) => setDeleteTag(event.target.checked)}
                    type="checkbox"
                  />
                  Also delete the git tag
                </label>
                <label className="mt-4 grid gap-2 t-sm">
                  Type tag name to confirm
                  <input
                    className="input"
                    disabled={disabled}
                    onChange={(event) => setDeleteConfirm(event.target.value)}
                    placeholder={release?.tagName}
                    value={deleteConfirm}
                  />
                </label>
                <button
                  aria-disabled={disabled || !deleteReady}
                  className="btn mt-4 w-full"
                  disabled={disabled || !deleteReady}
                  onClick={deleteRelease}
                  type="button"
                >
                  Delete release
                </button>
              </section>
            ) : null}
          </aside>
        </div>
        {message ? (
          <p
            className="mt-5 t-sm"
            role={message.kind === "error" ? "alert" : "status"}
            style={{
              color: `var(--${message.kind === "error" ? "err" : "ok"})`,
            }}
          >
            {message.text}
          </p>
        ) : null}
      </section>
    </RepositoryShell>
  );
}
