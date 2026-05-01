import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  abandonRepositoryPullRequestReviewDraftFromCookie,
  submitRepositoryPullRequestReviewFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);

  try {
    const result = await submitRepositoryPullRequestReviewFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        body: readOptionalString(body, "body"),
        state: readReviewState(body),
      },
    );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Review could not be submitted.");
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;

  try {
    const result = await abandonRepositoryPullRequestReviewDraftFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
    );
    return NextResponse.json(result);
  } catch (error) {
    return errorResponse(error, "Review draft could not be abandoned.");
  }
}

function errorResponse(error: unknown, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "review_failed",
        message,
      },
      status: 500,
    },
    { status: envelope?.status ?? 500 },
  );
}

function readOptionalString(source: unknown, key: string) {
  if (typeof source !== "object" || source === null) {
    return null;
  }
  const value = (source as Record<string, unknown>)[key];
  return typeof value === "string" ? value : null;
}

function readReviewState(source: unknown) {
  const value = readOptionalString(source, "state");
  if (
    value === "approved" ||
    value === "changes_requested" ||
    value === "commented"
  ) {
    return value;
  }
  return "commented";
}
