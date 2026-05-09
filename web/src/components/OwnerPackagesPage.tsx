import Link from "next/link";
import type { OwnerPackageFilters, OwnerPackageList } from "@/lib/api";
import { ownerPackagesHref } from "@/lib/navigation";

type OwnerPackagesPageProps = {
  list: OwnerPackageList;
  owner: string;
  ownerKind: "user" | "organization";
};

const TYPE_OPTIONS = [
  ["all", "All"],
  ["container", "Container"],
  ["npm", "npm"],
  ["rubygems", "RubyGems"],
  ["maven", "Maven"],
  ["nuget", "NuGet"],
] as const;

const VISIBILITY_OPTIONS = [
  ["all", "All"],
  ["public", "Public"],
  ["internal", "Internal"],
  ["private", "Private"],
] as const;

const SORT_OPTIONS = [
  ["downloads-desc", "Most downloads"],
  ["downloads-asc", "Least downloads"],
] as const;

const TYPE_ICON: Record<string, string> = {
  container: "▣",
  npm: "◇",
  rubygems: "◆",
  maven: "◫",
  nuget: "▧",
  generic: "□",
};

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Published recently";
  }
  return `Published ${new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
    year: "numeric",
  }).format(date)}`;
}

function filtersForHref(filters: OwnerPackageFilters) {
  return {
    artifactTab: filters.artifactTab,
    page: filters.page,
    pageSize: filters.pageSize,
    query: filters.query,
    sort: filters.sort,
    type: filters.packageType,
    visibility: filters.visibility,
  };
}

function PackageTabs({
  filters,
  owner,
  ownerKind,
}: {
  filters: OwnerPackageFilters;
  owner: string;
  ownerKind: "user" | "organization";
}) {
  const base = filtersForHref(filters);
  const tabs = [
    { label: "GitHub Packages", value: "packages" },
    { label: "Linked artifacts", value: "artifacts" },
  ];
  return (
    <nav aria-label="Package result tabs" className="tabs">
      {tabs.map((tab) => {
        const active = filters.artifactTab === tab.value;
        return (
          <Link
            aria-current={active ? "page" : undefined}
            className={`tab ${active ? "active" : ""}`}
            href={ownerPackagesHref(ownerKind, owner, base, {
              artifactTab: tab.value,
              page: null,
            })}
            key={tab.value}
          >
            {tab.label}
          </Link>
        );
      })}
    </nav>
  );
}

function FilterSelect({
  label,
  name,
  options,
  value,
}: {
  label: string;
  name: string;
  options: readonly (readonly [string, string])[];
  value: string;
}) {
  const id = `package-filter-${name}`;
  return (
    <label className="grid gap-1" htmlFor={id}>
      <span className="t-label" style={{ color: "var(--ink-3)" }}>
        {label}
      </span>
      <select
        className="input min-w-36"
        id={id}
        name={name}
        defaultValue={value}
      >
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionLabel}
          </option>
        ))}
      </select>
    </label>
  );
}

function PackageFilters({
  filters,
  owner,
  ownerKind,
}: {
  filters: OwnerPackageFilters;
  owner: string;
  ownerKind: "user" | "organization";
}) {
  return (
    <form action={ownerPackagesHref(ownerKind, owner)} className="card p-4">
      {ownerKind === "user" ? (
        <input name="tab" type="hidden" value="packages" />
      ) : null}
      <input name="artifactTab" type="hidden" value={filters.artifactTab} />
      <div className="grid gap-3 lg:grid-cols-[1fr_auto_auto_auto_auto] lg:items-end">
        <label className="grid gap-1" htmlFor="package-search-input">
          <span className="t-label" style={{ color: "var(--ink-3)" }}>
            Search packages
          </span>
          <span className="relative block">
            <span
              aria-hidden="true"
              className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2"
              style={{ color: "var(--ink-4)" }}
            >
              ⌕
            </span>
            <input
              aria-label="Search packages"
              className="input w-full pl-9"
              id="package-search-input"
              defaultValue={filters.query ?? ""}
              name="q"
              placeholder="Search by package name"
              type="search"
            />
          </span>
        </label>
        <FilterSelect
          label="Type"
          name="type"
          options={TYPE_OPTIONS}
          value={filters.packageType}
        />
        <FilterSelect
          label="Visibility"
          name="visibility"
          options={VISIBILITY_OPTIONS}
          value={filters.visibility}
        />
        <FilterSelect
          label="Sort"
          name="sort"
          options={SORT_OPTIONS}
          value={filters.sort}
        />
        <button className="btn sm" type="submit">
          Apply filters
        </button>
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-3">
        <Link
          className="t-sm underline"
          href={ownerPackagesHref(ownerKind, owner)}
        >
          Clear filters
        </Link>
        <span className="t-xs">
          Changing filters updates the URL and reloads package results.
        </span>
      </div>
    </form>
  );
}

