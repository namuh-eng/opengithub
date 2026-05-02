"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { RepositoryShell } from "@/components/RepositoryShell";
import type { RepositoryOverview, RepositoryRefSummary } from "@/lib/api";

type BranchTab = "overview" | "active" | "stale" | "all";

// Row fields are deliberately derived from the existing refs/repository seam.
// Future branch activity, check, PR, protection, and ruleset tables can replace
// these values without changing the rendering contract below.
type BranchRow = {
  ref: RepositoryRefSummary;
  classification: "default" | "active" | "stale";
  updatedLabel: string;
  checkLabel: string;
  checkTone: "soft" | "ok" | "warn" | "err";
  protectionLabel: string;
  protectionTone: "soft" | "info";
  protectionHref: string;
  ahead: number;
  behind: number;
  pullRequestLabel: string;
  pullRequestHref: string;
  activityHref: string;
  compareHref: string;
};

type RepositoryBranchesPageProps = {
  repository: RepositoryOverview;
  refs: RepositoryRefSummary[];
};

const BRANCH_TABS: { value: BranchTab; label: string; description: string }[] =
  [
    {
      value: "overview",
      label: "Overview",
      description: "Default branch plus recently active branches.",
    },
    {
      value: "active",
      label: "Active",
      description: "Branches updated in the last 90 days.",
    },
    {
      value: "stale",
      label: "Stale",
      description: "Branches with no updates for at least 90 days.",
    },
    {
      value: "all",
      label: "All",
      description: "Every repository branch returned by the refs API.",
    },
  ];

