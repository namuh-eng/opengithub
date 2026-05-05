"use client";

import { useState } from "react";
import type {
  RepositoryDependencyExportState,
  RepositorySbomExport,
} from "@/lib/api";

type RepositoryDependencyExportButtonProps = {
  exportState: RepositoryDependencyExportState;
  owner: string;
  repo: string;
};

function exportStatusLabel(status: string | null) {
  if (!status) return "No export yet";
  if (status === "ready") return "Latest SBOM ready";
  if (status === "pending") return "Latest SBOM pending";
  if (status === "failed") return "Latest SBOM failed";
  return `Latest SBOM ${status}`;
}

export function RepositoryDependencyExportButton({
  exportState,
  owner,
  repo,
}: RepositoryDependencyExportButtonProps) {
  const [job, setJob] = useState<RepositorySbomExport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function startExport() {
    if (!exportState.supported || pending) return;
    setPending(true);
    setError(null);
    try {
      const response = await fetch(
        `/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/network/dependencies/sbom`,
        { method: "POST" },
      );
      const body = (await response.json().catch(() => null)) as
        | RepositorySbomExport
        | { error?: { message?: string } }
        | null;
      if (!response.ok) {
        throw new Error(
          body && "error" in body
            ? body.error?.message || "SBOM export failed."
            : "SBOM export failed.",
        );
      }
      setJob(body as RepositorySbomExport);
    } catch (caught) {
      setError(
        caught instanceof Error ? caught.message : "SBOM export failed.",
      );
    } finally {
      setPending(false);
    }
  }

  const downloadHref = job?.downloadHref;

  return (
    <div className="flex flex-wrap items-center gap-2">
      <button
        aria-describedby="sbom-export-status"
        className={`btn ${exportState.supported ? "primary" : ""}`}
        disabled={!exportState.supported || pending}
        onClick={startExport}
        type="button"
      >
        {pending ? "Exporting SBOM" : "Export SBOM"}
      </button>
      <span className="chip soft" id="sbom-export-status">
        {job
          ? exportStatusLabel(job.status)
          : exportStatusLabel(exportState.latestStatus)}
      </span>
      {downloadHref ? (
        <a className="btn" href={downloadHref}>
          Download SBOM
        </a>
      ) : null}
      {error ? (
        <span className="chip err" role="status">
          {error}
        </span>
      ) : null}
    </div>
  );
}
