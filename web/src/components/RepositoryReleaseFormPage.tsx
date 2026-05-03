"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { MarkdownEditor } from "@/components/MarkdownEditor";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  ReleaseManagementContext,
  ReleaseRefOption,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryReleaseFormPageProps = {
  context: ReleaseManagementContext | ApiErrorEnvelope;
  mode: "new" | "edit";
  repository: RepositoryOverview;
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

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
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
  const release = context.release;
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
  const [notesPreviewOpen, setNotesPreviewOpen] = useState(false);

  const activeTag = tagMode === "new" ? newTagName : tagName;
  const generatedPreviewHtml = useMemo(() => {
    const safeTitle = escapeHtml(title.trim() || "Generated release notes");
    const from = escapeHtml(previousTag || "the previous release");
    const to = escapeHtml(target || context.defaultTarget);
    return `<h2>${safeTitle}</h2><p>Preview source: ${from} to ${to}.</p><p>The server-generated notes action is available in the next release-management step.</p>`;
  }, [context.defaultTarget, previousTag, target, title]);

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
              publication policy before the server-confirmed action step.
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
                        disabled={locked || context.availableTags.length === 0}
                        onChange={() => setTagMode("existing")}
                        type="radio"
                      />{" "}
                      Existing
                    </label>
                    <label className="chip soft">
                      <input
                        checked={tagMode === "new"}
                        disabled={locked}
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
                      disabled={locked || context.availableTags.length === 0}
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
                      disabled={locked}
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
                    disabled={locked}
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
                  disabled={locked}
                  onChange={(event) => setTitle(event.target.value)}
                  placeholder="Release title"
                  value={title}
                />
              </label>
            </section>

            <MarkdownEditor
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
              <div
                className="mt-4 rounded-[var(--radius)] border border-dashed p-5 text-center"
                style={{ borderColor: "var(--line)" }}
              >
                <p className="t-sm">Drag files here or choose assets.</p>
                <input
                  aria-label="Release asset files"
                  className="input mt-3"
                  disabled={locked}
                  multiple
                  type="file"
                />
                <p className="t-xs mt-2">
                  Upload intent creation is server-confirmed before assets are
                  attached to a release.
                </p>
              </div>
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
                      <button
                        aria-disabled="true"
                        className="btn sm"
                        disabled
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
                  disabled={locked}
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
                disabled={locked}
                onClick={() => setNotesPreviewOpen((value) => !value)}
                type="button"
              >
                {notesPreviewOpen ? "Hide preview" : "Preview generated notes"}
              </button>
              {notesPreviewOpen ? (
                <div className="mt-4">
                  <MarkdownBody html={generatedPreviewHtml} />
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
                    disabled={locked}
                    onChange={(event) => setDraft(event.target.checked)}
                    type="checkbox"
                  />
                  Save as draft
                </label>
                <label className="flex items-center gap-2 t-sm">
                  <input
                    checked={prerelease}
                    disabled={locked}
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
                        disabled={locked}
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
                  aria-disabled="true"
                  className="btn accent"
                  disabled
                  type="button"
                >
                  {editing ? "Update release" : "Publish release"}
                </button>
                <button
                  aria-disabled="true"
                  className="btn"
                  disabled
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
                    disabled={locked}
                    onChange={(event) => setDeleteTag(event.target.checked)}
                    type="checkbox"
                  />
                  Also delete the git tag
                </label>
                <button
                  aria-disabled="true"
                  className="btn mt-4 w-full"
                  disabled
                  type="button"
                >
                  Delete release
                </button>
              </section>
            ) : null}
          </aside>
        </div>
      </section>
    </RepositoryShell>
  );
}
