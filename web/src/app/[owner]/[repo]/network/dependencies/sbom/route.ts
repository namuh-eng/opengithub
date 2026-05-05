import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  startRepositorySbomExportFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
  }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo } = await context.params;

  try {
    const exportJob = await startRepositorySbomExportFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
    );
    return NextResponse.json(
      {
        ...exportJob,
        downloadHref: exportJob.downloadHref
          ? `/${owner}/${repo}/network/dependencies/sbom/${exportJob.id}`
          : null,
      },
      { status: 201 },
    );
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "sbom_export_failed",
          message: "SBOM export failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
