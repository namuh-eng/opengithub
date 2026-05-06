import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  updateProjectDraftItemFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; itemId: string }>;
};

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, itemId } = await context.params;
  const body = await request.json().catch(() => null);
  try {
    const detail = await updateProjectDraftItemFromCookie(
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
          code: "project_draft_update_failed",
          message: "Draft project item could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
