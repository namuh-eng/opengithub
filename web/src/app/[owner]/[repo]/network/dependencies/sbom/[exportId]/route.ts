import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  downloadRepositorySbomExportFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    exportId: string;
  }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, exportId } = await context.params;

  try {
    const response = await downloadRepositorySbomExportFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(exportId),
    );
    const body = await response.arrayBuffer();
    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-disposition":
          response.headers.get("content-disposition") ??
          `attachment; filename="${owner}-${repo}-sbom.spdx.json"`,
        "content-type":
          response.headers.get("content-type") ?? "application/spdx+json",
      },
    });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "sbom_download_failed",
          message: "SBOM download failed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
