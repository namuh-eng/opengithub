"use client";

import { useMemo, useState } from "react";
import type { RepositoryTrafficSeriesPoint } from "@/lib/api";

type RepositoryTrafficChartProps = {
  label: string;
  points: RepositoryTrafficSeriesPoint[];
  totalLabel: string;
  uniqueLabel: string;
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

function pointLabel(
  point: RepositoryTrafficSeriesPoint,
  totalLabel: string,
  uniqueLabel: string,
) {
  return `${formatDate(point.date)}: ${formatNumber(point.total)} ${totalLabel.toLowerCase()}, ${formatNumber(point.unique)} ${uniqueLabel.toLowerCase()}`;
}

export function RepositoryTrafficChart({
  label,
  points,
  totalLabel,
  uniqueLabel,
}: RepositoryTrafficChartProps) {
  const maxTotal = Math.max(1, ...points.map((point) => point.total));
  const [selectedDate, setSelectedDate] = useState(
    points.find((point) => point.total > 0)?.date ??
      points[points.length - 1]?.date,
  );
  const selectedPoint = useMemo(
    () =>
      points.find((point) => point.date === selectedDate) ??
      points[points.length - 1],
    [points, selectedDate],
  );
  const detailsId = `${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-traffic-point-details`;

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
        <span className="chip soft">Focus points for exact values</span>
      </div>

      <div
        aria-label={`${label} line chart`}
        className="mt-5 grid min-h-52 grid-cols-[repeat(auto-fit,minmax(34px,1fr))] items-end gap-2"
        role="img"
      >
        {points.map((point) => {
          const height = Math.max(10, (point.total / maxTotal) * 100);
          const selected = point.date === selectedPoint?.date;
          return (
            <button
              aria-describedby={selected ? detailsId : undefined}
              aria-label={`${label} ${pointLabel(point, totalLabel, uniqueLabel)}`}
              className="group grid min-w-0 gap-2 rounded-md p-1 text-left focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
              key={point.date}
              onBlur={() => setSelectedDate(point.date)}
              onFocus={() => setSelectedDate(point.date)}
              onMouseEnter={() => setSelectedDate(point.date)}
              style={{
                outlineColor: "var(--accent)",
              }}
              type="button"
            >
              <span
                aria-hidden="true"
                className="relative flex h-32 items-end rounded-md"
                style={{ background: "var(--surface-2)" }}
              >
                <span
                  className="w-full rounded-md transition-[height,background]"
                  style={{
                    background:
                      point.total > 0 ? "var(--accent)" : "var(--line-strong)",
                    height: `${height}%`,
                    boxShadow: selected
                      ? "0 0 0 2px var(--accent-soft)"
                      : "none",
                  }}
                />
                {selected ? (
                  <span
                    className="absolute left-1/2 top-2 h-2 w-2 -translate-x-1/2 rounded-full"
                    style={{ background: "var(--ink-1)" }}
                  />
                ) : null}
              </span>
              <span className="t-mono-sm truncate text-center">
                {formatNumber(point.total)}
              </span>
            </button>
          );
        })}
      </div>

      {selectedPoint ? (
        <div
          aria-live="polite"
          className="mt-4 rounded-md border p-3"
          id={detailsId}
          style={{
            background: "var(--surface-2)",
            borderColor: "var(--line-soft)",
          }}
        >
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Selected point
          </p>
          <p className="t-sm mt-1" style={{ color: "var(--ink-1)" }}>
            {pointLabel(selectedPoint, totalLabel, uniqueLabel)}
          </p>
        </div>
      ) : null}

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
