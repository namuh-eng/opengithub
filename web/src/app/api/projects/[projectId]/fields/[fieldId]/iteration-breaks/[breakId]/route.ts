import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectIterationBreakFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; fieldId: string; breakId: string }>;
};

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, fieldId, breakId } = await context.params;
  try {
    const settings = await deleteProjectIterationBreakFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(fieldId),
      decodeURIComponent(breakId),
    );
    return NextResponse.json(settings);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_iteration_break_delete_failed",
          message: "Project iteration break could not be deleted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
