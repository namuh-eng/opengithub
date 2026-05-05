import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryDiscussionReplyFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
    commentId: string;
  }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number, commentId } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await createRepositoryDiscussionReplyFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      decodeURIComponent(commentId),
      {
        body:
          typeof body === "object" && body !== null && "body" in body
            ? String(body.body)
            : "",
        attachmentDrafts: [],
      },
    );
    return NextResponse.json(detail, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_reply_failed",
          message: "Discussion reply could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
