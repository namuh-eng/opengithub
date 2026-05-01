import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteRepositoryPullRequestReviewDraftCommentFromCookie,
  updateRepositoryPullRequestReviewDraftCommentFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
    draftId: string;
  }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number, draftId } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result =
      await updateRepositoryPullRequestReviewDraftCommentFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(number),
        decodeURIComponent(draftId),
        { body: readString(body, "body") },
      );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Review comment draft could not be updated.");
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, number, draftId } = await context.params;

  try {
    const result =
      await deleteRepositoryPullRequestReviewDraftCommentFromCookie(
        request.headers.get("cookie"),
        decodeURIComponent(owner),
        decodeURIComponent(repo),
        decodeURIComponent(number),
        decodeURIComponent(draftId),
      );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Review comment draft could not be deleted.");
  }
}

function errorResponse(error: unknown, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "review_draft_failed",
        message,
      },
      status: 500,
    },
    { status: envelope?.status ?? 500 },
  );
}

function readString(source: unknown, key: string) {
  if (typeof source !== "object" || source === null) {
    return "";
  }
  const value = (source as Record<string, unknown>)[key];
  return typeof value === "string" ? value : "";
}
