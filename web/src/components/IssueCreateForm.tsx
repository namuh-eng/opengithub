"use client";

import Link from "next/link";
import type { KeyboardEvent } from "react";
import { useMemo, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import type {
  ApiErrorEnvelope,
  CreatedIssue,
  IssueAttachmentInput,
  IssueFormField,
  RenderedMarkdown,
} from "@/lib/api";

type IssueCreateFormProps = {
  owner: string;
  repo: string;
  initialTitle?: string;
  initialBody?: string;
  defaultLabelIds?: string[];
  defaultAssigneeUserIds?: string[];
  defaultMilestoneId?: string | null;
  templateId?: string | null;
  templateSlug?: string | null;
  templateName?: string | null;
  formFields?: IssueFormField[];
  cancelHref: string;
  onCreated?: (issue: CreatedIssue) => void;
  previewMarkdown?: (markdown: string) => Promise<RenderedMarkdown>;
};

type ToolbarAction = {
  label: string;
  ariaLabel: string;
  prefix: string;
  suffix: string;
  placeholder: string;
};

type LocalIssueAttachment = IssueAttachmentInput & {
  clientId: string;
};

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
    label: "Task",
    ariaLabel: "Task list",
    prefix: "- [ ] ",
    suffix: "",
    placeholder: "task",
  },
  {
    label: "Quote",
    ariaLabel: "Quote",
    prefix: "> ",
    suffix: "",
    placeholder: "quote",
  },
];

const EMPTY_PREVIEW = "<p>Nothing to preview</p>";

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
  return envelope?.error.message ?? "Issue could not be created.";
}

function initialFieldValue(field: IssueFormField) {
  return field.value ?? "";
}

