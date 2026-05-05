import { type NextRequest, NextResponse } from "next/server";
import {
  type ApiErrorEnvelope,
  type ProjectViewLayoutRequest,
  updateProjectViewLayoutFromCookie,
} from "@/lib/api";

type RouteContext = {
  params: Promise<{ projectId: string; viewId: string }>;
};

const LAYOUTS = new Set(["table", "board", "roadmap"]);

function optionalId(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseViewLayoutRequest(
  input: unknown,
): ProjectViewLayoutRequest | null {
  if (!input || typeof input !== "object" || Array.isArray(input)) return null;
  const body = input as Record<string, unknown>;
  const layout = typeof body.layout === "string" ? body.layout.trim() : "";
  const expectedUpdatedAt =
    typeof body.expectedUpdatedAt === "string"
      ? body.expectedUpdatedAt.trim()
      : "";
  if (!LAYOUTS.has(layout) || !expectedUpdatedAt) return null;
  return {
    layout: layout as ProjectViewLayoutRequest["layout"],
    columnFieldId: optionalId(body.columnFieldId),
    swimlaneFieldId: optionalId(body.swimlaneFieldId),
    startFieldId: optionalId(body.startFieldId),
    targetFieldId: optionalId(body.targetFieldId),
    expectedUpdatedAt,
  };
}

export async function PATCH(request: NextRequest, context: RouteContext) {
  const { projectId, viewId } = await context.params;
  const mutation = parseViewLayoutRequest(
    await request.json().catch(() => null),
  );
  if (!mutation) {
    return NextResponse.json(
      {
        error: {
          code: "validation_failed",
          message:
            "Layout and expected view timestamp are required for layout saves.",
        },
        status: 422,
      },
      { status: 422 },
    );
  }

  try {
    const workspace = await updateProjectViewLayoutFromCookie(
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
          code: "project_view_layout_failed",
          message: "Project view layout could not be saved.",
        },
        status: 502,
      },
      { status: envelope?.status ?? 502 },
    );
  }
}
