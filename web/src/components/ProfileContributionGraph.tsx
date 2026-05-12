import Link from "next/link";
import type { ProfileContributionSummary } from "@/lib/api";

type ProfileContributionGraphProps = {
  login: string;
  summary: ProfileContributionSummary;
};

const MONTH_LABELS = new Intl.DateTimeFormat("en", {
  month: "short",
  timeZone: "UTC",
});
const DAY_LABELS = new Intl.DateTimeFormat("en", {
  day: "numeric",
  timeZone: "UTC",
  month: "long",
  year: "numeric",
});

function contributionLabel(count: number) {
  if (count === 0) {
    return "No contributions";
  }
  if (count === 1) {
    return "1 contribution";
  }
  return `${count.toLocaleString()} contributions`;
}

function dateLabel(date: string) {
  return DAY_LABELS.format(new Date(`${date}T00:00:00Z`));
}

function monthLabels(days: ProfileContributionSummary["days"]) {
  const labels: { key: string; label: string }[] = [];
  for (const day of days) {
    const date = new Date(`${day.date}T00:00:00Z`);
    const key = `${date.getUTCFullYear()}-${date.getUTCMonth()}`;
    if (labels.at(-1)?.key !== key) {
      labels.push({ key, label: MONTH_LABELS.format(date) });
    }
  }
  return labels;
}

function yearHref(login: string, year: number) {
  return `/${encodeURIComponent(login)}?year=${year}`;
}

export function ProfileContributionGraph({
  login,
  summary,
}: ProfileContributionGraphProps) {
  const days = summary.days;
  const months = monthLabels(days);
  const years = [summary.year, summary.year - 1, summary.year - 2];

  return (
    <section className="card p-5" aria-labelledby="profile-contributions">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Contributions
          </p>
          <h2 className="t-h2 mt-1" id="profile-contributions">
            {summary.total.toLocaleString()} contributions in {summary.year}
          </h2>
        </div>
        <nav
          aria-label="Contribution years"
          className="flex flex-wrap items-center gap-2"
        >
          {years.map((year) => (
            <Link
              aria-current={year === summary.year ? "page" : undefined}
              className={year === summary.year ? "chip active" : "chip soft"}
              href={yearHref(login, year)}
              key={year}
            >
              {year}
            </Link>
          ))}
        </nav>
      </div>

      {days.length > 0 ? (
        <div className="mt-5 overflow-x-auto pb-1">
          <fieldset className="min-w-[640px] border-0 p-0">
            <legend className="sr-only">
              {summary.total.toLocaleString()} contributions in {summary.year}
            </legend>
            <div
              aria-hidden="true"
              className="mb-2 grid gap-1"
              style={{
                gridTemplateColumns: `repeat(${Math.max(months.length, 1)}, minmax(0, 1fr))`,
              }}
            >
              {months.map((month) => (
                <span
                  className="t-mono-sm"
                  key={month.key}
                  style={{ color: "var(--ink-3)" }}
                >
                  {month.label}
                </span>
              ))}
            </div>
            <div className="grid grid-flow-col grid-rows-7 gap-1">
              {days.map((day) => {
                const label = `${contributionLabel(day.count)} on ${dateLabel(day.date)}`;
                return (
                  <button
                    aria-label={label}
                    className="h-3 w-3 rounded-[2px] border outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]"
                    key={day.date}
                    style={{
                      background:
                        day.intensity > 0
                          ? "var(--accent-soft)"
                          : "var(--surface-2)",
                      borderColor: "var(--line-soft)",
                      opacity:
                        day.intensity > 0 ? 0.28 + day.intensity * 0.16 : 1,
                    }}
                    title={label}
                    type="button"
                  />
                );
              })}
            </div>
          </fieldset>
        </div>
      ) : (
        <p className="t-body mt-4" style={{ color: "var(--ink-3)" }}>
          No public contributions are visible for {summary.year}.
        </p>
      )}

      <div className="mt-4 flex flex-wrap items-center gap-2">
        <span className="t-xs">Less</span>
        {[0, 1, 2, 3, 4].map((intensity) => (
          <span
            aria-hidden="true"
            className="h-3 w-3 rounded-[2px] border"
            key={intensity}
            style={{
              background:
                intensity > 0 ? "var(--accent-soft)" : "var(--surface-2)",
              borderColor: "var(--line-soft)",
              opacity: intensity > 0 ? 0.28 + intensity * 0.16 : 1,
            }}
          />
        ))}
        <span className="t-xs">More</span>
      </div>
    </section>
  );
}
