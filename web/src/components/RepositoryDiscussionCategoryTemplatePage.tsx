"use client";

import Link from "next/link";
import { type FormEvent, useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type {
  ApiErrorEnvelope,
  DiscussionCategoryTemplateView,
  DiscussionFormDefinition,
  RepositoryOverview,
} from "@/lib/api";

type RepositoryDiscussionCategoryTemplatePageProps = {
  repository: RepositoryOverview;
  template: DiscussionCategoryTemplateView | ApiErrorEnvelope;
};

function isApiError(
  template: DiscussionCategoryTemplateView | ApiErrorEnvelope,
): template is ApiErrorEnvelope {
  return "error" in template;
}

function templateEndpoint(owner: string, repo: string, categoryId: string) {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/settings/discussions/categories/${encodeURIComponent(categoryId)}/template`;
}

function previewEndpoint(owner: string, repo: string, categoryId: string) {
  return `${templateEndpoint(owner, repo, categoryId)}/preview`;
}

function fieldSummary(form: DiscussionFormDefinition) {
  if (!form.fields.length) return "No YAML fields parsed yet.";
  return `${form.fields.length} field${form.fields.length === 1 ? "" : "s"} parsed`;
}

function FormPreview({ form }: { form: DiscussionFormDefinition }) {
  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Parsed preview
          </p>
          <h2 className="t-h3 mt-1">{form.title}</h2>
          {form.description ? (
            <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
              {form.description}
            </p>
          ) : null}
        </div>
        <span className={form.valid ? "chip ok" : "chip warn"}>
          {form.valid ? fieldSummary(form) : "Fallback composer"}
        </span>
      </div>

      {form.parseError ? (
        <div
          className="mt-4 rounded-[var(--radius)] border p-3 t-sm"
          style={{ background: "var(--warn-soft)", borderColor: "var(--warn)" }}
        >
          {form.parseError}
        </div>
      ) : null}

      <div className="mt-4 grid gap-3">
        {form.body ? (
          <div className="rounded-[var(--radius)] border p-3">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Default body
            </p>
            <p className="t-sm mt-1 whitespace-pre-wrap">{form.body}</p>
          </div>
        ) : null}
        {form.fields.map((field) => (
          <div className="rounded-[var(--radius)] border p-3" key={field.id}>
            <div className="flex flex-wrap items-center gap-2">
              <span className="t-h3">{field.label}</span>
              <span className="chip soft">{field.fieldType}</span>
              {field.required ? (
                <span className="chip accent">Required</span>
              ) : null}
            </div>
            {field.description ? (
              <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
                {field.description}
              </p>
            ) : null}
            {field.options.length ? (
              <div className="mt-2 flex flex-wrap gap-2">
                {field.options.map((option) => (
                  <span className="chip soft" key={option}>
                    {option}
                  </span>
                ))}
              </div>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}

function UnavailableTemplate({
  repository,
  template,
}: {
  repository: RepositoryOverview;
  template: ApiErrorEnvelope;
}) {
  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/discussions`}
      frameClassName="max-w-5xl"
      repository={repository}
    >
      <section className="card p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Discussion template
        </p>
        <h1 className="t-h2 mt-1">Template editor is unavailable.</h1>
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          {template.error.message}
        </p>
      </section>
    </RepositoryShell>
  );
}

