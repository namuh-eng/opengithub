import Link from "next/link";
import { MarkdownBody } from "@/components/MarkdownBody";
import type { PackageDetail, PackageDetailFetchResult } from "@/lib/api";
import { ownerPackagesHref, packageDetailHref } from "@/lib/navigation";

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

function formatBytes(value: number | null) {
  if (value === null) {
    return "Size unknown";
  }
  const units = ["B", "KB", "MB", "GB"];
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size >= 10 || unit === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[unit]}`;
}

function platformLabel(os: string | null, arch: string | null) {
  return [os, arch].filter(Boolean).join(" / ") || "Any platform";
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

function InstallCard({ detail }: { detail: PackageDetail }) {
  return (
    <section className="card overflow-hidden" aria-labelledby="package-install">
      <div className="border-b border-[var(--line)] p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Installation
        </p>
        <h2 className="t-h2 mt-1" id="package-install">
          Install from the command line
        </h2>
      </div>
      <div className="grid gap-3 p-5">
        {detail.installCommands.length > 0 ? (
          detail.installCommands.map((command) => (
            <div
              className="rounded-[var(--radius)] bg-[var(--surface-2)] p-4"
              key={`${command.label}-${command.command}`}
            >
              <div className="mb-2 flex flex-wrap items-center gap-2">
                <span className="chip soft">{command.label}</span>
                {command.platform ? (
                  <span className="chip soft">{command.platform}</span>
                ) : null}
              </div>
              <code
                className="t-mono block overflow-x-auto whitespace-pre"
                style={{ color: "var(--ink-1)" }}
              >
                {command.command}
              </code>
            </div>
          ))
        ) : (
          <p className="t-body" style={{ color: "var(--ink-2)" }}>
            Install commands will appear after the first version is published.
          </p>
        )}
      </div>
    </section>
  );
}

function RecentVersions({
  detail,
  ownerKind,
}: {
  detail: PackageDetail;
  ownerKind: "user" | "organization";
}) {
  return (
    <section
      className="card overflow-hidden"
      aria-labelledby="package-versions"
    >
      <div className="border-b border-[var(--line)] p-5">
        <p className="t-label" style={{ color: "var(--ink-3)" }}>
          Versions
        </p>
        <h2 className="t-h2 mt-1" id="package-versions">
          Recent versions
        </h2>
      </div>
      <div>
        {detail.versions.length > 0 ? (
          detail.versions.slice(0, 8).map((version) => (
            <article className="list-row p-5" key={version.id}>
              <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto]">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <Link
                      className="chip"
                      href={packageDetailHref(
                        ownerKind,
                        detail.owner.login,
                        detail.packageType,
                        detail.name,
                        version.version,
                      )}
                    >
                      {version.version}
                    </Link>
                    {detail.selectedVersion?.id === version.id ? (
                      <span className="chip accent">Selected</span>
                    ) : null}
                    {version.shortDigest ? (
                      <span
                        className="t-mono-sm"
                        style={{ color: "var(--ink-3)" }}
                      >
                        {version.shortDigest}
                      </span>
                    ) : null}
                  </div>
                  <p className="t-sm mt-2" style={{ color: "var(--ink-2)" }}>
                    Published {formatDate(version.publishedAt)} by{" "}
                    <Link className="underline" href={version.publisher.href}>
                      {version.publisher.name ?? version.publisher.login}
                    </Link>
                  </p>
                </div>
                <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                  {platformLabel(version.platformOs, version.platformArch)} ·{" "}
                  {formatBytes(version.sizeBytes)}
                </span>
              </div>
            </article>
          ))
        ) : (
          <p className="t-body p-5" style={{ color: "var(--ink-2)" }}>
            No versions are visible for this package yet.
          </p>
        )}
      </div>
    </section>
  );
}

function BlobSummary({ detail }: { detail: PackageDetail }) {
  if (detail.blobs.length === 0) {
    return null;
  }
  return (
    <section className="card p-5" aria-labelledby="package-blobs">
      <p className="t-label" style={{ color: "var(--ink-3)" }}>
        OS / Arch
      </p>
      <h2 className="t-h3 mt-1" id="package-blobs">
        Published artifacts
      </h2>
      <div className="mt-4 grid gap-3">
        {detail.blobs.slice(0, 6).map((blob) => (
          <div
            className="rounded-[var(--radius)] border border-[var(--line)] p-3"
            key={blob.id}
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <span className="chip soft">
                {platformLabel(blob.platformOs, blob.platformArch)}
              </span>
              <span className="t-mono-sm" style={{ color: "var(--ink-3)" }}>
                {formatBytes(blob.sizeBytes)}
              </span>
            </div>
            <p
              className="t-mono-sm mt-2 break-all"
              style={{ color: "var(--ink-3)" }}
            >
              {blob.shortDigest}
            </p>
          </div>
        ))}
      </div>
    </section>
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
      <BlobSummary detail={detail} />
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
          <InstallCard detail={detail} />
          <RecentVersions detail={detail} ownerKind={ownerKind} />
          <AboutContent detail={detail} />
        </div>
        <DetailSidebar detail={detail} />
      </div>
    </div>
  );
}
