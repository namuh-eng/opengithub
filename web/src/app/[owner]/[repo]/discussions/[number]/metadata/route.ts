import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  updateRepositoryDiscussionMetadataFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ owner: string; repo: string; number: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const labelIds =
    typeof body === "object" && body !== null && Array.isArray(body.labelIds)
      ? body.labelIds.map(String)
      : undefined;
  try {
    const detail = await updateRepositoryDiscussionMetadataFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      {
        categorySlug:
          typeof body === "object" && body !== null && "categorySlug" in body
            ? String(body.categorySlug)
            : undefined,
        labelIds,
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
          code: "discussion_metadata_failed",
          message: "Discussion metadata could not be updated.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
