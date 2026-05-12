"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import type { DashboardTopRepository } from "@/lib/api";

type DashboardTopRepositoriesProps = {
  repositories: DashboardTopRepository[];
};

function formatUpdatedAt(value: string): string {
  const updatedAt = new Date(value);
  if (Number.isNaN(updatedAt.getTime())) {
    return "Updated recently";
  }

  const formatter = new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  });
  return `Updated ${formatter.format(updatedAt)}`;
}

function matchesRepository(repository: DashboardTopRepository, query: string) {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return true;
  }

  return `${repository.ownerLogin}/${repository.name}`
    .toLowerCase()
    .includes(normalized);
}

function VisibilityBadge({
  visibility,
}: {
  visibility: DashboardTopRepository["visibility"];
}) {
  return <span className="chip soft t-xs capitalize">{visibility}</span>;
}

function LanguageLabel({ repository }: { repository: DashboardTopRepository }) {
  if (!repository.primaryLanguage) {
    return null;
  }

  return (
    <span className="inline-flex min-w-0 items-center gap-1.5">
      <span
        aria-hidden="true"
        className="h-3 w-3 rounded-full border border-black/10"
        style={{
          backgroundColor: repository.primaryLanguageColor ?? "var(--ink-3)",
        }}
      />
      <span className="truncate">{repository.primaryLanguage}</span>
    </span>
  );
}

function RepositoryRow({ repository }: { repository: DashboardTopRepository }) {
  return (
    <li>
      <Link
        className="list-row block rounded-md px-2 py-2"
        href={repository.href}
      >
        <div className="flex min-w-0 items-center gap-2">
          <span
            className="min-w-0 flex-1 truncate t-sm font-semibold"
            style={{ color: "var(--accent)" }}
          >
            {repository.ownerLogin}/{repository.name}
          </span>
          <VisibilityBadge visibility={repository.visibility} />
        </div>
        <div
          className="mt-1 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 t-xs"
          style={{ color: "var(--ink-3)" }}
        >
          <LanguageLabel repository={repository} />
          <span>{formatUpdatedAt(repository.updatedAt)}</span>
        </div>
      </Link>
    </li>
  );
}

export function DashboardTopRepositories({
  repositories,
}: DashboardTopRepositoriesProps) {
  const [query, setQuery] = useState("");
  const filteredRepositories = useMemo(
    () =>
      repositories.filter((repository) => matchesRepository(repository, query)),
    [repositories, query],
  );
  const hasRepositories = repositories.length > 0;

  return (
    <aside
      aria-labelledby="top-repositories-heading"
      className="w-full space-y-4 lg:w-[296px]"
    >
      <div className="flex items-center justify-between gap-3">
        <h2 className="t-h3" id="top-repositories-heading">
          Top repositories
        </h2>
        <Link className="btn primary sm" href="/new">
          New
        </Link>
      </div>
      <label className="sr-only" htmlFor="repository-filter">
        Find a repository
      </label>
      <input
        className="input w-full"
        id="repository-filter"
        name="repository-filter"
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Find a repository..."
        type="search"
        value={query}
      />
      <div className="min-h-32">
        {filteredRepositories.length > 0 ? (
          <ul className="space-y-1">
            {filteredRepositories.map((repository) => (
              <RepositoryRow
                key={`${repository.ownerLogin}/${repository.name}`}
                repository={repository}
              />
            ))}
          </ul>
        ) : hasRepositories ? (
          <p
            className="card px-3 py-4 t-sm leading-6"
            style={{ color: "var(--ink-3)" }}
          >
            No repositories match your filter.
          </p>
        ) : (
          <div
            className="card px-4 py-5 t-sm leading-6"
            style={{ color: "var(--ink-3)" }}
          >
            <p>You do not have any repositories yet.</p>
            <Link className="btn ghost sm mt-3" href="/new">
              Create repository
            </Link>
          </div>
        )}
      </div>
    </aside>
  );
}
