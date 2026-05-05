import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  recategorizeRepositoryDiscussionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await recategorizeRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        categorySlug:
          typeof body === "object" && body !== null && "categorySlug" in body
            ? String(body.categorySlug)
            : "",
      },
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_category_failed",
          message: "Discussion category could not be changed.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
