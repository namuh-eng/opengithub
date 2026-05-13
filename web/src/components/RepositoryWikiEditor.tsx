"use client";

import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import type {
  RepositoryOverview,
  RepositoryWikiEditView,
  RepositoryWikiMarkupFormat,
  RepositoryWikiMutationResult,
  RepositoryWikiPagesIndex,
  RepositoryWikiPreviewResult,
} from "@/lib/api";
import { repositoryWikiHref } from "@/lib/navigation";

type RepositoryWikiEditorProps = {
  repository: RepositoryOverview;
  pagesIndex: RepositoryWikiPagesIndex;
  editView?: RepositoryWikiEditView | null;
};

type Tab = "write" | "preview";

const DEFAULT_FORMATS: RepositoryWikiMarkupFormat[] = [
  { mode: "markdown", label: "Markdown", extension: ".md" },
];

function ImageIcon() {
  return (
    <svg
      aria-hidden="true"
      fill="none"
      height="16"
      viewBox="0 0 16 16"
      width="16"
    >
      <path
        d="M2.75 3.25h10.5v9.5H2.75v-9.5Z"
        stroke="currentColor"
        strokeLinejoin="round"
        strokeWidth="1.35"
      />
      <path
        d="m3.2 11.1 3.05-3 2.1 2 1.35-1.35 3.1 3.1"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.35"
      />
      <circle cx="10.8" cy="5.75" r=".75" fill="currentColor" />
    </svg>
  );
}

function initialMarkdown() {
  return "# New page\n\nStart writing this wiki page.";
}

