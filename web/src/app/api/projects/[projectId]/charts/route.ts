import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  createProjectInsightsChartFromCookie,
  type ProjectInsightsChartMutationRequest,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string }>;
};

function parseChartMutation(
  input: unknown,
): ProjectInsightsChartMutationRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const title = typeof body.title === "string" ? body.title.trim() : "";
  const chartType =
    typeof body.chartType === "string" ? body.chartType.trim() : "";
  const visibility =
    typeof body.visibility === "string" ? body.visibility.trim() : "";
  if (!title || !chartType || !visibility) return null;
  return {
    title,
    description:
      typeof body.description === "string" ? body.description.trim() : null,
    chartType,
    filter: typeof body.filter === "string" ? body.filter.trim() : null,
    xFieldId:
      typeof body.xFieldId === "string" && body.xFieldId.trim()
        ? body.xFieldId.trim()
        : null,
    yFieldId:
      typeof body.yFieldId === "string" && body.yFieldId.trim()
        ? body.yFieldId.trim()
        : null,
    groupFieldId:
      typeof body.groupFieldId === "string" && body.groupFieldId.trim()
        ? body.groupFieldId.trim()
        : null,
    visibility,
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string" &&
      body.expectedUpdatedAt.trim()
        ? body.expectedUpdatedAt.trim()
        : null,
  };
}

export async function POST(request: NextRequest, context: RouteContext) {
  const { projectId } = await context.params;
  const mutation = parseChartMutation(await request.json().catch(() => null));
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message: "Chart title, type, and visibility are required.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const insights = await createProjectInsightsChartFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      mutation,
    );
    return NextResponse.json(insights, { status: 201 });
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_chart_create_failed",
          message: "Project chart could not be created.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
