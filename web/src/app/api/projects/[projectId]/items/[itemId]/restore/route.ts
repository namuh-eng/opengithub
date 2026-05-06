import { type NextRequest, NextResponse } from "next/server";
import { type ApiErrorEnvelope, restoreProjectItemFromCookie } from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  try {
    const detail = await restoreProjectItemFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(itemId),
    );
    return NextResponse.json(detail);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_item_restore_failed",
          message: "Project item could not be restored.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
