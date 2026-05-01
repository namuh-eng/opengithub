import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  updateRepositoryPullRequestViewedFileFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const fileId =
    typeof body === "object" &&
    body !== null &&
    "fileId" in body &&
    typeof body.fileId === "string"
      ? body.fileId
      : "";
  const versionKey =
    typeof body === "object" &&
    body !== null &&
    "versionKey" in body &&
    typeof body.versionKey === "string"
      ? body.versionKey
      : "";
  const viewed =
    typeof body === "object" && body !== null && "viewed" in body
      ? Boolean(body.viewed)
      : false;

  try {
    const result = await updateRepositoryPullRequestViewedFileFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      { fileId, versionKey, viewed },
    );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "viewed_file_failed",
          message: "Viewed file state could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
