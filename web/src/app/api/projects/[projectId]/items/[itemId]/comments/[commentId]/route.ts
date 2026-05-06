import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectItemCommentFromCookie,
  updateProjectItemCommentFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string; commentId: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, itemId, commentId } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await updateProjectItemCommentFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
      decodeURIComponent(commentId),
      body,
    );
    return NextResponse.json(detail);
  } catch (error) {
    return commentErrorResponse(
      error,
      "Project item comment could not be saved.",
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, itemId, commentId } = await context.params;
  try {
    const detail = await deleteProjectItemCommentFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
      decodeURIComponent(commentId),
    );
    return NextResponse.json(detail);
  } catch (error) {
    return commentErrorResponse(
      error,
      "Project item comment could not be deleted.",
    );
  }
}

function commentErrorResponse(error: unknown, message: string) {
  const envelope = (
    error instanceof Error ? error.cause : null
  ) as ApiErrorEnvelope | null;
  return NextResponse.json(
    envelope ?? {
      error: {
        code: "project_item_comment_failed",
        message,
      },
      status: 502,
    },
    { status: envelope?.status ?? 502 },
  );
}
