"use client";

import Link from "next/link";
import { type FormEvent, useState } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RenderedMarkdown,
  RepositoryOverview,
  RepositorySecurityPolicyFetchResult,
  RepositorySecurityPolicyView,
} from "@/lib/api";

type RepositorySecurityPolicyEditorPageProps = {
  repository: RepositoryOverview;
  policyResult: RepositorySecurityPolicyFetchResult;
};

const starterMarkdown =
  "# Security policy\n\n## Reporting a vulnerability\n\nPlease email security@example.com with a summary, affected versions, and reproduction steps.\n\n## Supported versions\n\nSecurity fixes are published for the default branch and the latest release line.";

function policyActionPath(policy: RepositorySecurityPolicyView) {
  return `/${encodeURIComponent(policy.repository.ownerLogin)}/${encodeURIComponent(policy.repository.name)}/security/policy/actions`;
}

async function renderPreview(markdown: string): Promise<RenderedMarkdown> {
  const response = await fetch("/markdown/preview", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ markdown }),
  });
  if (!response.ok) {
    throw new Error("Preview failed");
  }
  return (await response.json()) as RenderedMarkdown;
}

function EditorUnavailable({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositorySecurityPolicyFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="policy" repository={repository}>
      <section className="card p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Security policy
        </p>
        <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
          Policy editor unavailable
        </h1>
        <p
          className="t-sm mt-3"
          role="status"
          style={{ color: "var(--ink-3)" }}
        >
          {result.message}
        </p>
      </section>
    </RepositorySecurityShell>
  );
}

function PolicyEditor({
  securityPolicy,
}: {
  securityPolicy: RepositorySecurityPolicyView;
}) {
  const { policy, viewer, repository } = securityPolicy;
  const [markdown, setMarkdown] = useState(policy.markdown ?? starterMarkdown);
  const [commitMessage, setCommitMessage] = useState(
    policy.exists ? "Update security policy" : "Create security policy",
  );
  const [tab, setTab] = useState<"write" | "preview">("write");
  const [preview, setPreview] = useState<RenderedMarkdown | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [savedPolicy, setSavedPolicy] =
    useState<RepositorySecurityPolicyView | null>(null);

  const path = policy.path ?? "SECURITY.md";
  const branch = policy.ref ?? repository.defaultBranch;
  const canSave = viewer.canEditPolicy && !saving;

  async function showPreview() {
    setTab("preview");
    if (!markdown.trim()) {
      setPreview(null);
      return;
    }
    setPreviewing(true);
    setError(null);
    try {
      setPreview(await renderPreview(markdown));
    } catch {
      setError("Preview could not be rendered.");
    } finally {
      setPreviewing(false);
    }
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setStatus(null);
    if (!markdown.trim()) {
      setError("Security policy content is required.");
      return;
    }
    if (!commitMessage.trim()) {
      setError("Commit message is required.");
      return;
    }
    setSaving(true);
    try {
      const response = await fetch(policyActionPath(securityPolicy), {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          action: policy.exists ? "update" : "create",
          commitMessage,
          expectedContentSha: policy.contentSha,
          markdown,
          path,
          ref: branch,
        }),
      });
      const body = await response.json();
      if (!response.ok) {
        throw new Error(
          body?.error?.message ?? "Security policy could not be saved.",
        );
      }
      const nextPolicy = body as RepositorySecurityPolicyView;
      setSavedPolicy(nextPolicy);
      setStatus("Security policy saved to the default branch.");
    } catch (saveError) {
      setError(
        saveError instanceof Error
          ? saveError.message
          : "Security policy could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="grid gap-6">
      <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Security and quality
          </p>
          <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
            {policy.exists ? "Edit security policy" : "Start security policy"}
          </h1>
          <p className="t-sm mt-3 max-w-3xl" style={{ color: "var(--ink-3)" }}>
            Commit changes to <span className="t-mono-sm">{path}</span> on{" "}
            <span className="t-mono-sm">{branch}</span>. Saved changes update
            the repository file, raw view, history, and policy metadata.
          </p>
        </div>
        <Link className="btn" href={securityPolicy.links.policyHref}>
          Back to policy
        </Link>
      </section>

      {!viewer.canEditPolicy ? (
        <section className="card p-5">
          <span className="chip warn">Reader view</span>
          <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
            You need write access to create or edit the repository security
            policy.
          </p>
        </section>
      ) : null}

      <form className="card overflow-hidden" onSubmit={submit}>
        <div
          className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-4"
          style={{ borderColor: "var(--line)" }}
        >
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Policy file
            </p>
            <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              {path}
            </h2>
          </div>
          <div className="tabs" role="tablist">
            <button
              className={`tab${tab === "write" ? " active" : ""}`}
              onClick={() => setTab("write")}
              role="tab"
              type="button"
            >
              Write
            </button>
            <button
              className={`tab${tab === "preview" ? " active" : ""}`}
              onClick={() => void showPreview()}
              role="tab"
              type="button"
            >
              Preview
            </button>
          </div>
        </div>

        <div className="grid gap-5 p-5">
          {tab === "write" ? (
            <label className="grid gap-2">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Markdown
              </span>
              <textarea
                className="input min-h-[360px] resize-y p-4 font-mono text-sm leading-6"
                disabled={!viewer.canEditPolicy}
                onChange={(event) => setMarkdown(event.target.value)}
                value={markdown}
              />
            </label>
          ) : (
            <section
              aria-label="Policy preview"
              className="min-h-[360px] rounded-[var(--radius)] border p-4"
              style={{
                background: "var(--surface)",
                borderColor: "var(--line)",
              }}
            >
              {preview?.html ? (
                <MarkdownBody html={preview.html} />
              ) : (
                <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                  {previewing ? "Rendering preview..." : "Nothing to preview."}
                </p>
              )}
            </section>
          )}

          <label className="grid gap-2">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              Commit message
            </span>
            <input
              className="input"
              disabled={!viewer.canEditPolicy}
              onChange={(event) => setCommitMessage(event.target.value)}
              value={commitMessage}
            />
          </label>

          {error ? (
            <p className="chip err justify-self-start" role="alert">
              {error}
            </p>
          ) : null}
          {status ? (
            <p className="chip ok justify-self-start" role="status">
              {status}
            </p>
          ) : null}

          <div className="flex flex-wrap items-center gap-2">
            <button className="btn primary" disabled={!canSave} type="submit">
              {saving
                ? "Saving..."
                : policy.exists
                  ? "Save changes"
                  : "Commit policy"}
            </button>
            {savedPolicy?.policy.sourceHref ? (
              <Link className="btn" href={savedPolicy.policy.sourceHref}>
                View file
              </Link>
            ) : null}
            {savedPolicy?.policy.rawHref ? (
              <Link className="btn" href={savedPolicy.policy.rawHref}>
                Open raw
              </Link>
            ) : null}
          </div>
        </div>
      </form>
    </div>
  );
}

export function RepositorySecurityPolicyEditorPage({
  repository,
  policyResult,
}: RepositorySecurityPolicyEditorPageProps) {
  if (!policyResult.ok) {
    return <EditorUnavailable repository={repository} result={policyResult} />;
  }

  return (
    <RepositorySecurityShell activeSection="policy" repository={repository}>
      <PolicyEditor securityPolicy={policyResult.securityPolicy} />
    </RepositorySecurityShell>
  );
}
