import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  setRepositoryDiscussionAnswerFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

async function updateAnswer(
  request: NextRequest,
  context: RouteContext,
  marked: boolean,
) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await setRepositoryDiscussionAnswerFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      typeof body === "object" && body !== null && "commentId" in body
        ? String(body.commentId)
        : "",
      marked,
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_answer_failed",
          message: "Discussion answer state could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateAnswer(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateAnswer(request, context, false);
}
