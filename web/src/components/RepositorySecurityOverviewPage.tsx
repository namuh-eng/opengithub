import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { RepositorySecurityShell } from "@/components/RepositorySecurityShell";
import type {
  RepositoryOverview,
  RepositorySecurityAdvisorySummary,
  RepositorySecurityFeatureCard,
  RepositorySecurityOverviewFetchResult,
  RepositorySecurityOverviewView,
  RepositorySecurityPolicySummary,
} from "@/lib/api";

type RepositorySecurityOverviewPageProps = {
  repository: RepositoryOverview;
  securityResult: RepositorySecurityOverviewFetchResult;
};

function formatDate(value: string | null) {
  if (!value) return "Not recorded";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

function formatCount(value: number | null) {
  if (value === null) return "Hidden";
  return new Intl.NumberFormat("en").format(value);
}

function statusChipClass(status: string) {
  const normalized = status.toLowerCase();
  if (["enabled", "published", "active"].includes(normalized)) {
    return "chip ok";
  }
  if (["disabled", "draft", "needs_setup"].includes(normalized)) {
    return "chip warn";
  }
  if (["critical", "high"].includes(normalized)) {
    return "chip err";
  }
  if (["medium", "low"].includes(normalized)) {
    return "chip soft";
  }
  return "chip soft";
}

function humanize(value: string) {
  return value.replaceAll("_", " ");
}

function FeatureCard({ feature }: { feature: RepositorySecurityFeatureCard }) {
  return (
    <article className="card grid gap-4 p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 className="t-h3" style={{ color: "var(--ink-1)" }}>
            <Link className="hover:underline" href={feature.href}>
              {feature.label}
            </Link>
          </h3>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            {feature.summary}
          </p>
        </div>
        <span className={statusChipClass(feature.status)}>
          {humanize(feature.status)}
        </span>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Alerts
          </p>
          <p className="t-h2 t-num mt-1" style={{ color: "var(--ink-1)" }}>
            {formatCount(feature.alertCount)}
          </p>
        </div>
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Private
          </p>
          <p className="t-h2 t-num mt-1" style={{ color: "var(--ink-1)" }}>
            {formatCount(feature.privateCount)}
          </p>
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        <Link className="btn sm" href={feature.href}>
          View
        </Link>
        {feature.configHref ? (
          <Link className="btn sm primary" href={feature.configHref}>
            Configure
          </Link>
        ) : null}
      </div>
      <p className="t-xs">Updated {formatDate(feature.updatedAt)}</p>
    </article>
  );
}

function PolicyPreview({
  policy,
  canEdit,
}: {
  policy: RepositorySecurityPolicySummary;
  canEdit: boolean;
}) {
  if (!policy.exists) {
    return (
      <section className="card p-5">
        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security policy
            </p>
            <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              No published policy
            </h2>
            <p className="t-sm mt-3" style={{ color: "var(--ink-3)" }}>
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

  return (
    <section className="card overflow-hidden">
      <div
        className="flex flex-wrap items-start justify-between gap-3 border-b px-5 py-4"
        style={{ borderColor: "var(--line)" }}
      >
        <div className="min-w-0">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Security policy
          </p>
          <h2
            className="t-h2 mt-2 break-words"
            id="security-policy-title"
            style={{ color: "var(--ink-1)" }}
          >
            {policy.path ?? "SECURITY.md"}
          </h2>
          <p className="t-xs mt-2">
            {policy.ref ? `${policy.ref} · ` : ""}
            Updated {formatDate(policy.updatedAt)}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {policy.sourceHref ? (
            <Link className="btn sm" href={policy.sourceHref}>
              Source
            </Link>
          ) : null}
          {policy.historyHref ? (
            <Link className="btn sm" href={policy.historyHref}>
              History
            </Link>
          ) : null}
          {canEdit && policy.editHref ? (
            <Link className="btn sm primary" href={policy.editHref}>
              Edit policy
            </Link>
          ) : null}
        </div>
      </div>
      <div className="px-5 py-5">
        {policy.html ? (
          <MarkdownBody html={policy.html} labelledBy="security-policy-title" />
        ) : (
          <p className="t-sm" style={{ color: "var(--ink-3)" }}>
            The policy content is unavailable, but the source file still exists.
          </p>
        )}
      </div>
      {policy.rawHref ? (
        <div
          className="border-t px-5 py-3"
          style={{ borderColor: "var(--line)" }}
        >
          <Link className="t-sm hover:underline" href={policy.rawHref}>
            Open raw policy
          </Link>
        </div>
      ) : null}
    </section>
  );
}

function AdvisoryRow({
  advisory,
}: {
  advisory: RepositorySecurityAdvisorySummary;
}) {
  return (
    <article className="list-row grid gap-3 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <Link
            className="break-words t-sm font-semibold hover:underline"
            href={advisory.href}
          >
            {advisory.title}
          </Link>
          <span className={statusChipClass(advisory.severity)}>
            {advisory.severity}
          </span>
          <span className={statusChipClass(advisory.status)}>
            {advisory.status}
          </span>
        </div>
        <p className="t-sm mt-2 break-words" style={{ color: "var(--ink-3)" }}>
          {advisory.summary}
        </p>
        <p className="t-xs mt-2">
          {advisory.packageName ?? "Repository advisory"}
          {advisory.vulnerableRange ? ` · ${advisory.vulnerableRange}` : ""}
        </p>
      </div>
      <div className="text-left md:text-right">
        <p className="t-mono-sm">{advisory.identifier}</p>
        <p className="t-xs mt-1">
          Published {formatDate(advisory.publishedAt)}
        </p>
      </div>
    </article>
  );
}

function SecurityReadyPage({
  repository,
  security,
}: {
  repository: RepositoryOverview;
  security: RepositorySecurityOverviewView;
}) {
  const hasAdvisories = security.advisories.length > 0;

  return (
    <RepositorySecurityShell activeSection="overview" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Security and quality
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Security overview
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Review the repository policy, published advisories, and enabled
              security features without exposing private alert counts to
              read-only viewers.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip soft">
              {humanize(security.viewer.permission)}
            </span>
            {security.viewer.canViewPrivateAlertCounts ? (
              <span className="chip ok">Private counts visible</span>
            ) : (
              <span className="chip warn">Private counts hidden</span>
            )}
            <Link className="btn" href={security.links.policyHref}>
              Security policy
            </Link>
          </div>
        </section>

        <PolicyPreview
          canEdit={security.viewer.canEditPolicy}
          policy={security.policy}
        />

        <section
          aria-label="Security feature cards"
          className="grid gap-4 md:grid-cols-2"
        >
          {security.features.map((feature) => (
            <FeatureCard feature={feature} key={feature.key} />
          ))}
        </section>

        <section className="card overflow-hidden">
          <div
            className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-4"
            style={{ borderColor: "var(--line)" }}
          >
            <div>
              <p className="t-label" style={{ color: "var(--ink-3)" }}>
                Advisories
              </p>
              <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
                Recent published advisories
              </h2>
            </div>
            <Link className="btn sm" href={security.links.advisoriesHref}>
              View all
            </Link>
          </div>
          {hasAdvisories ? (
            <div>
              {security.advisories.map((advisory) => (
                <AdvisoryRow advisory={advisory} key={advisory.id} />
              ))}
            </div>
          ) : (
            <div className="px-5 py-8">
              <p className="t-sm" style={{ color: "var(--ink-3)" }}>
                No published advisories are available for this repository.
              </p>
            </div>
          )}
        </section>
      </div>
    </RepositorySecurityShell>
  );
}

function SecurityUnavailablePage({
  repository,
  result,
}: {
  repository: RepositoryOverview;
  result: Extract<RepositorySecurityOverviewFetchResult, { ok: false }>;
}) {
  return (
    <RepositorySecurityShell activeSection="overview" repository={repository}>
      <section className="card p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Security and quality
        </p>
        <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
          Security overview unavailable
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

export function RepositorySecurityOverviewPage({
  repository,
  securityResult,
}: RepositorySecurityOverviewPageProps) {
  if (!securityResult.ok) {
    return (
      <SecurityUnavailablePage
        repository={repository}
        result={securityResult}
      />
    );
  }

  return (
    <SecurityReadyPage
      repository={repository}
      security={securityResult.security}
    />
  );
}
