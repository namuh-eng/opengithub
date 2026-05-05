import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  convertIssueToDiscussionFromCookie,
  getIssueDiscussionConversionFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{
    owner: string;
    repo: string;
    number: string;
  }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  try {
    const view = await getIssueDiscussionConversionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
    );
    return NextResponse.json(view);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "issue_conversion_metadata_failed",
          message: "Discussion conversion metadata could not be loaded.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { owner, repo, number } = await context.params;
  const body = await request.json().catch(() => null);
  const categorySlug =
    typeof body === "object" && body !== null && "categorySlug" in body
      ? String(body.categorySlug)
      : "";

  try {
    const response = await convertIssueToDiscussionFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(owner),
      decodeURIComponent(repo),
      decodeURIComponent(number),
      categorySlug,
    );
    return NextResponse.json(response);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "issue_convert_to_discussion_failed",
          message: "Issue could not be converted to a discussion.",
        },
        status: 500,
      },
      { status: envelope?.status ?? 500 },
    );
  }
}
