import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, removeProjectItemFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  try {
    const workspace = await removeProjectItemFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
    );
    return NextResponse.json(workspace);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_item_remove_failed",
          message: "Project item could not be removed.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
