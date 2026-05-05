import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectRoadmapSettingsRequest,
  updateProjectRoadmapSettingsFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; viewId: string }>;
};

const ZOOMS = new Set(["month", "quarter", "year"]);

function requiredId(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseRoadmapSettingsRequest(
  input: unknown,
): ProjectRoadmapSettingsRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const startFieldId = requiredId(body.startFieldId);
  const targetFieldId = requiredId(body.targetFieldId);
  const zoom = typeof body.zoom === "string" ? body.zoom.trim() : "";
  const expectedUpdatedAt =
    typeof body.expectedUpdatedAt === "string"
      ? body.expectedUpdatedAt.trim()
      : "";
  if (
    !startFieldId ||
    !targetFieldId ||
    !ZOOMS.has(zoom) ||
    !expectedUpdatedAt
  ) {
    return null;
  }
  const markerFieldIds = Array.isArray(body.markerFieldIds)
    ? body.markerFieldIds.filter(
        (value): value is string =>
          typeof value === "string" && value.trim().length > 0,
      )
    : [];
  return {
    startFieldId,
    targetFieldId,
    markerFieldIds,
    zoom,
    expectedUpdatedAt,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, viewId } = await context.params;
  const mutation = parseRoadmapSettingsRequest(
    await request.json().catch(() => null),
  );
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message:
            "Start field, target field, zoom, and expected view timestamp are required for roadmap settings.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const workspace = await updateProjectRoadmapSettingsFromCookie(
      request.headers.get("cookie"),
      decodeURIComponent(projectId),
      decodeURIComponent(viewId),
      mutation,
    );
    return NextResponse.json(workspace);
  } catch (error) {
    const envelope = (
      error instanceof Error ? error.cause : null
    ) as ApiErrorEnvelope | null;
    return NextResponse.json(
      envelope ?? {
        error: {
          code: "project_roadmap_settings_failed",
          message: "Project roadmap settings could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
