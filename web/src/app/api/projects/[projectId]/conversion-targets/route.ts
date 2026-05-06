import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  getProjectConversionTargetsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  try {
    const targets = await getProjectConversionTargetsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
    );
    return NextResponse.json(targets);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_conversion_targets_failed",
          message: "Project conversion targets could not be loaded.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
