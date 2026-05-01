import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryPullRequestReviewDraftCommentFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result =
      await createRepositoryPullRequestReviewDraftCommentFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(number),
        {
          fileId: readString(body, "fileId"),
          body: readString(body, "body"),
          side: readString(body, "side") || "right",
          oldLine: readNullableNumber(body, "oldLine"),
          newLine: readNullableNumber(body, "newLine"),
          position: readNumber(body, "position"),
        },
      );
    return NextResponse.json(result);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "review_draft_failed",
          message: "Review comment draft could not be saved.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}

function readString(source: unknown, key: string) {
  if (typeof source !== "object" || source === null) {
    return "";
  }
  const value = (source as Record<string, unknown>)[key];
  return typeof value === "string" ? value : "";
}

function readNullableNumber(source: unknown, key: string) {
  if (typeof source !== "object" || source === null) {
    return null;
  }
  const value = (source as Record<string, unknown>)[key];
  return typeof value === "number" ? value : null;
}

function readNumber(source: unknown, key: string) {
  return readNullableNumber(source, key) ?? 0;
}