export function RepositoryWikiEditor({
  editView,
  repository,
  pagesIndex,
}: RepositoryWikiEditorProps) {
  const formats =
    editView && editView.supportedFormats.length > 0
      ? editView.supportedFormats
      : DEFAULT_FORMATS;
  const editingPage = editView?.page ?? null;
  const [title, setTitle] = useState(editingPage?.title ?? "");
  const [markdown, setMarkdown] = useState(
    editingPage?.markdown ?? initialMarkdown,
  );
  const [message, setMessage] = useState(
    editingPage ? `Update ${editingPage.title}` : "Create wiki page",
  );
  const [editMode, setEditMode] = useState<RepositoryWikiMarkupFormat["mode"]>(
    editingPage?.editMode ?? "markdown",
  );
  const [tab, setTab] = useState<Tab>("write");
  const [preview, setPreview] = useState<RepositoryWikiPreviewResult | null>(
    null,
  );
  const [imageUrl, setImageUrl] = useState("");
  const [imageAlt, setImageAlt] = useState("");
  const [status, setStatus] = useState<{
    kind: "idle" | "pending" | "success" | "error";
    message: string | null;
  }>({ kind: "idle", message: null });

  const lineCount = useMemo(() => markdown.split("\n").length, [markdown]);
  const canEdit = pagesIndex.viewer.canEditWiki;
  const owner = pagesIndex.repository.ownerLogin;
  const repo = pagesIndex.repository.name;

  async function requestJson<T>(
    url: string,
    method: "POST" | "PATCH",
    body: unknown,
  ): Promise<T> {
    const response = await fetch(url, {
      method,
      headers: {
        accept: "application/json",
        "content-type": "application/json",
      },
      body: JSON.stringify(body),
    });
    const payload = (await response.json().catch(() => null)) as
      | { error?: { message?: string } }
      | T
      | null;
    if (!response.ok) {
      throw new Error(
        (payload as { error?: { message?: string } } | null)?.error?.message ??
          "Wiki request failed.",
      );
    }
    return payload as T;
  }

  async function renderPreview() {
    setTab("preview");
    setStatus({ kind: "pending", message: "Rendering preview..." });
    try {
      const result = await requestJson<RepositoryWikiPreviewResult>(
        `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/preview`,
        "POST",
        { markdown, editMode },
      );
      setPreview(result);
      setStatus({ kind: "success", message: "Preview rendered." });
    } catch (error) {
      setStatus({
        kind: "error",
        message:
          error instanceof Error ? error.message : "Preview could not render.",
      });
    }
  }

  function insertImage() {
    const url = imageUrl.trim();
    const alt = imageAlt.trim() || "image";
    if (!url) {
      setStatus({ kind: "error", message: "Enter an image URL first." });
      return;
    }
    setMarkdown((current) => `${current.trimEnd()}\n\n![${alt}](${url})\n`);
    setImageUrl("");
    setImageAlt("");
    setTab("write");
    setStatus({ kind: "success", message: "Image reference inserted." });
  }

  function clientValidationMessage() {
    const trimmedTitle = title.trim();
    if (!trimmedTitle) return "wiki page title is required";
    const titleSegments = trimmedTitle.split("/").filter(Boolean);
    if (
      titleSegments.some(
        (segment) =>
          segment === "." || segment === ".." || segment.endsWith(".git"),
      )
    ) {
      return "wiki page slug is invalid";
    }
    if (
      trimmedTitle.startsWith("_") &&
      !["_sidebar", "_footer"].includes(trimmedTitle.toLowerCase())
    ) {
      return "wiki page slug is invalid";
    }
    if (!markdown.trim()) return "wiki page body is required";
    if (!message.trim()) return "wiki edit message is required";
    if (!formats.some((format) => format.mode === editMode)) {
      return `wiki edit mode ${editMode} is not supported`;
    }
    return null;
  }

  async function savePage() {
    const validationMessage = clientValidationMessage();
    if (validationMessage) {
      setStatus({ kind: "error", message: validationMessage });
      return;
    }

    setStatus({ kind: "pending", message: "Saving wiki page..." });
    try {
      const savePath = editingPage
        ? `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/${editingPage.slug
            .split("/")
            .filter(Boolean)
            .map((segment) => encodeURIComponent(segment))
            .join("/")}`
        : `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/wiki/pages`;
      const result = await requestJson<RepositoryWikiMutationResult>(
        savePath,
        editingPage ? "PATCH" : "POST",
        {
          title,
          markdown,
          message,
          editMode,
          expectedRevisionId: editingPage?.latestRevisionId ?? null,
        },
      );
      setStatus({ kind: "success", message: "Wiki page saved." });
      window.location.assign(result.redirectHref);
    } catch (error) {
      setStatus({
        kind: "error",
        message:
          error instanceof Error ? error.message : "Wiki page failed to save.",
      });
    }
  }

  return (
    <div className="grid gap-5">
      <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository wiki
          </p>
          <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
            {editingPage ? `Edit ${editingPage.title}` : "New Page"}
          </h1>
          <p className="t-sm mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            Draft a new page for{" "}
            <a
              className="t-mono-sm hover:underline"
              href={repositoryWikiHref(owner, repo)}
            >
              {owner}/{repo}
            </a>{" "}
            on <span className="t-mono-sm">{repository.default_branch}</span>.
            {editingPage ? (
              <>
                {" "}
                Saving includes the latest revision guard{" "}
                <span className="t-mono-sm">
                  {editingPage.latestRevisionId.slice(0, 8)}
                </span>
                .
              </>
            ) : (
              " Preview is rendered by the Rust API before the page is published."
            )}
          </p>
        </div>
        <a className="btn" href={repositoryWikiHref(owner, repo, "_pages")}>
          Pages
        </a>
      </section>

      {!canEdit ? (
        <section className="card p-5">
          <h2 className="t-h2" style={{ color: "var(--ink-1)" }}>
            Editor unavailable
          </h2>
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            Your repository role can read this wiki but cannot create pages.
          </p>
        </section>
      ) : (
        <section className="card overflow-hidden">
          <div
            className="grid gap-4 border-b px-5 py-4 lg:grid-cols-[minmax(0,1fr)_220px]"
            style={{
              borderColor: "var(--line)",
              background: "var(--surface-2)",
            }}
          >
            <div>
              <label className="t-label block" htmlFor="wiki-title">
                Page title
              </label>
              <input
                className="input mt-2 w-full"
                id="wiki-title"
                onChange={(event) => setTitle(event.target.value)}
                placeholder="Operations Guide"
                value={title}
              />
            </div>
            <div>
              <label className="t-label block" htmlFor="wiki-edit-mode">
                Edit mode
              </label>
              <select
                className="input mt-2 w-full"
                id="wiki-edit-mode"
                onChange={(event) =>
                  setEditMode(
                    event.target.value as RepositoryWikiMarkupFormat["mode"],
                  )
                }
                value={editMode}
              >
                {formats.map((format) => (
                  <option key={format.mode} value={format.mode}>
                    {format.label} ({format.extension})
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="px-5 pt-4">
            <div
              aria-label="Wiki editor tabs"
              className="tabs flex"
              role="tablist"
            >
              <button
                aria-selected={tab === "write"}
                className={`tab${tab === "write" ? " active" : ""}`}
                onClick={() => setTab("write")}
                role="tab"
                type="button"
              >
                Write
              </button>
              <button
                aria-selected={tab === "preview"}
                className={`tab${tab === "preview" ? " active" : ""}`}
                onClick={() => void renderPreview()}
                role="tab"
                type="button"
              >
                Preview
              </button>
            </div>
          </div>

          <div
            className="flex flex-wrap items-end gap-2 px-5 py-4"
            role="toolbar"
            aria-label="Wiki formatting toolbar"
          >
            <label className="grid gap-1">
              <span className="t-label">Image URL</span>
              <input
                className="input w-64 max-w-full"
                onChange={(event) => setImageUrl(event.target.value)}
                placeholder="https://example.com/diagram.png"
                value={imageUrl}
              />
            </label>
            <label className="grid gap-1">
              <span className="t-label">Alt text</span>
              <input
                className="input w-52 max-w-full"
                onChange={(event) => setImageAlt(event.target.value)}
                placeholder="Architecture diagram"
                value={imageAlt}
              />
            </label>
            <button className="btn sm" onClick={insertImage} type="button">
              <ImageIcon />
              Insert image
            </button>
          </div>

          <div className="px-5 pb-5">
            {tab === "write" ? (
              <div>
                <label className="sr-only" htmlFor="wiki-markdown">
                  Wiki page source
                </label>
                <textarea
                  className="input min-h-80 w-full resize-y p-3 t-mono leading-6"
                  id="wiki-markdown"
                  onChange={(event) => setMarkdown(event.target.value)}
                  value={markdown}
                />
                <p className="t-xs mt-2">{lineCount} lines</p>
              </div>
            ) : (
              <div
                className="min-h-80 rounded-md border p-4"
                style={{ borderColor: "var(--line)" }}
              >
                {preview ? (
                  <MarkdownBody html={preview.html} />
                ) : (
                  <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                    Preview has not rendered yet.
                  </p>
                )}
              </div>
            )}
          </div>

          <div
            className="grid gap-4 border-t px-5 py-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end"
            style={{ borderColor: "var(--line)" }}
          >
            <label>
              <span className="t-label block">Edit message</span>
              <input
                className="input mt-2 w-full"
                onChange={(event) => setMessage(event.target.value)}
                placeholder="Create wiki page"
                value={message}
              />
            </label>
            <button
              className="btn primary"
              disabled={status.kind === "pending"}
              onClick={() => void savePage()}
              type="button"
            >
              Save Page
            </button>
          </div>
          {status.message ? (
            <p
              className="t-sm px-5 pb-5"
              role={status.kind === "error" ? "alert" : "status"}
              style={{
                color:
                  status.kind === "error"
                    ? "var(--err)"
                    : status.kind === "success"
                      ? "var(--ok)"
                      : "var(--ink-3)",
              }}
            >
              {status.message}
            </p>
          ) : null}
        </section>
      )}
    </div>
  );
}
