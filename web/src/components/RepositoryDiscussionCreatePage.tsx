"use client";

import Link from "next/link";
import type { KeyboardEvent } from "react";
import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import type {
  ApiErrorEnvelope,
  CreateDiscussionResponse,
  DiscussionAttachmentDraft,
  DiscussionCreationView,
  RenderedMarkdown,
} from "@/lib/api";
import {
  repositoryDiscussionChooseCategoryHref,
  repositoryDiscussionsHref,
} from "@/lib/navigation";

type RepositoryDiscussionCreatePageProps = {
  creation: DiscussionCreationView;
  owner: string;
  repo: string;
};

type LocalDiscussionAttachment = DiscussionAttachmentDraft & {
  clientId: string;
};

type ToolbarAction = {
  label: string;
  ariaLabel: string;
  prefix: string;
  suffix: string;
  placeholder: string;
};

const EMPTY_PREVIEW = "<p>Nothing to preview</p>";
const MAX_ATTACHMENT_BYTES = 25 * 1024 * 1024;

const TOOLBAR_ACTIONS: ToolbarAction[] = [
  {
    label: "B",
    ariaLabel: "Bold",
    prefix: "**",
    suffix: "**",
    placeholder: "bold",
  },
  {
    label: "I",
    ariaLabel: "Italic",
    prefix: "_",
    suffix: "_",
    placeholder: "italic",
  },
  {
    label: "Code",
    ariaLabel: "Code",
    prefix: "`",
    suffix: "`",
    placeholder: "code",
  },
  {
    label: "Link",
    ariaLabel: "Link",
    prefix: "[",
    suffix: "](https://example.com)",
    placeholder: "link",
  },
  {
    label: "Quote",
    ariaLabel: "Quote",
    prefix: "> ",
    suffix: "",
    placeholder: "quote",
  },
  {
    label: "List",
    ariaLabel: "List",
    prefix: "- ",
    suffix: "",
    placeholder: "item",
  },
];

