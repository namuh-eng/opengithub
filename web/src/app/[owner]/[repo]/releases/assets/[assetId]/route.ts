import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  getRepositoryReleaseAssetDownloadFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    assetId: string;
  }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, assetId } = await context.params;
  try {
    const metadata = await getRepositoryReleaseAssetDownloadFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(assetId),
    );
    return NextResponse.json(metadata);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "asset_download_failed",
          message: "Release asset could not be prepared.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
