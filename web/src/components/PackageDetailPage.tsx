import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import { PackageDetailInteractions } from "@/components/PackageDetailInteractions";
import type { PackageDetail, PackageDetailFetchResult } from "@/lib/api";
import { ownerPackagesHref } from "@/lib/navigation";

type PackageDetailPageProps = {
  owner: string;
  ownerKind: "user" | "organization";
  result: PackageDetailFetchResult;
};

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
    return "recently";
  }
  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

function PackageUnavailable({
  result,
}: {
  result: Exclude<PackageDetailFetchResult, { ok: true }>;
}) {
  const forbidden = result.status === 403;
  const missing = result.status === 404;
  return (
    <section className="card p-6" role="status">
      <span className={`chip ${forbidden ? "warn" : "err"}`}>
        {forbidden
          ? "Read access required"
          : missing
            ? "Not found"
            : "Unavailable"}
      </span>
      <h1 className="t-h1 mt-4">Package could not load</h1>
      <p className="t-body mt-3 max-w-2xl" style={{ color: "var(--ink-2)" }}>
        {forbidden
          ? "Private and internal package metadata is visible only to viewers with package or linked repository access."
          : result.message}
      </p>
    </section>
  );
}

function DetailHeader({
  detail,
  ownerKind,
}: {
  detail: PackageDetail;
  ownerKind: "user" | "organization";
}) {
  const selected = detail.selectedVersion;
  return (
    <header className="grid gap-4">
      <div className="flex flex-wrap items-center gap-2">
        <Link
          className="t-sm underline"
          href={ownerPackagesHref(ownerKind, detail.owner.login)}
        >
          Packages
        </Link>
        <span className="t-xs">/</span>
        <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
          {detail.packageType}
        </span>
      </div>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-3">
            <span className="chip soft text-lg" aria-hidden="true">
              {TYPE_ICON[detail.packageType] ?? "□"}
            </span>
            <h1 className="t-h1 min-w-0 break-words">{detail.name}</h1>
            <span className="chip soft">{detail.visibility}</span>
            {selected ? <span className="chip accent">Latest</span> : null}
          </div>
          <p className="t-sm mt-3" style={{ color: "var(--ink-2)" }}>
            Published by{" "}
            <Link className="underline" href={detail.publisher.href}>
              {detail.publisher.name ?? detail.publisher.login}
            </Link>{" "}
            on {formatDate(detail.publishedAt)}
            {selected?.shortDigest ? ` · ${selected.shortDigest}` : ""}
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            <span className="chip soft">{detail.typeLabel}</span>
            <span className="chip soft">
              {detail.downloadCount.toLocaleString()} downloads
            </span>
            {detail.linkedRepository ? (
              <Link className="chip" href={detail.linkedRepository.href}>
                {detail.linkedRepository.fullName}
              </Link>
            ) : (
              <span className="chip soft">Owner scoped</span>
            )}
          </div>
        </div>
        {detail.admin.canAdmin && detail.admin.settingsHref ? (
          <Link className="btn primary" href={detail.admin.settingsHref}>
            Settings
          </Link>
        ) : null}
      </div>
    </header>
  );
}

function AboutContent({ detail }: { detail: PackageDetail }) {
  return (
    <section className="card p-6" aria-labelledby="package-about">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        About
      </p>
      <h2 className="t-h2 mt-1" id="package-about">
        README
      </h2>
      {detail.about.empty || !detail.about.html ? (
        <p className="t-body mt-4" style={{ color: "var(--ink-2)" }}>
          This package does not have README or about content yet.
        </p>
      ) : (
        <div className="mt-5">
          <MarkdownBody html={detail.about.html} labelledBy="package-about" />
        </div>
      )}
    </section>
  );
}

function DetailSidebar({ detail }: { detail: PackageDetail }) {
  return (
    <aside className="grid content-start gap-4">
      <section className="card p-5" aria-labelledby="package-details">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Details
        </p>
        <h2 className="t-h3 mt-1" id="package-details">
          Package metadata
        </h2>
        <dl className="mt-4 grid gap-3">
          <div>
            <dt className="t-xs">Owner</dt>
            <dd>
              <Link className="t-sm underline" href={detail.owner.href}>
                {detail.owner.login}
              </Link>
            </dd>
          </div>
          <div>
            <dt className="t-xs">Last published</dt>
            <dd className="t-sm">{formatDate(detail.updatedAt)}</dd>
          </div>
          <div>
            <dt className="t-xs">Downloads</dt>
            <dd className="t-num text-2xl">
              {detail.downloadCount.toLocaleString()}
            </dd>
          </div>
          {detail.linkedRepository ? (
            <div>
              <dt className="t-xs">Source repository</dt>
              <dd>
                <Link
                  className="t-sm underline"
                  href={detail.linkedRepository.href}
                >
                  {detail.linkedRepository.fullName}
                </Link>
              </dd>
            </div>
          ) : null}
        </dl>
      </section>
    </aside>
  );
}

export function PackageDetailPage({
  ownerKind,
  result,
}: PackageDetailPageProps) {
  if (!result.ok) {
    return <PackageUnavailable result={result} />;
  }

  const detail = result.package;
  return (
    <div className="grid gap-6">
      <DetailHeader detail={detail} ownerKind={ownerKind} />
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_300px]">
        <div className="grid min-w-0 gap-6">
          <PackageDetailInteractions detail={detail} ownerKind={ownerKind} />
          <AboutContent detail={detail} />
        </div>
        <DetailSidebar detail={detail} />
      </div>
    </div>
  );
}