function defaultRendered(markdown: string): RenderedMarkdown {
  return {
    contentSha: "local-preview",
    html: markdown.trim() ? `<p>${escapeHtml(markdown)}</p>` : EMPTY_PREVIEW,
    cached: false,
  };
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function errorMessageFromEnvelope(envelope: ApiErrorEnvelope | null) {
  return envelope?.error.message ?? "Discussion could not be created.";
}

function attachmentFromFile(file: File): LocalDiscussionAttachment {
  const randomId =
    globalThis.crypto?.randomUUID?.() ??
    `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const safeName = file.name.replaceAll("/", "-").slice(0, 180);
  return {
    clientId: `${safeName}-${file.size}-${file.lastModified}-${randomId}`,
    fileName: safeName,
    contentType: file.type || "application/octet-stream",
    byteSize: file.size,
    storageKey: `discussion-drafts/${randomId}/${safeName}`,
  };
}

function attachmentPayload(
  attachment: LocalDiscussionAttachment,
): DiscussionAttachmentDraft {
  return {
    fileName: attachment.fileName,
    contentType: attachment.contentType,
    byteSize: attachment.byteSize,
    storageKey: attachment.storageKey,
  };
}

function similarHref(owner: string, repo: string, title: string) {
  const query = title.trim() ? `is:open ${title.trim()}` : "is:open";
  return repositoryDiscussionsHref(owner, repo, { q: query });
}

export function RepositoryDiscussionCreatePage({
  creation,
  owner,
  repo,
}: RepositoryDiscussionCreatePageProps) {
  const selected = creation.selectedCategory;
  const [title, setTitle] = useState(creation.form.title);
  const [body, setBody] = useState(creation.form.body);
  const [tab, setTab] = useState<"write" | "preview">("write");
  const [rendered, setRendered] = useState<RenderedMarkdown>(
    defaultRendered(creation.form.body),
  );
  const [attachments, setAttachments] = useState<LocalDiscussionAttachment[]>(
    [],
  );
  const [acknowledged, setAcknowledged] = useState(false);
  const [titleTouched, setTitleTouched] = useState(false);
  const [ackTouched, setAckTouched] = useState(false);
  const [isPreviewPending, setIsPreviewPending] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [created, setCreated] = useState<CreateDiscussionResponse | null>(null);

  const titleError = useMemo(
    () => (title.trim() ? null : "Title is required."),
    [title],
  );
  const attachmentError = useMemo(() => {
    if (attachments.length > 10)
      return "A discussion can attach at most 10 files.";
    const tooLarge = attachments.find(
      (attachment) => attachment.byteSize > MAX_ATTACHMENT_BYTES,
    );
    return tooLarge ? `${tooLarge.fileName} is larger than 25 MiB.` : null;
  }, [attachments]);
  const canSubmit =
    Boolean(selected) &&
    creation.enabled &&
    creation.viewer.canCreate &&
    !titleError &&
    !attachmentError &&
    acknowledged &&
    !isSubmitting;
  const createEndpoint = `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/discussions/new/create`;
  const searchHref = similarHref(owner, repo, title);

  async function showPreview(nextBody = body) {
    setTab("preview");
    setIsPreviewPending(true);
    setError(null);
    try {
      const nextRendered = await fetch("/markdown/preview", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          markdown: nextBody,
          owner,
          repo,
          enableTaskToggles: true,
        }),
      }).then((response) => {
        if (!response.ok) throw new Error("Preview failed");
        return response.json() as Promise<RenderedMarkdown>;
      });
      setRendered(
        nextBody.trim()
          ? nextRendered
          : { ...nextRendered, html: EMPTY_PREVIEW },
      );
    } catch {
      setRendered(defaultRendered(nextBody));
      setError("Preview could not be rendered.");
    } finally {
      setIsPreviewPending(false);
    }
  }

  function applyToolbarAction(action: ToolbarAction) {
    setBody((current) =>
      current
        ? `${current}\n${action.prefix}${action.placeholder}${action.suffix}`
        : `${action.prefix}${action.placeholder}${action.suffix}`,
    );
    setTab("write");
  }

  async function submit() {
    setTitleTouched(true);
    setAckTouched(true);
    setError(null);
    setCreated(null);

    if (!selected) {
      setError("Choose a discussion category before starting.");
      return;
    }
    if (titleError) return;
    if (attachmentError) {
      setError(attachmentError);
      return;
    }
    if (!acknowledged) {
      setError("Confirm that you searched for similar discussions first.");
      return;
    }

    setIsSubmitting(true);
    try {
      const response = await fetch(createEndpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          categorySlug: selected.slug,
          title: title.trim(),
          body: body.trim() ? body : null,
          similarSearchAcknowledged: acknowledged,
          attachmentDrafts: attachments.map(attachmentPayload),
        }),
      });
      const payload = (await response.json().catch(() => null)) as
        | CreateDiscussionResponse
        | ApiErrorEnvelope
        | null;
      if (!response.ok) {
        setError(errorMessageFromEnvelope(payload as ApiErrorEnvelope | null));
        return;
      }
      const discussion = payload as CreateDiscussionResponse;
      setCreated(discussion);
      window.location.assign(discussion.href);
    } catch {
      setError("Discussion could not be created.");
    } finally {
      setIsSubmitting(false);
    }
  }

  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      void submit();
    }
  }

  if (!selected) {
    return (
      <section className="card p-6">
        <h1 className="t-h2">Choose a category</h1>
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          Select one of the repository discussion categories before starting a
          thread.
        </p>
        <Link
          className="btn primary mt-4"
          href={repositoryDiscussionChooseCategoryHref(owner, repo)}
        >
          Choose category
        </Link>
      </section>
    );
  }

  return (
    <>
      <main className="min-w-0 space-y-5">
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            New discussion
          </p>
          <h1 className="t-h2 mt-1 break-words">
            {selected.emoji} {selected.name}
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {selected.description ?? "Start a focused repository conversation."}
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            {selected.acceptsAnswers ? (
              <span className="chip ok">Answers enabled</span>
            ) : null}
            {creation.form.fallback ? (
              <span className="chip soft">Generic composer</span>
            ) : null}
            <Link
              className="chip soft hover:underline"
              href={repositoryDiscussionChooseCategoryHref(owner, repo)}
            >
              Choose a different category
            </Link>
          </div>
        </section>

        {creation.enabled && creation.viewer.canCreate ? null : (
          <section
            className="card p-4"
            style={{ background: "var(--warn-soft)" }}
          >
            <p className="t-label" style={{ color: "var(--warn)" }}>
              Creation unavailable
            </p>
            <p className="t-sm mt-1" style={{ color: "var(--ink-2)" }}>
              {creation.disabledReason ??
                "You do not have permission to create discussions in this repository."}
            </p>
          </section>
        )}

        <section className="card overflow-hidden">
          <div className="border-b p-4" style={{ borderColor: "var(--line)" }}>
            <label className="t-label" htmlFor="discussion-title">
              Title <span aria-hidden="true">*</span>
            </label>
            <input
              aria-describedby={
                titleTouched && titleError
                  ? "discussion-title-error"
                  : undefined
              }
              aria-invalid={titleTouched && titleError ? "true" : "false"}
              aria-required="true"
              className="input mt-2 w-full"
              id="discussion-title"
              onBlur={() => setTitleTouched(true)}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Ask a question or propose an idea"
              required
              value={title}
            />
            {titleTouched && titleError ? (
              <p
                className="mt-2 t-sm"
                id="discussion-title-error"
                role="alert"
                style={{ color: "var(--err)" }}
              >
                {titleError}
              </p>
            ) : null}
          </div>

          <div>
            <div
              className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-2"
              style={{
                borderColor: "var(--line)",
                background: "var(--surface-2)",
              }}
            >
              <div
                aria-label="Markdown formatting toolbar"
                className="flex flex-wrap gap-1"
                role="toolbar"
              >
                {TOOLBAR_ACTIONS.map((action) => (
                  <button
                    aria-label={action.ariaLabel}
                    className="btn ghost sm"
                    key={action.ariaLabel}
                    onClick={() => applyToolbarAction(action)}
                    type="button"
                  >
                    {action.label}
                  </button>
                ))}
              </div>
              <span className="kbd">Command+Enter</span>
            </div>
            <div
              aria-label="Discussion body tabs"
              className="tabs px-4 pt-3"
              role="tablist"
            >
              <button
                aria-controls="discussion-body-write-panel"
                aria-selected={tab === "write"}
                className={`tab${tab === "write" ? " active" : ""}`}
                id="discussion-body-write-tab"
                onClick={() => setTab("write")}
                role="tab"
                type="button"
              >
                Write
              </button>
              <button
                aria-controls="discussion-body-preview-panel"
                aria-selected={tab === "preview"}
                className={`tab${tab === "preview" ? " active" : ""}`}
                id="discussion-body-preview-tab"
                onClick={() => void showPreview()}
                role="tab"
                type="button"
              >
                Preview
              </button>
            </div>
            <div className="p-4">
              {tab === "write" ? (
                <div
                  aria-labelledby="discussion-body-write-tab"
                  id="discussion-body-write-panel"
                  role="tabpanel"
                >
                  <label className="sr-only" htmlFor="discussion-body">
                    Discussion body
                  </label>
                  <textarea
                    className="input min-h-72 w-full resize-y p-3 t-mono leading-6"
                    id="discussion-body"
                    onChange={(event) => setBody(event.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Add context, examples, screenshots, or proposed next steps."
                    value={body}
                  />
                  <p className="mt-2 t-xs" style={{ color: "var(--ink-3)" }}>
                    Markdown preview is rendered by the Rust sanitizer before
                    anything is created.
                  </p>
                </div>
              ) : (
                <div
                  aria-labelledby="discussion-body-preview-tab"
                  id="discussion-body-preview-panel"
                  role="tabpanel"
                >
                  <MarkdownBody html={rendered.html} />
                  {isPreviewPending ? (
                    <p
                      className="mt-3 t-sm"
                      role="status"
                      style={{ color: "var(--ink-3)" }}
                    >
                      Rendering preview...
                    </p>
                  ) : null}
                </div>
              )}
            </div>
          </div>
        </section>

        <section className="card p-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <label className="t-label" htmlFor="discussion-attachments">
                Attachments
              </label>
              <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                Add screenshots or logs. Files are recorded as bounded draft
                metadata for the Rust API to attach to the opening comment.
              </p>
            </div>
            <label className="btn sm" htmlFor="discussion-attachments">
              Add files
            </label>
            <input
              className="sr-only"
              id="discussion-attachments"
              multiple
              onChange={(event) => {
                const files = Array.from(event.target.files ?? []);
                setAttachments((current) => [
                  ...current,
                  ...files.map(attachmentFromFile),
                ]);
                event.currentTarget.value = "";
              }}
              type="file"
            />
          </div>
          {attachments.length ? (
            <ul
              className="mt-4 divide-y"
              style={{ borderColor: "var(--line)" }}
            >
              {attachments.map((attachment) => (
                <li
                  className="flex flex-wrap items-center justify-between gap-3 py-3"
                  key={attachment.clientId}
                >
                  <div>
                    <p className="t-sm">{attachment.fileName}</p>
                    <p className="t-xs">
                      {Math.max(0, attachment.byteSize).toLocaleString()} bytes
                      {attachment.contentType
                        ? ` · ${attachment.contentType}`
                        : ""}
                    </p>
                  </div>
                  <button
                    className="btn ghost sm"
                    onClick={() =>
                      setAttachments((current) =>
                        current.filter(
                          (item) => item.clientId !== attachment.clientId,
                        ),
                      )
                    }
                    type="button"
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-4 t-xs" style={{ color: "var(--ink-3)" }}>
              No attachments selected.
            </p>
          )}
          {attachmentError ? (
            <p
              className="mt-3 t-sm"
              role="alert"
              style={{ color: "var(--err)" }}
            >
              {attachmentError}
            </p>
          ) : null}
        </section>

        <section className="card p-4">
          <label
            className="flex items-start gap-3 t-sm"
            htmlFor="discussion-similar-ack"
          >
            <input
              aria-describedby={
                ackTouched && !acknowledged
                  ? "discussion-similar-error"
                  : undefined
              }
              checked={acknowledged}
              id="discussion-similar-ack"
              onBlur={() => setAckTouched(true)}
              onChange={(event) => setAcknowledged(event.target.checked)}
              type="checkbox"
            />
            <span>
              I have done a search for similar discussions.
              <Link className="ml-2 underline" href={searchHref}>
                Search using this title
              </Link>
            </span>
          </label>
          {ackTouched && !acknowledged ? (
            <p
              className="mt-2 t-sm"
              id="discussion-similar-error"
              role="alert"
              style={{ color: "var(--err)" }}
            >
              Similar-search acknowledgement is required.
            </p>
          ) : null}
        </section>

        {error ? (
          <p className="chip err" role="alert">
            {error}
          </p>
        ) : null}
        {created ? (
          <p className="chip ok" role="status">
            Discussion created. Opening #{created.discussionNumber}.
          </p>
        ) : null}

        <div className="flex flex-wrap justify-end gap-2">
          <Link className="btn" href={repositoryDiscussionsHref(owner, repo)}>
            Cancel
          </Link>
          <button
            className="btn accent"
            disabled={!canSubmit}
            onClick={() => void submit()}
            type="button"
          >
            {isSubmitting ? "Starting..." : "Start discussion"}
          </button>
        </div>
      </main>

      <aside className="space-y-4">
        <section className="card p-4">
          <h2 className="t-h3">Similar discussions</h2>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            The search link updates from the title you type.
          </p>
          <Link
            className="t-sm mt-3 inline-block hover:underline"
            href={searchHref}
          >
            Search before posting
          </Link>
        </section>

        <section className="card p-4">
          <h2 className="t-h3">Community resources</h2>
          <div className="mt-3 grid gap-2">
            {creation.communityLinks.length ? (
              creation.communityLinks.map((link) => (
                <Link
                  className="t-sm hover:underline"
                  href={link.href}
                  key={link.id}
                >
                  {link.label}
                </Link>
              ))
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                Community links have not been published for this repository.
              </p>
            )}
          </div>
        </section>
      </aside>
    </>
  );
}
