import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteRepositoryDiscussionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const response = await deleteRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        confirmation:
          typeof body === "object" && body !== null && "confirmation" in body
            ? String(body.confirmation)
            : "",
        reason:
          typeof body === "object" && body !== null && "reason" in body
            ? String(body.reason)
            : undefined,
      },
    );
    return NextResponse.json(response);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_delete_failed",
          message: "Discussion could not be deleted.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
