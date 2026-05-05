import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  setRepositoryDiscussionLockFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function errorJson(error: unknown) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "discussion_lock_failed",
        message: "Discussion lock state could not be updated.",
      },
      status: 500,
    },
    { status: envelope?.status ?? 500 },
  );
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await setRepositoryDiscussionLockFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      true,
      {
        allowReactions:
          typeof body === "object" && body !== null && "allowReactions" in body
            ? body.allowReactions !== false
            : true,
      },
    );
    return NextResponse.json(detail);
  } catch (error) {
    return errorJson(error);
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  try {
    const detail = await setRepositoryDiscussionLockFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      false,
    );
    return NextResponse.json(detail);
  } catch (error) {
    return errorJson(error);
  }
}
