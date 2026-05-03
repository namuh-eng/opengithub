"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { CopyButton } from "@/components/CopyButton";
import type { PackageDetail, PackageDetailVersion } from "@/lib/api";
import { packageDetailHref } from "@/lib/navigation";

type PackageDetailInteractionsProps = {
  detail: PackageDetail;
  ownerKind: "user" | "organization";
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

function commandForVersion(
  detail: PackageDetail,
  selected: PackageDetailVersion | null,
) {
  const version = selected?.version ?? "latest";
  const digest = selected?.digest ?? null;
  const namespace = `${detail.owner.login}/${detail.name}`;
  if (detail.packageType === "container") {
    return digest
      ? `docker pull ghcr.io/${namespace}:${version}@${digest}`
      : `docker pull ghcr.io/${namespace}:${version}`;
  }
  if (detail.packageType === "npm") {
    return `npm install @${namespace}`;
  }
  if (detail.packageType === "maven") {
    return `mvn dependency:get -Dartifact=${detail.owner.login}:${detail.name}:${version}`;
  }
  if (detail.packageType === "nuget") {
    return `dotnet add package ${detail.name} --version ${version}`;
  }
  if (detail.packageType === "rubygems") {
    return `gem install ${detail.name} -v ${version}`;
  }
  return `curl -O https://packages.opengithub.local/${namespace}/${version}`;
}

function selectedCommand(
  detail: PackageDetail,
  selected: PackageDetailVersion | null,
) {
  const digest = selected?.digest ?? null;
  const version = selected?.version ?? null;
  const versionCommand = detail.installCommands.find(
    (command) =>
      command.version === version &&
      (digest === null || command.digest === digest || command.digest === null),
  );
  return (
    versionCommand ??
    (selected
      ? {
          command: commandForVersion(detail, selected),
          digest,
          label: "Selected version",
          platform: [selected.platformOs, selected.platformArch]
            .filter(Boolean)
            .join("/"),
          version,
        }
      : (detail.installCommands[0] ?? null))
  );
}

export function PackageDetailInteractions({
  detail,
  ownerKind,
}: PackageDetailInteractionsProps) {
  const initialVersionId =
    detail.selectedVersion?.id ?? detail.versions[0]?.id ?? "";
  const [selectedVersionId, setSelectedVersionId] = useState(initialVersionId);
  const selectedVersion =
    detail.versions.find((version) => version.id === selectedVersionId) ??
    detail.selectedVersion ??
    null;
  const selectedBlobs = useMemo(
    () =>
      selectedVersion
        ? detail.blobs.filter(
            (blob) =>
              blob.versionId === selectedVersion.id ||
              detail.blobs.length === 1,
          )
        : detail.blobs,
    [detail.blobs, selectedVersion],
  );
  const command = selectedCommand(detail, selectedVersion);
  const digestCommand =
    detail.packageType === "container" && selectedVersion?.digest
      ? `docker pull ghcr.io/${detail.owner.login}/${detail.name}@${selectedVersion.digest}`
      : null;

  return (
    <>
      <section
        className="card overflow-hidden"
        aria-labelledby="package-install"
      >
        <div className="border-b border-[var(--line)] p-5">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            Installation
          </p>
          <h2 className="t-h2 mt-1" id="package-install">
            Install from the command line
          </h2>
        </div>
        <div className="grid gap-4 p-5">
          {detail.versions.length > 0 ? (
            <label className="grid gap-2">
              <span className="t-label" style={{ color: "var(--ink-3)" }}>
                Version
              </span>
              <select
                aria-label="Package version"
                className="input max-w-xs"
                onChange={(event) => setSelectedVersionId(event.target.value)}
                value={selectedVersionId}
              >
                {detail.versions.map((version) => (
                  <option key={version.id} value={version.id}>
                    {version.version}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          {command ? (
            <div className="rounded-[var(--radius)] bg-[var(--surface-2)] p-4">
              <div className="mb-2 flex flex-wrap items-center gap-2">
                <span className="chip soft">{command.label}</span>
                {selectedVersion?.shortDigest ? (
                  <span className="chip soft">
                    {selectedVersion.shortDigest}
                  </span>
                ) : null}
              </div>
              <code
                className="t-mono block overflow-x-auto whitespace-pre"
                style={{ color: "var(--ink-1)" }}
              >
                {command.command}
              </code>
              <div className="mt-3">
                <CopyButton
                  copiedLabel="Command copied"
                  label="Copy install command"
                  value={command.command}
                />
              </div>
            </div>
          ) : (
            <p className="t-body" style={{ color: "var(--ink-2)" }}>
              Install commands will appear after the first version is published.
            </p>
          )}
          {digestCommand ? (
            <details className="rounded-[var(--radius)] border border-[var(--line)] p-4">
              <summary className="t-sm cursor-pointer font-medium">
                Pull this immutable digest
              </summary>
              <code
                className="t-mono mt-3 block overflow-x-auto whitespace-pre"
                style={{ color: "var(--ink-1)" }}
              >
                {digestCommand}
              </code>
              <div className="mt-3">
                <CopyButton
                  copiedLabel="Digest command copied"
                  label="Copy digest command"
                  value={digestCommand}
                />
              </div>
            </details>
          ) : null}
        </div>
      </section>

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
                          version.digest ?? version.version,
                        )}
                      >
                        {version.version}
                      </Link>
                      {selectedVersion?.id === version.id ? (
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

      {selectedBlobs.length > 0 ? (
        <section className="card p-5" aria-labelledby="package-blobs">
          <p className="t-label" style={{ color: "var(--ink-3)" }}>
            OS / Arch
          </p>
          <h2 className="t-h3 mt-1" id="package-blobs">
            Published artifacts
          </h2>
          <div className="mt-4 grid gap-3">
            {selectedBlobs.slice(0, 6).map((blob) => (
              <details
                className="rounded-[var(--radius)] border border-[var(--line)] p-3"
                key={blob.id}
              >
                <summary className="cursor-pointer">
                  <span className="inline-flex flex-wrap items-center gap-2">
                    <span className="chip soft">
                      {platformLabel(blob.platformOs, blob.platformArch)}
                    </span>
                    <span
                      className="t-mono-sm"
                      style={{ color: "var(--ink-3)" }}
                    >
                      {formatBytes(blob.sizeBytes)}
                    </span>
                  </span>
                </summary>
                <p
                  className="t-mono-sm mt-2 break-all"
                  style={{ color: "var(--ink-3)" }}
                >
                  {blob.digest}
                </p>
              </details>
            ))}
          </div>
        </section>
      ) : null}
    </>
  );
}
