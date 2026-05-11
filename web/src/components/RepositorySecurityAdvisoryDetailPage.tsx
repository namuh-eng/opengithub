"use client";

import Link from "next/link";
import { useMemo, useState, useTransition } from "react";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecurityAdvisoryDetail,
  RepositorySecurityAdvisoryDetailFetchResult,
  RepositorySecurityAdvisoryMutation,
} from "@/lib/api";

type RepositorySecurityAdvisoryDetailPageProps = {
  repository: RepositoryOverview;
  advisoryResult: RepositorySecurityAdvisoryDetailFetchResult;
};

function chipForSeverity(severity: string) {
  if (severity === "critical" || severity === "high") return "chip err";
  if (severity === "moderate") return "chip warn";
  if (severity === "low") return "chip info";
  return "chip soft";
}

function formatDate(value: string | null) {
  if (!value) return "Not published";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function linesToArray(value: string) {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function AdvisoryEditor({
  detail,
  onSaved,
}: {
  detail: RepositorySecurityAdvisoryDetail;
  onSaved: (next: RepositorySecurityAdvisoryDetail) => void;
}) {
  const [isPending, startTransition] = useTransition();
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [draft, setDraft] = useState(() => ({
    title: detail.advisory.title,
    summary: detail.markdown.summaryMarkdown,
    detailsMarkdown: detail.markdown.detailsMarkdown,
    cveId: detail.advisory.cveId ?? "",
    severity: detail.advisory.severity,
    packageEcosystem: detail.advisory.package?.ecosystem ?? "",
    packageName: detail.advisory.package?.name ?? "",
    affectedVersions: detail.advisory.package?.affectedVersions ?? "",
    patchedVersions: detail.advisory.package?.patchedVersions ?? "",
    cvssVector: detail.advisory.cvss?.vector ?? "",
    cvssScore:
      typeof detail.advisory.cvss?.score === "number"
        ? String(detail.advisory.cvss.score)
        : "",
    cwes: detail.advisory.cwes.map((cwe) => cwe.id).join("\n"),
    credits: detail.credits.map((credit) => credit.actor.login).join("\n"),
    collaborators: detail.collaborators
      .map((collaborator) => collaborator.actor.login)
      .join("\n"),
  }));
  const update = (field: keyof typeof draft, value: string) => {
    setSaved(false);
    setError(null);
    setDraft((current) => ({ ...current, [field]: value }));
  };

  const submit = () => {
    const score = draft.cvssScore.trim() ? Number(draft.cvssScore) : null;
    const mutation: RepositorySecurityAdvisoryMutation = {
      title: draft.title,
      summary: draft.summary,
      detailsMarkdown: draft.detailsMarkdown,
      cveId: draft.cveId.trim() || null,
      severity: draft.severity,
      packageEcosystem: draft.packageEcosystem.trim() || null,
      packageName: draft.packageName.trim() || null,
      affectedVersions: draft.affectedVersions.trim() || null,
      patchedVersions: draft.patchedVersions.trim() || null,
      cvssVector: draft.cvssVector.trim() || null,
      cvssScore: Number.isFinite(score) ? score : null,
      cvssMetrics: detail.advisory.cvss?.metrics ?? {},
      cwes: linesToArray(draft.cwes).map((id) => ({
        id,
        name:
          detail.advisory.cwes.find((cwe) => cwe.id === id)?.name ??
          "Common Weakness Enumeration",
        href: detail.advisory.cwes.find((cwe) => cwe.id === id)?.href ?? null,
      })),
      credits: linesToArray(draft.credits).map((login) => ({
        login,
        creditType:
          detail.credits.find((credit) => credit.actor.login === login)
            ?.creditType ?? "reporter",
      })),
      collaborators: linesToArray(draft.collaborators).map((login) => ({
        login,
        role:
          detail.collaborators.find(
            (collaborator) => collaborator.actor.login === login,
          )?.role ?? "collaborator",
      })),
    };

    startTransition(async () => {
      const response = await fetch(`${detail.advisory.href}/actions`, {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(mutation),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as {
          error?: { message?: string };
        } | null;
        setError(
          body?.error?.message ?? "Repository security advisory update failed.",
        );
        return;
      }
      const next = (await response.json()) as RepositorySecurityAdvisoryDetail;
      onSaved(next);
      setSaved(true);
    });
  };

  return (
    <section className="card grid gap-4 p-5" aria-label="Edit advisory">
      <div>
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Maintainer edit
        </p>
        <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
          Advisory metadata
        </h2>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        <label className="grid gap-1 t-sm">
          Title
          <input
            className="input"
            onChange={(event) => update("title", event.target.value)}
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
          CVSS score
          <input
            className="input"
            inputMode="decimal"
            onChange={(event) => update("cvssScore", event.target.value)}
            placeholder="8.1"
            value={draft.cvssScore}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Ecosystem
          <input
            className="input"
            onChange={(event) => update("packageEcosystem", event.target.value)}
            value={draft.packageEcosystem}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Package
          <input
            className="input"
            onChange={(event) => update("packageName", event.target.value)}
            value={draft.packageName}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Affected versions
          <input
            className="input"
            onChange={(event) => update("affectedVersions", event.target.value)}
            value={draft.affectedVersions}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Patched versions
          <input
            className="input"
            onChange={(event) => update("patchedVersions", event.target.value)}
            value={draft.patchedVersions}
          />
        </label>
      </div>
      <label className="grid gap-1 t-sm">
        CVSS vector
        <input
          className="input t-mono-sm"
          onChange={(event) => update("cvssVector", event.target.value)}
          value={draft.cvssVector}
        />
      </label>
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
          onChange={(event) => update("detailsMarkdown", event.target.value)}
          value={draft.detailsMarkdown}
        />
      </label>
      <div className="grid gap-3 md:grid-cols-3">
        <label className="grid gap-1 t-sm">
          CWE ids
          <textarea
            className="input min-h-24 t-mono-sm"
            onChange={(event) => update("cwes", event.target.value)}
            value={draft.cwes}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Credits
          <textarea
            className="input min-h-24"
            onChange={(event) => update("credits", event.target.value)}
            value={draft.credits}
          />
        </label>
        <label className="grid gap-1 t-sm">
          Collaborators
          <textarea
            className="input min-h-24"
            onChange={(event) => update("collaborators", event.target.value)}
            value={draft.collaborators}
          />
        </label>
      </div>
      <div className="between flex-wrap gap-3">
        <p className="t-xs" role="status">
          {error ??
            (saved
              ? "Advisory metadata saved."
              : "Server validates CVE, CVSS, CWE, package, credit, and collaborator fields.")}
        </p>
        <button
          className="btn primary"
          disabled={isPending}
          onClick={submit}
          type="button"
        >
          {isPending ? "Saving..." : "Save advisory"}
        </button>
      </div>
    </section>
  );
}

function PublishPanel({
  detail,
  onPublished,
}: {
  detail: RepositorySecurityAdvisoryDetail;
  onPublished: (
    next: RepositorySecurityAdvisoryDetail,
    message: string,
  ) => void;
}) {
  const [isPending, startTransition] = useTransition();
  const [message, setMessage] = useState<string | null>(null);
  const checks = [
    {
      label: "Title is ready",
      ready: detail.advisory.title.trim().length > 0,
    },
    {
      label: "Package advisories include patched versions",
      ready:
        !detail.advisory.package?.name ||
        Boolean(detail.advisory.package.patchedVersions),
    },
    {
      label: "Markdown details render safely",
      ready: detail.markdown.detailsHtml.trim().length > 0,
    },
  ];
  const canPublish = checks.every((check) => check.ready);
  const publish = () => {
    setMessage(null);
    startTransition(async () => {
      const response = await fetch(`${detail.advisory.href}/publish`, {
        method: "POST",
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as {
          error?: { message?: string };
        } | null;
        setMessage(
          body?.error?.message ??
            "Repository security advisory publish failed.",
        );
        return;
      }
      const next = (await response.json()) as RepositorySecurityAdvisoryDetail;
      onPublished(next, "Advisory published.");
    });
  };

  return (
    <section className="card grid gap-4 p-5" aria-label="Publish advisory">
      <div className="between flex-wrap gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Publish
          </p>
          <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Make this advisory public
          </h2>
        </div>
        <span className={canPublish ? "chip ok" : "chip warn"}>
          {canPublish ? "Ready" : "Needs review"}
        </span>
      </div>
      <ul className="grid gap-2">
        {checks.map((check) => (
          <li className="between gap-3 t-sm" key={check.label}>
            <span>{check.label}</span>
            <span className={check.ready ? "chip ok" : "chip warn"}>
              {check.ready ? "Ready" : "Required"}
            </span>
          </li>
        ))}
      </ul>
      <div className="between flex-wrap gap-3">
        <p className="t-xs" role="status">
          {message ??
            "Publishing records audit events, notifies advisory collaborators, and links package advisories when package metadata is present."}
        </p>
        <button
          className="btn primary"
          disabled={isPending}
          onClick={publish}
          type="button"
        >
          {isPending ? "Publishing..." : "Publish advisory"}
        </button>
      </div>
    </section>
  );
}

function DetailReadyPage({
  detail,
  repository,
}: {
  detail: RepositorySecurityAdvisoryDetail;
  repository: RepositoryOverview;
}) {
  const [current, setCurrent] = useState(detail);
  const [notice, setNotice] = useState<string | null>(null);
  const metricEntries = useMemo(
    () => Object.entries(current.advisory.cvss?.metrics ?? {}),
    [current.advisory.cvss],
  );
  return (
    <RepositorySecurityShell activeSection="advisories" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div className="min-w-0">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository advisory
            </p>
            <h1
              className="t-h1 mt-2 break-words"
              style={{ color: "var(--ink-1)" }}
            >
              {current.advisory.title}
            </h1>
            <div className="mt-3 flex flex-wrap gap-2">
              <span className={chipForSeverity(current.advisory.severity)}>
                {current.advisory.severity}
              </span>
              <span className="chip soft">{current.advisory.state}</span>
              <span className="chip soft t-mono-sm">
                {current.advisory.ghsaId}
              </span>
              {current.advisory.cveId ? (
                <span className="chip soft t-mono-sm">
                  {current.advisory.cveId}
                </span>
              ) : null}
            </div>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              {current.markdown.summaryMarkdown}
            </p>
          </div>
          <div className="flex flex-wrap gap-2 md:justify-end">
            <Link className="btn" href={current.links.listHref}>
              All advisories
            </Link>
            {current.viewer.canPublish ? (
              <span className="chip warn">Ready for publish flow</span>
            ) : null}
          </div>
        </section>

        {notice ? (
          <p className="chip ok w-fit" role="status">
            {notice}
          </p>
        ) : null}

        <section className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_300px]">
          <article className="card overflow-hidden">
            <div className="px-5 py-5">
              <MarkdownBody
                html={current.markdown.detailsHtml}
                labelledBy="advisory-detail-title"
              />
            </div>
          </article>
          <aside className="grid content-start gap-4">
            <section className="card grid gap-2 p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Package
              </p>
              <p className="t-mono-sm break-words">
                {current.advisory.package?.ecosystem ?? "unknown"}:
                {current.advisory.package?.name ?? "unscoped"}
              </p>
              <p className="t-xs">
                Affected{" "}
                {current.advisory.package?.affectedVersions ?? "not set"}
              </p>
              <p className="t-xs">
                Patched {current.advisory.package?.patchedVersions ?? "not set"}
              </p>
            </section>
            <section className="card grid gap-2 p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                CVSS
              </p>
              <button className="btn sm w-fit" type="button">
                Score {current.advisory.cvss?.score ?? "not set"}
              </button>
              {current.advisory.cvss?.vector ? (
                <p className="t-mono-sm break-words">
                  {current.advisory.cvss.vector}
                </p>
              ) : null}
              {metricEntries.length > 0 ? (
                <dl className="grid gap-1">
                  {metricEntries.map(([key, value]) => (
                    <div className="between gap-3" key={key}>
                      <dt className="t-xs">{key}</dt>
                      <dd className="t-mono-sm break-words">{String(value)}</dd>
                    </div>
                  ))}
                </dl>
              ) : null}
            </section>
            <section className="card grid gap-2 p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                CWE
              </p>
              {current.advisory.cwes.length > 0 ? (
                current.advisory.cwes.map((cwe) => (
                  <span className="chip soft t-mono-sm" key={cwe.id}>
                    {cwe.id} {cwe.name}
                  </span>
                ))
              ) : (
                <span className="chip soft">No CWE metadata</span>
              )}
            </section>
          </aside>
        </section>

        <section className="grid gap-4 md:grid-cols-3">
          <article className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Credits
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              {current.credits.map((credit) => (
                <Link
                  className="chip soft"
                  href={credit.actor.profileHref}
                  key={credit.id}
                >
                  {credit.actor.login} · {credit.creditType}
                </Link>
              ))}
            </div>
          </article>
          <article className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Collaborators
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              {current.collaborators.map((collaborator) => (
                <Link
                  className="chip soft"
                  href={collaborator.actor.profileHref}
                  key={collaborator.id}
                >
                  {collaborator.actor.login} · {collaborator.role}
                </Link>
              ))}
            </div>
          </article>
          <article className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Timeline
            </p>
            <ol className="mt-3 grid gap-2">
              {current.timeline.map((event) => (
                <li className="t-sm" key={event.id}>
                  {event.message}
                  <span className="t-xs block">
                    {formatDate(event.createdAt)}
                    {event.actor ? ` by ${event.actor.login}` : ""}
                  </span>
                </li>
              ))}
            </ol>
          </article>
        </section>

        {current.viewer.canEdit ? (
          <AdvisoryEditor detail={current} onSaved={setCurrent} />
        ) : null}
        {current.viewer.canPublish ? (
          <PublishPanel
            detail={current}
            onPublished={(next, message) => {
              setCurrent(next);
              setNotice(message);
            }}
          />
        ) : null}
      </div>
    </RepositorySecurityShell>
  );
}

function DetailUnavailablePage({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositorySecurityAdvisoryDetailFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="advisories" repository={repository}>
      <section className="card grid gap-3 p-6">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Security advisory
        </p>
        <h1 className="t-h1" style={{ color: "var(--ink-1)" }}>
          Security advisory unavailable
        </h1>
        <p className="t-sm" style={{ color: "var(--ink-3)" }}>
          {result.message}
        </p>
        <Link
          className="btn w-fit"
          href={`/${repository.owner_login}/${repository.name}/security/advisories`}
        >
          Back to advisories
        </Link>
      </section>
    </RepositorySecurityShell>
  );
}

export function RepositorySecurityAdvisoryDetailPage({
  advisoryResult,
  repository,
}: RepositorySecurityAdvisoryDetailPageProps) {
  if (!advisoryResult.ok) {
    return (
      <DetailUnavailablePage repository={repository} result={advisoryResult} />
    );
  }
  return (
    <DetailReadyPage detail={advisoryResult.advisory} repository={repository} />
  );
}
