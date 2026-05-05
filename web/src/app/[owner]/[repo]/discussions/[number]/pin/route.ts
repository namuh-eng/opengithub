import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  pinRepositoryDiscussionFromCookie,
  unpinRepositoryDiscussionFromCookie,
  updateRepositoryDiscussionPinFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

function errorJson(error: unknown, code: string, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? { error: { code, message }, status: 500 },
    { status: envelope?.status ?? 500 },
  );
}

export async function PUT(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await pinRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        target:
          typeof body === "object" &&
          body !== null &&
          "target" in body &&
          body.target === "category"
            ? "category"
            : "global",
        categorySlug:
          typeof body === "object" && body !== null && "categorySlug" in body
            ? String(body.categorySlug)
            : undefined,
        title:
          typeof body === "object" && body !== null && "title" in body
            ? String(body.title)
            : undefined,
        body:
          typeof body === "object" && body !== null && "body" in body
            ? String(body.body)
            : undefined,
      },
    );
    return NextResponse.json(detail);
  } catch (error) {
    return errorJson(
      error,
      "discussion_pin_failed",
      "Discussion could not be pinned.",
    );
  }
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await updateRepositoryDiscussionPinFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        title:
          typeof body === "object" && body !== null && "title" in body
            ? String(body.title)
            : undefined,
        body:
          typeof body === "object" && body !== null && "body" in body
            ? String(body.body)
            : undefined,
      },
    );
    return NextResponse.json(detail);
  } catch (error) {
    return errorJson(
      error,
      "discussion_pin_update_failed",
      "Discussion pin could not be updated.",
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  try {
    const detail = await unpinRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
    );
    return NextResponse.json(detail);
  } catch (error) {
    return errorJson(
      error,
      "discussion_unpin_failed",
      "Discussion could not be unpinned.",
    );
  }
}
