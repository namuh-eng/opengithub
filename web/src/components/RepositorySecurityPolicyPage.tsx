import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecurityPolicyFetchResult,
  RepositorySecurityPolicyView,
} from "@/lib/api";

type RepositorySecurityPolicyPageProps = {
  repository: RepositoryOverview;
  policyResult: RepositorySecurityPolicyFetchResult;
};

function formatDate(value: string | null) {
  if (!value) return "Not recorded";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function MissingPolicyState({
  policy,
  canEdit,
}: {
  policy: RepositorySecurityPolicyView["policy"];
  canEdit: boolean;
}) {
  return (
    <section className="card p-5">
      <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Security policy
          </p>
          <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
            No published policy
          </h1>
          <p className="t-sm mt-3 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {policy.emptyState}
          </p>
        </div>
        {canEdit && policy.editHref ? (
          <Link className="btn primary" href={policy.editHref}>
            Start setup
          </Link>
        ) : (
          <span className="chip warn">Reader view</span>
        )}
      </div>
    </section>
  );
}

function PolicyReader({
  securityPolicy,
}: {
  securityPolicy: RepositorySecurityPolicyView;
}) {
  const { policy, viewer } = securityPolicy;

  if (!policy.exists) {
    return (
      <MissingPolicyState canEdit={viewer.canEditPolicy} policy={policy} />
    );
  }

  return (
    <div className="grid gap-6">
      <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Security and quality
          </p>
          <h1
            className="t-h1 mt-2 break-words"
            style={{ color: "var(--ink-1)" }}
          >
            Security policy
          </h1>
          <p className="t-sm mt-3 max-w-3xl" style={{ color: "var(--ink-3)" }}>
            Responsible disclosure guidance from{" "}
            <span className="t-mono-sm">{policy.path ?? "SECURITY.md"}</span> on{" "}
            <span className="t-mono-sm">{policy.ref ?? "default"}</span>.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <span className="chip soft">{viewer.permission}</span>
          {viewer.canEditPolicy && policy.editHref ? (
            <Link className="btn primary" href={policy.editHref}>
              Edit policy
            </Link>
          ) : null}
        </div>
      </section>

      <section className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_260px]">
        <article className="card overflow-hidden">
          <div
            className="flex flex-wrap items-start justify-between gap-3 border-b px-5 py-4"
            style={{ borderColor: "var(--line)" }}
          >
            <div className="min-w-0">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Policy file
              </p>
              <h2
                className="t-h2 mt-2 break-words"
                id="security-policy-reader-title"
                style={{ color: "var(--ink-1)" }}
              >
                {policy.path ?? "SECURITY.md"}
              </h2>
              <p className="t-xs mt-2">
                Updated {formatDate(policy.updatedAt)}
                {policy.latestCommit ? (
                  <>
                    {" "}
                    by{" "}
                    <Link
                      className="t-mono-sm hover:underline"
                      href={policy.latestCommit.href}
                    >
                      {policy.latestCommit.shortOid}
                    </Link>
                  </>
                ) : null}
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              {policy.sourceHref ? (
                <Link className="btn sm" href={policy.sourceHref}>
                  Source
                </Link>
              ) : null}
              {policy.rawHref ? (
                <Link className="btn sm" href={policy.rawHref}>
                  Raw
                </Link>
              ) : null}
              {policy.historyHref ? (
                <Link className="btn sm" href={policy.historyHref}>
                  History
                </Link>
              ) : null}
            </div>
          </div>
          <div className="px-5 py-5">
            {policy.html ? (
              <MarkdownBody
                html={policy.html}
                labelledBy="security-policy-reader-title"
              />
            ) : (
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                The policy content is unavailable, but the file metadata is
                still available.
              </p>
            )}
          </div>
        </article>

        <aside className="grid gap-4 content-start">
          <section className="card p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              File actions
            </p>
            <div className="mt-3 grid gap-2">
              {policy.sourceHref ? (
                <Link className="btn sm" href={policy.sourceHref}>
                  View source
                </Link>
              ) : null}
              {policy.rawHref ? (
                <Link className="btn sm" href={policy.rawHref}>
                  Open raw
                </Link>
              ) : null}
              {policy.historyHref ? (
                <Link className="btn sm" href={policy.historyHref}>
                  View history
                </Link>
              ) : null}
            </div>
          </section>

          {policy.outline.length > 0 ? (
            <section className="card p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                On this page
              </p>
              <nav className="mt-3 grid gap-2" aria-label="Policy headings">
                {policy.outline.map((heading) => (
                  <Link
                    className="t-sm break-words hover:underline"
                    href={heading.href}
                    key={heading.id}
                    style={{
                      color: "var(--ink-3)",
                      paddingLeft: `${Math.max(heading.level - 1, 0) * 10}px`,
                    }}
                  >
                    {heading.text}
                  </Link>
                ))}
              </nav>
            </section>
          ) : null}

          {policy.latestCommit ? (
            <section className="card p-4">
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Latest policy commit
              </p>
              <Link
                className="t-sm mt-3 block break-words font-semibold hover:underline"
                href={policy.latestCommit.href}
              >
                {policy.latestCommit.message}
              </Link>
              <p className="t-xs mt-2">
                <span className="t-mono-sm">
                  {policy.latestCommit.shortOid}
                </span>{" "}
                · {formatDate(policy.latestCommit.committedAt)}
              </p>
            </section>
          ) : null}
        </aside>
      </section>
    </div>
  );
}

function PolicyUnavailablePage({
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
          Security policy unavailable
        </h1>
        <p
          className="t-sm mt-3"
          role="status"
          style={{ color: "var(--ink-3)" }}
        >
          {result.message}
        </p>
        <p className="t-xs mt-3">
          {result.status}
          {result.code ? ` · ${result.code}` : ""}
        </p>
      </section>
    </RepositorySecurityShell>
  );
}

export function RepositorySecurityPolicyPage({
  repository,
  policyResult,
}: RepositorySecurityPolicyPageProps) {
  if (!policyResult.ok) {
    return (
      <PolicyUnavailablePage repository={repository} result={policyResult} />
    );
  }

  return (
    <RepositorySecurityShell activeSection="policy" repository={repository}>
      <PolicyReader securityPolicy={policyResult.securityPolicy} />
    </RepositorySecurityShell>
  );
}
