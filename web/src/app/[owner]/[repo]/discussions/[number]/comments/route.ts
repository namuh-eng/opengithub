import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createRepositoryDiscussionCommentFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await createRepositoryDiscussionCommentFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
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
          code: "discussion_comment_failed",
          message: "Discussion comment could not be created.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