const STALE_AFTER_DAYS = 90;
const DAY_MS = 24 * 60 * 60 * 1000;

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  if (!Number.isFinite(timestamp)) {
    return "recently";
  }
  const seconds = Math.max(1, Math.floor((Date.now() - timestamp) / 1000));
  if (seconds < 60) {
    return "just now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  const days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days}d ago`;
  }
  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${months}mo ago`;
  }
  return `${Math.floor(months / 12)}y ago`;
}

function classifyBranch(
  repository: RepositoryOverview,
  ref: RepositoryRefSummary,
) {
  if (ref.shortName === repository.default_branch) {
    return "default" as const;
  }
  const updatedAt = new Date(ref.updatedAt).getTime();
  if (
    Number.isFinite(updatedAt) &&
    Date.now() - updatedAt >= STALE_AFTER_DAYS * DAY_MS
  ) {
    return "stale" as const;
  }
  return "active" as const;
}

function branchCompareHref(repository: RepositoryOverview, branch: string) {
  return `/${repository.owner_login}/${repository.name}/compare?base=${encodeURIComponent(
    repository.default_branch,
  )}&head=${encodeURIComponent(branch)}`;
}

function branchPullRequestHref(repository: RepositoryOverview, branch: string) {
  const query = `head:${repository.owner_login}:${branch}`;
  return `/${repository.owner_login}/${repository.name}/pulls?q=${encodeURIComponent(
    query,
  )}`;
}

function toBranchRows(
  repository: RepositoryOverview,
  refs: RepositoryRefSummary[],
): BranchRow[] {
  const branchRefs = refs.filter((ref) => ref.kind === "branch");
  return branchRefs.map((ref) => {
    const classification = classifyBranch(repository, ref);
    const isDefault = classification === "default";
    const rulesHref = `/${repository.owner_login}/${repository.name}/settings/branches?pattern=${encodeURIComponent(
      ref.shortName,
    )}`;
    const compareHref = branchCompareHref(repository, ref.shortName);

    return {
      ref,
      classification,
      updatedLabel: relativeTime(ref.updatedAt),
      checkLabel: ref.targetShortOid
        ? "Checks not reported"
        : "No commit target",
      checkTone: ref.targetShortOid ? "soft" : "warn",
      protectionLabel: isDefault ? "Default branch" : "Unprotected",
      protectionTone: isDefault ? "info" : "soft",
      protectionHref: rulesHref,
      ahead: 0,
      behind: 0,
      pullRequestLabel: "Find PRs",
      pullRequestHref: branchPullRequestHref(repository, ref.shortName),
      activityHref: `/${repository.owner_login}/${repository.name}/activity?branch=${encodeURIComponent(
        ref.shortName,
      )}`,
      compareHref,
    };
  });
}

function tabMatches(tab: BranchTab, row: BranchRow) {
  if (tab === "all") {
    return true;
  }
  if (tab === "active") {
    return row.classification === "active" || row.classification === "default";
  }
  if (tab === "stale") {
    return row.classification === "stale";
  }
  return row.classification !== "stale";
}

function BranchName({ row }: { row: BranchRow }) {
  return (
    <div className="min-w-0">
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <Link
          className="t-mono-sm truncate font-semibold hover:underline"
          href={row.ref.href}
          style={{ color: "var(--ink-1)" }}
        >
          {row.ref.shortName}
        </Link>
        {row.classification === "default" ? (
          <span className="chip active">default</span>
        ) : null}
        <button
          aria-label={`Copy branch name ${row.ref.shortName}`}
          className="btn ghost sm"
          onClick={() => void navigator.clipboard?.writeText(row.ref.shortName)}
          type="button"
        >
          Copy
        </button>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-2">
        <Link
          className={`chip ${row.protectionTone}`}
          href={row.protectionHref}
        >
          {row.protectionLabel}
        </Link>
        {row.ref.targetShortOid ? (
          <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
            {row.ref.targetShortOid}
          </span>
        ) : null}
      </div>
    </div>
  );
}

function BranchTable({ rows, title }: { rows: BranchRow[]; title: string }) {
  if (rows.length === 0) {
    return (
      <section className="card p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          {title}
        </p>
        <p className="t-sm mt-2" style={{ color: "var(--ink-3)" }}>
          No branches match this view.
        </p>
      </section>
    );
  }

  return (
    <section className="card overflow-hidden">
      <div
        className="border-b px-4 py-3"
        style={{ borderColor: "var(--line)" }}
      >
        <h2 className="t-h3" style={{ color: "var(--ink-1)" }}>
          {title}
        </h2>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[920px] text-left">
          <thead>
            <tr
              className="t-label border-b"
              style={{ borderColor: "var(--line)", color: "var(--ink-3)" }}
            >
              <th className="px-4 py-3 font-medium">Branch</th>
              <th className="px-3 py-3 font-medium">Updated</th>
              <th className="px-3 py-3 font-medium">Check status</th>
              <th className="px-3 py-3 font-medium">Behind</th>
              <th className="px-3 py-3 font-medium">Ahead</th>
              <th className="px-3 py-3 font-medium">Pull request</th>
              <th className="px-4 py-3 text-right font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr
                className="list-row align-top"
                key={row.ref.name}
                style={{ borderColor: "var(--line-soft)" }}
              >
                <td className="px-4 py-4">
                  <BranchName row={row} />
                </td>
                <td
                  className="t-sm px-3 py-4"
                  style={{ color: "var(--ink-3)" }}
                >
                  {row.updatedLabel}
                </td>
                <td className="px-3 py-4">
                  <span className={`chip ${row.checkTone}`}>
                    {row.checkLabel}
                  </span>
                </td>
                <td
                  className="t-num px-3 py-4"
                  style={{ color: "var(--ink-2)" }}
                >
                  {row.behind}
                </td>
                <td
                  className="t-num px-3 py-4"
                  style={{ color: "var(--ink-2)" }}
                >
                  {row.ahead}
                </td>
                <td className="px-3 py-4">
                  <Link
                    className="t-sm hover:underline"
                    href={row.pullRequestHref}
                  >
                    {row.pullRequestLabel}
                  </Link>
                </td>
                <td className="px-4 py-4 text-right">
                  <details className="relative inline-block text-left">
                    <summary className="btn ghost sm list-none cursor-pointer">
                      Actions
                    </summary>
                    <div
                      className="absolute right-0 z-10 mt-2 grid min-w-40 gap-1 rounded-md border p-2 text-left shadow-sm"
                      style={{
                        background: "var(--surface)",
                        borderColor: "var(--line)",
                        boxShadow: "var(--shadow-sm)",
                      }}
                    >
                      <Link
                        className="t-sm rounded px-2 py-1 hover:underline"
                        href={row.activityHref}
                      >
                        Activity
                      </Link>
                      <Link
                        className="t-sm rounded px-2 py-1 hover:underline"
                        href={row.protectionHref}
                      >
                        View rules
                      </Link>
                      <Link
                        className="t-sm rounded px-2 py-1 hover:underline"
                        href={row.compareHref}
                      >
                        Compare
                      </Link>
                    </div>
                  </details>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function BranchSummaryCards({ rows }: { rows: BranchRow[] }) {
  const defaultBranch = rows.find((row) => row.classification === "default");
  const activeCount = rows.filter(
    (row) =>
      row.classification === "active" || row.classification === "default",
  ).length;
  const staleCount = rows.filter(
    (row) => row.classification === "stale",
  ).length;
  return (
    <div className="grid gap-3 md:grid-cols-3">
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Default
        </p>
        <p className="t-h3 mt-2" style={{ color: "var(--ink-1)" }}>
          {defaultBranch?.ref.shortName ?? "Not set"}
        </p>
      </div>
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Active
        </p>
        <p className="t-num mt-2 text-2xl" style={{ color: "var(--ink-1)" }}>
          {activeCount}
        </p>
      </div>
      <div className="card p-4">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Stale
        </p>
        <p className="t-num mt-2 text-2xl" style={{ color: "var(--ink-1)" }}>
          {staleCount}
        </p>
      </div>
    </div>
  );
}

export function RepositoryBranchesPage({
  repository,
  refs,
}: RepositoryBranchesPageProps) {
  const [tab, setTab] = useState<BranchTab>("overview");
  const [query, setQuery] = useState("");
  const rows = useMemo(
    () => toBranchRows(repository, refs),
    [repository, refs],
  );
  const normalizedQuery = query.trim().toLowerCase();
  const visibleRows = rows.filter(
    (row) =>
      tabMatches(tab, row) &&
      (normalizedQuery.length === 0 ||
        row.ref.shortName.toLowerCase().includes(normalizedQuery) ||
        row.ref.name.toLowerCase().includes(normalizedQuery)),
  );
  const defaultRows = visibleRows.filter(
    (row) => row.classification === "default",
  );
  const activeRows = visibleRows.filter(
    (row) => row.classification === "active",
  );
  const currentTab =
    BRANCH_TABS.find((item) => item.value === tab) ?? BRANCH_TABS[0];

  return (
    <RepositoryShell
      activePath={`/${repository.owner_login}/${repository.name}/branches`}
      frameClassName="max-w-6xl"
      repository={repository}
    >
      <div className="grid gap-5">
        <header className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              Repository refs
            </p>
            <h1 className="t-h2 mt-2" style={{ color: "var(--ink-1)" }}>
              Branches
            </h1>
            <p
              className="t-sm mt-2 max-w-2xl"
              style={{ color: "var(--ink-3)" }}
            >
              Review branch freshness, default-branch rules, status hints, and
              safe branch actions from the live refs API.
            </p>
          </div>
          <Link
            className="btn"
            href={`/${repository.owner_login}/${repository.name}/tree/${repository.default_branch}`}
          >
            Open default branch
          </Link>
        </header>

        <BranchSummaryCards rows={rows} />

        <section className="card p-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <div
                aria-label="Branch views"
                className="tabs flex gap-1"
                role="tablist"
              >
                {BRANCH_TABS.map((item) => (
                  <button
                    aria-selected={tab === item.value}
                    className={`tab px-3 py-2 ${tab === item.value ? "active" : ""}`}
                    key={item.value}
                    onClick={() => setTab(item.value)}
                    role="tab"
                    type="button"
                  >
                    {item.label}
                  </button>
                ))}
              </div>
              <p className="t-xs mt-2" style={{ color: "var(--ink-3)" }}>
                {currentTab.description}
              </p>
            </div>
            <label className="min-w-[260px] flex-1 md:max-w-sm">
              <span className="sr-only">Search branches</span>
              <input
                aria-label="Search branches"
                className="input w-full"
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search branches"
                type="search"
                value={query}
              />
            </label>
          </div>
        </section>

        {tab === "overview" ? (
          <>
            <BranchTable rows={defaultRows} title="Default branch" />
            <BranchTable rows={activeRows} title="Active branches" />
          </>
        ) : (
          <BranchTable
            rows={visibleRows}
            title={`${currentTab.label} branches`}
          />
        )}
      </div>
    </RepositoryShell>
  );
}
