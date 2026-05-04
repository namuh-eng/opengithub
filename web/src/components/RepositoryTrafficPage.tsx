import Link from "next/link";
import { RepositoryInsightsShell } from "@/components/RepositoryInsightsShell";
import type {
  RepositoryOverview,
  RepositoryTrafficFetchResult,
  RepositoryTrafficSeriesPoint,
  RepositoryTrafficView,
} from "@/lib/api";
import { repositoryCommitHistoryHref } from "@/lib/navigation";

type RepositoryTrafficPageProps = {
  repository: RepositoryOverview;
  trafficResult: RepositoryTrafficFetchResult;
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("en").format(value);
}

function formatDate(value: string) {
  const date = new Date(`${value}T00:00:00Z`);
  if (!Number.isFinite(date.getTime())) return value;
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(date);
}

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) return "recently";
  const diffMs = Date.now() - timestamp;
  const absMs = Math.abs(diffMs);
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 1000 * 60 * 60 * 24 * 365],
    ["month", 1000 * 60 * 60 * 24 * 30],
    ["day", 1000 * 60 * 60 * 24],
    ["hour", 1000 * 60 * 60],
    ["minute", 1000 * 60],
  ];
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  for (const [unit, unitMs] of units) {
    if (absMs >= unitMs) {
      return formatter.format(Math.round(-diffMs / unitMs), unit);
    }
  }
  return "just now";
}

function TrafficMetric({
  label,
  total,
  unique,
}: {
  label: string;
  total: number;
  unique: number;
}) {
  return (
    <article className="card min-h-36 p-4">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        {label}
      </p>
      <div className="mt-3 flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="t-h1 t-num" style={{ color: "var(--ink-1)" }}>
            {formatNumber(total)}
          </p>
          <p className="t-xs mt-1">total events</p>
        </div>
        <span className="chip soft">
          <span className="t-num">{formatNumber(unique)}</span> unique
        </span>
      </div>
    </article>
  );
}

