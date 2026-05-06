import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  deleteProjectInsightsChartFromCookie,
  type ProjectInsightsChartMutationRequest,
  updateProjectInsightsChartFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; chartId: string }>;
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
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

function parseDelete(input: unknown) {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return { expectedUpdatedAt: null };
  }
  const body = input as Record<string, unknown>;
  return {
    expectedUpdatedAt:
      typeof body.expectedUpdatedAt === "string"
        ? body.expectedUpdatedAt
        : null,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, chartId } = await context.params;
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
    const insights = await updateProjectInsightsChartFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(chartId),
      mutation,
    );
    return NextResponse.json(insights);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_chart_update_failed",
          message: "Project chart could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}

export async function DELETE(request: NextRequest, context: RouteContext) {
  const { projectId, chartId } = await context.params;
  try {
    const insights = await deleteProjectInsightsChartFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(chartId),
      parseDelete(await request.json().catch(() => null)),
    );
    return NextResponse.json(insights);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_chart_delete_failed",
          message: "Project chart could not be deleted.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