function PackageRow({ item }: { item: OwnerPackageList["items"][number] }) {
  return (
    <article className="list-row py-5">
      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <span className="chip soft" aria-hidden="true">
              {TYPE_ICON[item.packageType] ?? "□"}
            </span>
            <Link className="t-h3 min-w-0 no-underline" href={item.href}>
              {item.name}
            </Link>
            <span className="chip soft">{item.typeLabel}</span>
            {item.visibility !== "public" ? (
              <span className="chip warn">{item.visibility}</span>
            ) : null}
          </div>
          <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
            {formatDate(item.publishedAt)} by{" "}
            <Link className="underline" href={item.publisher.href}>
              {item.publisher.name ?? item.publisher.login}
            </Link>
            {item.latestVersion ? ` · Latest ${item.latestVersion}` : ""}
          </p>
          {item.linkedRepository ? (
            <p className="t-mono-sm mt-2" style={{ color: "var(--ink-3)" }}>
              Linked repository{" "}
              <Link className="underline" href={item.linkedRepository.href}>
                {item.linkedRepository.fullName}
              </Link>
            </p>
          ) : (
            <p className="t-mono-sm mt-2" style={{ color: "var(--ink-3)" }}>
              No repository link recorded
            </p>
          )}
        </div>
        <span
          className="t-mono-sm inline-flex items-center gap-2"
          style={{ color: "var(--ink-3)" }}
        >
          <span aria-hidden="true">⇩</span>
          {item.downloadCount.toLocaleString()} downloads
        </span>
      </div>
    </article>
  );
}

function EmptyPackages({
  filters,
  owner,
  ownerKind,
}: {
  filters: OwnerPackageFilters;
  owner: string;
  ownerKind: "user" | "organization";
}) {
  const filtered =
    Boolean(filters.query) ||
    filters.packageType !== "all" ||
    filters.visibility !== "all" ||
    filters.sort !== "downloads-desc";
  return (
    <section className="card p-6" aria-labelledby="packages-empty">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        Empty state
      </p>
      <h2 className="t-h2 mt-2" id="packages-empty">
        {filtered ? "No packages matched" : "No visible packages yet"}
      </h2>
      <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
        {filtered
          ? "No packages matched the current search, type, visibility, and sort filters."
          : "Public packages will appear here when this owner publishes them."}
      </p>
      {filtered ? (
        <Link
          className="btn sm mt-4"
          href={ownerPackagesHref(ownerKind, owner)}
        >
          Back to all packages
        </Link>
      ) : null}
    </section>
  );
}

export function OwnerPackagesPage({
  list,
  owner,
  ownerKind,
}: OwnerPackagesPageProps) {
  if (list.filters.artifactTab === "artifacts") {
    return (
      <div className="grid gap-4">
        <PackageTabs
          filters={list.filters}
          owner={owner}
          ownerKind={ownerKind}
        />
        <section className="card p-6" aria-labelledby="linked-artifacts">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Placeholder tab
          </p>
          <h2 className="t-h2 mt-2" id="linked-artifacts">
            Linked artifacts
          </h2>
          <p className="t-body mt-3" style={{ color: "var(--ink-2)" }}>
            {list.linkedArtifacts.message}
          </p>
          <Link
            className="btn sm mt-4"
            href={ownerPackagesHref(
              ownerKind,
              owner,
              filtersForHref(list.filters),
              {
                artifactTab: "packages",
              },
            )}
          >
            View GitHub Packages
          </Link>
        </section>
      </div>
    );
  }

  return (
    <div className="grid gap-4">
      <PackageTabs filters={list.filters} owner={owner} ownerKind={ownerKind} />
      <PackageFilters
        filters={list.filters}
        owner={owner}
        ownerKind={ownerKind}
      />
      <section className="card p-5" aria-labelledby="package-count">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <p className="t-label" style={{ color: "var(--ink-3)" }}>
              GitHub Packages
            </p>
            <h2 className="t-h2 mt-1" id="package-count">
              {list.total.toLocaleString()} package{list.total === 1 ? "" : "s"}
            </h2>
          </div>
          <span className="t-xs">Public viewers see public packages only.</span>
        </div>
        <div className="mt-4">
          {list.items.length > 0 ? (
            list.items.map((item) => <PackageRow item={item} key={item.id} />)
          ) : (
            <EmptyPackages
              filters={list.filters}
              owner={owner}
              ownerKind={ownerKind}
            />
          )}
        </div>
      </section>
    </div>
  );
}
