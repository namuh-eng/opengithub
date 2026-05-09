"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositoryContributorsPeriodSelector } from "@/components/RepositoryContributorsPeriodSelector";
import type { RepositoryContributorsWeek } from "@/lib/api";
import { repositoryContributorsHref } from "@/lib/navigation";

type RepositoryContributorsControlsProps = {
  owner: string;
  repo: string;
  activePeriod: string;
  weeks: RepositoryContributorsWeek[];
};

function shortDate(value: string) {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return value;
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  }).format(date);
}

export function RepositoryContributorsControls({
  owner,
  repo,
  activePeriod,
  weeks,
}: RepositoryContributorsControlsProps) {
  const lastIndex = Math.max(0, weeks.length - 1);
  const [startIndex, setStartIndex] = useState(0);
  const [endIndex, setEndIndex] = useState(lastIndex);
  const selected = useMemo(() => {
    const start = weeks[Math.min(startIndex, endIndex)] ?? null;
    const end = weeks[Math.max(startIndex, endIndex)] ?? null;
    return { start, end };
  }, [endIndex, startIndex, weeks]);
  const hasRangeChoices = weeks.length > 1;
  const applyHref =
    selected.start && selected.end
      ? repositoryContributorsHref(owner, repo, {
          period: activePeriod,
          start: selected.start.weekStart,
          end: selected.end.weekEnd,
        })
      : repositoryContributorsHref(owner, repo, { period: activePeriod });
  const clearHref = repositoryContributorsHref(owner, repo, {
    period: activePeriod,
  });

  function setStart(value: string) {
    const next = Number(value);
    setStartIndex(next);
    if (next > endIndex) setEndIndex(next);
  }

  function setEnd(value: string) {
    const next = Number(value);
    setEndIndex(next);
    if (next < startIndex) setStartIndex(next);
  }

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap gap-2">
        <RepositoryContributorsPeriodSelector
          activePeriod={activePeriod}
          owner={owner}
          repo={repo}
        />
        <Link className="btn" href="#contributors-data-table-panel">
          View as data table
        </Link>
      </div>

      <div
        className="card grid gap-3 p-3"
        style={{ background: "var(--surface)" }}
      >
        <div className="flex flex-wrap items-center gap-2">
          <span className="chip soft">Range</span>
          <span className="t-xs">
            {selected.start && selected.end
              ? `${shortDate(selected.start.weekStart)} - ${shortDate(
                  selected.end.weekEnd,
                )}`
              : "No weekly buckets"}
          </span>
          <span className="flex-1" />
          <Link
            aria-disabled={!hasRangeChoices}
            className={`btn sm ${hasRangeChoices ? "" : "disabled"}`}
            href={hasRangeChoices ? applyHref : clearHref}
          >
            Apply
          </Link>
          <Link className="btn sm ghost" href={clearHref}>
            Clear
          </Link>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
          <label className="grid gap-1">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              Start week
            </span>
            <input
              aria-label="Start week"
              disabled={!hasRangeChoices}
              max={lastIndex}
              min={0}
              onChange={(event) => setStart(event.currentTarget.value)}
              type="range"
              value={Math.min(startIndex, endIndex)}
            />
          </label>
          <label className="grid gap-1">
            <span className="t-label" style={{ color: "var(--ink-3)" }}>
              End week
            </span>
            <input
              aria-label="End week"
              disabled={!hasRangeChoices}
              max={lastIndex}
              min={0}
              onChange={(event) => setEnd(event.currentTarget.value)}
              type="range"
              value={Math.max(startIndex, endIndex)}
            />
          </label>
        </div>
      </div>
    </div>
  );
}
