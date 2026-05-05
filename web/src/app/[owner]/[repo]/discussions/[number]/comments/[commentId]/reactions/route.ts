import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type DiscussionReactionContent,
  setRepositoryDiscussionReactionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
    commentId: string;
  }>;
};

const reactions = new Set([
  "+1",
  "-1",
  "laugh",
  "hooray",
  "confused",
  "heart",
  "rocket",
  "eyes",
]);

async function updateReaction(
  request: NextRequest,
  context: RouteContext,
  reacted: boolean,
) {
  const { owner, repo, number, commentId } = await context.params;
  const body = await request.json().catch(() => null);
  const content =
    typeof body === "object" && body !== null && "content" in body
      ? String(body.content)
      : "";
  if (!reactions.has(content)) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "reaction is not supported",
        },
        status: 422,
      },
      { status: 422 },
    );
  }
  try {
    const summaries = await setRepositoryDiscussionReactionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      content as DiscussionReactionContent,
      reacted,
      decodeURIComponent(commentId),
    );
    return NextResponse.json(summaries);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_reaction_failed",
          message: "Discussion reaction could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}

export async function PUT(request: NextRequest, context: RouteContext) {
  return updateReaction(request, context, true);
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  return updateReaction(request, context, false);
}
