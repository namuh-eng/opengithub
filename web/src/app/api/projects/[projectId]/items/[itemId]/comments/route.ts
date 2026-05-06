import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectItemCommentFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await createProjectItemCommentFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
      body,
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_item_comment_failed",
          message: "Project item comment could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