function TrafficChart({
  label,
  points,
  totalLabel,
  uniqueLabel,
}: {
  label: string;
  points: RepositoryTrafficSeriesPoint[];
  totalLabel: string;
  uniqueLabel: string;
}) {
  const maxTotal = Math.max(1, ...points.map((point) => point.total));

  return (
    <section className="card p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            {label}
          </p>
          <h2 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Last 14 days
          </h2>
        </div>
        <span className="chip soft">Accessible table included</span>
      </div>

      <div
        aria-label={`${label} line chart`}
        className="mt-5 grid min-h-48 grid-cols-[repeat(auto-fit,minmax(28px,1fr))] items-end gap-2"
        role="img"
      >
        {points.map((point) => {
          const height = Math.max(10, (point.total / maxTotal) * 100);
          return (
            <div className="grid min-w-0 gap-2" key={point.date}>
              <div
                aria-hidden="true"
                className="flex h-32 items-end rounded-md"
                style={{ background: "var(--surface-2)" }}
              >
                <div
                  className="w-full rounded-md"
                  style={{
                    background:
                      point.total > 0 ? "var(--accent)" : "var(--line-strong)",
                    height: `${height}%`,
                  }}
                />
              </div>
              <span className="t-mono-sm text-center">
                {formatNumber(point.total)}
              </span>
            </div>
          );
        })}
      </div>

      <div className="mt-5 overflow-x-auto">
        <table className="w-full text-left t-sm">
          <caption className="sr-only">{label} data table</caption>
          <thead className="t-label" style={{ color: "var(--ink-3)" }}>
            <tr>
              <th className="py-2 pr-3">Date</th>
              <th className="py-2 pr-3 text-right">{totalLabel}</th>
              <th className="py-2 text-right">{uniqueLabel}</th>
            </tr>
          </thead>
          <tbody>
            {points.map((point) => (
              <tr
                className="border-t"
                key={`${label}-${point.date}`}
                style={{ borderColor: "var(--line-soft)" }}
              >
                <td className="py-2 pr-3">{formatDate(point.date)}</td>
                <td className="py-2 pr-3 text-right t-num">
                  {formatNumber(point.total)}
                </td>
                <td className="py-2 text-right t-num">
                  {formatNumber(point.unique)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function TrafficRows({ traffic }: { traffic: RepositoryTrafficView }) {
  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <section className="card overflow-hidden">
        <div
          className="border-b px-4 py-3"
          style={{ borderColor: "var(--line)" }}
        >
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Referrers
          </p>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            External sites ordered by views and unique visitors.
          </p>
        </div>
        {traffic.referrers.length > 0 ? (
          traffic.referrers.map((referrer) => (
            <div className="list-row px-4 py-3" key={referrer.referrer}>
              <div className="min-w-0 flex-1">
                <a
                  className="break-words t-sm font-semibold hover:underline"
                  href={referrer.href}
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  {referrer.referrer}
                </a>
                <p className="t-xs mt-1">
                  <span className="t-num">
                    {formatNumber(referrer.totalViews)}
                  </span>{" "}
                  views ·{" "}
                  <span className="t-num">
                    {formatNumber(referrer.uniqueVisitors)}
                  </span>{" "}
                  unique visitors
                </p>
              </div>
            </div>
          ))
        ) : (
          <div className="px-4 py-5">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No external referrers were recorded for this window.
            </p>
          </div>
        )}
      </section>

      <section className="card overflow-hidden">
        <div
          className="border-b px-4 py-3"
          style={{ borderColor: "var(--line)" }}
        >
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Popular content
          </p>
          <p className="t-sm mt-1" style={{ color: "var(--ink-3)" }}>
            Repository paths ordered by visitor demand.
          </p>
        </div>
        {traffic.popularContent.length > 0 ? (
          traffic.popularContent.map((content) => (
            <div className="list-row px-4 py-3" key={content.path}>
              <div className="min-w-0 flex-1">
                <Link
                  className="break-words t-sm font-semibold hover:underline"
                  href={content.href}
                >
                  {content.title || content.path}
                </Link>
                <p className="t-xs mt-1 break-all">
                  <span className="t-mono-sm">{content.path}</span> ·{" "}
                  <span className="t-num">
                    {formatNumber(content.totalViews)}
                  </span>{" "}
                  views ·{" "}
                  <span className="t-num">
                    {formatNumber(content.uniqueVisitors)}
                  </span>{" "}
                  unique visitors
                </p>
              </div>
            </div>
          ))
        ) : (
          <div className="px-4 py-5">
            <p className="t-sm" style={{ color: "var(--ink-3)" }}>
              No repository paths were viewed during this window.
            </p>
          </div>
        )}
      </section>
    </div>
  );
}

function TrafficReadyPage({
  repository,
  traffic,
}: {
  repository: RepositoryOverview;
  traffic: RepositoryTrafficView;
}) {
  const owner = traffic.repository.ownerLogin;
  const repo = traffic.repository.name;
  const dateRange = `${formatDate(traffic.window.startedOn)} - ${formatDate(
    traffic.window.endedOn,
  )}`;
  const commitHistoryHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName: traffic.repository.defaultBranch,
  });

  return (
    <RepositoryInsightsShell activeSection="traffic" repository={repository}>
      <div className="grid gap-6">
        <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Traffic
            </p>
            <h1 className="t-h1 mt-2" style={{ color: "var(--ink-1)" }}>
              Traffic analytics
            </h1>
            <p
              className="t-sm mt-3 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Clone and visitor activity for {owner}/{repo} across {dateRange}.
            </p>
            <p
              className="t-sm mt-2 max-w-3xl"
              style={{ color: "var(--ink-3)" }}
            >
              Dates are reported in {traffic.window.timezone}. Clone and visitor
              series update {traffic.window.clonesUpdateCadence}. Referrers and
              popular content update {traffic.window.referrersUpdateCadence}.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <span className="chip active">{traffic.window.label}</span>
            <span className={traffic.snapshot.stale ? "chip warn" : "chip ok"}>
              {traffic.snapshot.stale ? "Stale snapshot" : "Fresh snapshot"}
            </span>
            <Link className="btn primary" href={commitHistoryHref}>
              Commit history
            </Link>
          </div>
        </section>

        <section
          aria-label="Traffic summary metrics"
          className="grid gap-4 md:grid-cols-2 xl:grid-cols-4"
        >
          <TrafficMetric
            label="Full clones"
            total={traffic.summaries.clonesTotal}
            unique={traffic.summaries.clonesUnique}
          />
          <TrafficMetric
            label="Visitors"
            total={traffic.summaries.visitorsTotal}
            unique={traffic.summaries.visitorsUnique}
          />
          <article className="card min-h-36 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Referrers
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(traffic.summaries.referrersTotal)}
            </p>
            <p className="t-xs mt-1">ranked sources</p>
          </article>
          <article className="card min-h-36 p-4">
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Popular content
            </p>
            <p className="t-h1 t-num mt-3" style={{ color: "var(--ink-1)" }}>
              {formatNumber(traffic.summaries.popularContentTotal)}
            </p>
            <p className="t-xs mt-1">ranked paths</p>
          </article>
        </section>

        <div className="grid gap-4 xl:grid-cols-2">
          <TrafficChart
            label="Clones"
            points={traffic.clones}
            totalLabel="Clones"
            uniqueLabel="Unique cloners"
          />
          <TrafficChart
            label="Visitors"
            points={traffic.visitors}
            totalLabel="Views"
            uniqueLabel="Unique visitors"
          />
        </div>

        <TrafficRows traffic={traffic} />

        <section className="card p-4">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Freshness
          </p>
          <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
            Snapshot computed {formatRelativeTime(traffic.snapshot.computedAt)}.
            It expires {formatRelativeTime(traffic.snapshot.expiresAt)}.
          </p>
        </section>
      </div>
    </RepositoryInsightsShell>
  );
}

export function RepositoryTrafficPage({
  repository,
  trafficResult,
}: RepositoryTrafficPageProps) {
  if (!trafficResult.ok) {
    return (
      <RepositoryInsightsShell activeSection="traffic" repository={repository}>
        <section className="card p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Traffic
          </p>
          <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
            Traffic unavailable
          </h1>
          <p className="t-sm mt-2 max-w-2xl" style={{ color: "var(--ink-3)" }}>
            {trafficResult.message}
          </p>
          <p className="t-xs mt-3">
            Traffic counts are visible only to repository users with push
            access.
          </p>
          <Link
            className="btn mt-4"
            href={`/${repository.owner_login}/${repository.name}`}
          >
            Back to Code
          </Link>
        </section>
      </RepositoryInsightsShell>
    );
  }

  return (
    <TrafficReadyPage repository={repository} traffic={trafficResult.traffic} />
  );
}
