import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  transferRepositoryDiscussionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const transfer = await transferRepositoryDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        repositoryId:
          typeof body === "object" && body !== null && "repositoryId" in body
            ? String(body.repositoryId)
            : "",
        categorySlug:
          typeof body === "object" && body !== null && "categorySlug" in body
            ? String(body.categorySlug)
            : "",
      },
    );
    return NextResponse.json(transfer);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "discussion_transfer_failed",
          message: "Discussion could not be transferred.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
