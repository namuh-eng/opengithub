"use client";

import Link from "next/link";
import { useMemo, useRef, useState } from "react";
import { repositoryCommitHistoryHref } from "@/lib/navigation";

type RepositoryCommitDateFilterProps = {
  owner: string;
  repo: string;
  refName: string;
  path: string | null;
  author: string | null;
  until: string | null;
};

function dateOnly(value: string | null) {
  if (!value) {
    return "";
  }
  return value.slice(0, 10);
}

function isValidDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false;
  }
  const parsed = new Date(`${value}T00:00:00Z`);
  return (
    !Number.isNaN(parsed.getTime()) && parsed.toISOString().startsWith(value)
  );
}

export function RepositoryCommitDateFilter({
  owner,
  repo,
  refName,
  path,
  author,
  until,
}: RepositoryCommitDateFilterProps) {
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const [selectedDate, setSelectedDate] = useState(dateOnly(until));
  const invalid = selectedDate.length > 0 && !isValidDate(selectedDate);
  const applyHref = useMemo(
    () =>
      repositoryCommitHistoryHref({
        owner,
        repo,
        refName,
        path,
        author,
        until: selectedDate ? `${selectedDate}T23:59:59Z` : null,
      }),
    [author, owner, path, refName, repo, selectedDate],
  );
  const clearHref = repositoryCommitHistoryHref({
    owner,
    repo,
    refName,
    path,
    author,
  });

  return (
    <details className="relative" ref={detailsRef}>
      <summary
        aria-label={`Filter commits by date. Current date ${until ? dateOnly(until) : "All time"}`}
        className="btn sm inline-flex cursor-pointer list-none"
      >
        {until ? `Until ${dateOnly(until)}` : "All time"}
      </summary>
      <div
        className="absolute left-0 z-20 mt-2 w-80 rounded-md p-3 max-sm:w-[calc(100vw-3rem)]"
        role="dialog"
        aria-label="Filter commits by date"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--line)",
          boxShadow: "var(--shadow-md)",
        }}
      >
        <p className="font-semibold" style={{ color: "var(--ink-1)" }}>
          Date
        </p>
        <p className="mt-1 t-xs">
          Show commits made on or before the selected day.
        </p>
        <label className="mt-3 block t-label" htmlFor="commit-until-date">
          Until date
        </label>
        <input
          aria-invalid={invalid}
          className="input mt-2 h-10 w-full px-3"
          id="commit-until-date"
          onChange={(event) => setSelectedDate(event.target.value)}
          type="date"
          value={selectedDate}
        />
        {invalid ? (
          <p className="mt-2 t-xs" role="alert" style={{ color: "var(--err)" }}>
            Enter a valid date.
          </p>
        ) : null}
        <div className="mt-4 flex flex-wrap items-center gap-2">
          {selectedDate && !invalid ? (
            <Link
              className="btn sm primary"
              href={applyHref}
              onClick={() => {
                if (detailsRef.current) {
                  detailsRef.current.open = false;
                }
              }}
            >
              Apply date
            </Link>
          ) : (
            <button className="btn sm" disabled type="button">
              Apply date
            </button>
          )}
          <Link
            className="btn sm"
            href={clearHref}
            onClick={() => {
              if (detailsRef.current) {
                detailsRef.current.open = false;
              }
            }}
          >
            Clear date
          </Link>
        </div>
      </div>
    </details>
  );
}