function attachmentInputFromFile(file: File): LocalIssueAttachment {
  const randomId =
    globalThis.crypto?.randomUUID?.() ??
    `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return {
    clientId: `${file.name}-${file.size}-${file.lastModified}-${randomId}`,
    fileName: file.name,
    byteSize: file.size,
    contentType: file.type || null,
  };
}

function attachmentPayload(
  attachment: LocalIssueAttachment,
): IssueAttachmentInput {
  return {
    fileName: attachment.fileName,
    byteSize: attachment.byteSize,
    contentType: attachment.contentType,
  };
}

export function IssueCreateForm({
  owner,
  repo,
  initialTitle = "",
  initialBody = "",
  defaultLabelIds = [],
  defaultAssigneeUserIds = [],
  defaultMilestoneId = null,
  templateId = null,
  templateSlug = null,
  templateName = null,
  formFields = [],
  cancelHref,
  onCreated,
  previewMarkdown,
}: IssueCreateFormProps) {
  const [title, setTitle] = useState(initialTitle);
  const [body, setBody] = useState(initialBody);
  const [tab, setTab] = useState<"write" | "preview">("write");
  const [fieldValues, setFieldValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(
      formFields.map((field) => [field.fieldKey, initialFieldValue(field)]),
    ),
  );
  const [fieldTabs, setFieldTabs] = useState<
    Record<string, "write" | "preview">
  >(() =>
    Object.fromEntries(formFields.map((field) => [field.fieldKey, "write"])),
  );
  const [fieldPreviews, setFieldPreviews] = useState<
    Record<string, RenderedMarkdown>
  >(() =>
    Object.fromEntries(
      formFields.map((field) => [
        field.fieldKey,
        defaultRendered(initialFieldValue(field)),
      ]),
    ),
  );
  const [fieldTouched, setFieldTouched] = useState<Record<string, boolean>>({});
  const [attachments, setAttachments] = useState<LocalIssueAttachment[]>([]);
  const [pendingPreviewKey, setPendingPreviewKey] = useState<string | null>(
    null,
  );
  const [createMore, setCreateMore] = useState(false);
  const [rendered, setRendered] = useState<RenderedMarkdown>(
    defaultRendered(initialBody),
  );
  const [isPreviewPending, setIsPreviewPending] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [titleTouched, setTitleTouched] = useState(false);
  const [createdIssue, setCreatedIssue] = useState<CreatedIssue | null>(null);

  const titleError = useMemo(
    () => (title.trim() ? null : "Title is required."),
    [title],
  );
  const fieldErrors = useMemo(
    () =>
      Object.fromEntries(
        formFields.map((field) => [
          field.fieldKey,
          field.required && !fieldValues[field.fieldKey]?.trim()
            ? `${field.label} is required.`
            : null,
        ]),
      ) as Record<string, string | null>,
    [fieldValues, formFields],
  );
  const hasFieldErrors = Object.values(fieldErrors).some(Boolean);
  const canSubmit = !titleError && !hasFieldErrors && !isSubmitting;
  const createEndpoint = `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/new/create`;

  async function showPreview(nextBody = body) {
    setTab("preview");
    setIsPreviewPending(true);
    setError(null);
    try {
      const nextRendered =
        previewMarkdown !== undefined
          ? await previewMarkdown(nextBody)
          : await fetch("/markdown/preview", {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify({
                markdown: nextBody,
                owner,
                repo,
                enableTaskToggles: true,
              }),
            }).then((response) => {
              if (!response.ok) {
                throw new Error("Preview failed");
              }
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

  async function showFieldPreview(field: IssueFormField) {
    const value = fieldValues[field.fieldKey] ?? "";
    setFieldTabs((current) => ({ ...current, [field.fieldKey]: "preview" }));
    setPendingPreviewKey(field.fieldKey);
    setError(null);
    try {
      const nextRendered =
        previewMarkdown !== undefined
          ? await previewMarkdown(value)
          : await fetch("/markdown/preview", {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify({
                markdown: value,
                owner,
                repo,
                enableTaskToggles: true,
              }),
            }).then((response) => {
              if (!response.ok) {
                throw new Error("Preview failed");
              }
              return response.json() as Promise<RenderedMarkdown>;
            });
      setFieldPreviews((current) => ({
        ...current,
        [field.fieldKey]: value.trim()
          ? nextRendered
          : { ...nextRendered, html: EMPTY_PREVIEW },
      }));
    } catch {
      setFieldPreviews((current) => ({
        ...current,
        [field.fieldKey]: defaultRendered(value),
      }));
      setError("Preview could not be rendered.");
    } finally {
      setPendingPreviewKey(null);
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
    setError(null);
    setCreatedIssue(null);
    setFieldTouched(
      Object.fromEntries(formFields.map((field) => [field.fieldKey, true])),
    );
    if (titleError) {
      return;
    }
    if (hasFieldErrors) {
      setError("Complete the required template fields before creating.");
      return;
    }

    setIsSubmitting(true);
    try {
      const requestBody = {
        title: title.trim(),
        body: body.trim() ? body : null,
        ...(templateId ? { templateId } : {}),
        ...(templateSlug ? { templateSlug } : {}),
        ...(formFields.length ? { fieldValues } : {}),
        labelIds: defaultLabelIds,
        assigneeUserIds: defaultAssigneeUserIds,
        ...(defaultMilestoneId ? { milestoneId: defaultMilestoneId } : {}),
        ...(attachments.length
          ? { attachments: attachments.map(attachmentPayload) }
          : {}),
      };
      const response = await fetch(createEndpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(requestBody),
      });
      const payload = (await response.json().catch(() => null)) as
        | CreatedIssue
        | ApiErrorEnvelope
        | null;

      if (!response.ok) {
        setError(errorMessageFromEnvelope(payload as ApiErrorEnvelope | null));
        return;
      }

      const issue = payload as CreatedIssue;
      if (createMore) {
        setCreatedIssue(issue);
        setTitle("");
        setBody("");
        setFieldValues(
          Object.fromEntries(formFields.map((field) => [field.fieldKey, ""])),
        );
        setFieldPreviews(
          Object.fromEntries(
            formFields.map((field) => [field.fieldKey, defaultRendered("")]),
          ),
        );
        setFieldTabs(
          Object.fromEntries(
            formFields.map((field) => [field.fieldKey, "write"]),
          ),
        );
        setFieldTouched({});
        setRendered(defaultRendered(""));
        setAttachments([]);
        setTab("write");
        setTitleTouched(false);
        return;
      }

      if (onCreated) {
        onCreated(issue);
      } else {
        window.location.assign(
          issue.href ?? `/${owner}/${repo}/issues/${issue.number}`,
        );
      }
    } catch {
      setError("Issue could not be created.");
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

  function handleFieldKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      void submit();
    }
  }

  return (
    <section aria-labelledby="issue-create-title" className="space-y-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="t-label" style={{ color: "var(--accent)" }}>
            Issues
          </p>
          <h1 className="t-h1 mt-2" id="issue-create-title">
            Create new issue
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {templateName
              ? `Using the ${templateName} template. Review the defaults, then create the issue.`
              : "Start with a focused title and a Markdown body."}
          </p>
        </div>
        <Link className="btn" href={cancelHref}>
          Cancel
        </Link>
      </div>

      {formFields.length ? (
        <div className="space-y-4">
          {formFields.map((field) => {
            const value = fieldValues[field.fieldKey] ?? "";
            const fieldError = fieldErrors[field.fieldKey];
            const fieldId = `issue-field-${field.fieldKey}`;
            const errorId = `${fieldId}-error`;
            const writeTabId = `${fieldId}-write-tab`;
            const previewTabId = `${fieldId}-preview-tab`;
            const writePanelId = `${fieldId}-write-panel`;
            const previewPanelId = `${fieldId}-preview-panel`;
            const isMarkdown =
              field.fieldType === "markdown" || field.fieldType === "textarea";
            return (
              <div className="card overflow-hidden" key={field.id}>
                <div
                  className="border-b p-4"
                  style={{ borderColor: "var(--line)" }}
                >
                  <label className="t-label" htmlFor={fieldId}>
                    {field.label}{" "}
                    {field.required ? <span aria-hidden="true">*</span> : null}
                  </label>
                  {field.description ? (
                    <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
                      {field.description}
                    </p>
                  ) : null}
                </div>
                {isMarkdown ? (
                  <div>
                    <div
                      aria-label={`${field.label} tabs`}
                      className="tabs px-4 pt-3"
                      role="tablist"
                    >
                      <button
                        aria-controls={writePanelId}
                        aria-selected={fieldTabs[field.fieldKey] !== "preview"}
                        className={`tab${
                          fieldTabs[field.fieldKey] !== "preview"
                            ? " active"
                            : ""
                        }`}
                        id={writeTabId}
                        onClick={() =>
                          setFieldTabs((current) => ({
                            ...current,
                            [field.fieldKey]: "write",
                          }))
                        }
                        role="tab"
                        type="button"
                      >
                        Write
                      </button>
                      <button
                        aria-controls={previewPanelId}
                        aria-selected={fieldTabs[field.fieldKey] === "preview"}
                        className={`tab${
                          fieldTabs[field.fieldKey] === "preview"
                            ? " active"
                            : ""
                        }`}
                        id={previewTabId}
                        onClick={() => void showFieldPreview(field)}
                        role="tab"
                        type="button"
                      >
                        Preview
                      </button>
                    </div>
                    <div className="p-4">
                      {fieldTabs[field.fieldKey] === "preview" ? (
                        <div
                          aria-labelledby={previewTabId}
                          id={previewPanelId}
                          role="tabpanel"
                        >
                          <MarkdownBody
                            html={
                              fieldPreviews[field.fieldKey]?.html ??
                              EMPTY_PREVIEW
                            }
                          />
                          {pendingPreviewKey === field.fieldKey ? (
                            <p
                              className="mt-3 t-sm"
                              role="status"
                              style={{ color: "var(--ink-3)" }}
                            >
                              Rendering preview...
                            </p>
                          ) : null}
                        </div>
                      ) : (
                        <div
                          aria-labelledby={writeTabId}
                          id={writePanelId}
                          role="tabpanel"
                        >
                          <textarea
                            aria-describedby={
                              fieldTouched[field.fieldKey] && fieldError
                                ? errorId
                                : undefined
                            }
                            aria-invalid={
                              fieldTouched[field.fieldKey] && fieldError
                                ? "true"
                                : "false"
                            }
                            aria-required={field.required}
                            className="input min-h-40 w-full resize-y p-3 t-mono leading-6"
                            id={fieldId}
                            onBlur={() =>
                              setFieldTouched((current) => ({
                                ...current,
                                [field.fieldKey]: true,
                              }))
                            }
                            onChange={(event) =>
                              setFieldValues((current) => ({
                                ...current,
                                [field.fieldKey]: event.target.value,
                              }))
                            }
                            onKeyDown={handleFieldKeyDown}
                            placeholder={field.placeholder ?? ""}
                            required={field.required}
                            value={value}
                          />
                        </div>
                      )}
                    </div>
                  </div>
                ) : (
                  <div className="p-4">
                    <input
                      aria-describedby={
                        fieldTouched[field.fieldKey] && fieldError
                          ? errorId
                          : undefined
                      }
                      aria-invalid={
                        fieldTouched[field.fieldKey] && fieldError
                          ? "true"
                          : "false"
                      }
                      aria-required={field.required}
                      className="input w-full"
                      id={fieldId}
                      onBlur={() =>
                        setFieldTouched((current) => ({
                          ...current,
                          [field.fieldKey]: true,
                        }))
                      }
                      onChange={(event) =>
                        setFieldValues((current) => ({
                          ...current,
                          [field.fieldKey]: event.target.value,
                        }))
                      }
                      placeholder={field.placeholder ?? ""}
                      required={field.required}
                      value={value}
                    />
                  </div>
                )}
                {fieldTouched[field.fieldKey] && fieldError ? (
                  <p
                    className="px-4 pb-4 t-sm"
                    id={errorId}
                    role="alert"
                    style={{ color: "var(--err)" }}
                  >
                    {fieldError}
                  </p>
                ) : null}
              </div>
            );
          })}
        </div>
      ) : null}

      {defaultLabelIds.length || defaultAssigneeUserIds.length ? (
        <div className="flex flex-wrap gap-2">
          {defaultLabelIds.length ? (
            <span className="chip soft">
              {defaultLabelIds.length} default{" "}
              {defaultLabelIds.length === 1 ? "label" : "labels"}
            </span>
          ) : null}
          {defaultAssigneeUserIds.length ? (
            <span className="chip soft">
              {defaultAssigneeUserIds.length} default{" "}
              {defaultAssigneeUserIds.length === 1 ? "assignee" : "assignees"}
            </span>
          ) : null}
        </div>
      ) : null}

      <div className="card overflow-hidden">
        <div className="border-b p-4" style={{ borderColor: "var(--line)" }}>
          <label className="t-label" htmlFor="issue-title">
            Title <span aria-hidden="true">*</span>
          </label>
          <input
            aria-describedby={
              titleTouched && titleError ? "title-error" : undefined
            }
            aria-invalid={titleTouched && titleError ? "true" : "false"}
            aria-required="true"
            className="input mt-2 w-full"
            id="issue-title"
            onBlur={() => setTitleTouched(true)}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="Briefly describe the work"
            required
            value={title}
          />
          {titleTouched && titleError ? (
            <p
              className="mt-2 t-sm"
              id="title-error"
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
                  key={action.label}
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
            aria-label="Issue body tabs"
            className="tabs px-4 pt-3"
            role="tablist"
          >
            <button
              aria-controls="issue-body-write-panel"
              aria-selected={tab === "write"}
              className={`tab${tab === "write" ? " active" : ""}`}
              id="issue-body-write-tab"
              onClick={() => setTab("write")}
              role="tab"
              type="button"
            >
              Write
            </button>
            <button
              aria-controls="issue-body-preview-panel"
              aria-selected={tab === "preview"}
              className={`tab${tab === "preview" ? " active" : ""}`}
              id="issue-body-preview-tab"
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
                aria-labelledby="issue-body-write-tab"
                id="issue-body-write-panel"
                role="tabpanel"
              >
                <label className="sr-only" htmlFor="issue-body">
                  Issue body
                </label>
                <textarea
                  className="input min-h-72 w-full resize-y p-3 t-mono leading-6"
                  id="issue-body"
                  onChange={(event) => setBody(event.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Add context, reproduction steps, screenshots, or a task list."
                  value={body}
                />
                <p className="mt-2 t-xs" style={{ color: "var(--ink-3)" }}>
                  Markdown is supported. Selected attachments are recorded as
                  metadata until binary upload storage is connected.
                </p>
              </div>
            ) : (
              <div
                aria-labelledby="issue-body-preview-tab"
                id="issue-body-preview-panel"
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
      </div>

      <div className="card p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <label className="t-label" htmlFor="issue-attachments">
              Attachments
            </label>
            <p className="mt-2 t-sm" style={{ color: "var(--ink-3)" }}>
              Add screenshots or logs. Files are not uploaded yet; opengithub
              stores filename, size, and type with this issue.
            </p>
          </div>
          <label className="btn sm" htmlFor="issue-attachments">
            Add files
          </label>
          <input
            className="sr-only"
            id="issue-attachments"
            multiple
            onChange={(event) => {
              const files = Array.from(event.target.files ?? []);
              setAttachments((current) => [
                ...current,
                ...files.map(attachmentInputFromFile),
              ]);
              event.currentTarget.value = "";
            }}
            type="file"
          />
        </div>
        {attachments.length ? (
          <ul className="mt-4 divide-y" style={{ borderColor: "var(--line)" }}>
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
      </div>

      {error ? (
        <p className="chip err" role="alert">
          {error}
        </p>
      ) : null}
      {createdIssue ? (
        <p className="chip ok" role="status">
          Created{" "}
          <Link
            className="underline"
            href={createdIssue.href ?? `${cancelHref}/${createdIssue.number}`}
          >
            issue #{createdIssue.number}
          </Link>
          . The form is ready for another issue.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center justify-between gap-4">
        <label className="flex items-center gap-2 t-sm" htmlFor="create-more">
          <input
            checked={createMore}
            id="create-more"
            onChange={(event) => setCreateMore(event.target.checked)}
            type="checkbox"
          />
          Create more
        </label>
        <div className="flex flex-wrap gap-2">
          <Link className="btn" href={cancelHref}>
            Cancel
          </Link>
          <button
            className="btn accent"
            disabled={!canSubmit}
            onClick={() => void submit()}
            type="button"
          >
            {isSubmitting ? "Creating..." : "Create issue"}
          </button>
        </div>
      </div>
    </section>
  );
}
