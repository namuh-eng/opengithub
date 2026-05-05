"use client";

import Link from "next/link";
import { useState, useTransition } from "react";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecurityAdvisoryCreate,
  RepositorySecurityAdvisoryDetail,
} from "@/lib/api";

type RepositorySecurityAdvisoryCreatePageProps = {
  repository: RepositoryOverview;
};

function toNullable(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

export function RepositorySecurityAdvisoryCreatePage({
  repository,
}: RepositorySecurityAdvisoryCreatePageProps) {
  const [isPending, startTransition] = useTransition();
  const [created, setCreated] =
    useState<RepositorySecurityAdvisoryDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState({
    title: "",
    summary: "",
    detailsMarkdown: "",
    severity: "moderate",
    cveId: "",
    packageEcosystem: "",
    packageName: "",
    affectedVersions: "",
    patchedVersions: "",
    cvssVector: "",
    cvssScore: "",
  });
  const baseHref = `/${repository.owner_login}/${repository.name}/security/advisories`;
  const update = (field: keyof typeof draft, value: string) => {
    setCreated(null);
    setError(null);
    setDraft((current) => ({ ...current, [field]: value }));
  };
  const submit = () => {
    const score = draft.cvssScore.trim() ? Number(draft.cvssScore) : null;
    const body: RepositorySecurityAdvisoryCreate = {
      title: draft.title,
      summary: toNullable(draft.summary),
      detailsMarkdown: toNullable(draft.detailsMarkdown),
      cveId: toNullable(draft.cveId),
      severity: toNullable(draft.severity),
      packageEcosystem: toNullable(draft.packageEcosystem),
      packageName: toNullable(draft.packageName),
      affectedVersions: toNullable(draft.affectedVersions),
      patchedVersions: toNullable(draft.patchedVersions),
      cvssVector: toNullable(draft.cvssVector),
      cvssScore: Number.isFinite(score) ? score : null,
      cvssMetrics: null,
      cwes: [],
      credits: [],
      collaborators: [],
    };

    startTransition(async () => {
      const response = await fetch(`${baseHref}/actions`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const payload = (await response.json().catch(() => null)) as
        | RepositorySecurityAdvisoryDetail
        | { error?: { message?: string } }
        | null;
      if (!response.ok) {
        setError(
          (payload as { error?: { message?: string } } | null)?.error
            ?.message ?? "Draft security advisory creation failed.",
        );
        return;
      }
      setCreated(payload as RepositorySecurityAdvisoryDetail);
    });
  };

  return (
    <RepositorySecurityShell activeSection="advisories" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-3">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Repository advisory
          </p>
          <div className="between flex-wrap gap-3">
            <div>
              <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
                New draft security advisory
              </h1>
              <p
                className="t-sm mt-2 max-w-2xl"
                style={{ color: "var(--ink-3)" }}
              >
                Drafts stay private to maintainers and advisory collaborators
                until a maintainer publishes them.
              </p>
            </div>
            <Link className="btn" href={baseHref}>
              All advisories
            </Link>
          </div>
        </section>

        <section className="card grid gap-4 p-5" aria-label="Create advisory">
          <div className="grid gap-3 md:grid-cols-2">
            <label className="grid gap-1 t-sm md:col-span-2">
              Title
              <input
                className="input"
                onChange={(event) => update("title", event.target.value)}
                placeholder="Briefly describe the vulnerability"
                value={draft.title}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Severity
              <select
                className="input"
                onChange={(event) => update("severity", event.target.value)}
                value={draft.severity}
              >
                <option value="low">Low</option>
                <option value="moderate">Moderate</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </label>
            <label className="grid gap-1 t-sm">
              CVE
              <input
                className="input"
                onChange={(event) => update("cveId", event.target.value)}
                placeholder="CVE-2026-1234"
                value={draft.cveId}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Ecosystem
              <input
                className="input"
                onChange={(event) =>
                  update("packageEcosystem", event.target.value)
                }
                placeholder="cargo"
                value={draft.packageEcosystem}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Package
              <input
                className="input"
                onChange={(event) => update("packageName", event.target.value)}
                placeholder="opengithub-api"
                value={draft.packageName}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Affected versions
              <input
                className="input"
                onChange={(event) =>
                  update("affectedVersions", event.target.value)
                }
                placeholder="< 1.2.3"
                value={draft.affectedVersions}
              />
            </label>
            <label className="grid gap-1 t-sm">
              Patched versions
              <input
                className="input"
                onChange={(event) =>
                  update("patchedVersions", event.target.value)
                }
                placeholder=">= 1.2.3"
                value={draft.patchedVersions}
              />
            </label>
          </div>
          <label className="grid gap-1 t-sm">
            Summary
            <textarea
              className="input min-h-24"
              onChange={(event) => update("summary", event.target.value)}
              value={draft.summary}
            />
          </label>
          <label className="grid gap-1 t-sm">
            Markdown details
            <textarea
              className="input min-h-40 t-mono-sm"
              onChange={(event) =>
                update("detailsMarkdown", event.target.value)
              }
              value={draft.detailsMarkdown}
            />
          </label>
          <div className="grid gap-3 md:grid-cols-2">
            <label className="grid gap-1 t-sm">
              CVSS vector
              <input
                className="input t-mono-sm"
                onChange={(event) => update("cvssVector", event.target.value)}
                value={draft.cvssVector}
              />
            </label>
            <label className="grid gap-1 t-sm">
              CVSS score
              <input
                className="input"
                inputMode="decimal"
                onChange={(event) => update("cvssScore", event.target.value)}
                placeholder="8.1"
                value={draft.cvssScore}
              />
            </label>
          </div>
          <div className="between flex-wrap gap-3">
            <p className="t-xs" role="status">
              {error ??
                (created
                  ? `Draft ${created.advisory.ghsaId} created.`
                  : "Server validates title, CVE, CVSS, package metadata, and Markdown.")}
            </p>
            <div className="flex flex-wrap gap-2">
              {created ? (
                <Link className="btn" href={created.advisory.href}>
                  Open draft
                </Link>
              ) : null}
              <button
                className="btn primary"
                disabled={isPending}
                onClick={submit}
                type="button"
              >
                {isPending ? "Creating..." : "Create draft"}
              </button>
            </div>
          </div>
        </section>
      </div>
    </RepositorySecurityShell>
  );
}
