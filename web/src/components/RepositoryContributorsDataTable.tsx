"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { CopyButton } from "@/components/CopyButton";
import type {
  RepositoryContributorRow,
  RepositoryContributorsWeek,
} from "@/lib/api";

type RepositoryContributorsDataTableProps = {
  contributors: RepositoryContributorRow[];
  weeks: RepositoryContributorsWeek[];
};

function formatNumber(value: number | null | undefined) {
  if (value == null) return "omitted";
  return new Intl.NumberFormat("en").format(value);
}

function formatDate(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return "recently";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function csvCell(value: string | number | null | undefined) {
  const text = value == null ? "omitted" : String(value);
  return `"${text.replaceAll('"', '""')}"`;
}

function contributorsCsv(
  contributors: RepositoryContributorRow[],
  weeks: RepositoryContributorsWeek[],
) {
  const rows = [
    ["scope", "week", "commits", "additions", "deletions"],
    ...weeks.map((week) => [
      "Repository",
      week.weekStart,
      week.commits,
      week.additions,
      week.deletions,
    ]),
    ...contributors.flatMap((contributor) =>
      contributor.weeks.map((week) => [
        contributor.login,
        week.weekStart,
        week.commits,
        week.additions,
        week.deletions,
      ]),
    ),
  ];
  return rows.map((row) => row.map(csvCell).join(",")).join("\n");
}

export function RepositoryContributorsDataTable({
  contributors,
  weeks,
}: RepositoryContributorsDataTableProps) {
  const [open, setOpen] = useState(false);
  const csv = useMemo(
    () => contributorsCsv(contributors, weeks),
    [contributors, weeks],
  );
  const csvHref = `data:text/csv;charset=utf-8,${encodeURIComponent(csv)}`;

  return (
    <section className="card overflow-hidden" id="contributors-data-table">
      <div
        className="flex flex-wrap items-center gap-3 border-b px-4 py-3"
        style={{ borderColor: "var(--line)" }}
      >
        <div>
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Data table
          </p>
          <h2 className="t-h3 mt-1" style={{ color: "var(--ink-1)" }}>
            Weekly contributor values
          </h2>
        </div>
        <span className="flex-1" />
        <button
          aria-controls="contributors-data-table-panel"
          aria-expanded={open}
          className="btn sm"
          onClick={() => setOpen((value) => !value)}
          type="button"
        >
          {open ? "Hide table" : "View as data table"}
        </button>
        <a
          className="btn sm"
          download="repository-contributors.csv"
          href={csvHref}
        >
          Download CSV
        </a>
        <CopyButton
          className="btn sm ghost"
          copiedLabel="CSV copied"
          label="Copy CSV"
          value={csv}
        />
      </div>
      {open ? (
        <div className="overflow-x-auto p-4" id="contributors-data-table-panel">
          <table className="w-full text-left t-sm">
            <caption className="sr-only">
              Repository contributors data table
            </caption>
            <thead className="t-label" style={{ color: "var(--ink-3)" }}>
              <tr>
                <th className="py-2 pr-3">Scope</th>
                <th className="py-2 pr-3">Week</th>
                <th className="py-2 pr-3 text-right">Commits</th>
                <th className="py-2 pr-3 text-right">Additions</th>
                <th className="py-2 text-right">Deletions</th>
              </tr>
            </thead>
            <tbody>
              {weeks.map((week) => (
                <tr
                  className="border-t"
                  key={`repo-${week.weekStart}`}
                  style={{ borderColor: "var(--line-soft)" }}
                >
                  <td className="py-2 pr-3">Repository</td>
                  <td className="py-2 pr-3">{formatDate(week.weekStart)}</td>
                  <td className="py-2 pr-3 text-right t-num">
                    {formatNumber(week.commits)}
                  </td>
                  <td className="py-2 pr-3 text-right t-num">
                    {formatNumber(week.additions)}
                  </td>
                  <td className="py-2 text-right t-num">
                    {formatNumber(week.deletions)}
                  </td>
                </tr>
              ))}
              {contributors.flatMap((contributor) =>
                contributor.weeks.map((week) => (
                  <tr
                    className="border-t"
                    key={`${contributor.login}-${week.weekStart}`}
                    style={{ borderColor: "var(--line-soft)" }}
                  >
                    <td className="py-2 pr-3">
                      <Link
                        className="break-words hover:underline"
                        href={contributor.profileHref}
                      >
                        {contributor.login}
                      </Link>
                    </td>
                    <td className="py-2 pr-3">{formatDate(week.weekStart)}</td>
                    <td className="py-2 pr-3 text-right t-num">
                      <Link
                        className="hover:underline"
                        href={contributor.commitsHref}
                      >
                        {formatNumber(week.commits)}
                      </Link>
                    </td>
                    <td className="py-2 pr-3 text-right t-num">
                      {formatNumber(week.additions)}
                    </td>
                    <td className="py-2 text-right t-num">
                      {formatNumber(week.deletions)}
                    </td>
                  </tr>
                )),
              )}
            </tbody>
          </table>
        </div>
      ) : null}
    </section>
  );
}