export function RepositoryDiscussionCategoryTemplatePage({
  repository,
  template,
}: RepositoryDiscussionCategoryTemplatePageProps) {
  const [current, setCurrent] = useState(template);
  const [content, setContent] = useState(
    isApiError(template) ? "" : template.content,
  );
  const [commitMessage, setCommitMessage] = useState(
    isApiError(template)
      ? "Update discussion template"
      : `Update ${template.category.name} discussion template`,
  );
  const [branch, setBranch] = useState(
    isApiError(template) ? "" : template.branch,
  );
  const [proposeChange, setProposeChange] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<"preview" | "commit" | null>(null);
  const owner = repository.owner_login;
  const repo = repository.name;

  const form = useMemo(
    () => (isApiError(current) ? null : current.form),
    [current],
  );

  if (isApiError(current)) {
    return <UnavailableTemplate repository={repository} template={current} />;
  }
  const templateView = current;

  async function preview() {
    setPending("preview");
    setNotice(null);
    setError(null);
    const response = await fetch(
      previewEndpoint(owner, repo, templateView.category.id),
      {
        body: JSON.stringify({ content }),
        headers: { "content-type": "application/json" },
        method: "POST",
      },
    );
    const payload = await response.json().catch(() => null);
    setPending(null);
    if (!response.ok) {
      const envelope = payload as ApiErrorEnvelope | null;
      setError(envelope?.error.message ?? "Template preview failed.");
      return;
    }
    setCurrent({ ...templateView, form: payload as DiscussionFormDefinition });
    setNotice("Template preview refreshed.");
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending("commit");
    setNotice(null);
    setError(null);
    const response = await fetch(
      templateEndpoint(owner, repo, templateView.category.id),
      {
        body: JSON.stringify({
          branch,
          commitMessage,
          content,
          expectedContentSha: templateView.contentSha,
          proposeChange,
        }),
        headers: { "content-type": "application/json" },
        method: "PUT",
      },
    );
    const payload = await response.json().catch(() => null);
    setPending(null);
    if (!response.ok) {
      const envelope = payload as ApiErrorEnvelope | null;
      setError(envelope?.error.message ?? "Template commit failed.");
      return;
    }
    const committed = payload as {
      commitHref: string;
      proposed: boolean;
      template: DiscussionCategoryTemplateView;
    };
    setCurrent(committed.template);
    setContent(committed.template.content);
    setNotice(
      committed.proposed
        ? "Template change was committed on a proposed branch."
        : "Template change was committed to the selected branch.",
    );
  }

  const settingsHref = `/${owner}/${repo}/discussions/categories/edit`;

  return (
    <RepositoryShell
      activePath={`/${owner}/${repo}/discussions`}
      frameClassName="grid grid-cols-[minmax(0,1fr)_320px] gap-8 max-lg:grid-cols-1"
      repository={repository}
    >
      <main className="min-w-0 space-y-5">
        <section className="card p-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Discussion template
              </p>
              <h1 className="t-h2 mt-1">
                {current.category.emoji} {current.category.name}
              </h1>
              <p
                className="t-sm mt-2 max-w-3xl"
                style={{ color: "var(--ink-3)" }}
              >
                Edit the YAML form stored at{" "}
                <span className="t-mono-sm">{current.path}</span>.
              </p>
            </div>
            <Link className="btn ghost sm" href={settingsHref}>
              Back to categories
            </Link>
          </div>
        </section>

        {error ? (
          <section
            className="card p-4"
            style={{ background: "var(--err-soft)", borderColor: "var(--err)" }}
          >
            <p className="t-label" style={{ color: "var(--err)" }}>
              Template error
            </p>
            <p className="t-sm mt-1">{error}</p>
          </section>
        ) : null}

        {notice ? (
          <section className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Notice
            </p>
            <p className="t-sm mt-1">{notice}</p>
          </section>
        ) : null}

        <form className="card overflow-hidden" onSubmit={submit}>
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3"
            style={{
              background: "var(--surface-2)",
              borderColor: "var(--line)",
            }}
          >
            <div>
              <h2 className="t-h3">YAML editor</h2>
              <p className="t-xs mt-1">
                <span className="t-mono-sm">{current.path}</span>
              </p>
            </div>
            <button
              className="btn sm"
              disabled={pending !== null || current.category.isPoll}
              onClick={preview}
              type="button"
            >
              {pending === "preview" ? "Previewing..." : "Preview"}
            </button>
          </div>
          <textarea
            aria-label="Discussion template YAML"
            className="w-full resize-y border-0 bg-transparent p-5 t-mono-sm outline-none"
            disabled={current.category.isPoll}
            onChange={(event) => setContent(event.target.value)}
            rows={18}
            spellCheck={false}
            style={{ color: "var(--ink-1)" }}
            value={content}
          />
          <div
            className="grid gap-4 border-t p-5"
            style={{ borderColor: "var(--line)" }}
          >
            <label className="grid gap-2">
              <span className="t-label">Commit message</span>
              <input
                aria-label="Commit message"
                className="input"
                maxLength={240}
                onChange={(event) => setCommitMessage(event.target.value)}
                value={commitMessage}
              />
            </label>
            <div className="grid gap-4 sm:grid-cols-2">
              <label className="grid gap-2">
                <span className="t-label">Branch</span>
                <input
                  aria-label="Branch"
                  className="input"
                  maxLength={120}
                  onChange={(event) => setBranch(event.target.value)}
                  value={branch}
                />
              </label>
              <label className="flex items-center gap-2 self-end t-sm">
                <input
                  checked={proposeChange}
                  onChange={(event) => setProposeChange(event.target.checked)}
                  type="checkbox"
                />
                Propose on a separate branch
              </label>
            </div>
            <div className="flex flex-wrap justify-end gap-2">
              {current.blobHref ? (
                <Link className="btn ghost" href={current.blobHref}>
                  View file
                </Link>
              ) : null}
              <button
                className="btn primary"
                disabled={pending !== null || current.category.isPoll}
                type="submit"
              >
                {pending === "commit" ? "Committing..." : "Commit template"}
              </button>
            </div>
          </div>
        </form>
      </main>

      <aside className="space-y-4">
        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Category
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            <span
              className={current.category.isPoll ? "chip warn" : "chip soft"}
            >
              {current.category.format.replaceAll("_", " ")}
            </span>
            {current.category.acceptsAnswers ? (
              <span className="chip ok">Answers</span>
            ) : null}
            <span className="chip soft">
              <span className="t-num">{current.category.count}</span>{" "}
              discussions
            </span>
          </div>
        </section>
        {form ? <FormPreview form={form} /> : null}
      </aside>
    </RepositoryShell>
  );
}
